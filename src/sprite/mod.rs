//! This module provides the ability to manage objects (hardware sprites) in video memory.
//! The interface is allocator-like, with the ability to allocate and free sprites.
//!
//! Note that all sprites must share a palette.
//!
//! `DISPCNT` also has to be set for 1D mapping.
//!
//! Heavily inspired by [this article](https://www.gamasutra.com/view/feature/131491/gameboy_advance_resource_management.php?print=1).
//!
//! # TODO:
//! * Consider upstreaming to GBA crate.
//! * Writes to OAM should only happen on VBlank, we should implement some sort of shadow OAM and copy on interrupt.

mod hw_sprite;
mod hw_sprite_alloc;
mod hw_sprite_handle;
mod sprite_dma;
pub use hw_sprite::HWSpriteSize;
pub use hw_sprite_alloc::HWSpriteAllocator;
pub use hw_sprite_handle::HWSpriteHandle;

#[cfg(test)]
mod hw_sprite_alloc_test;
