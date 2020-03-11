#![no_std]
#![feature(start)]

extern crate gba;

use gba::mgba::{MGBADebug, MGBADebugLevel};

mod game;
mod map;
mod testmap;

use core::fmt::Write;

use game::Game;

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    // This kills the emulation with a message if we're running within mGBA.
    let mut fatal = MGBADebug::new().unwrap();
    writeln!(fatal, "{}", info).unwrap();
    fatal.send(MGBADebugLevel::Fatal);
    // If we're _not_ running within mGBA then we still need to not return, so
    // loop forever doing nothing.
    loop {}
}

#[start]
fn main(_argc: isize, _argv: *const *const u8) -> isize {
    gba::info!("Starting game!");
    let mut game = Game::new();
    game.run();
    loop {}
}

//#[no_mangle]
//static __IRQ_HANDLER: extern "C" fn() = irq_handler;

//extern "C" fn irq_handler() {}
