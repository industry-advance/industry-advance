use alloc::vec::Vec;
use core::convert::TryInto;

// TODO: Implement automatic deallocation of sprite on handle going out of scope via Drop

/// The sizes of sprite that the hardware supports, and are therefore possible to allocate.
pub(crate) enum HWSpriteSize {
    EightByEight,
    SixteenBySixteen,
    ThirtyTwoByThirtyTwo,
    SixtyFourBySixtyFour,
    SixteenByEight,
    ThirtyTwoBySixteen,
    SixtyFourByThirtyTwo,
    EightBySixteen,
    SixteenByThirtyTwo,
    ThirtyTwoBySixtyFour,
}

impl HWSpriteSize {
    pub(crate) fn to_size_in_bytes(&self) -> u32 {
        use HWSpriteSize::*;
        match self {
            EightByEight => 32,
            SixteenBySixteen => 128,
            ThirtyTwoByThirtyTwo => 512,
            SixtyFourBySixtyFour => 1024, // FIXME: Correct?
            SixteenByEight => 64,
            ThirtyTwoBySixteen => 256,
            SixtyFourByThirtyTwo => 512, // FIXME: Correct?
            EightBySixteen => 64,
            SixteenByThirtyTwo => 256,
            ThirtyTwoBySixtyFour => 512, // FIXME: Correct?
        }
    }

    pub(crate) fn to_num_of_32_byte_blocks(&self) -> usize {
        return (self.to_size_in_bytes() / 32).try_into().unwrap();
    }
}

/// A hardware sprite in a representation ready to be loaded into VRAM by the sprite allocator.
pub(crate) struct HWSprite {
    pub tiles: Vec<u32>,
    pub size: HWSpriteSize,
}

impl HWSprite {
    /// Takes a slice of u32 representing the 8x8 tiles of a sprite in GBA format
    /// and a target sprite size, and creates a ready-to-be-allocated hardware sprite.
    ///
    /// The sprite must be using 8bpp color mode and 1D mapping.
    ///
    /// If the slice is smaller than the target size, the rest is padded.
    ///
    /// If the slice is larger than the target size, the function panics.
    pub fn from_u32_slice(tiles: &[u32], size: HWSpriteSize) -> HWSprite {
        let num_of_u32s: usize = (size.to_size_in_bytes() / 4).try_into().unwrap();
        if tiles.len() > num_of_u32s {
            panic!("Attempt to create hardware sprite that is larger than the specified size");
        }

        // Copy and pad the tiles
        let mut padded_tiles = Vec::<u32>::new();
        for tile in tiles {
            padded_tiles.push(*tile);
        }
        for _ in 0..(num_of_u32s - tiles.len()) {
            padded_tiles.push(0);
        }

        return HWSprite {
            tiles: padded_tiles,
            size: size,
        };
    }
}
