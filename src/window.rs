//! This module enables creation of windows for lists, menus etc.

use crate::components::InventoryComponent;
use crate::item::{Item, ITEM_SPRITE_SIZE};
use crate::shared_constants::{SCREEN_HEIGHT_TILES, SCREEN_WIDTH_TILES};
use crate::shared_constants::{WINDOW_0_SCREENBLOCK, WINDOW_1_SCREENBLOCK};
use crate::shared_types::Background;
use crate::sprite::{HWSpriteAllocator, HWSpriteHandle, HWSpriteSize};
use crate::text::{TextArea, TextEngine, TextEngineError, CHARA_SIZE_IN_PX};
use crate::{debug_log, debug_log::Subsystems};

use alloc::string::String;
use alloc::vec::Vec;
use core::cmp;
use core::convert::AsRef;
use core::fmt;
use core::fmt::Write;

use gba::mmio_addresses::*;
use gba::mmio_types::*;

use spinning_top::{const_spinlock, Spinlock};

static WINDOW_0_TAKEN: Spinlock<bool> = const_spinlock(false);
static WINDOW_1_TAKEN: Spinlock<bool> = const_spinlock(false);

/// Which window this is
enum WindowNum {
    Zero,
    One,
}

/// A description of the size and positioning of the window on-screen, in tiles.
#[derive(Debug, Copy, Clone)]
pub struct WindowArea {
    tl_x: u8,
    tl_y: u8,
    br_x: u8,
    br_y: u8,
}

impl WindowArea {
    /// Dimensions for a window which takes up the entire screen.
    pub const FULLSCREEN: WindowArea = WindowArea {
        tl_x: 0,
        tl_y: 0,
        br_x: SCREEN_WIDTH_TILES as u8,
        br_y: SCREEN_HEIGHT_TILES as u8,
    };

    /// Creates a new `WindowArea` object.
    ///
    /// Returns `None` if
    /// * the dimensions would be larger than the screen
    /// * the bottom-left corner would be above / to the right of the top-right corner
    /// * the window would have an area of 0
    pub const fn new(tl_x: u8, tl_y: u8, br_x: u8, br_y: u8) -> Option<WindowArea> {
        // Screen boundaries and positive area
        if (br_x >= tl_x)
            || (br_y >= tl_y)
            || (br_x - tl_x) > (SCREEN_WIDTH_TILES as u8)
            || (br_y - tl_y) > (SCREEN_HEIGHT_TILES as u8)
        {
            return None;
        }

        return Some(WindowArea {
            tl_x,
            tl_y,
            br_x,
            br_y,
        });
    }
}

impl Into<TextArea> for WindowArea {
    fn into(self) -> TextArea {
        return TextArea::new(self.tl_x, self.tl_y, self.br_x, self.br_y).unwrap();
    }
}

#[derive(Debug, Copy, Clone)]
pub enum WindowError {
    TooManyWindowsActive,
    TextEngine(TextEngineError),
}

impl fmt::Display for WindowError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WindowError::TooManyWindowsActive => write!(
                f,
                "There are already 2 active windows. This is a hardware limitation."
            ),
            WindowError::TextEngine(err) => write!(f, "{}", err),
        }
    }
}

impl From<TextEngineError> for WindowError {
    fn from(error: TextEngineError) -> Self {
        WindowError::TextEngine(error)
    }
}

/// A GBA hardware window.
pub struct Window {
    num: WindowNum,
    text: TextEngine,
    bg: Background,
    area: WindowArea,
}

