use crate::sprite::{HWSprite, HWSpriteAllocator, HWSpriteHandle, HWSpriteSize};
/// An ECS component which controls the on-screen sprite of the entity.
pub(crate) struct SpriteComponent {
    handle: HWSpriteHandle,
}

impl SpriteComponent {
    /// Initialize a new sprite and make it visible.
    /// The sprite allocator is expected to be initialized.
    pub fn init(
        alloc: &mut HWSpriteAllocator,
        sprite_data: &[u32],
        sprite_size: HWSpriteSize,
    ) -> SpriteComponent {
        let mut handle = alloc.alloc(HWSprite::from_u32_slice(sprite_data, sprite_size));
        // TODO: Consider whether to make visible by default
        return SpriteComponent { handle: handle };
    }

    /// Returns a handle to the underlying sprite.
    pub fn get_handle(&mut self) -> &mut HWSpriteHandle {
        return &mut self.handle;
    }
}
