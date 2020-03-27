mod hw_sprite;
mod hw_sprite_alloc;
pub(crate) use hw_sprite::{HWSprite, HWSpriteSize};
pub(crate) use hw_sprite_alloc::{HWSpriteAllocator, HWSpriteHandle};

#[cfg(test)]
mod hw_sprite_alloc_test;
