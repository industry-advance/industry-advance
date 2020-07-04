//! This module introduces a simple, fixed-width, 8x8,
//! monochrome text display system.
//!
//! Because it's implemented using a map layer, you have to keep in mind that
//! only up to 512 unique characters may be used.
//! If the font contains more, a panic will occur on load.
//! # NOTE
//! Keep the reserved resources documented by the README in mind.

use crate::shared_constants::*;
use crate::shared_types::Background;
use crate::FS;
use crate::{debug_log, Subsystems::Text};

use gba::io::background::{BGSize, BackgroundControlSetting};
use gba::io::dma::{DMAControlSetting, DMA3};
use gba::palram;
use gba::vram::text::TextScreenblockEntry;
use gba::{
    vram::{SCREEN_BASE_BLOCKS, VRAM_BASE_USIZE},
    Color,
};

use gbfs_rs::Filename;
use hashbrown::hash_map::HashMap;
use twox_hash::XxHash64;

use core::convert::TryInto;
use core::fmt;
use core::hash::BuildHasherDefault;
use core::str;

const BG_WIDTH_TILES: usize = 32;
/// Structure representing the text engine's current state.
///
/// # SAFETY
/// Only a single instance may exist. Otherwise, you'll get funky text rendering.
pub struct TextEngine {
    char_to_tile_id: HashMap<char, u16, BuildHasherDefault<XxHash64>>,
    /// X position of cursor, in tiles
    cursor_x: u8,
    /// Y position of cursor, in tiles
    cursor_y: u8,
    /// Screenblock to draw on
    screenblock: u16,
}

impl TextEngine {
    fn init(
        font_tile_filename: &str,
        font_chars_filename: &str,
        screenblock: u16,
        background: Background,
        make_visible: bool,
    ) -> TextEngine {
        let font_tile_filename = Filename::try_from_str(font_tile_filename).unwrap();
        let font_tiles = FS
            .get_file_data_by_name_as_u32_slice(font_tile_filename)
            .unwrap();

        // Create character -> tile number lookup table
        // TODO: Make this more efficient, both in terms of memory for the mapping and CPU time (maybe use some const map)
        let mut hashmap: HashMap<char, u16, BuildHasherDefault<XxHash64>> = Default::default();
        let font_chars_filename = Filename::try_from_str(font_chars_filename).unwrap();
        let font_chars = FS.get_file_data_by_name(font_chars_filename).unwrap();
        let font_chars: &str = str::from_utf8(font_chars).unwrap();
        for (i, chara) in font_chars.chars().enumerate() {
            debug_log!(Text, "Inserting char {} with tile ID {}", chara, i);
            hashmap.insert(chara, i as u16);
        }

        // Load characters into VRAM charblock
        // There are 512 4bpp tiles per charblock, each one is 32 bytes (=8 u32's) in length
        if font_tiles.len() > 512 * 8 {
            panic!(
                "Font is too large! May contain at most 512 glyphs, actually contains {}",
                font_tiles.len() / 8
            );
        }
        // DMA transfer font tiles
        unsafe {
            DMA3::set_source(font_tiles.as_ptr());
            DMA3::set_dest((VRAM_BASE_USIZE + (TEXT_CHARBLOCK * CHARBLOCK_SIZE_BYTES)) as *mut u32);
            DMA3::set_count(font_tiles.len().try_into().unwrap());
            DMA3::set_control(
                DMAControlSetting::new()
                    .with_use_32bit(true)
                    .with_enabled(true),
            );
        }

        // TODO: Load colors other than black
        let idx = palram::index_palram_bg_4bpp(0, 0);
        idx.write(Color::from_rgb(31, 31, 31));
        for i in 1..15 {
            let idx = palram::index_palram_bg_4bpp(0, i);
            idx.write(Color::from_rgb(0, 0, 0));
        }

        let mut engine = TextEngine {
            char_to_tile_id: hashmap,
            cursor_x: 0,
            cursor_y: 0,
            screenblock,
        };

        // Ensure there's no residual gunk in our screenblock
        engine.clear();

        // Set text to display on top of everything else
        background.write(
            BackgroundControlSetting::new()
                .with_bg_priority(0)
                .with_char_base_block(TEXT_CHARBLOCK as u16)
                .with_screen_base_block(screenblock)
                .with_size(BGSize::Zero)
                .with_is_8bpp(false),
        );
        background.set_visible(make_visible);

        debug_log!(Text, "Text engine init done");
        return engine;
    }
    /// Initializes a text engine with the default font from GBFS on the default text engine screenblock.
    /// The filename must be "font", and a UTF8 file "font_chars.txt" must also exist,
    /// containing all characters in order of appearance in the tile file.
    /// The file is assumed to contain the font in a 4bpp format, where each tile is exactly
    /// 1 character.
    /// No more than 512 glyphs are permitted.
    pub fn with_default_font_screenblock_and_background() -> TextEngine {
        return TextEngine::init(
            "fontTiles",
            "font_chars.txt",
            TEXT_SCREENBLOCK as u16,
            Background::Two,
            true,
        );
    }

