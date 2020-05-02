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
// Disable a bunch of clippy lints I disagree with
#![allow(clippy::needless_return)]
#![allow(clippy::module_inception)]

#[macro_use]
extern crate alloc;

#[cfg(test)]
use ansi_rgb::green;

use ansi_rgb::{red, Foreground};
use gba::mgba::{MGBADebug, MGBADebugLevel};
use gbfs_rs::GBFSFilesystem;

extern crate arrayref;

mod components;
mod entities;
mod ewram_alloc;
mod game;
mod map;
mod shared_types;
mod sprite;
mod systems;

use core::fmt::Write;

use game::Game;

// Filesystem containing assets
const FS_DATA: &[u8] = include_bytes!("../assets.gbfs");
const FS: GBFSFilesystem<'static> = GBFSFilesystem::from_slice(FS_DATA);

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
    {
        test_main();
        loop {}
    }

    // Initialize the allocator
    unsafe {
        ewram_alloc::create_new_block(ewram_alloc::EWRAM_BASE, ewram_alloc::EWRAM_SIZE);
    }

    gba::info!("[MAIN] Starting game!");

    let mut game = Game::init();
    // Start game loop
    game.run();
    // Don't return
    gba::debug!("[MAIN] Done running game loop, looping forever");
    loop {}
}

// Heap allocator config
#[global_allocator]
static ALLOCATOR: ewram_alloc::MyBigAllocator = ewram_alloc::MyBigAllocator;

#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("allocation error: {:?}", layout)
}

// Custom testing framework (the standard one can't be used because it depends on std)
#[cfg(test)]
fn test_runner(tests: &[&dyn Fn()]) {
    // Prepare memory allocator for tests that require dynamic allocation
    unsafe {
        ewram_alloc::create_new_block(ewram_alloc::EWRAM_BASE, ewram_alloc::EWRAM_SIZE);
    }
    gba::info!("[TEST RUNNER] Running {} tests", tests.len());
    // Actually run tests
    for test in tests {
        test();
    }
    gba::info!("[TEST RUNNER] {}", "ALL TESTS DONE".fg(green()));

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
