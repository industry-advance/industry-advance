use super::*;
use gba::oam;

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
    /// Returns the OAM object attributes for the sprite.
    pub fn read_obj_attributes(&self) -> oam::ObjectAttributes {
        return oam::read_obj_attributes(self.oam_slot).unwrap();
    }

    /// Writes the OAM object attributes for the sprite.
    ///
    /// # Safety
    ///
    /// Messing with the sprite's shape, size or base tile will cause graphical glitches.
    /// If you want to change those attributes, free the sprite and allocate a new one.
    ///
    /// The only reason why those fields are exposed is because it'd be too much work to create
    /// a wrapper for the OAM functionality of the gba crate that disallows this.
    pub fn write_obj_attributes(&self, attrs: oam::ObjectAttributes) {
        oam::write_obj_attributes(self.oam_slot, attrs).unwrap();
    }

    // These are some wrappers to avoid the tedium of having to get an object, modify it, and write it back
    // for commonly used object attributes.

    /// Set the visibility of the sprite.
    ///
    /// Do not use to enable affine sprites.
    pub fn set_visibility(&self, visible: bool) {
        let mut attrs = self.read_obj_attributes();
        if visible {
            attrs.attr0 = attrs.attr0.with_obj_rendering(oam::ObjectRender::Normal);
        } else {
            attrs.attr0 = attrs.attr0.with_obj_rendering(oam::ObjectRender::Disabled);
        }
        self.write_obj_attributes(attrs);
    }

    /// Gets the visibility of the sprite.
    pub fn get_visibility(&self) -> bool {
        let attrs = self.read_obj_attributes();
        if attrs.attr0.obj_rendering() != oam::ObjectRender::Disabled {
            return true;
        }
        return false;
    }

    /// Sets the X position of the sprite.
    ///
    /// # Safety
    ///
    /// The position is not checked to be in bounds.
    pub fn set_x_pos(&self, pos: u16) {
        let mut attrs = self.read_obj_attributes();
        attrs.attr1 = attrs.attr1.with_col_coordinate(pos);
        self.write_obj_attributes(attrs);
    }

    /// Sets the Y position of the sprite.
    ///
    /// # Safety
    ///
    /// The position is not checked to be in bounds.
    pub fn set_y_pos(&self, pos: u16) {
        let mut attrs = self.read_obj_attributes();
        attrs.attr0 = attrs.attr0.with_row_coordinate(pos);
        self.write_obj_attributes(attrs);
    }
}

// TODO: impl Drop for HWSpriteHandle {} (currently causes a VRAM leak)
