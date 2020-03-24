/// This module provides the ability to manage objects (hardware sprites) in video memory.
/// The interface is allocator-like, with the ability to allocate and free sprites.
/// Note that all sprites must share a palette.
/// Heavily inspired by this article: https://www.gamasutra.com/view/feature/131491/gameboy_advance_resource_management.php?print=1
/// TODO: Consider upstreaming to GBA crate
use core::mem;

use alloc::boxed::Box;
use gba::{oam, palram, vram, Color};

use super::HWSprite;

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
/// TODO: Enforce singleton via type system
pub(crate) struct HWSpriteAllocator {
    allocation_map: Box<[SpriteBlockState; 1024]>, // Allocated on the heap to save valuable IWRAM
    palette: Box<[Color; 256]>,
    oam_free_list: Box<[bool; 128]>,
}

impl HWSpriteAllocator {
    /// Create a new hardware sprite allocator for sprites with the given palette.
    pub fn new(palette: &[u16]) -> HWSpriteAllocator {
        // TODO: This transmutation should be replaced with an API to load u16 as color in the gba crate
        let mut palette_as_gba_colors: [Color; 256] = [Color::default(); 256];
        for (i, color) in palette.iter().enumerate() {
            let color_as_gba_color: Color = unsafe { mem::transmute::<u16, Color>(*color) };
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
        let mut pal_block = palram::PALRAM_OBJ;
        for (i, color) in self.palette.iter().enumerate() {
            pal_block.index(i).write(*color);
        }
    }

    /// Allocate the given sprite in VRAM.
    ///
    /// This will panic if insufficient space is available or too many sprites are already active.
    pub fn alloc(&mut self, sprite: &HWSprite) -> HWSpriteHandle {
        let num_blocks = sprite.size.to_num_of_32_byte_blocks();
        // Find first spot with enough contiguous free blocks to hold the sprite
        let begin_index = self
            .find_contiguous_free_blocks(num_blocks)
            .expect("No contiguous free block of VRAM available to allocate hardware sprite");

        // TODO: Copy the sprite into the matching area of VRAM

        // Assign a slot in OAM
        let oam_slot = self.find_free_oam_slot();
        self.oam_free_list[oam_slot] = true;
        self.reset_oam_slot(oam_slot);

        // Mark blocks as occupied
        self.allocation_map[begin_index] = SpriteBlockState::Used;
        for i in 1..=num_blocks {
            self.allocation_map[begin_index + i] = SpriteBlockState::Continue;
        }

        return HWSpriteHandle {
            starting_block: begin_index,
            oam_slot: oam_slot,
        };
    }

    /// Resets a slot in OAM to sane defaults.
    fn reset_oam_slot(&self, oam_slot: usize) {
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
                attr0: oam::OBJAttr0::new(),
                attr1: oam::OBJAttr1::new(),
                attr2: oam::OBJAttr2::new(),
            },
        );
    }
    /// Find a free slot in OAM.
    /// If none are available, panic.
    fn find_free_oam_slot(&self) -> usize {
        match self.oam_free_list.iter().position(|&x| x == true) {
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
    /// but is removed from OAM and therefore not displayed.
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

        // TODO: Set the sprite to not render in OAM

        // Mark slot in OAM as available
        self.oam_free_list[handle.oam_slot] = false;
    }
}
/// A handle to a hardware sprite allocated in VRAM/OAM.
pub(crate) struct HWSpriteHandle {
    starting_block: usize,
    oam_slot: usize,
}

// TODO: Implement reasonably safe access to OAM attributes
impl HWSpriteHandle {}

// TODO: impl Drop for HWSpriteHandle {} (currently causes a VRAM leak)
