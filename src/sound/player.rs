use super::wave::*;
use crate::FS;

use gba::io::{dma::*, sound, timers};
use spinning_top::{const_spinlock, Spinlock};

static PLAYER_EXISTS: Spinlock<bool> = const_spinlock(false);

// TODO: Make variable/read from wav
const SOUND_SAMPLE_RATE: u32 = 8000;
const CPU_CYCLES_PER_SEC: u32 = 16777216;
const TIMER_MAX_VALUE: u32 = u16::MAX as u32;

/// A mono sound player for direct sound channel A.
pub struct Player {}

impl Player {
    /// Initializes the audio player.
    /// As long as there's a `Player` instance active, you shouldn't touch DMA 1 or timer 0.
    /// Also, only a single `Player` instance can be active at a time,
    /// trying to create more will lead to a panic by the constructor.
    pub fn init() -> Player {
        let mut exists = PLAYER_EXISTS.lock();
        if *exists {
            panic!("A sound player already exists. Drop it first before creating a new one.");
        }
        *exists = true;

        return Player {};
    }

    /// Play the given raw PCM samples on channel A.
    ///
    /// File must be 8000Hz, 8 bit signed.
    ///
    /// Note that this function does not block until playback finishes, it returns as soon as the HW is configured.
    fn play_raw(&self, samples: &[u32], sample_rate: u32) {
        // Master sound enable: has to be set before any of the registers are usable
        sound::SOUNDCNT_X
            .write(sound::SoundMasterSetting::new().with_psg_fifo_master_enabled(true));

        // Configure sound timer initial value such that it overflows exactly when a sample is about to run out
        timers::TM0CNT_L.write((TIMER_MAX_VALUE - (CPU_CYCLES_PER_SEC / SOUND_SAMPLE_RATE)) as u16);
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
                .with_dma_sound_a_enable_right(true)
                .with_dma_sound_a_enable_left(true)
                // Sample each time timer 0 (rather than 1) runs out
                .with_dma_sound_a_timer_select(false)
                // Ensure the FIFO is prepared
                .with_dma_sound_a_reset_fifo(true),
        );

        // Configure DMA 1 to continuously transfer samples from the buffer
        unsafe {
            DMA1::set_source(samples.as_ptr() as *const u32);
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

        // TODO: Use timer 1 to stop playback when sound is over
    }

    /// Play the given rav file on channel A.
    ///
    /// File must be 8000Hz, 8 bit signed PCM.
    ///
    /// Note that this function does not block until playback finishes, it returns as soon as the HW is configured.
    ///
    /// This function currently panics if the file can't be opened,
    /// TODO: proper error handling.
    pub fn play_raw_file(&self, name: &str) {
        let data = FS.get_file_data_by_name_as_u32_slice(name).unwrap();
        self.play_raw(data, SOUND_SAMPLE_RATE);
    }

    /// Play the given wav file on channel A.
    ///
    /// File must be 8000Hz, 8 bit signed PCM.
    ///
    /// Note that this function does not block until playback finishes, it returns as soon as the HW is configured.
    pub fn play_wav_file(&self, name: &str) -> Result<(), WaveError> {
        // Configure DMA 1 to continuously transfer samples from the file
        let wav = Wave::from_file(name)?;
        self.play_raw(wav.audio, wav.sample_rate);
        return Ok(());
    }

    // TODO: Mixer
}

impl Drop for Player {
    fn drop(&mut self) {
        let mut exists = PLAYER_EXISTS.lock();
        *exists = false;
    }
}
