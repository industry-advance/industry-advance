mod hw_sprite;
mod hw_sprite_alloc;
mod sprite_dma;
pub(crate) use hw_sprite::HWSpriteSize;
pub(crate) use hw_sprite_alloc::{HWSpriteAllocator, HWSpriteHandle};

#[cfg(test)]
mod hw_sprite_alloc_test;
