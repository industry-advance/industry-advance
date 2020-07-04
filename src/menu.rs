//! This module contains a simple menu system for stuff like crafting, inventory etc.

use crate::shared_constants::{SCREEN_HEIGHT, SCREEN_HEIGHT_TILES, SCREEN_WIDTH};
use crate::shared_constants::{WINDOW_0_SCREENBLOCK, WINDOW_1_SCREENBLOCK};
use crate::shared_types::Background;
use crate::text::TextEngine;
use crate::{debug_log, debug_log::Subsystems};

use core::fmt::Write;

use gba::io::{display, keypad, window};

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
    bg: Background,
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
                    // Initialize window control registers with size for fullscreen
                    window::WIN1H.write(
                        window::HorizontalWindowSetting::new()
                            .with_col_start(0)
                            .with_col_end(SCREEN_WIDTH as u16),
                    );
                    window::WIN1V.write(
                        window::VerticalWindowSetting::new()
                            .with_row_start(0)
                            .with_row_end(SCREEN_HEIGHT as u16),
                    );
                }
            } else {
                num = WindowNum::Zero;
                WINDOW_0_TAKEN = true;
                // Initialize window control registers with size for fullscreen
                window::WIN0H.write(
                    window::HorizontalWindowSetting::new()
                        .with_col_start(0)
                        .with_col_end(SCREEN_WIDTH as u16),
                );
                window::WIN0V.write(
                    window::VerticalWindowSetting::new()
                        .with_row_start(0)
                        .with_row_end(SCREEN_HEIGHT as u16),
                );
            }
        }
        // Initialize a new text engine which draws to our window for ANSI art
        let text: TextEngine;
        match num {
            WindowNum::Zero => {
                text = TextEngine::with_default_font(WINDOW_0_SCREENBLOCK, Background::Two, false)
            }
            WindowNum::One => {
                text = TextEngine::with_default_font(WINDOW_1_SCREENBLOCK, Background::Two, false)
            }
        }

        return Window {
            num,
            text,
            bg: Background::Two,
        };
    }

    /// Make the window visible.
    pub fn show(&mut self) {
        match self.num {
            WindowNum::Zero => {
                // Enable window display
                let disp = display::display_control().with_win0(true);
                display::set_display_control(disp);
                // Set the window background
                window::WININ.write(window::InsideWindowSetting::new().with_win0_bg2(true));
            }
            WindowNum::One => {
                // Enable window display
                let disp = display::display_control().with_win1(true);
                display::set_display_control(disp);
                // Set the window background
                window::WININ.write(window::InsideWindowSetting::new().with_win1_bg2(true));
            }
        }
        // We have to enable it in DISPCNT as well
        self.bg.set_visible(true);
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
        // Redraws the cursor
        fn update_menu(text: &mut TextEngine, menu_entries: &[&str], cursor_pos: u8) {
            debug_log!(Subsystems::Menu, "Updating");
            // Remove the cursor from other positions
            for (i, _) in menu_entries[0..(SCREEN_HEIGHT_TILES - 1)]
                .iter()
                .enumerate()
            {
                text.put_char(' ', 0, (i + 1) as u8);
            }

            // Set the cursor
            text.put_char('>', 0, cursor_pos);
        }

        let mut last_keys = keypad::read_key_input();
        let mut keys_dirty = false;
        let mut current_selection = 0;

        writeln!(&mut self.text, "{}", title).unwrap();
        debug_log!(Subsystems::Menu, "Foo");
        // FIXME: Implement scrolling so that more stuff fits on screen
        // Until that happens, we only show as many entries as can fit
        for (i, entry) in menu_entries[0..(SCREEN_HEIGHT_TILES - 1)]
            .iter()
            .enumerate()
        {
            // Special case: Last line should not end on a newline (or it violates a text engine assertion)
            if i == SCREEN_HEIGHT_TILES - 2 {
                write!(&mut self.text, "  {}", entry).unwrap();
            } else {
                writeln!(&mut self.text, "  {}", entry).unwrap();
            }
        }
        update_menu(&mut self.text, menu_entries, 1);

        loop {
            // Read the current state of the keypad
            let keys = keypad::read_key_input();
            let delta = keys.difference(last_keys);
            last_keys = keys;
            if delta.up() && !keys_dirty && keys.up() {
                keys_dirty = true;
                // Move cursor up
                // Unless we're at the very top already
                if current_selection == 0 {
                    current_selection = SCREEN_HEIGHT_TILES - 2;
                } else {
                    current_selection -= 1;
                }
            } else if delta.down() && !keys_dirty && keys.down() {
                keys_dirty = true;
                // Move cursor down
                // Unless we're at the very bottom
                if current_selection == SCREEN_HEIGHT_TILES - 2 {
                    current_selection = 0;
                    debug_log!(Subsystems::Menu, "Moving down");
                } else {
                    current_selection += 1;
                }
            } else if delta.a() || delta.start() {
                // Confirm selection
                return current_selection;
            }

            // Draw menu with updated cursor position
            if keys_dirty {
                update_menu(&mut self.text, menu_entries, (current_selection + 1) as u8);
                keys_dirty = false;
            }
        }
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
