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
use crate::vram;
use crate::FS;
use crate::{debug_log, Subsystems::Text};

use alloc::vec::Vec;
use core::convert::TryInto;
use core::fmt;
use core::hash::BuildHasherDefault;
use core::str;

use arrayref::array_ref;
use gba::prelude::*;
use hashbrown::hash_map::HashMap;
use twox_hash::XxHash64;

const BG_WIDTH_TILES: usize = 32;
/// The size of a single character. Useful to know for laying out graphics which accompany the text.
pub const CHARA_SIZE_IN_PX: u8 = 8;

#[must_use]
#[derive(Copy, Clone, Debug)]
pub enum TextEngineError {
    TileCharAmountMismatch,
    TooManyTiles,
    NonExistentScreenblock,
    NonExistentCharblock,
    OOBCursorPos,
    UnknownChar(char),
}
impl fmt::Display for TextEngineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use TextEngineError::*;
        match self {
            TileCharAmountMismatch => write!(
                f,
                "TextEngineError: The amount of characters provided differs from the amount of glyph tiles"
            ),
            TooManyTiles => write!(f, "TextEngineError: At most 512 characters may be provided"),
            NonExistentScreenblock => write!(f, "TextEngineError: The provided screenblock doesn't exist"),
            NonExistentCharblock => write!(f, "TextEngineError: The provided charblock doesn't exist"),
            OOBCursorPos => write!(f, "The given cursor position is outside the rectangle this text engine is allowed to draw into"),
            UnknownChar(c) => write!(f, "The character '{}' does not exist in the font", c),
        }
    }
}

/// The area of the background where the text engine may render, in tiles.
#[derive(Debug, Copy, Clone)]
pub struct TextArea {
    tl_x: u8,
    tl_y: u8,
    br_x: u8,
    br_y: u8,
}

impl TextArea {
    /// Dimensions for a text area which takes up an entire screen.
    pub const FULLSCREEN: TextArea = TextArea {
        tl_x: 0,
        tl_y: 0,
        br_x: SCREEN_WIDTH_TILES as u8,
        br_y: SCREEN_HEIGHT_TILES as u8,
    };

    /// Creates a new `TextArea` object.
    ///
    /// Returns `None` if
    /// * The dimensions would be larger than the screen
    /// * The bottom-left corner would be above / to the right of the top-right corner
    /// * The `TextArea` would have an area of 0
    pub const fn new(tl_x: u8, tl_y: u8, br_x: u8, br_y: u8) -> Option<TextArea> {
        // Screen boundaries and positive area
        if (tl_x >= br_x)
            || (tl_y >= br_y)
            || (br_x > (SCREEN_WIDTH_TILES as u8))
            || (br_y > (SCREEN_HEIGHT_TILES as u8))
        {
            return None;
        }

        return Some(TextArea {
            br_x,
            br_y,
            tl_x,
            tl_y,
        });
    }

    // Checks whether the given tile is within the `TextArea`.
    const fn contains_tile(&self, x: u8, y: u8) -> bool {
        return x >= self.tl_x && x <= self.br_x && y >= self.tl_y && y <= self.br_y;
    }
}

/// Structure representing the text engine's current state.
#[derive(Debug)]
pub struct TextEngine {
    char_to_tile_id: HashMap<char, u16, BuildHasherDefault<XxHash64>>,
    /// X position of cursor, in tiles
    cursor_x: u8,
    /// Y position of cursor, in tiles
    cursor_y: u8,
    /// Screenblock to draw on
    screenblock: u8,
    /// The offset into the background palette where text colors start
    palette_start: u16,
    /// The area of the background that text may be rendered on
    ta: TextArea,
}

