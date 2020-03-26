use alloc::vec::Vec;
use core::convert::TryInto;
use gba::oam::{ObjectShape, ObjectSize};
use gba::vram::Tile8bpp;

#[allow(dead_code)]
/// The sizes of sprite that the hardware supports, and are therefore possible to allocate.
pub(crate) enum HWSpriteSize {
    EightByEight,
    SixteenBySixteen,
    ThirtyTwoByThirtyTwo,
    SixtyFourBySixtyFour,

    EightBySixteen,
    EightByThirtyTwo,

    SixteenByEight,
    SixteenByThirtyTwo,

    ThirtyTwoByEight,
    ThirtyTwoBySixteen,
    ThirtyTwoBySixtyFour,

    SixtyFourByThirtyTwo,
}

impl HWSpriteSize {
    /// Returns the sprite's size in bytes.
    /// Note that this assumes 8bpp sprites for now.
    pub fn to_size_in_bytes(&self) -> u32 {
        use HWSpriteSize::*;
        match self {
            EightByEight => 64,
            SixteenBySixteen => 256,
            ThirtyTwoByThirtyTwo => 1024,
            SixtyFourBySixtyFour => 4096,

            EightBySixteen => 128,
            EightByThirtyTwo => 256,

            SixteenByEight => 128,
            SixteenByThirtyTwo => 512,

            ThirtyTwoByEight => 256,
            ThirtyTwoBySixteen => 512,
            ThirtyTwoBySixtyFour => 2048,

            SixtyFourByThirtyTwo => 2048,
        }
    }

    /// Calculates the amount of 32 byte blocks of VRAM required to fit the sprite.
    pub fn to_num_of_32_byte_blocks(&self) -> usize {
        return (self.to_size_in_bytes() / 32).try_into().unwrap();
    }

    /// Calculates the sprite's size and shape in the format required by OAM.
    pub fn to_obj_size_and_shape(&self) -> (ObjectSize, ObjectShape) {
        use HWSpriteSize::*;
        use ObjectShape::*;
        use ObjectSize::*;
        match self {
            EightByEight => (Zero, Square),
            SixteenBySixteen => (One, Square),
            ThirtyTwoByThirtyTwo => (Two, Square),
            SixtyFourBySixtyFour => (Three, Square),

            EightBySixteen => (Zero, Vertical),
            EightByThirtyTwo => (One, Vertical),

            SixteenByEight => (Zero, Horizontal),
            SixteenByThirtyTwo => (Two, Vertical),

            ThirtyTwoByEight => (One, Horizontal),
            ThirtyTwoBySixteen => (Two, Horizontal),
            ThirtyTwoBySixtyFour => (Three, Vertical),

            SixtyFourByThirtyTwo => (Three, Horizontal),
        }
    }
}

/// A hardware sprite in a representation ready to be loaded into VRAM by the sprite allocator.
/// Currently, only 8bpp color sprites are supported.
pub(crate) struct HWSprite {
    pub tiles: Vec<Tile8bpp>,
    pub size: HWSpriteSize,
}

impl HWSprite {
    /// Takes a slice of u32 representing the 8x8 tiles of a sprite in GBA format
    /// and a target sprite size, and creates a ready-to-be-allocated hardware sprite.
    ///
    /// The sprite must be using 8bpp color mode and 1D mapping.
    ///
    /// TODO: Reasonably pad smaller sprites
    /// TODO: (so that the padding empty space is in the bottom-right, while the sprite itself is in the top-left,
    /// TODO: as that's the corner OAM coordinates refer to)
    ///
    /// If the slice is larger than the target size, the function panics.
    pub fn from_u32_slice(tile_fragments: &[u32], size: HWSpriteSize) -> HWSprite {
        let expected_tile_fragments: usize = (size.to_size_in_bytes() / 4).try_into().unwrap();
        if tile_fragments.len() > expected_tile_fragments {
            panic!("Attempt to create hardware sprite that is larger than the specified size");
        }

        // Copy and pad the tiles
        let mut padded_tile_fragments = Vec::<u32>::new();
        for tile_fragment in tile_fragments {
            padded_tile_fragments.push(*tile_fragment);
        }
        for _ in 0..(expected_tile_fragments - tile_fragments.len()) {
            padded_tile_fragments.push(0);
        }

        // Chunk them up into Tile8bpp
        let mut tiles: Vec<Tile8bpp> = Vec::with_capacity(8);
        for tile_as_u32_chunk in padded_tile_fragments.chunks_exact(16) {
            let chunk_with_known_length: &[u32; 16] = tile_as_u32_chunk
                .try_into()
                .expect("This should not happen unless chunks_exact() is broken");
            tiles.push(Tile8bpp(*chunk_with_known_length));
        }

        return HWSprite {
            tiles: tiles,
            size: size,
        };
    }
}
