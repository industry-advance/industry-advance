//! This module contains types that don't really belong into other modules,
//! yet are used by several of them.

use fixed::{types::extra::U8, FixedI32, FixedU32};
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
pub enum Background {
    Zero,
    One,
    Two,
    Three,
}

use gba::io::background;
use gba::io::display;
impl Background {
    /// Apply the given setting to the background
    pub fn write(&self, setting: background::BackgroundControlSetting) {
        // Utilize the various control register setting functions
        use Background::*;
        match self {
            Zero => background::BG0CNT.write(setting),
            One => background::BG1CNT.write(setting),
            Two => background::BG2CNT.write(setting),
            Three => background::BG3CNT.write(setting),
        }
    }

    /// Set background visibility in DISPCNT
    pub fn set_visible(&self, visible: bool) {
        use Background::*;
        match self {
            Zero => display::DISPCNT.write(display::DISPCNT.read().with_bg0(visible)),
            One => display::DISPCNT.write(display::DISPCNT.read().with_bg1(visible)),
            Two => display::DISPCNT.write(display::DISPCNT.read().with_bg2(visible)),
            Three => display::DISPCNT.write(display::DISPCNT.read().with_bg3(visible)),
        }
    }
}
