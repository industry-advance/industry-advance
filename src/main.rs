#![no_std]
#![feature(start)]

mod game;
use game::Game;

use gba::{
    fatal,
    io::{
        display::{DisplayControlSetting, DisplayMode, DISPCNT, VBLANK_SCANLINE, VCOUNT},
        keypad, *,
    },
    palram::index_palram_bg_4bpp,
    vram::{get_4bpp_character_block, CHAR_BASE_BLOCKS},
    Color,
};

const BLACK: Color = Color::from_rgb(0, 0, 0);
const WHITE: Color = Color::from_rgb(31, 31, 31);

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    // This kills the emulation with a message if we're running within mGBA.
    fatal!("{}", info);
    // If we're _not_ running within mGBA then we still need to not return, so
    // loop forever doing nothing.
    loop {}
}

#[start]
fn main(_argc: isize, _argv: *const *const u8) -> isize {
    let mut game = Game::new();
    game.draw_bg(); // Draw a 1024*1024 tiled background
    loop {
        // Simulate
        game.update();
        // Wait for VBlank
        // TODO: Optimize via interrupts
        spin_until_vblank();
        // Draw changed tiles/sprites
        game.draw();
    }
}

fn spin_until_vblank() {
    while VCOUNT.read() < VBLANK_SCANLINE {}
}
