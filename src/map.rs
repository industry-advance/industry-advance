use crate::testmap::*;

use gba::{
    io::background::{BGSize, BackgroundControlSetting, BG0CNT},
    io::display::{DisplayControlSetting, DisplayMode, DISPCNT},
    mgba, palram, vram,
};
use typenum::consts::U1024;
use voladdress::VolBlock;

use core::convert::TryInto;
use core::mem;

const PALETTE_SIZE: usize = 256;
// TODO: Variable size
const TILEMAP_SIZE: usize = 1024;

// TODO: Use dynamic allocator/borrow checker to prevent collisions with memory areas already in use
const CHARBLOCK: usize = 0;
const SCREENBLOCK: usize = 8; // First to not overlap w/ charblock 0

pub(crate) struct Map<'a> {
    tileset: &'a [u32],               // Tileset to display
    palette: &'a [u16; PALETTE_SIZE], // Palette for tiles
    tilemap: &'a [u16],               // Part of tilemap currently in VRAM
}

impl Map<'_> {
    pub(crate) fn new_testmap() -> Map<'static> {
        return Map {
            tileset: &TESTMAP_TILES,
            tilemap: &TESTMAP_MAP,
            palette: &TESTMAP_PAL,
        };
    }

    /// Draw a freshly-loaded map.
    /// Use draw_updated() to draw a map already loaded into VRAM that just needs to be updated.
    pub(crate) fn draw_initial(&self) {
        // Ensure we're in correct graphics mode
        const MODE: DisplayControlSetting = DisplayControlSetting::new()
            .with_mode(DisplayMode::Mode0)
            // We use background layer 0 for the map
            .with_bg0(true);
        DISPCNT.write(MODE);

        // Copy palette into PALRAM
        for (i, entry) in self.palette.iter().enumerate() {
            let idx = palram::index_palram_bg_8bpp(i as u8);
            // This unsafe block is needed because the GBA crate doesn't expose a safe API to cast u16 to color
            let col: gba::Color;
            unsafe {
                col = mem::transmute::<u16, gba::Color>(*entry);
            }
            idx.write(col);
        }

        // Copy tiles into VRAM
        // Character block 0 is always used for background map layer data
        let charblock = vram::get_8bpp_character_block(CHARBLOCK);
        for (i, b) in charblock.iter().enumerate() {
            // Check whether we have exhausted tileset
            if i * 16 >= self.tileset.len() {
                break;
            }

            // Otherwise, copy a subslice into VRAM
            let subslice = &self.tileset[(16 * i)..(16 * i + 16)];
            let mut tiles: [u32; 16] = [
                0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
            ];
            tiles.copy_from_slice(subslice);
            b.write(vram::Tile8bpp(tiles));
        }

        // Copy tilemap into VRAM
        let screenblock = vram::get_screen_block(SCREENBLOCK); // Important to choose one that's not used by charblock (see https://www.coranac.com/tonc/text/regbg.htm)!
        for (i, entry) in screenblock.iter().enumerate() {
            // There's no direct API to interpret u16's as tile data, making transmutation necessary
            let tile_entry: vram::text::TextScreenblockEntry;
            unsafe {
                tile_entry =
                    mem::transmute::<u16, vram::text::TextScreenblockEntry>(self.tilemap[i]);
            }
            entry.write(tile_entry);
        }

        // Set the parameters for the background
        BG0CNT.write(
            BackgroundControlSetting::new()
                .with_is_8bpp(true)
                .with_screen_base_block(SCREENBLOCK.try_into().unwrap())
                .with_char_base_block(CHARBLOCK.try_into().unwrap())
                .with_size(BGSize::Zero),
        );
    }
}
