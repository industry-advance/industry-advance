//! This module provides an "interrupt switchboard" where other modules may
//! register their interrupt handlers.
//!
//! Please, always use the facilities provided here for interrupt management!
//! That way, conflicts can be detected early.
//!
//! Not all interrupts are yet supported. Add them as you need them.

use crate::debug_log::Subsystems::Interrupt;

use gba::io::irq::*;

/// Can't use a mutex here because they internally require disabling interrupts
/// to lock due to the way we've implemented atomics emulation.
///
/// This also must have a static lifetime because we don't know how long an
/// ISR will be relevant.
static mut TIMER1_HANDLER: Option<&'static dyn Fn()> = None;

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
    if IME.read() == IrqEnableSetting::IRQ_NO {
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
    IE.write(flags);
}

/// Initializes the module.
///
/// Repeated calls will have no effect.
///
/// Do not call in a context where enabling interrupts is undesirable.
pub fn init() {
    if IME.read() == IrqEnableSetting::IRQ_NO {
        IME.write(IrqEnableSetting::IRQ_YES);
    }
    set_irq_handler(irq_handler);
}

/// This function has to have exactly this signature so that the hardware knows what to do with it.
extern "C" fn irq_handler(flags: IrqFlags) {
    if flags.timer1() {
        // Run the handler if one is defined
        unsafe {
            match TIMER1_HANDLER {
                Some(handler) => handler(),
                None => panic!("Received timer1 interrupt even though no handler has been registered. Have you registered your handler with the interrupt module?"),
            }
        }
    }
}
