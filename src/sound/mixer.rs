//! This module provides a mixer to play multiple audio files simultaneously.
//!
//! In brief, the way it works is by utilizing double buffers (one is played while the other is mixed)
//! which are switched on vblank.
//!
//! TODO: Consider moving away from vblank to some timer or so to free up that IRQ.

use core::convert::TryInto;

use super::wave;
use crate::debug_log::Subsystems::Sound;
use crate::interrupt;
use crate::FS;

use byte_slice_cast::AsSliceOf;
use gba::prelude::*;
use gbfs_rs::GBFSError;
use spinning_top::{const_spinlock, Spinlock};

/// The maximum number of streams that may be mixed.
const NUM_MIXABLE_STREAMS: usize = 4;

/// Size of a single audio buffer. Equivalent to the number of samples per frame (we switch buffers on vblank).
const AUDIO_BUF_SIZE: usize = 304;

/// The supported sample rate in Hz. All supplied audio must adhere to this.
pub const SAMPLE_RATE: u32 = 18157;

/// CPU cycles per second.
const CPU_CYCLES_PER_SEC: u32 = 16777216;

/// The maximum value of a hardware timer.
const TIMER_MAX_VALUE: u32 = u16::MAX as u32;

/// Which of the buffers are currently being used for playback.
enum FrontBuf {
    Zero,
    One,
}

/// Audio buffer suitable for feeding to the hardware for playback.
type AudioBuf = [i8; AUDIO_BUF_SIZE];
/// Audio buffer only used for mixing (widened to prevent clipping caused by overflow).
type MixBuf = [i16; AUDIO_BUF_SIZE];

