#![no_std]
#![no_main]
// Allow defining a custom entrypoint
#![feature(start)]
// Needed for the allocator
#![feature(alloc_error_handler)]
#![feature(const_fn)]
// Test harness setup
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![feature(const_in_array_repeat_expressions)]

extern crate alloc;

use ansi_rgb::{green, red, Foreground};
use gba::mgba::{MGBADebug, MGBADebugLevel};

mod assets;
mod ewram_alloc;
mod game;
mod map;
mod sprite;
mod testmap;

use core::fmt::Write;

use game::Game;
use sprite::HWSpriteAllocator;

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    // This kills the emulation with a message if we're running within mGBA.
    let mut writer = MGBADebug::new().expect("Failed to acquire MGBA debug writer");
    writeln!(writer, "{}", info.fg(red())).expect("Failed to write panic to MGBA debug register");
    writer.send(MGBADebugLevel::Fatal);
    // If we're _not_ running within mGBA then we still need to not return, so
    // loop forever doing nothing.
    // TODO: Consider implementing output over serial/graphical error message display
    loop {}
}

#[no_mangle]
#[start]
fn main(_argc: isize, _argv: *const *const u8) -> isize {
    // Run only tests if asked
    #[cfg(test)]
    test_main();
    #[cfg(test)]
    loop {}

    gba::info!("Starting game!");

    // Initialize the heap
    unsafe { ALLOCATOR.init(ewram_alloc::EWRAM_BASE, ewram_alloc::EWRAM_SIZE) };

    // Initialize hardware sprite (object) management
    let sprite_allocator = HWSpriteAllocator::new(&assets::sprites::palette::PAL);
    sprite_allocator.init();

    // Pack the various hardware component handlers into a single struct to make passing around easier
    // Start game loop
    let mut game = Game::new(sprite_allocator);
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

// Custom testing framework (the standard one can't be used because it depends on std)
#[cfg(test)]
fn test_runner(tests: &[&dyn Fn()]) {
    // Prepare memory allocator for tests that require dynamic allocation
    unsafe { ALLOCATOR.init(ewram_alloc::EWRAM_BASE, ewram_alloc::EWRAM_SIZE) };

    let mut writer = MGBADebug::new().expect("Failed to acquire MGBA debug writer");
    writeln!(writer, "Running {} tests", tests.len())
        .expect("Failed to write to MGBA debug message register");
    writer.send(MGBADebugLevel::Info);
    for test in tests {
        test();
    }
    writeln!(writer, "{}", "[ALL TESTS DONE]".fg(green()))
        .expect("Failed to write to MGBA debug message register");
    writer.send(MGBADebugLevel::Info);

    // Because mGBA has no feature to terminate emulation from within the game with a successful
    // exit code, we have to use a hack here.
    // We panic with a "magic string", and an external process looks for this string and exits with a
    // successful exit code.
    // Do not alter this string!
    panic!("Tests ran successfully, this panic is just here to quit mGBA");
}

// This test doesn't actually test anything; it's just here to ensure the testing framework works
#[test_case]
fn should_always_pass() {
    let mut writer = MGBADebug::new().expect("Failed to acquire MGBA debug writer");
    writeln!(writer, "This test should always pass...")
        .expect("Failed to write to MGBA debug message register");
    writer.send(MGBADebugLevel::Info);
    assert_eq!(1, 1);
    writeln!(writer, "{}", "[ok]".fg(green()))
        .expect("Failed to write to MGBA debug message register");
    writer.send(MGBADebugLevel::Info);
}
