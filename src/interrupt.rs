//! This module provides an "interrupt switchboard" where other modules may
//! register their interrupt handlers.
//!
//! Please, always use the facilities provided here for interrupt management!
//! That way, conflicts can be detected early.
//!
//! Not all interrupts are yet supported. Add them as you need them.

use crate::debug_log::Subsystems::Interrupt;

use gba::mmio_addresses::*;
use gba::mmio_types::InterruptFlags;

/// Can't use a mutex here because they internally require disabling interrupts
/// to lock due to the way we've implemented atomics emulation.
///
/// This also must have a static lifetime because we don't know how long an
/// ISR will be relevant.
static mut TIMER1_HANDLER: Option<&'static dyn Fn()> = None;
static mut VBLANK_HANDLER: Option<&'static dyn Fn()> = None;

/// Whether an ISR is currently active for `timer1`.
pub fn timer1_isr_active() -> bool {
    return IE.read().timer1();
}

/// Enable receiving interrupts when `timer1` fires.
///
/// Pass `None` as the handler function to disable again.
///
/// Will panic if interrupts are disabled (meaning you didn't call `init()` first).
pub fn set_timer1_handler(f: Option<&'static dyn Fn()>) {
    if IME.read() == false {
        panic!("Enable interrupts by calling init() before registering an ISR");
    }
    unsafe {
        TIMER1_HANDLER = f;
    }

    // Enable/disable receiving timer1 interrupts
    let mut flags = IE.read();
    match f {
        Some(_) => {
            debug_log!(Interrupt, "Enabling handler for timer1");
            flags = flags.with_timer1(true);
        }
        None => {
            debug_log!(Interrupt, "Disabling handler for timer1");
            flags = flags.with_timer1(false);
        }
    }
    unsafe {
        IE.write(flags);
    }
}

/// Whether an ISR is currently active for `vblank`.
pub fn vblank_isr_active() -> bool {
    return IE.read().vblank();
}

/// Enable receiving interrupts when `vblank` occurs.
///
/// Pass `None` as the handler function to disable again.
///
/// Will panic if interrupts are disabled (meaning you didn't call `init()` first).
pub fn set_vblank_handler(f: Option<&'static dyn Fn()>) {
    if IME.read() == false {
        panic!("Enable interrupts by calling init() before registering an ISR");
    }
    unsafe {
        VBLANK_HANDLER = f;
    }

    // Enable/disable receiving vblank interrupts
    let mut flags = IE.read();
    match f {
        Some(_) => {
            debug_log!(Interrupt, "Enabling handler for vblank");
            flags = flags.with_vblank(true);
            DISPSTAT.apply(|x| x.set_is_vblank(true));
        }
        None => {
            debug_log!(Interrupt, "Disabling handler for vblank");
            flags = flags.with_vblank(false);
            DISPSTAT.apply(|x| x.set_is_vblank(false));
        }
    }
    unsafe {
        IE.write(flags);
    }
}

/// Initializes the module.
///
/// Repeated calls will have no effect.
///
/// # Safety
/// Do not call in a context where enabling interrupts is undesirable.
pub unsafe fn init() {
    if IME.read() == false {
        IME.write(true);
    }
    USER_IRQ_HANDLER.write(Some(irq_handler));
}

/// This function has to have exactly this signature so that the hardware knows what to do with it.
unsafe extern "C" fn irq_handler() {
    let flags: InterruptFlags = IRQ_PENDING.read();
    // Run the handler if they're defined
    if flags.timer1() {
        unsafe {
            match TIMER1_HANDLER {
                Some(handler) => handler(),
                None => panic!("Received timer1 interrupt even though no handler has been registered. Have you registered your handler with the interrupt module?"),
            }
        }
    }
    if flags.vblank() {
        unsafe {
            match VBLANK_HANDLER {
                Some(handler) => handler(),
                None => panic!("Received vblank interrupt even though no handler has been registered. Have you registered your handler with the interrupt module?"),
            }
        }
    }
}
