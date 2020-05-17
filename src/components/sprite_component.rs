use crate::sprite::{HWSpriteAllocator, HWSpriteHandle, HWSpriteSize};
/// An ECS component which controls the on-screen sprite of the entity.
pub(crate) struct SpriteComponent {
    handle: HWSpriteHandle,
}

impl SpriteComponent {
    /// Initialize a new sprite and make it visible.
    /// The sprite allocator is expected to be initialized.
    pub fn with_pos(
        alloc: &mut HWSpriteAllocator,
        sprite_filename: &str,
        sprite_size: HWSpriteSize,
        x_pos: u16,
        y_pos: u16,
    ) -> SpriteComponent {
        let sprite_handle = alloc.alloc_from_fs_file(sprite_filename, sprite_size);
        sprite_handle.set_x_pos(x_pos);
        sprite_handle.set_y_pos(y_pos);
        sprite_handle.set_visibility(true);
        return SpriteComponent {
            handle: sprite_handle,
        };
    }

    /// Returns a handle to the underlying sprite.
    pub fn get_handle(&mut self) -> &mut HWSpriteHandle {
        return &mut self.handle;
    }
}
