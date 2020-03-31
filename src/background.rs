use core::convert::TryInto;

use alloc::vec::Vec;

use gba::io::dma;
use gba::palram;
use gba::Color;

pub const SCREENBLOCK_SIZE_IN_U16: usize = 32 * 32 * 1;
const TILE_SIZE_IN_PX: usize = 8;
const BACKING_MAP_LENGTH_IN_TILES: usize = 32;

// Used for DMA
const SCREENBLOCK_8_ADDR: usize = 0x0600_4000;
const SCREENBLOCK_SIZE: usize = 4 * 1024;

enum BackgroundSlots {
    BG0,
    BG1,
    BG2,
    BG3,
}
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
    center_x: usize,
    center_y: usize,
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
        center_x: usize,
        center_y: usize,
    ) -> LargeBackground<'a> {
        let mut lbg: LargeBackground<'a> = LargeBackground {
            tiles: tiles,
            backing_tilemaps: backing_tilemaps,
            center_x,
            center_y,
        };

        // Load palette into VRAM
        for (i, entry) in palette.iter().enumerate() {
            let idx = palram::index_palram_bg_4bpp((i / 16) as u8, (i % 16) as u8);
            idx.write(Color(*entry));
        }

        // TODO: Load tiles into VRAM

        // Determine which backing backgrounds are the closest around the initial coordinates
        // and load their respective tilemaps into VRAM.
        let (center_backing_map_x, center_backing_map_y) =
            coords_to_backing_tilemap_indices(center_x, center_y);
        lbg.load_backing_tilemap(
            BackgroundSlots::BG0,
            center_backing_map_x,
            center_backing_map_y,
        );

        return lbg;
    }

    /// Load the given backing tilemap into VRAM.
    fn load_backing_tilemap(
        &self,
        slot: BackgroundSlots,
        backing_map_x: usize,
        backing_map_y: usize,
    ) {
        let tilemap_ptr = self.backing_tilemaps[backing_map_x][backing_map_y].as_ptr();
        // Use DMA to speed up loading (and because I want to try it)
        unsafe {
            dma::DMA3::set_source(tilemap_ptr as *const u32);
            dma::DMA3::set_dest(SCREENBLOCK_8_ADDR as *mut u32);
            dma::DMA3::set_count((SCREENBLOCK_SIZE / 4).try_into().unwrap());
            dma::DMA3::set_control(dma::DMAControlSetting::new().with_enabled(true));
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
