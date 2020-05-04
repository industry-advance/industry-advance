//! This module contains types that don't really belong into other modules,
//! yet are used by several of them.

use fixed::{types::extra::U8, FixedI32, FixedU32};
/// A fixed-point velocity is used because the GBA has no FPU.
/// The velocity has 23 bits of precision before and 8 after the comma.
pub type Velocity = FixedI32<U8>;
/// The Zero value for a velocity
pub const ZERO_VELOCITY: Velocity = Velocity::from_bits(0b0);

pub type Coordinate = FixedU32<U8>;
pub const ZERO_COORDINATE: Coordinate = Coordinate::from_bits(0b0);
pub type Position = (Coordinate, Coordinate);
pub const ZERO_POSITION: Position = (ZERO_COORDINATE, ZERO_COORDINATE);