//! This module provides a mixer to play multiple audio files simultaneously.
//!
//! In brief, the way it works is by utilizing double buffers (one is played while the other is mixed)
//! which are switched on vblank.
//!
//! # Safety
//!
//! Due to the need to perform mixing in an ISR and the way atomics are emulated,
//! many of the data structures in this module are not locked and therefore not race-safe.
//! Therefore you should NEVER use these in concurrent contexts (like interrupts, async, etc!)
//!
//! TODO: Ponder how to improve safety (maybe just make adding a stream unsafe and push it onto the user?)

use super::wave;
use crate::debug_log::Subsystems::Sound;
use crate::interrupt;
use crate::FS;

use byte_slice_cast::{AsByteSlice, AsSliceOf};
use gba::io::{dma::*, irq, sound, timers};
use gbfs_rs::GBFSError;

/// The maximum number of streams that may be mixed.
const NUM_MIXABLE_STREAMS: usize = 4;

/// Size of a single audio buffer. Equivalent to the number of samples per frame (we switch buffers on vblank)
const AUDIO_BUF_SIZE: usize = 304;
/// How many samples to play per second
const SAMPLE_RATE: u32 = 18157;
const CPU_CYCLES_PER_SEC: u32 = 16777216;
const TIMER_MAX_VALUE: u32 = u16::MAX as u32;

/// Which of the buffers are currently being used for playback.
enum FrontBuf {
    Zero,
    One,
}

/// Audio buffer suitable for feeding to the hardware for playback.
type AudioBuf = [i8; AUDIO_BUF_SIZE];
/// Audio buffer only used for mixing to prevent clipping caused by overflow.
type MixBuf = [i16; AUDIO_BUF_SIZE];

/// Things that may go wrong when playing audio.
// TODO: Display trait
#[derive(Debug, Clone)]
pub enum SoundError {
    AllChannelsUsed,
    Filesystem(GBFSError),
    Wave(wave::WaveError),
}

impl From<GBFSError> for SoundError {
    fn from(err: GBFSError) -> Self {
        return SoundError::Filesystem(err);
    }
}

impl From<wave::WaveError> for SoundError {
    fn from(err: wave::WaveError) -> Self {
        return SoundError::Wave(err);
    }
}

struct Stream {
    /// The actual sample data
    data: &'static [i8],
    /// Where to start playback of next part of the sample
    offset: usize,
}

/// Double buffers (one plays, the other is used for mixing)
static mut BUF_0: AudioBuf = [0; AUDIO_BUF_SIZE];
static mut BUF_1: AudioBuf = [0; AUDIO_BUF_SIZE];

/// Which of the buffers are currently being used for playback
static mut FRONT_BUF: FrontBuf = FrontBuf::Zero;

/// The streams that should be mixed.
static mut STREAMS: [Option<Stream>; NUM_MIXABLE_STREAMS] = [None; NUM_MIXABLE_STREAMS];

/// Initializes the audio mixer.
///
/// Will panic if interrupts aren't enabled when called.
///
/// # Safety
/// This uses several hardware timers and a DMA engine.
/// After activation, you shouldn't touch `DMA1` or timer `0`.
///
/// Also, keep the module-wide safety warning in mind.
pub unsafe fn init() {
    // Before we do anything, make sure that interrupts are enabled
    // so that we can stop playback later.
    if irq::IME.read() == irq::IrqEnableSetting::IRQ_NO {
        panic!("Sound playback requires interrupts to be enabled")
    }

    // Master sound enable: has to be set before any of the registers are usable
    sound::SOUNDCNT_X.write(sound::SoundMasterSetting::new().with_psg_fifo_master_enabled(true));

    // Configure sound timer initial value such that it overflows exactly when a sample is about to run out.
    timers::TM0CNT_L.write((TIMER_MAX_VALUE - (CPU_CYCLES_PER_SEC / SAMPLE_RATE)) as u16);
    timers::TM0CNT_H.write(
        timers::TimerControlSetting::new()
            // Count up by 1 each CPU cycle
            .with_tick_rate(timers::TimerTickRate::CPU1)
            .with_enabled(true),
    );

    // Configure wave sound control register
    sound::SOUNDCNT_H.write(
        sound::WaveVolumeEnableSetting::new()
            // The other channels should not be audible
            .with_dma_sound_a_full_volume(true)
            // Mono sound (same track on both channels)
            .with_dma_sound_a_enable_left(true)
            .with_dma_sound_a_enable_right(true)
            // Sample each time timer 0 (rather than 1) runs out
            .with_dma_sound_a_timer_select(false)
            // Ensure the FIFO is prepared
            .with_dma_sound_a_reset_fifo(true),
    );

    // Configure the vblank interrupt to switch our buffers
    interrupt::set_vblank_handler(Some(&vblank_irq));

    // Configure DMA 1 to continuously transfer samples from the currently active buffer
    unsafe {
        DMA1::set_source(BUF_0.as_ptr() as *const u32);
        // Write into direct sound channel A's FIFO
        DMA1::set_dest(sound::FIFO_A_L.to_usize() as *mut u32);
        DMA1::set_control(
            DMAControlSetting::new()
                .with_dma_repeat(true)
                // Transfer a word at a time
                .with_use_32bit(true)
                // Start DMA when FIFO needs sample
                .with_start_time(DMAStartTiming::Special)
                // We want the next sample to be transferred each time
                .with_source_address_control(DMASrcAddressControl::Increment)
                // Total FIFO length is 32 bits, meaning we should always write to same address
                .with_dest_address_control(DMADestAddressControl::Fixed)
                .with_enabled(true),
        );
    }
}