    /// Initializes a text engine with the default font from GBFS on the given screenblock and background.
    /// The filename must be "font", and a UTF8 file "font_chars.txt" must also exist,
    /// containing all characters in order of appearance in the tile file.
    /// The file is assumed to contain the font in a 4bpp format, where each tile is exactly
    /// 1 character.
    /// No more than 512 glyphs are permitted.
    pub fn with_default_font(
        screenblock: usize,
        background: Background,
        make_visible: bool,
    ) -> TextEngine {
        return TextEngine::init(
            "fontTiles",
            "font_chars.txt",
            screenblock as u16,
            background,
            make_visible,
        );
    }

    /// Sets the X, Y onscreen position for the cursor on screen, in tiles.
    /// Value must not be greater than `SCREEN_WIDTH_TILES` and `SCREEN_HEIGHT_TILES`, respectively.
    pub fn set_cursor_pos(&mut self, x: u8, y: u8) {
        assert!(x < SCREEN_WIDTH_TILES as u8);
        assert!(y < SCREEN_HEIGHT_TILES as u8);
        self.cursor_x = x;
        self.cursor_y = y;
    }

    /// Puts selected character at current cursor position and advances it
    fn put_char_and_advance(&mut self, chara: char) {
        self.put_char(chara, self.cursor_x, self.cursor_y);
        // When line on screen is full, advance to next one
        if self.cursor_x >= (SCREEN_WIDTH_TILES - 1) as u8 {
            self.set_cursor_pos(0, self.cursor_y + 1);
        } else {
            self.set_cursor_pos(self.cursor_x + 1, self.cursor_y);
        }
        // When all lines are full, start overwriting from the top
        if self.cursor_y >= SCREEN_HEIGHT_TILES as u8 {
            self.set_cursor_pos(0, 0);
        }
    }

    /// Puts selected character at given screen position
    /// without advancing the cursor.
    pub fn put_char(&mut self, chara: char, x: u8, y: u8) {
        assert!(x < SCREEN_WIDTH_TILES as u8);
        assert!(y < SCREEN_HEIGHT_TILES as u8);
        // Look up the glyph tile ID
        let tile_id = match self.char_to_tile_id.get(&chara) {
            Some(id) => id,
            // TODO: Just print a black box instead
            None => panic!("Attempt to print nonexistent glyph"),
        };
        debug_log!(Text, "Character {} has tile ID {}", chara, *tile_id);
        let glyph = TextScreenblockEntry::from_tile_id(*tile_id);
        // TODO: This cast should be abstracted away by the lib; submit a PR
        unsafe {
            let offset_in_sb: isize = (x as isize) + (y as isize * BG_WIDTH_TILES as isize);
            let sb_entries = SCREEN_BASE_BLOCKS
                .index(self.screenblock as usize)
                .cast::<TextScreenblockEntry>();
            sb_entries.offset(offset_in_sb).write(glyph);
        }
    }

    /// Clear all text from the screen and reset cursor position to (0,0).
    pub fn clear(&mut self) {
        // Load blank tilemap into VRAM
        let blank_entry = TextScreenblockEntry::new();
        // TODO: This cast should be abstracted away by the lib; submit a PR
        unsafe {
            let sb_entries = SCREEN_BASE_BLOCKS
                .index(self.screenblock as usize)
                .cast::<TextScreenblockEntry>();
            for slot in sb_entries.iter_slots(32 * 32) {
                slot.write(blank_entry);
            }
        }
        self.set_cursor_pos(0, 0);
    }
    // TODO: Load correct palette
}

impl fmt::Write for TextEngine {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for chara in s.chars() {
            // Interpret newline correctly
            if chara == '\n' {
                self.set_cursor_pos(0, self.cursor_y + 1);
            } else {
                self.put_char_and_advance(chara);
            }
        }
        return fmt::Result::Ok(());
    }
}