impl TextEngine {
    /// Initialize a text engine using a default font and characters from global GBFS.
    pub fn init_from_gbfs(
        screenblock: u8,
        charblock: u8,
        background: Background,
        palette_start: u16,
        ta: TextArea,
    ) -> Result<TextEngine, TextEngineError> {
        let font_tiles: &[u32] = FS.get_file_data_by_name_as_u32_slice("fontTiles").unwrap();
        let font_chars: Vec<char> =
            str::from_utf8(FS.get_file_data_by_name("font_chars.txt").unwrap())
                .unwrap()
                .chars()
                .collect();
        let font_pal: Vec<Color> = FS
            .get_file_data_by_name_as_u16_slice("font_sharedPal")
            .unwrap()
            .iter()
            .map(|x| Color(*x))
            .collect();

        return TextEngine::init(
            &font_tiles,
            &font_chars,
            &font_pal,
            palette_start,
            screenblock,
            charblock,
            background,
            ta,
        );
    }

    /// Initializes a text engine.
    ///
    /// `font_tiles` must be a slice of tiles.
    /// `font_chars` must be Unicode characters, with each character at the same index as it's
    /// tile.
    ///
    /// No more than 512 tiles are permitted due to the limited amount of space in a charblock.
    ///
    /// `line_len` is the maximum amount of characters that may be put in a line. Must be <= to
    /// `SCREEN_WIDTH_TILES`.
    ///
    /// `line_amount` is the maximum number of lines to be written, must be <= to
    /// `SCREEN_HEIGHT_TILES`.
    ///
    /// `background` is the hardware background text should be drawn upon.
    ///
    /// `charblock` is the index of the GBA charblock to use (`0`-`3`)
    ///
    /// `screenblock` is the index of the GBA screenblock to use (`0`-`31`). Keep the [tonc
    /// table](https://www.coranac.com/tonc/text/regbg.htm) in mind and pick one that doesn't
    /// overlap with `charblock`.
    ///
    /// If any of the parameters do not fulfill the requirements, `None` is returned.
    ///
    /// You are responsible for ensuring that the hardware is in a compatible state (4bpp
    /// backgrounds, given background is enabled, given screenblock/charblock/background doesn't contain tiles where the
    /// text engine may overwrite it) before calling this.
    pub fn init(
        font_tiles: &[u32],
        font_chars: &[char],
        font_pal: &[Color],
        palette_start: u16,
        screenblock: u8,
        charblock: u8,
        background: Background,
        ta: TextArea,
    ) -> Result<TextEngine, TextEngineError> {
        // Check the validity of parameters
        if (font_tiles.len() / vram::TILE4BPP_NUM_U32S) != font_chars.len() {
            return Err(TextEngineError::TileCharAmountMismatch);
        }
        if (font_tiles.len() / vram::TILE4BPP_NUM_U32S) > 512 {
            return Err(TextEngineError::TooManyTiles);
        }
        if screenblock > vram::SCREENBLOCK_ID_MAX as u8 {
            return Err(TextEngineError::NonExistentScreenblock);
        }
        if charblock > vram::CHARBLOCK_ID_MAX as u8 {
            return Err(TextEngineError::NonExistentCharblock);
        }

        // Create character -> tile number lookup table
        // TODO: Make this more efficient, both in terms of memory for the mapping and CPU time (maybe use some const map)
        let mut hashmap: HashMap<char, u16, BuildHasherDefault<XxHash64>> = Default::default();
        for (i, chara) in font_chars.iter().enumerate() {
            debug_log!(Text, "Inserting char {} with tile ID {}", chara, i);
            hashmap.insert(*chara, i as u16);
        }

        // Load characters into VRAM charblock
        vram::copy_tiles_4bpp_dma(font_tiles, TEXT_CHARBLOCK, 0).unwrap();

        // Copy font palette into VRAM
        for (i, color) in font_pal.iter().enumerate() {
            BG_PALETTE.index(palette_start as usize + i).write(*color);
        }

        let mut engine = TextEngine {
            char_to_tile_id: hashmap,
            cursor_x: ta.tl_x,
            cursor_y: ta.tl_y,
            screenblock,
            palette_start,
            ta,
        };

        // Ensure there's no residual gunk in our screenblock
        engine.clear();

        background.write(
            BackgroundControl::new()
                .with_priority(0) // Display on top of everything
                .with_char_base_block(charblock)
                .with_screen_base_block(screenblock)
                .with_screen_size(BG_REG_32X32)
                .with_is_8bpp(false),
        );

        debug_log!(Text, "Text engine init done");
        return Ok(engine);
    }

