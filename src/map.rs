use crate::background::{LargeBackground, SCREENBLOCK_SIZE_IN_U16};

use alloc::vec::Vec;

pub(crate) struct Map<'a> {
    bg: LargeBackground<'a>,
}

impl Map<'_> {
    /// Create a new map.
    /// `x` and `y` are the size of the entire map, given in number of horizontal and vertical 32x32 sub-tilemaps, respectively.
    /// . Their number must match x*y and they must be in the vector in a left-to-right, top-to-bottom order.
    pub(crate) fn new_map<'a>(
        palette: &'a [u16],
        x: usize,
        y: usize,
        tiles: &'a [u32],
        tilemaps: Vec<&'a [u16; SCREENBLOCK_SIZE_IN_U16]>,
    ) -> Map<'a> {
        let mut two_d_indexed_tilemaps: Vec<Vec<&[u16; SCREENBLOCK_SIZE_IN_U16]>> =
            Vec::with_capacity(x);
        for i in 0..x {
            two_d_indexed_tilemaps[x] = Vec::with_capacity(y);
            for j in 0..y {
                two_d_indexed_tilemaps[x][y] = tilemaps[i * x + j];
            }
        }
        // TODO: Pass through center_x and center_y
        let bg = LargeBackground::init(tiles, two_d_indexed_tilemaps, palette, 1024, 1024);
        return Map { bg: bg };
    }
}
