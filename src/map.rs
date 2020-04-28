use crate::background::{LargeBackground, SCREENBLOCK_SIZE_IN_U8};

use alloc::vec::Vec;

pub(crate) struct Map<'a> {
    bg: LargeBackground<'a>,
}

impl Map<'_> {
    /// Create a new map.
    /// `x` and `y` are the size of the entire map, given in number of horizontal and vertical 32x32 sub-tilemaps, respectively.
    /// . Their number must match x*y and they must be in the vector in a left-to-right, top-to-bottom order.
    /// Each tilemap must be SCREENBLOCK_SIZE_IN_U8 large.
    /// If it isn't, this function will panic.
    pub(crate) fn new_map<'a>(
        palette: Vec<u16>,
        x: usize,
        y: usize,
        tiles: Vec<u32>,
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
}
