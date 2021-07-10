//! This module contains types that don't really belong into other modules,
//! yet are used by several of them.

use fixed::{types::extra::U8, FixedI32, FixedU32};
use gba::mmio_addresses::*;
use gba::mmio_types::*;

/// A fixed-point velocity is used because the GBA has no FPU.
/// The velocity has 23 bits of precision before and 8 after the comma.
pub type Velocity = FixedI32<U8>;
/// The Zero value for a velocity
pub const ZERO_VELOCITY: Velocity = Velocity::from_bits(0b0);

pub type Coordinate = FixedU32<U8>;
pub type Position = (Coordinate, Coordinate);

// FIXME: These should be upstreamed

/// This enum represents any background.
/// It's quite convenient, and should be upstreamed.
#[allow(dead_code)] // Not all variants used ATM
#[derive(Copy, Clone, Debug)]
pub enum Background {
    Zero,
    One,
    Two,
    Three,
}

impl Background {
    /// Apply the given setting to the background
    pub fn write(&self, setting: BackgroundControl) {
        // Utilize the various control register setting functions
        use Background::*;
        match self {
            Zero => BG0CNT.write(setting),
            One => BG1CNT.write(setting),
            Two => BG2CNT.write(setting),
            Three => BG3CNT.write(setting),
        }
    }

    /// Set background visibility in DISPCNT
    pub fn set_visible(&self, visible: bool) {
        use Background::*;
        match self {
            Zero => DISPCNT.apply(|x| x.set_display_bg0(visible)),
            One => DISPCNT.apply(|x| x.set_display_bg1(visible)),
            Two => DISPCNT.apply(|x| x.set_display_bg2(visible)),
            Three => DISPCNT.apply(|x| x.set_display_bg3(visible)),
        }
    }
}
