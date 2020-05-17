use core::convert::TryInto;

use alloc::vec::Vec;

use gba::io::{background, display, dma};
use gba::{palram, vram, Color};

use crate::shared_constants::*;

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
#[derive(Debug)]
pub(crate) struct LargeBackground<'a> {
    backing_tilemaps: Vec<Vec<&'a [u8]>>, // 2D map of
    // Absolute coordinates of the current top-left corner of the screen on the map.
    // Coordinate system starts at top-left (0,0) of the map.
    curr_x: u32,
    curr_y: u32,
    // Store which backing tilemap is currently mapped to which screenblock
    sb0_curr_backing: Option<(usize, usize)>,
    sb1_curr_backing: Option<(usize, usize)>,
    sb2_curr_backing: Option<(usize, usize)>,
    sb3_curr_backing: Option<(usize, usize)>,
}

/// A 64x64 tile background is made up of 4 32x32 tile screenblocks.
#[derive(Debug)]
enum BGScreenblockSlots {
    Zero,
    One,
    Two,
    Three,
}

impl<'a> LargeBackground<'a> {
    /// Create a new `LargeBackground`.
    /// and initialize the backing backgrounds by writing data to VRAM/background PALRAM.
    ///
    /// center_x and center_y are the coordinates for where to initially place the center of the displayed area.
    /// The coordinate system starts at the top-left corner.
    pub(crate) fn init(
        tiles: &'a [u32],
        backing_tilemaps: Vec<Vec<&'a [u8]>>,
        palette: &'a [u16],
    ) -> LargeBackground<'a> {
        // Ensure we have at least 1 backing tilemap
        if backing_tilemaps.is_empty() {
            panic!("No backing tilemaps supplied");
        }

        if backing_tilemaps[0].is_empty() {
            panic!("No backing tilemaps supplied");
        }

