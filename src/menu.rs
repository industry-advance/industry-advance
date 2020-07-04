//! This module contains a simple menu system for stuff like crafting, inventory etc.

use crate::shared_constants::SCREEN_HEIGHT_TILES;
use crate::shared_constants::{WINDOW_0_SCREENBLOCK, WINDOW_1_SCREENBLOCK};
use crate::text::TextEngine;

use core::fmt::Write;

use gba::io::display;
use gba::io::window;

static mut WINDOW_0_TAKEN: bool = false;
static mut WINDOW_1_TAKEN: bool = false;
/// Which window this is
enum WindowNum {
    Zero,
    One,
}
/// A window drawn entirely by using ANSI art via the `TextEngine`.
pub struct Window {
    num: WindowNum,
    text: TextEngine,
}

impl Window {
    /// Create a new window, but don't display it yet.
    /// No more than two instances may be live at once.
    /// Attempting to create any more will cause a panic.
    pub fn new() -> Window {
        let num: WindowNum;

        /* Accessing a static mut is unsafe.
        However, we can do that here because this function isn't called from an interrupt handler,
        so it's not reentrant and due to the single-core CPU a new() call must always complete before the next one starts. */
        unsafe {
            if WINDOW_0_TAKEN {
                if WINDOW_1_TAKEN {
                    panic!("Cannot create window because 2 already exist. Drop one first.");
                } else {
                    num = WindowNum::One;
                    WINDOW_1_TAKEN = true;
                }
            } else {
                num = WindowNum::Zero;
                WINDOW_0_TAKEN = true;
            }
        }
        // Initialize a new text engine which draws to our window for ANSI art
        let text: TextEngine;
        match num {
            WindowNum::Zero => text = TextEngine::with_default_font(WINDOW_0_SCREENBLOCK),
            WindowNum::One => text = TextEngine::with_default_font(WINDOW_1_SCREENBLOCK),
        }

        // TODO: Draw window border

        return Window { num, text };
    }

    /// Make the window visible.
    pub fn show(&mut self) {
        match self.num {
            WindowNum::Zero => {
                let disp = display::display_control().with_win0(true);
                display::set_display_control(disp);
            }
            WindowNum::One => {
                let disp = display::display_control().with_win1(true);
                display::set_display_control(disp);
            }
        }
    }

    /// Make the window invisible.
    pub fn hide(&mut self) {
        match self.num {
            WindowNum::Zero => {
                let disp = display::display_control().with_win0(false);
                display::set_display_control(disp);
            }
            WindowNum::One => {
                let disp = display::display_control().with_win1(false);
                display::set_display_control(disp);
            }
        }
    }

    /// Create a text-based menu, and return the index of the choice the player picked
    pub fn make_menu(&mut self, title: &str, menu_entries: &[&str]) -> usize {
        writeln!(&mut self.text, "{}", title).unwrap();
        // Caret starts on 1st entry
        writeln!(&mut self.text, "> {}", menu_entries[0]).unwrap();
        // FIXME: Implement scrolling so that more stuff fits on screen
        // Until that happens, we only show as many entries as can fit
        for entry in menu_entries[1..(SCREEN_HEIGHT_TILES - 2)].iter() {
            writeln!(&mut self.text, "  {}", entry).unwrap();
        }

        // TODO: Process user choice
        loop {}
        // FIXME: Correct
        return 0;
    }
}

impl Drop for Window {
    fn drop(&mut self) {
        self.hide();
        /* Modifying a static mut is unsafe.
        However, we can do that here because this function isn't called from an interrupt handler,
        so it's not reentrant and due to the single-core CPU a drop call must always complete before the next one starts. */
        unsafe {
            use WindowNum::*;
            match &self.num {
                Zero => WINDOW_0_TAKEN = false,
                One => WINDOW_1_TAKEN = false,
            }
        }
    }
}
