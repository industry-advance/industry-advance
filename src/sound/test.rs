use super::*;
use crate::test::test;

#[test_case]
fn test_sound_mixer_init() {
    test(
        &|| {
            mixer::init();
        },
        "test_sound_mixer_init",
        "ensure initializing the sound mixer works",
    );
}
#[test_case]
fn test_raw_sound_playback() {
    test(
        &|| {
            crate::interrupt::init();
            mixer::init();
            mixer::add_raw_file_stream("drill.wav").unwrap();
            mixer::spin_until_all_streams_inactive();
        },
        "test_raw_sound_playback",
        "ensure playing sound works",
    );
}

#[test_case]
fn test_wav_sound_playback() {
    test(
        &|| {
            crate::interrupt::init();
            mixer::init();
            mixer::add_wave_file_stream("drill.wav").unwrap();
            mixer::spin_until_all_streams_inactive();
        },
        "test_wav_sound_playback",
        "ensure playing a wave file works",
    );
}

// TODO: Test mixing multiple streams
