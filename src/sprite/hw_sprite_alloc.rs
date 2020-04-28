/// This module provides the ability to manage objects (hardware sprites) in video memory.
/// The interface is allocator-like, with the ability to allocate and free sprites.
///
/// Note that all sprites must share a palette.
///
/// DISPCNT also has to be set for 1D mapping.
///
/// Heavily inspired by this article: https://www.gamasutra.com/view/feature/131491/gameboy_advance_resource_management.php?print=1
///
/// # TODO:
/// Consider upstreaming to GBA crate.
///
/// Writes to OAM should only happen on VBlank; we should implement some sort of shadow OAM and copy on interrupt
use core::convert::TryInto;

use alloc::boxed::Box;
use gba::{oam, palram, vram, Color};

#[cfg(debug_assertions)]
use gba::debug;

use super::HWSprite;

const LOWER_SPRITE_BLOCK_AS_CHARBLOCK: usize = 4;
const UPPER_SPRITE_BLOCK_AS_CHARBLOCK: usize = 5;

#[derive(PartialEq)]
enum SpriteBlockState {
    Unused,
    Used,
    Continue,
}

/// An allocator for managing hardware sprites in VRAM.
///
/// # Safety
///
/// This assumes that it's in complete control over the memory storing the sprite palette,
/// sprite tiles and OAM.
/// Any usage of these memory areas without going through the methods/handles provided by
/// this struct is UB.
///
/// Using more than one HWSpriteAllocator is also UB.
pub(crate) struct HWSpriteAllocator {
    allocation_map: Box<[SpriteBlockState; 1024]>, // List of 32 byte regions in object VRAM
    palette: Box<[Color; 256]>,
    oam_free_list: Box<[bool; 128]>, // List keeping track of which slots in OAM are free
}

impl HWSpriteAllocator {
    /// Create a new hardware sprite allocator for sprites with the given palette.
    pub fn new(palette: &[u16]) -> HWSpriteAllocator {
        // Cast to palette color type
        let mut palette_as_gba_colors: [Color; 256] = [Color::default(); 256];
        for (i, color) in palette.iter().enumerate() {
            let color_as_gba_color: Color = Color(*color);
            palette_as_gba_colors[i] = color_as_gba_color;
        }

        let entries = Box::new([SpriteBlockState::Unused; 1024]);
        let pal = Box::new(palette_as_gba_colors);
        let oam_free_list = Box::new([false; 128]);
        return HWSpriteAllocator {
            allocation_map: entries,
            palette: pal,
            oam_free_list: oam_free_list,
        };
    }

    /// Initialize the allocator by copying the palette into VRAM.
    ///  
    /// # Safety
    ///
    /// Any other code manipulating the sprite palette after
    /// this function is called will lead to graphical glitches.
    pub fn init(&self) {
        let pal_block = palram::PALRAM_OBJ;
        for (i, color) in self.palette.iter().enumerate() {
            pal_block.index(i).write(*color);
        }
    }

    /// Allocate the given sprite in VRAM.
    ///
    /// This will panic if insufficient space is available or too many sprites are already active.
    pub fn alloc(&mut self, sprite: HWSprite) -> HWSpriteHandle {
        // Find first spot with enough contiguous free blocks to hold the sprite
        let num_32b_blocks = sprite.size.to_num_of_32_byte_blocks();
        let begin_index = self
            .find_contiguous_free_blocks(num_32b_blocks)
            .expect("No contiguous free block of VRAM available to allocate hardware sprite");

        #[cfg(debug_assertions)]
        debug!(
            "[HW_SPRITE_ALLOC] Beginning allocation at block #{} for {} block sprite",
            begin_index, num_blocks
        );

        // Sprites are stored across 2 charblocks.
        let mut charblock_index: usize = 0xDD; // Illegal values, will be replaced below
        let mut slot_in_charblock: usize = 0xDD;
        if begin_index < 512 {
            charblock_index = LOWER_SPRITE_BLOCK_AS_CHARBLOCK;
            slot_in_charblock = begin_index / 2;
        } else {
            charblock_index = UPPER_SPRITE_BLOCK_AS_CHARBLOCK;
            slot_in_charblock = (begin_index - 512) / 2;
        }
        let charblock = vram::get_8bpp_character_block(charblock_index);
        // FIXME: Handle case where sprite is in boundary between 2 charblocks
        for (i, tile) in sprite.tiles.into_iter().enumerate() {
            charblock.index(slot_in_charblock + i).write(tile);
        }

        // Assign a slot in OAM
        let oam_slot = self.find_free_oam_slot();
        self.oam_free_list[oam_slot] = true;
        let (size, shape) = sprite.size.to_obj_size_and_shape();
        let starting_vram_tile_id: u16 = ((charblock_index * 256) + (slot_in_charblock * 2))
            .try_into()
            .unwrap();
        self.prepare_oam_slot(starting_vram_tile_id, oam_slot, size, shape);

        // Mark blocks as occupied
        self.allocation_map[begin_index] = SpriteBlockState::Used;
        for i in 1..num_32b_blocks {
            self.allocation_map[begin_index + i] = SpriteBlockState::Continue;
        }

        return HWSpriteHandle {
            starting_block: begin_index,
            oam_slot: oam_slot,
        };
    }

