use super::background::LargeBackground;

use crate::shared_constants::SCREENBLOCK_SIZE_BYTES;
use crate::FS;

use gbfs_rs::Filename;

use alloc::vec::Vec;

#[derive(Debug)]
pub struct Map<'a> {
    bg: LargeBackground<'a>,
}

impl<'a> Map<'a> {
    /// Create a new map.
    /// `x` and `y` are the size of the entire map, given in number of horizontal and vertical 32x32 sub-tilemaps, respectively.
    /// . Their number must match x*y and they must be in the vector in a left-to-right, top-to-bottom order.
    /// Each tilemap must be SCREENBLOCK_SIZE_IN_U8 large.
    /// If it isn't, this function will panic.
    pub fn new_map(
        palette: &'a [u16],
        x_size_in_tilemaps: usize,
        y_size_in_tilemaps: usize,
        tiles: &'a [u32],
        tilemaps: Vec<&'a [u8]>,
    ) -> Map<'a> {
        for tilemap in tilemaps.clone() {
            assert_eq!(tilemap.len(), SCREENBLOCK_SIZE_BYTES);
        }
        let mut two_d_indexed_tilemaps: Vec<Vec<&'a [u8]>> = Vec::with_capacity(x_size_in_tilemaps);
        for i in 0..x_size_in_tilemaps {
            two_d_indexed_tilemaps.push(Vec::with_capacity(y_size_in_tilemaps));
            for j in 0..y_size_in_tilemaps {
                two_d_indexed_tilemaps[i].push(tilemaps[i * x_size_in_tilemaps + j]);
            }
        }
        let bg = LargeBackground::init(tiles, two_d_indexed_tilemaps, palette);
        return Map { bg };
    }

    /// Returns whether the given area (in pixels) is visible on screen right now.
    pub fn is_area_visible(
        &self,
        top_left_x: u32,
        top_left_y: u32,
        bottom_right_x: u32,
        bottom_right_y: u32,
    ) -> bool {
        return self
            .bg
            .is_area_visible(top_left_x, top_left_y, bottom_right_x, bottom_right_y);
    }

    // Returns the top-left corner (x, y) coordinates of the currently visible map area.
    pub fn get_top_left_corner_coords(&self) -> (u32, u32) {
        return self.bg.get_top_left_corner_coords();
    }
    /// Loads a test map from the filesystem.
    pub fn load_test_map_from_fs() -> Map<'a> {
        let map_0: &'static [u8] = FS
            .get_file_data_by_name(Filename::try_from_str("testmap_0Map").unwrap())
            .unwrap();
        let map_1: &'static [u8] = FS
            .get_file_data_by_name(Filename::try_from_str("testmap_1Map").unwrap())
            .unwrap();
        let map_2: &'static [u8] = FS
            .get_file_data_by_name(Filename::try_from_str("testmap_2Map").unwrap())
            .unwrap();
        let map_3: &'static [u8] = FS
            .get_file_data_by_name(Filename::try_from_str("testmap_3Map").unwrap())
            .unwrap();
        let mut tilemaps: Vec<&'static [u8]> = Vec::new();

        tilemaps.push(map_0);
        tilemaps.push(map_1);
        tilemaps.push(map_2);
        tilemaps.push(map_3);

        let pal: &'a [u16] = FS
            .get_file_data_by_name_as_u16_slice(Filename::try_from_str("map_sharedPal").unwrap())
            .unwrap();
        let tiles: &'a [u32] = FS
            .get_file_data_by_name_as_u32_slice(Filename::try_from_str("map_sharedTiles").unwrap())
            .unwrap();
        return Map::new_map(pal, 2, 2, tiles, tilemaps);
    }

    /// Scroll the map by xy pixels.
    pub fn scroll(&mut self, x: i32, y: i32) {
        if x != 0 || y != 0 {
            self.bg.scroll(x, y);
        }
    }
}