/// Switch the buffer that the sound hardware currently plays from.
fn switch_active_buf() {
    unsafe {
        match FRONT_BUF {
            FrontBuf::Zero => {
                FRONT_BUF = FrontBuf::One;
                DMA1::set_source(BUF_1.as_ptr() as *const u32);
            }
            FrontBuf::One => {
                FRONT_BUF = FrontBuf::Zero;
                DMA1::set_source(BUF_0.as_ptr() as *const u32);
            }
        }
    }
}

/// Add a stream from a wave file.
///
/// Returns an error if
/// the maximum number of streams is already playing or the file is unsuitable.
pub fn add_wave_file_stream(file_name: &str) -> Result<(), SoundError> {
    let (audio, sample_rate) = wave::from_file(file_name)?;
    debug_log!(
        Sound,
        "Adding wav file {} to mixer (sample rate: {} Hz, {} samples)",
        file_name,
        sample_rate,
        audio.len() * 4
    );
    // TODO: Deal with sample rate
    add_raw_stream(audio)?;
    return Ok(());
}

/// Add a stream from a raw file.
///
/// Returns an error if
/// the maximum number of streams is already playing or the file is unsuitable.
pub fn add_raw_file_stream(file_name: &str) -> Result<(), SoundError> {
    let data = FS.get_file_data_by_name_as_u32_slice(file_name)?;
    debug_log!(
        Sound,
        "Adding raw file {} to mixer ({} samples)",
        file_name,
        data.len() * 4
    );
    add_raw_stream(data)?;
    return Ok(());
}

/// Add a stream to be mixed.
/// The data must actually be valid i8 samples, but a u32 is taken to ensure that DMA works properly.
///
/// Returns an error if
/// the maximum number of streams is already playing.
fn add_raw_stream(data: &'static [u32]) -> Result<(), SoundError> {
    unsafe {
        // Find the first unused stream slot
        let mut first_free_stream: Option<usize> = None;
        for (i, stream) in STREAMS.iter().enumerate() {
            if stream.is_none() {
                first_free_stream = Some(i);
                break;
            }
        }
        if first_free_stream.is_none() {
            return Err(SoundError::AllChannelsUsed);
        }

        // Actually insert the stream there
        let stream_idx = first_free_stream.unwrap();
        debug_log!(
            Sound,
            "Adding stream with {} samples as stream no. {}",
            data.len() * 4,
            stream_idx
        );
        STREAMS[stream_idx] = Some(Stream {
            data: data.as_byte_slice().as_slice_of::<i8>().unwrap(),
            offset: 0,
        });
        return Ok(());
    }
}

/// Mix active streams in the back buffer.
fn mix_buffers() {
    fn mix_stream(buf: &mut MixBuf, stream: &mut Stream) {
        let mixer_in = &stream.data[stream.offset..];
        // Add the sample
        let max_idx = buf.len();
        for (i, sample) in mixer_in
            .iter()
            .enumerate()
            .take_while(|(i, _)| i < &max_idx)
        {
            buf[i] += *sample as i16;
        }

        // Add the mixed data to the offset
        stream.offset += buf.len();
    }

    fn disable_inactive_streams() {
        // Check whether any stream has been emptied now, meaning we can mark it as such
        unsafe {
            for (i, stream) in STREAMS
                .iter_mut()
                .enumerate()
                .filter(|(_, stream)| stream.is_some())
            {
                // TODO: Take looping into consideration

                if stream.as_ref().unwrap().data.len() <= stream.as_ref().unwrap().offset {
                    debug_log!(Sound, "Stream {} now empty, disabling", i);
                    *stream = None;
                }
            }
        }
    }

    // TODO: Per-channel volume control
    // TODO: Resampling
    // TODO: Looping

    let mut mix_buffer: MixBuf = [0; AUDIO_BUF_SIZE];
    // Iterate over the streams and mix
    unsafe {
        for stream in STREAMS.iter_mut() {
            match stream {
                Some(stream) => mix_stream(&mut mix_buffer, stream),
                None => {}
            }
        }
    }

    unsafe {
        // Copy the wider mixing buffer into the narrower hardware buffer
        // Allow the optimizer to elide bounds checks
        assert_eq!(BUF_0.len(), mix_buffer.len());
        assert_eq!(BUF_1.len(), mix_buffer.len());
        match FRONT_BUF {
            FrontBuf::Zero => {
                for (i, elem) in mix_buffer.iter().map(|x| *x as i8).enumerate() {
                    BUF_1[i] = elem;
                }
            }
            FrontBuf::One => {
                for (i, elem) in mix_buffer.iter().map(|x| *x as i8).enumerate() {
                    BUF_0[i] = elem;
                }
            }
        }
    }

    // TODO: Disable sound playback entirely if all streams are inactive
    disable_inactive_streams();
}

/// All the housekeeping tasks that must be performed to refill a buffer are called here.
fn vblank_irq() {
    switch_active_buf();
    mix_buffers();
}

/// A helper function to terminate tests once all streams are inactive.
/// Not to be used outside of tests.
pub(super) fn spin_until_all_streams_inactive() {
    unsafe {
        loop {
            if STREAMS.iter().filter(|x| x.is_some()).count() == 0 {
                return;
            }
        }
    }
}