    /// Prepares a slot in OAM for the sprite.
    fn prepare_oam_slot(
        &self,
        starting_vram_tile_id: u16,
        oam_slot: usize,
        obj_size: oam::ObjectSize,
        obj_shape: oam::ObjectShape,
    ) {
        oam::write_affine_parameters(
            oam_slot,
            oam::AffineParameters {
                pa: 1,
                pb: 0,
                pc: 0,
                pd: 1,
            },
        ); // Identity matrix
        oam::write_obj_attributes(
            oam_slot,
            oam::ObjectAttributes {
                attr0: oam::OBJAttr0::new()
                    .with_obj_rendering(oam::ObjectRender::Disabled)
                    .with_obj_shape(obj_shape)
                    .with_is_8bpp(true),
                attr1: oam::OBJAttr1::new().with_obj_size(obj_size),
                attr2: oam::OBJAttr2::new().with_tile_id(starting_vram_tile_id),
            },
        );
    }
    /// Find a free slot in OAM.
    /// If none are available, panic.
    fn find_free_oam_slot(&self) -> usize {
        match self.oam_free_list.iter().position(|&x| x == false) {
            Some(pos) => pos,
            None => panic!("Attempt to create sprite when OAM is full"),
        }
    }

    /// Return the index of the beginning of the first area in the allocation map
    /// with sufficient space.
    fn find_contiguous_free_blocks(&self, num_blocks: usize) -> Option<usize> {
        for (i, block) in self.allocation_map.iter().enumerate() {
            if *block == SpriteBlockState::Unused {
                let mut free_blocks: usize = 1;
                for j in 1..num_blocks {
                    if self.allocation_map[i + j] == SpriteBlockState::Unused {
                        free_blocks = free_blocks + 1;
                    } else {
                        break;
                    }
                }
                if free_blocks >= num_blocks {
                    return Some(i);
                }
            }
        }
        return None;
    }

    /// Drop the allocation of the given sprite.
    /// Note that the sprite still exists in VRAM until overwritten,
    /// but is marked as inactive in OAM and therefore not displayed.
    pub fn free(&mut self, handle: HWSpriteHandle) {
        // Mark the first block as unused
        self.allocation_map[handle.starting_block] = SpriteBlockState::Unused;

        // Deallocate all blocks that are marked CONTINUE after the first block
        // (therefore part of this sprite)
        let mut i = 1;
        loop {
            if self.allocation_map[handle.starting_block + i] == SpriteBlockState::Continue {
                self.allocation_map[handle.starting_block + i] = SpriteBlockState::Unused
            } else {
                break;
            }
            i = i + 1;
        }

        // Set the sprite to not render in OAM
        let mut attrs = oam::read_obj_attributes(handle.oam_slot).unwrap();
        attrs.attr0 = attrs.attr0.with_obj_rendering(oam::ObjectRender::Disabled);
        oam::write_obj_attributes(handle.oam_slot, attrs);

        // Mark slot in OAM as available
        self.oam_free_list[handle.oam_slot] = false;
    }
}

/// A handle to a hardware sprite allocated in VRAM/OAM.
/// Also provides some wrappers to avoid the tedium of having to get an object, modify it, and write it back
/// for commonly used object attributes.
pub(crate) struct HWSpriteHandle {
    starting_block: usize,
    oam_slot: usize,
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
