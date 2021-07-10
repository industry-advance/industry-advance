// TODO: This should probably be upstreamed into the gba crate

#[cfg(test)]
use crate::test::test;

use core::intrinsics;
use gba::mmio_addresses::IME;

// Helper function to disable and enable interrupts here so those operations are truly atomic
unsafe fn with_disabled_external_interrupts<F, R>(func: F) -> R
where
    F: FnOnce() -> R,
{
    IME.write(true);
    let result = func();
    IME.write(false);
    return result;
}

#[no_mangle]
pub unsafe extern "C" fn __sync_lock_test_and_set_1(p: *mut u8, val: u8) -> u8 {
    with_disabled_external_interrupts(move || {
        let result = *p;
        *p = val;
        intrinsics::atomic_singlethreadfence_acq();
        result
    })
}

#[no_mangle]
pub unsafe extern "C" fn __sync_lock_test_and_set_4(p: *mut u32, val: u32) -> u32 {
    with_disabled_external_interrupts(move || {
        let result = *p;
        *p = val;
        intrinsics::atomic_singlethreadfence_acq();
        result
    })
}

#[no_mangle]
pub unsafe extern "C" fn __sync_val_compare_and_swap_1(p: *mut u8, old: u8, new: u8) -> u8 {
    use core::ptr;

    with_disabled_external_interrupts(move || {
        let cur = ptr::read(p);
        if cur == old {
            ptr::write(p, new);
        }
        cur
    })
}

#[no_mangle]
pub unsafe extern "C" fn __sync_val_compare_and_swap_2(p: &mut u16, old: u16, new: u16) -> u16 {
    use core::ptr;

    with_disabled_external_interrupts(move || {
        let cur = ptr::read(p);
        if cur == old {
            ptr::write(p, new);
        }
        cur
    })
}

#[no_mangle]
pub unsafe extern "C" fn __sync_val_compare_and_swap_4(p: &mut u32, old: u32, new: u32) -> u32 {
    use core::ptr;

    with_disabled_external_interrupts(move || {
        let cur = ptr::read(p);
        if cur == old {
            ptr::write(p, new);
        }
        cur
    })
}

/// This test locks and unlocks a mutex, just to make sure that stuff builds.
#[test_case]
fn test_atomics_mutex() {
    test(
        &|| {
            use alloc::string::String;
            use spinning_top::Spinlock;
            let data = String::from("Hello");
            let spinlock = Spinlock::new(data);
            // Lock the spinlock to get a mutex guard for the data
            let mut locked_data = spinlock.lock();
            // The guard implements the `Deref` trait, so we can use it like a `&String`
            assert_eq!(locked_data.as_str(), "Hello");
            // It also implements `DerefMut` so mutation is possible too. This is safe
            // because the spinlock ensures mutual exclusion
            locked_data.make_ascii_uppercase();
            assert_eq!(locked_data.as_str(), "HELLO");
        },
        "test_atomics_mutex",
        "ensure that we can at least use a mutex",
    );
}
