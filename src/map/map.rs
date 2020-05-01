use super::background::{LargeBackground, SCREENBLOCK_SIZE_IN_U8};

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
        x: usize,
        y: usize,
        tiles: &'a [u32],
        tilemaps: Vec<&'a [u8]>,
    ) -> Map<'a> {
        for tilemap in tilemaps.clone() {
            assert_eq!(tilemap.len(), SCREENBLOCK_SIZE_IN_U8);
        }
        let mut two_d_indexed_tilemaps: Vec<Vec<&'a [u8]>> = Vec::with_capacity(x);
        for i in 0..x {
            two_d_indexed_tilemaps.push(Vec::with_capacity(y));
            for j in 0..y {
                two_d_indexed_tilemaps[i].push(tilemaps[i * x + j]);
            }
        }
        let bg = LargeBackground::init(tiles, two_d_indexed_tilemaps, palette);
        return Map { bg: bg };
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
        self.bg.scroll(x, y);
    }
}
