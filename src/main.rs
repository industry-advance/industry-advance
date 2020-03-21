#![no_std]
#![feature(start)]
#![feature(alloc_error_handler)]
#![feature(const_fn)]

extern crate alloc;

use gba::mgba::{MGBADebug, MGBADebugLevel};

mod ewram_alloc;
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

    // Initialize the heap
    unsafe { ALLOCATOR.init(ewram_alloc::EWRAM_BASE, ewram_alloc::EWRAM_SIZE) };

    // Start game loop
    let mut game = Game::new();
    game.run();
    loop {}
}

// Heap allocator config
#[global_allocator]
static ALLOCATOR: ewram_alloc::RaceyHeap = unsafe { ewram_alloc::RaceyHeap::empty() };

#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("allocation error: {:?}", layout)
}