    /// Sets the X, Y onscreen position for the cursor on the hardware background (NOT the screen,
    /// though it may be the same if your `TextArea` starts at the screen's top-left corner), in tiles.
    ///
    /// Value must be within the rectangle specified by your `TextArea`, or you'll get an Error
    /// return.
    pub fn set_cursor_pos(&mut self, x: u8, y: u8) -> Result<(), TextEngineError> {
        if !(self.ta.contains_tile(x, y)) {
            return Err(TextEngineError::OOBCursorPos);
        }
        self.cursor_x = x;
        self.cursor_y = y;
        return Ok(());
    }

    /// Get the current (x, y) position of the cursor in pixels (not tiles!) from the top-left corner of the `TextArea`.
    /// Useful if you want to draw things other than text onto the `TextArea` as well and avoid colliding.
    pub fn get_cursor_pos(&self) -> (u8, u8) {
        return (
            self.cursor_x * CHARA_SIZE_IN_PX,
            self.cursor_y * CHARA_SIZE_IN_PX,
        );
    }

    /// Puts selected character at current cursor position and advances it
    fn put_char_and_advance(&mut self, chara: char) -> Result<(), TextEngineError> {
        self.put_char(chara, self.cursor_x, self.cursor_y)?;
        // When line on screen is full, advance to next one
        if self.cursor_x >= (self.ta.br_x) {
            self.set_cursor_pos(self.ta.tl_x, self.ta.tl_y + 1)?;
        } else {
            self.set_cursor_pos(self.cursor_x + 1, self.cursor_y)?;
        }
        // When all lines are full, start overwriting from the top
        if self.cursor_y >= self.ta.br_y {
            self.set_cursor_pos(self.ta.tl_x, self.ta.tl_y)?;
        }

        return Ok(());
    }

    /// Puts selected character at given position (in tiles!)
    /// without advancing the cursor.
    ///
    /// Returns an `Error` if:
    /// * The coordinates are outside the `TextArea`
    /// * The character is not part of the font
    pub fn put_char(&mut self, chara: char, x: u8, y: u8) -> Result<(), TextEngineError> {
        if !(self.ta.contains_tile(x, y)) {
            return Err(TextEngineError::OOBCursorPos);
        }
        // Look up the glyph tile ID
        let glyph = match self.char_to_tile_id.get(&chara) {
            Some(id) => *id,
            None => return Err(TextEngineError::UnknownChar(chara)),
        };
        debug_log!(Text, "Character {} has glyph with tile ID {}", chara, glyph);

        let offset_in_sb: usize = ((x as isize) + (y as isize * BG_WIDTH_TILES as isize))
            .try_into()
            .unwrap();
        vram::set_tilemap_entry(glyph, self.screenblock as usize, offset_in_sb).unwrap();

        return Ok(());
    }

    /// Clear all text from the screen and reset cursor position to the top-left corner of the
    /// `TextArea`.
    pub fn clear(&mut self) {
        // Load blank tilemap into VRAM
        vram::fill_tilemap_dma(0, self.screenblock as usize).unwrap();
        // Reset cursor
        self.set_cursor_pos(self.ta.tl_x, self.ta.tl_y).expect("Failed to set the cursor position to the top-left of the allotted block. This is a bug in the text engine.");
    }
}

impl fmt::Write for TextEngine {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for chara in s.chars() {
            // Interpret newline correctly
            if chara == '\n' {
                self.set_cursor_pos(self.ta.tl_x as u8, self.cursor_y + 1).expect("Failed to set proper cursor position when writing. This is a bug in the text engine.");
            } else {
                match self.put_char_and_advance(chara) {
                    Ok(_) => {}
                    Err(e) => return Err(fmt::Error {}),
                }
            }
        }
        return fmt::Result::Ok(());
    }
}
