use alloc::vec::Vec;
use core::convert::TryInto;
use gba::oam::{ObjectShape, ObjectSize};

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
/// The sizes of sprite that the hardware supports, and are therefore possible to allocate.
pub enum HWSpriteSize {
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
