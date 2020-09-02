#![no_std]
#![no_main]
// Allow defining a custom entrypoint
#![feature(start)]
// Needed for the allocator
#![feature(alloc_error_handler)]
#![feature(const_fn)]
// Needed to deal with errors when compiling the FS
#![feature(const_panic)]
// Needed to implement emulation of atomics
#![feature(core_intrinsics)]
// Test harness setup
#![feature(custom_test_frameworks)]
#![test_runner(crate::test::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![feature(const_in_array_repeat_expressions)]
// Nice-to-have features
#![feature(try_trait)]
// Disable a bunch of clippy lints I disagree with
#![allow(clippy::needless_return)]
#![allow(clippy::module_inception)]

#[macro_use]
extern crate alloc;

use ansi_rgb::{red, Foreground};
use gba::mgba::{MGBADebug, MGBADebugLevel};
use gbfs_rs::GBFSFilesystem;

extern crate arrayref;

mod components;
#[cfg(test)]
mod test;
#[macro_use]
mod debug_log;
mod atomics;
mod entities;
mod ewram_alloc;
mod game;
mod interrupt;
mod item;
mod map;
mod shared_constants;
mod shared_types;
mod sound;
mod sprite;
mod systems;
mod text;
mod window;

use debug_log::*;
use game::Game;

use core::fmt::Write;

// Construct the filesystem containing assets.
// This macro magic is needed to ensure alignment so that casting files to &[u32] works,
// see https://users.rust-lang.org/t/can-i-conveniently-compile-bytes-into-a-rust-program-with-a-specific-alignment/24049/2
#[macro_use]
mod macros {
    #[repr(C)] // guarantee 'bytes' comes after '_align'
    pub struct AlignedAs<Align, Bytes: ?Sized> {
        pub _align: [Align; 0],
        pub bytes: Bytes,
    }

    macro_rules! include_bytes_align_as {
        ($align_ty:ty, $path:literal) => {{
            // const block expression to encapsulate the static
            use $crate::macros::AlignedAs;

            // this assignment is made possible by CoerceUnsized
            const ALIGNED: &AlignedAs<$align_ty, [u8]> = &AlignedAs {
                _align: [],
                bytes: *include_bytes!($path),
            };

            &ALIGNED.bytes
        }};
    }
}

const FS_DATA: &'static [u8] = include_bytes_align_as!(u32, "../assets.gbfs");
const FS: GBFSFilesystem<'static> = match GBFSFilesystem::from_slice(FS_DATA) {
    Ok(val) => val,
    Err(_) => panic!("Failed to convert GBFS"),
};

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
    debug_log!(Subsystems::Main, "Starting game!");
    let mut game = Game::init();
    // Start game loop
    game.run();
    // Don't return
    debug_log!(Subsystems::Main, "Done running game loop, looping forever");
    loop {}
}

// Heap allocator config
#[global_allocator]
static ALLOCATOR: ewram_alloc::MyBigAllocator = ewram_alloc::MyBigAllocator;

#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("allocation error: {:?}", layout)
}