/// Things that may go wrong when playing audio.
// TODO: Display trait
#[derive(Debug, Clone)]
pub enum SoundError {
    AllChannelsInUse,
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

#[derive(Debug, Copy, Clone)]
struct Stream {
    /// The actual sample data
    data: &'static [i8],
    /// Where to start playback of next part of the sample
    offset: usize,
}

/// Double buffers (one plays, the other is used for mixing).
static BUF_0: Spinlock<AudioBuf> = const_spinlock([0; AUDIO_BUF_SIZE]);
static BUF_1: Spinlock<AudioBuf> = const_spinlock([0; AUDIO_BUF_SIZE]);

/// Which of the buffers is currently being used for playback.
static FRONT_BUF: Spinlock<FrontBuf> = const_spinlock(FrontBuf::Zero);

/// The streams that should be mixed.
static STREAMS: Spinlock<[Option<Stream>; NUM_MIXABLE_STREAMS]> =
    const_spinlock([None; NUM_MIXABLE_STREAMS]);

/// Initializes the audio mixer.
///
/// Will panic if interrupts aren't enabled when called.
///
/// # Safety
///
/// This uses a hardware timer, a DMA engine and the vblank IRQ handler.
/// After activation, you shouldn't touch `DMA1`, `timer0` or the `vblank` IRQ.
pub fn init() {
    // Before we do anything, make sure that interrupts are enabled
    // so that we can mix on vblank
    if !IME.read() {
        panic!("Sound playback requires interrupts to be enabled")
    }

    // Master sound enable: has to be set before any of the registers are usable
    SOUND_STATUS.write(SoundStatus::new().with_enabled(true));

    // Configure sound timer initial value such that it overflows exactly when a sample is about to run out.
    TIMER0_RELOAD.write(
        (TIMER_MAX_VALUE - (CPU_CYCLES_PER_SEC / SAMPLE_RATE))
            .try_into()
            .unwrap(),
    );
    // Count up by 1 each CPU cycle
    TIMER0_CONTROL.write(
        TimerControl::new()
            .with_prescaler_selection(0)
            .with_enabled(true),
    );

    FIFO_RESET.write(FifoReset::new().with_reset_fifo_a(true));
    // Configure wave sound control register
    FIFO_CONTROL.write(
        FifoControl::new()
            // The other channels should not be audible
            .with_full_volume_a(true)
            // Mono sound (same track on both channels)
            .with_enable_left_a(true)
            .with_enable_right_a(true)
            // Sample each time timer 0 (rather than 1) runs out
            .with_use_timer1_a(false),
    );

    // Configure the vblank interrupt to switch our buffers
    interrupt::set_vblank_handler(Some(&sound_vblank_irq));

    // Configure DMA 1 to continuously transfer samples from the currently active buffer
    unsafe {
        let buf_0 = BUF_0.lock();
        DMA1SAD.write(buf_0.as_ptr() as usize);
        // Write into direct sound channel A's FIFO
        DMA1DAD.write(FIFO_A.as_usize());
        DMA1CNT_H.write(
            DmaControl::new()
                .with_dma_repeat(true)
                // Transfer a word at a time
                .with_transfer_u32(true)
                // Start DMA when FIFO needs sample
                .with_start_time(DmaStartTiming::Special)
                // We want the next sample to be transferred each time
                .with_src_addr(SrcAddrControl::Increment)
                // Total FIFO length is 32 bits, meaning we should always write to same address
                .with_dest_addr(DestAddrControl::Fixed)
                .with_enabled(true),
        );
    }
}

/// Switch the buffer that the sound hardware currently plays from.
fn switch_front_buf() {
    let mut front_buf = FRONT_BUF.lock();
    match *front_buf {
        FrontBuf::Zero => {
            *front_buf = FrontBuf::One;
            let buf_1 = BUF_1.lock();
            unsafe { DMA1SAD.write(buf_1.as_ptr() as usize) };
        }
        FrontBuf::One => {
            *front_buf = FrontBuf::Zero;
            let buf_0 = BUF_0.lock();
            unsafe { DMA1SAD.write(buf_0.as_ptr() as usize) };
        }
    }
}

/// Add a stream from a wave file.
///
/// Returns an error if
/// the maximum number of streams is already playing or the file is unsuitable.
pub fn add_wave_file_stream(file_name: &str) -> Result<(), SoundError> {
    let audio = wave::from_file(file_name)?;
    debug_log!(
        Sound,
        "Adding wav file {} to mixer ({} samples)",
        file_name,
        audio.len()
    );
    add_raw_stream(audio)?;
    return Ok(());
}

/// Add a stream from a raw file.
///
/// Returns an error if
/// the maximum number of streams is already playing or the file is unsuitable.
///
/// Keep in mind that raw files have no metadata, so it's on you to make sure
/// the samples are of the correct width and sample rate.
pub fn add_raw_file_stream(file_name: &str) -> Result<(), SoundError> {
    let audio = FS
        .get_file_data_by_name(file_name)?
        .as_slice_of::<i8>()
        .unwrap();
    debug_log!(
        Sound,
        "Adding raw file {} to mixer ({} samples)",
        file_name,
        audio.len()
    );
    add_raw_stream(audio)?;
    return Ok(());
}

/// Add a stream to be mixed.
///
/// Returns an error if
/// the maximum number of streams is already playing.
fn add_raw_stream(data: &'static [i8]) -> Result<(), SoundError> {
    let mut streams = STREAMS.lock();
    // Find the first unused stream slot
    let mut first_free_stream: Option<usize> = None;
    for (i, stream) in streams.iter().enumerate() {
        if stream.is_none() {
            first_free_stream = Some(i);
            break;
        }
    }
    if first_free_stream.is_none() {
        return Err(SoundError::AllChannelsInUse);
    }

    // Actually insert the stream there
    let stream_idx = first_free_stream.unwrap();
    debug_log!(
        Sound,
        "Adding stream with {} samples as stream no. {}",
        data.len(),
        stream_idx
    );
    streams[stream_idx] = Some(Stream {
        data: data,
        offset: 0,
    });
    return Ok(());
}

/// Mix active streams in the back buffer.
fn mix_buffers() {
    /// Perform the actual mixing of a single stream by adding samples and updating the offset into the stream afterwards
    fn mix_stream(mix_buf: &mut MixBuf, stream: &mut Stream) {
        let mixer_input = &stream.data[stream.offset..stream.offset + mix_buf.len()];
        for (i, sample) in mixer_input.iter().enumerate() {
            mix_buf[i] += *sample as i16;
        }

        // Mark the samples in the stream as played by updating the offset to next sample to play
        stream.offset += mix_buf.len();
    }

    /// Remove streams that are finished playing from the list of active streams.
    fn gc_finished_streams() {
        let mut streams = STREAMS.lock();
        // Check whether any stream has been emptied now, meaning we can mark it as such
        for (i, stream) in streams
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

        // TODO: If no more streams are active, disable mixing (and re-enable once new stream is added).
        //if streams.iter().filter(|stream| stream.is_some()).count() == 0 {
        //    interrupt::set_vblank_handler(None);
        //}
    }

    fn mix_all_streams(mix_buffer: &mut MixBuf) {
        let mut streams = STREAMS.lock();
        for stream in streams.iter_mut() {
            match stream {
                Some(stream) => mix_stream(mix_buffer, stream),
                None => {}
            }
        }
    }

    fn copy_from_mix_to_playback_buf(mix_buffer: &MixBuf) {
        let mut buf_0 = BUF_0.lock();
        let mut buf_1 = BUF_1.lock();
        // Copy the wider mixing buffer into the narrower hardware buffer
        // Allow the optimizer to elide bounds checks
        assert_eq!(buf_0.len(), mix_buffer.len());
        assert_eq!(buf_1.len(), mix_buffer.len());
        let front_buf = FRONT_BUF.lock();
        match *front_buf {
            FrontBuf::Zero => {
                for (i, elem) in mix_buffer.iter().map(|x| *x as i8).enumerate() {
                    buf_1[i] = elem;
                }
            }
            FrontBuf::One => {
                for (i, elem) in mix_buffer.iter().map(|x| *x as i8).enumerate() {
                    buf_0[i] = elem;
                }
            }
        }
    }

    // TODO: Per-channel volume control
    // TODO: Looping

    let mut mix_buffer: MixBuf = [0; AUDIO_BUF_SIZE];
    mix_all_streams(&mut mix_buffer);
    copy_from_mix_to_playback_buf(&mix_buffer);

    // TODO: Re-enable, causes ISR to freeze for some reason
    gc_finished_streams();
}

/// All the housekeeping tasks that must be performed to refill a buffer are called here.
fn sound_vblank_irq() {
    switch_front_buf();
    mix_buffers();
}

/// A helper function to terminate tests once all streams are inactive.
/// Not to be used outside of tests.
pub(super) fn spin_until_all_streams_inactive() {
    loop {
        if !interrupt::vblank_isr_active() {
            return;
        }
    }
}
