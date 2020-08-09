//! This module enables creation of windows for lists, menus etc.

use crate::components::InventoryComponent;
use crate::item::{Item, ITEM_SPRITE_SIZE};
use crate::shared_constants::{SCREEN_HEIGHT, SCREEN_HEIGHT_TILES, SCREEN_WIDTH};
use crate::shared_constants::{WINDOW_0_SCREENBLOCK, WINDOW_1_SCREENBLOCK};
use crate::shared_types::Background;
use crate::sprite::{HWSpriteAllocator, HWSpriteHandle, HWSpriteSize};
use crate::text::{TextEngine, CHARA_SIZE_IN_PX};
use crate::{debug_log, debug_log::Subsystems};

use alloc::string::String;
use alloc::vec::Vec;
use core::convert::AsRef;
use core::fmt::Write;

use gba::io::{display, keypad, window};

// TODO: Consider lazy_static-ing these or so to get rid of the unsafe blocks
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
        self.bg.set_visible(false);
    }

    /// Make the window display sprites.
    fn enable_sprites(&self) {
        match self.num {
            WindowNum::Zero => {
                window::WININ.write(window::WININ.read().with_win0_obj(true));
            }
            WindowNum::One => {
                window::WININ.write(window::WININ.read().with_win1_obj(true));
            }
        }
    }

    /// Make the window not display sprites.
    fn disable_sprites(&self) {
        match self.num {
            WindowNum::Zero => {
                window::WININ.write(window::WININ.read().with_win0_obj(false));
            }
            WindowNum::One => {
                window::WININ.write(window::WININ.read().with_win1_obj(false));
            }
        }
    }

    /// Display the contents of the given Inventory in the window.
    pub fn make_inventory_listing(
        &mut self,
        inv: &InventoryComponent,
        sprite_alloc: &mut HWSpriteAllocator,
    ) {
        let items: Vec<(&Item, &usize)> = inv.peek().iter().collect();
        // TODO: Sort the item list alphabetically based on item name (blocked on lexical-sort no_std support)
        let entries: Vec<(String, &str, HWSpriteSize)> = items
            .iter()
            .map(|item| {
                (
                    format!("{}: {}", item.0, item.1),
                    item.0.to_sprite_name(),
                    ITEM_SPRITE_SIZE,
                )
            })
            .collect();
        self.make_sprite_list("Inventory", entries.as_slice(), sprite_alloc);
    }
    /// Create a list of text and sprites to the right of the text.
    /// This will block until the player presses "A" or "Start".
    /// Note that all other sprites will be invisible while the list is open.
    /// The entries are given in a (description, sprite filename, sprite size) form.
    /// Internally, the sprites are allocated and disposed of once the window is closed.
    pub fn make_sprite_list<S: AsRef<str>, T: AsRef<str>, U: AsRef<str>>(
        &mut self,
        title: S,
        entries: &[(T, U, HWSpriteSize)],
        sprite_alloc: &mut HWSpriteAllocator,
    ) {
        self.text.clear();
        // TODO: Scrolling
        // Write title (TODO: Center)
        writeln!(&mut self.text, "{}", title.as_ref()).unwrap();
        self.enable_sprites();
        // Hide all the other sprites
        sprite_alloc.hide_sprites_push();
        let mut sprite_handles: Vec<HWSpriteHandle> = Vec::new();
        // Now we plot all sprites and their descriptions
        for (desc, sprite, size) in entries {
            // Sprite to the left, text to the right
            let (cursor_x, cursor_y) = self.text.get_cursor_pos();

            let sprite_handle = sprite_alloc
                .alloc_from_fs_file(sprite.as_ref(), *size)
                .unwrap();
            // Put the sprite as the first element on a new line
            sprite_handle.set_x_pos(0);
            sprite_handle.set_y_pos((cursor_y) as u16);
            sprite_handle.set_visibility(true);
            sprite_handles.push(sprite_handle);

            // Followed by the description text
            let (sprite_x_size, sprite_y_size) = size.to_size_in_px();
            self.text.set_cursor_pos(
                (cursor_x + sprite_x_size as u8) / CHARA_SIZE_IN_PX,
                cursor_y / CHARA_SIZE_IN_PX,
            );
            write!(&mut self.text, "{}", desc.as_ref()).unwrap();

            // Move the cursor onto the next line not used by the sprite
            self.text.set_cursor_pos(
                0,
                (sprite_y_size as u8 / CHARA_SIZE_IN_PX) + (cursor_y / CHARA_SIZE_IN_PX),
            );
        }

        // Wait for player to press "A" or "Start"
        debug_log!(Subsystems::Menu, "Waiting for player to dismiss list");
        loop {
            let keys = keypad::read_key_input();
            if keys.a() || keys.start() {
                break;
            }
        }

        // Cleanup
        debug_log!(Subsystems::Menu, "List dismissed, cleaning up");
        self.disable_sprites();
        for handle in sprite_handles {
            sprite_alloc.free(handle);
        }
        sprite_alloc.show_sprites_pop().unwrap(); // Restore all the other sprite's visibility
    }

    /// Create a text-based menu, and return the index of the choice the player picked
    pub fn make_text_menu(&mut self, title: &str, menu_entries: &[&str]) -> usize {
        // Redraws the cursor
        fn update_menu(text: &mut TextEngine, menu_entries: &[&str], cursor_pos: u8) {
            debug_log!(Subsystems::Menu, "Updating text-based menu");
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
        self.text.clear();
        /* Modifying a static mut is unsafe.
        However, we can do that here because this function isn't called from an interrupt handler,
        so it's not reentrant and due to the single-core CPU and no threads a drop call must always complete before the next one starts. */
        unsafe {
            use WindowNum::*;
            match &self.num {
                Zero => WINDOW_0_TAKEN = false,
                One => WINDOW_1_TAKEN = false,
            }
        }
    }
}
