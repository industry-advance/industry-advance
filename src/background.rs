use core::convert::TryInto;

use alloc::vec::Vec;

use gba::io::{background, display, dma};
use gba::palram;
use gba::vram;
use gba::Color;

pub const SCREENBLOCK_SIZE_IN_U16: usize = 32 * 32 * 1;
const TILE_SIZE_IN_PX: usize = 8;
const BACKING_MAP_LENGTH_IN_TILES: usize = 32;
const CHARBLOCK: u16 = 0;
// Sets which screenblocks are to be used for each loaded part of the background
const BG0_SCREEN_BASE_BLOCK: usize = 16;
const BG1_SCREEN_BASE_BLOCK: usize = 20;
const BG2_SCREEN_BASE_BLOCK: usize = 24;
const BG3_SCREEN_BASE_BLOCK: usize = 28;

// Used for DMA
const SCREENBLOCK_SIZE: usize = 2 * 1024;

/// By default, the GBA only allows up to 32x32 tiles per screenblock.
/// However, the hardware supports using adjacent screenblocks to produce up to 64x64 tile maps.
/// We exploit that here by wrapping the scroll control registers so that we know when the edge of
/// the screenblock is about to come into view, and can seamlessly swap out the screenblock in the direction
/// the player's heading.
///
/// This way, we can create the illusion of an arbitrarily sized background.
///
/// # Safety
///
/// This code assumes that it's in sole control of the display control register's background and size settings,
/// as well as all charblocks and screenblocks in VRAM and background PALRAM.
pub(crate) struct LargeBackground<'a> {
    tiles: &'a [u32],
    backing_tilemaps: Vec<Vec<&'a [u16; SCREENBLOCK_SIZE_IN_U16]>>, // 2D map of
}

impl LargeBackground<'_> {
    /// Create a new `LargeBackground`.
    /// and initialize the backing backgrounds by writing data to VRAM/background PALRAM.
    ///
    /// center_x and center_y are the coordinates for where to initially place the center of the displayed area.
    /// The coordinate system starts at the top-left corner.
    pub(crate) fn init<'a>(
        tiles: &'a [u32],
        backing_tilemaps: Vec<Vec<&'a [u16; SCREENBLOCK_SIZE_IN_U16]>>,
        palette: &'a [u16],
    ) -> LargeBackground<'a> {
        let mut lbg: LargeBackground<'a> = LargeBackground {
            tiles: tiles,
            backing_tilemaps: backing_tilemaps,
        };

        // Load palette into VRAM
        for (i, entry) in palette.iter().enumerate() {
            let idx = palram::index_palram_bg_4bpp((i / 16) as u8, (i % 16) as u8);
            idx.write(Color(*entry));
        }

        // TODO: Load tiles into VRAM
        // Use DMA to load tiles into VRAM

        // Load the top-left tilemaps into VRAM
        lbg.load_backing_tilemap(display::Background::BG0, 0, 0);
        if lbg.backing_tilemaps.len() > 1 {
            lbg.load_backing_tilemap(display::Background::BG1, 1, 0);
        }
        if lbg.backing_tilemaps[0].len() > 1 {
            lbg.load_backing_tilemap(display::Background::BG2, 0, 1);
        }
        lbg.load_backing_tilemap(display::Background::BG3, 1, 1);
        return lbg;
    }

    /// Load the given backing tilemap into VRAM.
    fn load_backing_tilemap(
        &self,
        bg_slot: display::Background,
        backing_map_x: usize,
        backing_map_y: usize,
    ) {
        let tilemap_ptr = self.backing_tilemaps[backing_map_x][backing_map_y].as_ptr();
        // Use DMA to speed up loading (and because I want to try it)
        use display::Background::*;
        let screenblock = match bg_slot {
            BG0 => BG0_SCREEN_BASE_BLOCK,
            BG1 => BG1_SCREEN_BASE_BLOCK,
            BG2 => BG2_SCREEN_BASE_BLOCK,
            BG3 => BG3_SCREEN_BASE_BLOCK,
        };
        unsafe {
            dma::DMA3::set_source(tilemap_ptr as *const u32);
            dma::DMA3::set_dest(
                (vram::VRAM_BASE_USIZE + (screenblock * SCREENBLOCK_SIZE)) as *mut u32,
            );
            dma::DMA3::set_count((SCREENBLOCK_SIZE).try_into().unwrap());
            dma::DMA3::set_control(dma::DMAControlSetting::new().with_enabled(true));
        }

        // Ensure the background is enabled
        let bg_settings = background::BackgroundControlSetting::new()
            .with_char_base_block(CHARBLOCK)
            .with_is_8bpp(false)
            .with_size(background::BGSize::Zero);
        let dispcnt = display::display_control();
        match bg_slot {
            BG0 => {
                display::set_background_settings(BG0, bg_settings.with_screen_base_block(16));
                display::set_display_control(dispcnt.with_bg0(true));
            }
            BG1 => {
                display::set_background_settings(BG1, bg_settings.with_screen_base_block(20));
                display::set_display_control(dispcnt.with_bg1(true));
            }
            BG2 => {
                display::set_background_settings(BG2, bg_settings.with_screen_base_block(24));
                display::set_display_control(dispcnt.with_bg2(true));
            }
            BG3 => {
                display::set_background_settings(BG3, bg_settings.with_screen_base_block(28));
                display::set_display_control(dispcnt.with_bg3(true));
            }
        }
    }
}

/// Looks up the indices of backing tilemap the coordinates belong to.
fn coords_to_backing_tilemap_indices(x: usize, y: usize) -> (usize, usize) {
    return (
        (x / (BACKING_MAP_LENGTH_IN_TILES * TILE_SIZE_IN_PX)),
        (y / (BACKING_MAP_LENGTH_IN_TILES * TILE_SIZE_IN_PX)),
    );
}