        let mut lbg: LargeBackground<'a> = LargeBackground {
            backing_tilemaps,
            curr_x: 0,
            curr_y: 0,
            sb0_curr_backing: None,
            sb1_curr_backing: None,
            sb2_curr_backing: None,
            sb3_curr_backing: None,
        };

        // Load palette into VRAM
        for (i, entry) in palette.iter().enumerate() {
            let idx = palram::index_palram_bg_4bpp((i / 16) as u8, (i % 16) as u8);
            idx.write(Color(*entry));
        }

        // Use DMA to load tiles into VRAM
        // We only use charblock 0 for now.
        if tiles.len() > (CHARBLOCK_SIZE_BYTES / 4) {
            panic!(
                "Too many tiles in charblock! Expected up to {}, got {}",
                (CHARBLOCK_SIZE_BYTES / 4),
                tiles.len()
            );
        }
        unsafe {
            dma::DMA3::set_source(tiles.as_ptr());
            dma::DMA3::set_dest(
                (vram::VRAM_BASE_USIZE + (BACKGROUND_CHARBLOCK * CHARBLOCK_SIZE_BYTES)) as *mut u32,
            );
            dma::DMA3::set_count(tiles.len().try_into().unwrap());
            dma::DMA3::set_control(
                dma::DMAControlSetting::new()
                    .with_enabled(true)
                    .with_use_32bit(true),
            );
        }

        // Load the four top-left tilemaps into VRAM (if they exist)
        lbg.load_backing_tilemap(BGScreenblockSlots::Zero, 0, 0);
        if lbg.backing_tilemaps.len() > 1 {
            lbg.load_backing_tilemap(BGScreenblockSlots::One, 1, 0);
        }
        if lbg.backing_tilemaps[0].len() > 1 {
            lbg.load_backing_tilemap(BGScreenblockSlots::Two, 0, 1);
        }
        if lbg.backing_tilemaps[0].len() > 1 && lbg.backing_tilemaps.len() > 1 {
            lbg.load_backing_tilemap(BGScreenblockSlots::Three, 1, 1);
        }

        // Enable BG0 (which we use)
        let bg_settings = background::BackgroundControlSetting::new()
            .with_char_base_block(BACKGROUND_CHARBLOCK.try_into().unwrap())
            .with_screen_base_block(BACKGROUND_SCREEN_BASE_BLOCK.try_into().unwrap())
            .with_is_8bpp(false)
            .with_size(background::BGSize::Three)
            .with_bg_priority(3);
        background::BG0CNT.write(bg_settings);
        let dispcnt = display::display_control();
        display::set_display_control(dispcnt.with_bg0(true).with_force_vblank(false));

        return lbg;
    }

    /// Load the given backing tilemap into VRAM.
    fn load_backing_tilemap(
        &mut self,
        slot: BGScreenblockSlots,
        backing_map_x: usize,
        backing_map_y: usize,
    ) {
        // Mark the screenblock as occupied
        let screenblock_index: usize;
        use BGScreenblockSlots::*;
        match slot {
            Zero => {
                self.sb0_curr_backing = Some((backing_map_x, backing_map_y));
                screenblock_index = BACKGROUND_SCREEN_BASE_BLOCK;
            }

            One => {
                self.sb1_curr_backing = Some((backing_map_x, backing_map_y));
                screenblock_index = BACKGROUND_SCREEN_BASE_BLOCK + 1;
            }
            Two => {
                self.sb2_curr_backing = Some((backing_map_x, backing_map_y));
                screenblock_index = BACKGROUND_SCREEN_BASE_BLOCK + 2;
            }
            Three => {
                self.sb3_curr_backing = Some((backing_map_x, backing_map_y));
                screenblock_index = BACKGROUND_SCREEN_BASE_BLOCK + 3;
            }
        }

        let tilemap_ptr = self.backing_tilemaps[backing_map_x][backing_map_y].as_ptr();
        // Ensure that the pointer is aligned
        assert_eq!((tilemap_ptr as usize) % 4, 0);
        // Use DMA to speed up loading
        let dest_ptr =
            (vram::VRAM_BASE_USIZE + (screenblock_index * SCREENBLOCK_SIZE_BYTES)) as *mut u32;
        let num_u32s_to_copy: u16 = self.backing_tilemaps[backing_map_x][backing_map_y]
            .len()
            .try_into()
            .unwrap();
        // We checked alignment above
        #[allow(clippy::cast_ptr_alignment)]
        unsafe {
            dma::DMA3::set_source(tilemap_ptr as *const u32);
            dma::DMA3::set_dest(dest_ptr);
            dma::DMA3::set_count(num_u32s_to_copy);
            dma::DMA3::set_control(
                dma::DMAControlSetting::new()
                    .with_enabled(true)
                    .with_use_32bit(true),
            );
        }
    }

    /// Returns whether the given area is visible on screen right now.
    pub fn is_area_visible(
        &self,
        top_left_x: u32,
        top_left_y: u32,
        bottom_right_x: u32,
        bottom_right_y: u32,
    ) -> bool {
        let bottom_right_screen_x = self.curr_x + (SCREEN_WIDTH as u32);
        let bottom_right_screen_y = self.curr_y + (SCREEN_HEIGHT as u32);
        return (bottom_right_x >= self.curr_x)
            && (bottom_right_y >= self.curr_y)
            && (top_left_x <= bottom_right_screen_x)
            && (top_left_y <= bottom_right_screen_y);
    }

    /// Scroll the large background by xy pixels.
    /// If the indices are positive, scrolling happens down/to the right, if negative up/to the left.
    /// Parts of the map are dynamically loaded and no longer visible parts vacated.
    ///
    /// If scrolling into an area of the map for which no map fragment exists, a panic will occur.
    /// If scrolling into an area that would have negative absolute coordinates visible on screen, a
    /// panic will occur.
    /// The coordinates referenced in the panics are always related to the top-left corner of the displayed area.
    pub fn scroll(&mut self, delta_x: i32, delta_y: i32) {
        // New coords of the top-left screen corner
        let new_x = self.curr_x as i32 + delta_x;
        let new_y = self.curr_y as i32 + delta_y;
        // Ensure no negative coordinates would be visible on screen
        if new_x < 0 || new_y < 0 {
            panic!("Attempt to scroll into negative coordinates");
        }
        self.curr_x = new_x.try_into().unwrap();
        self.curr_y = new_y.try_into().unwrap();
        // Load new backing tilemaps if needed
        self.ensure_correct_backing_tilemaps_are_loaded();

        // Perform actual hardware scroll
        background::BG0HOFS.write(self.curr_x.try_into().unwrap());
        background::BG0VOFS.write(self.curr_y.try_into().unwrap());
    }

    fn ensure_correct_backing_tilemaps_are_loaded(&mut self) {
        // Calculate which backing tilemaps would be in view by scrolling
        // by checking where the 4 corners of the screen would end up in.
        let new_top_left_x: usize = self.curr_x.try_into().unwrap();
        let new_top_left_y: usize = self.curr_y.try_into().unwrap();
        let (
            (new_top_right_x, new_top_right_y),
            (new_bottom_right_x, new_bottom_right_y),
            (new_bottom_left_x, new_bottom_left_y),
        ) = coords_from_top_left_for_all_screen_corners(new_top_left_x, new_top_left_y);
        let (new_top_left_backing_x, new_top_left_backing_y) =
            coords_to_backing_tilemap_indices(new_top_left_x, new_top_left_y);
        let (new_top_right_backing_x, new_top_right_backing_y) =
            coords_to_backing_tilemap_indices(new_top_right_x, new_top_right_y);
        let (new_bottom_right_backing_x, new_bottom_right_backing_y) =
            coords_to_backing_tilemap_indices(new_bottom_right_x, new_bottom_right_y);
        let (new_bottom_left_backing_x, new_bottom_left_backing_y) =
            coords_to_backing_tilemap_indices(new_bottom_left_x, new_bottom_left_y);

        // For each corner, check whether the backing tilemaps are already present in VRAM
        // TODO: Ensure they're in the correct one of the four screenblocks
        if let None =
            self.get_backing_tilemap_loaded_slot(new_top_left_backing_x, new_top_left_backing_y)
        {
            self.load_backing_tilemap(
                BGScreenblockSlots::Zero,
                new_top_left_backing_x,
                new_top_left_backing_y,
            );
        }
        if let None =
            self.get_backing_tilemap_loaded_slot(new_top_right_backing_x, new_top_right_backing_y)
        {
            self.load_backing_tilemap(
                BGScreenblockSlots::One,
                new_top_right_backing_x,
                new_top_right_backing_y,
            );
        }
        if let None = self
            .get_backing_tilemap_loaded_slot(new_bottom_right_backing_x, new_bottom_right_backing_y)
        {
            self.load_backing_tilemap(
                BGScreenblockSlots::Three,
                new_bottom_right_backing_x,
                new_bottom_right_backing_y,
            );
        }
        if let None = self
            .get_backing_tilemap_loaded_slot(new_bottom_left_backing_x, new_bottom_left_backing_y)
        {
            self.load_backing_tilemap(
                BGScreenblockSlots::Two,
                new_bottom_left_backing_x,
                new_bottom_left_backing_y,
            );
        }
    }

    fn get_backing_tilemap_loaded_slot(
        &mut self,
        backing_x: usize,
        backing_y: usize,
    ) -> Option<BGScreenblockSlots> {
        match self.sb0_curr_backing {
            Some((_, _)) => return Some(BGScreenblockSlots::Zero),
            None => match self.sb1_curr_backing {
                Some((_, _)) => return Some(BGScreenblockSlots::One),
                None => match self.sb2_curr_backing {
                    Some((_, _)) => return Some(BGScreenblockSlots::Two),
                    None => match self.sb3_curr_backing {
                        Some((_, _)) => return Some(BGScreenblockSlots::Three),
                        None => return None,
                    },
                },
            },
        }
    }

    /// Returns the (x,y) coordinates of the top-left corner of the currently visible
    /// background area.
    pub fn get_top_left_corner_coords(&self) -> (u32, u32) {
        return (self.curr_x, self.curr_y);
    }
}

/// Looks up the indices of backing tilemap the coordinates belong to.
fn coords_to_backing_tilemap_indices(x: usize, y: usize) -> (usize, usize) {
    return (
        (x / (BACKING_MAP_LENGTH_IN_TILES * TILE_SIZE_IN_PX)),
        (y / (BACKING_MAP_LENGTH_IN_TILES * TILE_SIZE_IN_PX)),
    );
}

/// Given the coordinates for the top-left corner of the visible area,
/// calculate the coords for top-right, bottom-right, bottom-left.
fn coords_from_top_left_for_all_screen_corners(
    x: usize,
    y: usize,
) -> ((usize, usize), (usize, usize), (usize, usize)) {
    let top_right_x = x + SCREEN_WIDTH;
    let top_right_y = y;
    let bottom_right_x = x + SCREEN_WIDTH;
    let bottom_right_y = y + SCREEN_HEIGHT;
    let bottom_left_x = x;
    let bottom_left_y = y + SCREEN_HEIGHT;
    return (
        (top_right_x, top_right_y),
        (bottom_right_x, bottom_right_y),
        (bottom_left_x, bottom_left_y),
    );
}