impl Window {
    /// Create a new window, but don't display it yet.
    ///
    /// Due to hardware limitations, no more than 2 windows may be active at one time.
    /// Attempting to create more will cause an Error to be returned.
    pub fn new(area: WindowArea, bg: Background) -> Result<Window, WindowError> {
        let num: WindowNum;
        let mut window_0_taken = WINDOW_0_TAKEN.lock();
        let mut window_1_taken = WINDOW_1_TAKEN.lock();
        if *window_0_taken {
            if *window_1_taken {
                return Err(WindowError::TooManyWindowsActive);
            } else {
                num = WindowNum::One;
                *window_1_taken = true;
                // Initialize window control registers with size for fullscreen
                WIN1H_LEFT.write(area.tl_x);
                WIN1V_TOP.write(area.tl_y);
                WIN1H_RIGHT.write(area.br_x);
                WIN1V_BOTTOM.write(area.br_y);
            }
        } else {
            num = WindowNum::Zero;
            *window_0_taken = true;
            // Initialize window control registers with size for fullscreen
            WIN0H_LEFT.write(area.tl_x);
            WIN0V_TOP.write(area.tl_y);
            WIN0H_RIGHT.write(area.br_x);
            WIN0V_BOTTOM.write(area.br_y);
        }

        // Initialize a new text engine which draws to our window
        let text: TextEngine;
        match num {
            WindowNum::Zero => {
                text = TextEngine::init_from_gbfs(
                    WINDOW_0_SCREENBLOCK as u8,
                    crate::shared_constants::TEXT_CHARBLOCK as u8,
                    bg.clone(),
                    crate::shared_constants::TEXT_BG_PALETTE_START as u16,
                    area.into(),
                )?;
            }
            WindowNum::One => {
                text = TextEngine::init_from_gbfs(
                    WINDOW_1_SCREENBLOCK as u8,
                    crate::shared_constants::TEXT_CHARBLOCK as u8,
                    bg.clone(),
                    crate::shared_constants::TEXT_BG_PALETTE_START as u16,
                    area.into(),
                )?;
            }
        }

        return Ok(Window {
            num,
            text,
            bg,
            area,
        });
    }

    /// Make the window visible.
    pub fn show(&mut self) {
        // FIXME: Respect passed-in background
        match self.num {
            WindowNum::Zero => {
                // Enable window display
                DISPCNT.apply(|x| x.set_display_win0(true));
                // Set the window background
                WIN_IN_0.write(WindowEnable::new().with_bg2(true));
            }
            WindowNum::One => {
                DISPCNT.apply(|x| x.set_display_win1(true));
                WIN_IN_1.write(WindowEnable::new().with_bg2(true));
            }
        }
        // We have to enable it in DISPCNT as well
        self.bg.set_visible(true);
    }

    /// Make the window invisible.
    pub fn hide(&mut self) {
        match self.num {
            WindowNum::Zero => DISPCNT.apply(|x| x.set_display_win0(false)),
            WindowNum::One => DISPCNT.apply(|x| x.set_display_win1(false)),
        }
        self.bg.set_visible(false);
    }

    /// Make the window display sprites.
    fn enable_sprites(&self) {
        match self.num {
            WindowNum::Zero => WIN_IN_0.apply(|x| x.set_obj(true)),
            WindowNum::One => WIN_IN_1.apply(|x| x.set_obj(true)),
        }
    }

    /// Make the window not display sprites.
    fn disable_sprites(&self) {
        match self.num {
            WindowNum::Zero => WIN_IN_0.apply(|x| x.set_obj(false)),
            WindowNum::One => WIN_IN_1.apply(|x| x.set_obj(false)),
        }
    }

    /// Display the contents of the given Inventory in the window.
    /// TODO: Factor out someplace where it belongs more
    pub fn make_inventory_listing(
        &mut self,
        inv: &InventoryComponent,
        sprite_alloc: &mut HWSpriteAllocator,
    ) -> Result<(), WindowError> {
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
        self.make_sprite_list("Inventory", entries.as_slice(), sprite_alloc)?;
        return Ok(());
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
    ) -> Result<(), WindowError> {
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
            )?;
            write!(&mut self.text, "{}", desc.as_ref()).unwrap();

