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
            let player = Player::init();
            player.play_raw_file("drill.wav");
        },
        "test_sound_playback",
        "ensure playing sound works",
    );
}

// TODO: Test wave playback, parsing etc
