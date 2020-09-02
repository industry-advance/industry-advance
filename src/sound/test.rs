use super::*;
use crate::test::test;

#[test_case]
fn test_sound_player_init() {
    test(
        &|| {
            let _player = Player::init();
        },
        "test_sound_player_init",
        "ensure initializing the sound player works",
    );
}

#[test_case]
fn test_sound_playback() {
    test(
        &|| {
            crate::interrupt::init();
            let player = Player::init();
            player.play_raw_file("drill.wav").unwrap();
            player.spin_until_playback_completes();
        },
        "test_sound_playback",
        "ensure playing sound works",
    );
}

#[test_case]
fn test_wave_playback() {
    test(
        &|| {
            crate::interrupt::init();
            let player = Player::init();
            player.play_wav_file("drill.wav").unwrap();
            player.spin_until_playback_completes();
        },
        "test_wave_playback",
        "ensure playing a wave file works",
    );
}