            // Move the cursor onto the next line not used by the sprite
            self.text.set_cursor_pos(
                0,
                (sprite_y_size as u8 / CHARA_SIZE_IN_PX) + (cursor_y / CHARA_SIZE_IN_PX),
            )?;
        }

        // Wait for player to press "A" or "Start"
        debug_log!(Subsystems::Menu, "Waiting for player to dismiss list");
        loop {
            let keys: Keys = KEYINPUT.read().into();
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

        return Ok(());
    }

    /// Create a text-based menu, and return the index of the choice the player picked
    pub fn make_text_menu(
        &mut self,
        title: &str,
        menu_entries: &[&str],
    ) -> Result<usize, WindowError> {
        // Redraws the cursor
        let num_lines: usize = (self.area.br_x - self.area.tl_x) as usize;
        fn update_menu(
            text: &mut TextEngine,
            menu_entries: &[&str],
            cursor_pos: u8,
            num_lines: usize,
        ) -> Result<(), WindowError> {
            debug_log!(Subsystems::Menu, "Updating text-based menu");
            // Remove the cursor from other positions
            for (i, _) in menu_entries[0..(num_lines - 1)].iter().enumerate() {
                text.put_char(' ', 0, (i + 1) as u8)?;
            }

            // Set the cursor
            text.put_char('>', 0, cursor_pos)?;
            return Ok(());
        }

        // Calculate the difference between to key states.
        // Keys that have changed state are marked as active (even if the change was from active to inactive).
        fn compute_key_delta(new: Keys, old: Keys) -> Keys {
            if new == old {
                return Keys::new();
            }

            let mut delta = Keys::new();
            if new.a() != old.a() {
                delta.set_a(true);
            }
            if new.b() != old.b() {
                delta.set_b(true);
            }
            if new.select() != old.select() {
                delta.set_select(true);
            }
            if new.start() != old.start() {
                delta.set_select(true);
            }
            if new.right() != old.right() {
                delta.set_right(true);
            }
            if new.left() != old.left() {
                delta.set_left(true);
            }
            if new.up() != old.up() {
                delta.set_up(true);
            }
            if new.down() != old.down() {
                delta.set_down(true);
            }
            if new.l() != old.l() {
                delta.set_l(true);
            }
            if new.r() != old.r() {
                delta.set_r(true);
            }
            return delta;
        }

        writeln!(&mut self.text, "{}", title).unwrap();
        // FIXME: Implement scrolling so that more stuff fits on screen
        // Until that happens, we only show as many entries as can fit
        let num_lines: usize = cmp::min(
            (self.area.br_x - self.area.tl_x) as usize,
            menu_entries.len(),
        );
        for (i, entry) in menu_entries[0..(num_lines - 1)].iter().enumerate() {
            // Special case: Last line should not end on a newline (or it violates a text engine assertion)
            if i == num_lines - 2 {
                write!(&mut self.text, "  {}", entry).unwrap();
            } else {
                writeln!(&mut self.text, "  {}", entry).unwrap();
            }
        }
        update_menu(&mut self.text, menu_entries, 1, num_lines)?;

        // Read key state and scroll menu appropriately
        let mut old_keys: Keys = KEYINPUT.read().into();
        let mut keys_dirty = false;
        let mut current_selection_idx = 0;
        loop {
            // Read the current state of the keypad
            let new_keys: Keys = KEYINPUT.read().into();
            let delta: Keys = compute_key_delta(new_keys, old_keys);
            old_keys = new_keys;
            if delta.up() && !keys_dirty && new_keys.up() {
                keys_dirty = true;
                // Move cursor up
                // Unless we're at the very top already
                if current_selection_idx == 0 {
                    current_selection_idx = num_lines - 2;
                } else {
                    current_selection_idx -= 1;
                }
            } else if delta.down() && !keys_dirty && new_keys.down() {
                keys_dirty = true;
                // Move cursor down
                // Unless we're at the very bottom
                if current_selection_idx == num_lines - 2 {
                    current_selection_idx = 0;
                } else {
                    current_selection_idx += 1;
                }
            } else if delta.a() || delta.start() {
                // Confirm selection
                return Ok(current_selection_idx);
            }

            // Draw menu with updated cursor position
            if keys_dirty {
                update_menu(
                    &mut self.text,
                    menu_entries,
                    (current_selection_idx + 1) as u8,
                    num_lines,
                )?;
                keys_dirty = false;
            }
        }
    }
}

impl Drop for Window {
    fn drop(&mut self) {
        self.hide();
        self.text.clear();
        let mut window_0_taken = WINDOW_0_TAKEN.lock();
        let mut window_1_taken = WINDOW_1_TAKEN.lock();
        use WindowNum::*;
        match &self.num {
            Zero => *window_0_taken = false,
            One => *window_1_taken = false,
        }
    }
}
