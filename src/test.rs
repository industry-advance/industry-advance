//! This module contains code for the custom test framework.

use crate::ewram_alloc;
use ansi_rgb::{yellow_green, Foreground};

/// Custom testing framework (the standard one can't be used because it depends on std)
pub(super) fn test_runner(tests: &[&dyn Fn()]) {
    // Prepare memory allocator for tests that require dynamic allocation
    unsafe {
        ewram_alloc::create_new_block(ewram_alloc::EWRAM_BASE, ewram_alloc::EWRAM_SIZE);
    }
    gba::info!("[TEST RUNNER] Running {} tests", tests.len());
    // Actually run tests
    for test in tests {
        test();
    }
    gba::info!("[TEST RUNNER] All tests done");

    // Because mGBA has no feature to terminate emulation from within the game with a successful
    // exit code, we have to use a hack here.
    // We panic with a "magic string", and an external process looks for this string and exits with a
    // successful exit code.
    // Do not alter this string!
    panic!("Tests ran successfully, this panic is just here to quit mGBA");
}

/// Helper function to print test info and run it.
pub fn test(test: &dyn Fn(), name: &str, description: &str) {
    gba::info!("[TEST] {}: {}...", name, description);
    test();
    gba::info!("{}", "ok".fg(yellow_green()));
}

/// This test doesn't actually test anything; it's just here to ensure the testing framework works
#[test_case]
fn test_test() {
    test(
        &|| assert_eq!(1, 1),
        "test_test",
        "ensure the testing framework works",
    );
}
