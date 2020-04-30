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

#[macro_use]
extern crate alloc;

use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;

// TODO: Consider moving this stuff into gba crate (there's an open issue for that)
pub(crate) const EWRAM_BASE: usize = 0x200_0000;
pub(crate) const EWRAM_END: usize = 0x203_FFFF;
pub(crate) const EWRAM_SIZE: usize = EWRAM_END - EWRAM_BASE;

#[cfg(test)]
use ansi_rgb::green;

use ansi_rgb::{red, Foreground};
use gba::mgba::{MGBADebug, MGBADebugLevel};
use gbfs_rs::GBFSFilesystem;

#[macro_use]
extern crate arrayref;

mod background;
mod components;
mod entities;
mod ewram_alloc;
mod game;
mod map;
mod sprite;
mod systems;

use core::fmt::Write;
use core::str::FromStr;

use game::Game;

// Filesystem containing assets
const FS_DATA: &'static [u8] = include_bytes!("../assets.gbfs");
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

    gba::info!("Starting game!");

    let mut game = Game::init();
    // Start game loop
    game.run();
    // Don't return
    gba::debug!("Done running game loop, looping forever");
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
    gba::info!("Running {} tests", tests.len());
    // Actually run tests
    for test in tests {
        test();
    }
    gba::info!(writer, "{}", "[ALL TESTS DONE]".fg(green()));

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

#[test_case]
fn test_allocator() {
    // Perform some small allocations and ensure that what we expect was allocated
    gba::debug!("Allocating box 1");
    let test_box: Box<u32> = Box::new(3);
    assert_eq!(*test_box, 3);
    gba::debug!("Finished allocating box 1");

    gba::debug!("Allocating box 2");
    let test_box2: Box<u32> = Box::new(5);
    assert_eq!(*test_box2, 5);
    gba::debug!("Finished allocating box 2");

    gba::debug!("Allocating string 1");
    let str = String::from("FOOFOOOFOOOFOFOFOOFOFOF");
    assert_eq!(str.as_str(), "FOOFOOOFOOOFOFOFOOFOFOF");
    gba::debug!("Finished allocating string 1");

    gba::debug!("Alloc tests passed!");
}

#[test_case]
fn allocator_stress_test() {
    // Perform an allocator "stress test" by continuously allocating and dropping large data structures.
    let mut size_bytes: usize = 100;
    let num_objects_per_round = 10;
    for _s in 0..3 {
        gba::info!("[XXXXXXXXXXXXXXXXXXXX] Allocator stress test");
        let mut all_boxes: Vec<Box<[u8]>> = Vec::new();
        for _i in 0..num_objects_per_round {
            // Hack to ensure we don't blow our stack by not first writing to the stack and then copying to the heap
            let test_vec: Box<[u8]> = vec![0xFF; size_bytes].into_boxed_slice();
            all_boxes.push(test_vec);
        }
        gba::info!(
            "[XXXXXXXXXXXXXXXX] Survived allocation of {} byte objects for {} times",
            size_bytes,
            num_objects_per_round
        );
        size_bytes *= 10;
    }
}
