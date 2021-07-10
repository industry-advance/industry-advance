use super::*;
use gba::prelude::*;

/// A handle to a hardware sprite allocated in VRAM/OAM.
/// Also provides some wrappers to avoid the tedium of having to get an object, modify it, and write it back
/// for commonly used object attributes.
pub struct HWSpriteHandle {
    pub sprite_size: HWSpriteSize,
    pub(super) starting_block: usize,
    pub(super) data_hash: u64,
    pub(super) oam_slot: usize,
}

impl HWSpriteHandle {
    // These are some wrappers to avoid the tedium of having to get an object, modify it, and write it back
    // for commonly used object attributes.

    /// Set the visibility of the sprite.
    ///
    /// Do not use to enable affine sprites.
    pub fn set_visibility(&self, visible: bool) {
        OAM_ATTR0
            .index(self.oam_slot)
            .apply(|x| x.set_double_disabled(!visible));
    }

    /// Gets the visibility of the sprite.
    pub fn get_visibility(&self) -> bool {
        return !OAM_ATTR0.index(self.oam_slot).read().double_disabled();
    }

    /// Sets the X position of the sprite.
    ///
    /// # Safety
    ///
    /// The position is not checked to be in bounds.
    pub fn set_x_pos(&self, pos: u16) {
        OAM_ATTR1.index(self.oam_slot).apply(|x| x.set_x_pos(pos));
    }

    /// Sets the Y position of the sprite.
    ///
    /// # Safety
    ///
    /// The position is not checked to be in bounds.
    pub fn set_y_pos(&self, pos: u16) {
        OAM_ATTR1.index(self.oam_slot).apply(|x| x.set_x_pos(pos));
    }
}

// TODO: impl Drop for HWSpriteHandle {} (currently causes a VRAM leak)
