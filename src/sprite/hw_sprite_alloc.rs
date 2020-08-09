use super::*;
use crate::debug_log::*;

use core::convert::TryInto;
use core::hash::{BuildHasher, BuildHasherDefault, Hash, Hasher};

use alloc::boxed::Box;
use alloc::vec::Vec;
use gba::{oam, palram, Color};
use hashbrown::HashMap;
use twox_hash::XxHash64;

#[derive(Debug, PartialEq)]
enum SpriteBlockState {
    Unused,
    Used,
    Continue,
}

/// An allocator for managing hardware sprites in VRAM.
///
/// Sprites which have identical tiles to a sprite already in VRAM won't be loaded in order to
/// save VRAM, instead reference counting is used to keep the number of tiles down.
/// # Safety
///
/// This assumes that it's in complete control over the memory storing the sprite palette,
/// sprite tiles and OAM.
/// Any usage of these memory areas without going through the methods/handles provided by
/// this struct is UB.
///
/// Using more than one HWSpriteAllocator is also UB.
pub struct HWSpriteAllocator {
    /// List of 32 byte regions in object VRAM (1 tile per region),
    /// as well as how many OAM sprites use that particular tile.
    allocation_map: Box<[(u16, SpriteBlockState); 1024]>,
    /// Maps the hash of a sprite's tile data to a slot, if any.
    /// Note that we don't use the sprite data directly here, in order to avoid dealing with lifetimes.
    allocation_hashmap: HashMap<u64, usize, BuildHasherDefault<XxHash64>>,
    /// The hasher used for initially hashing the sprite data. This hash is what's stored in the HashMap.
    hasher: XxHash64,
    /// Sprite palette.
    palette: Box<[Color; 256]>,
    /// List keeping track of which slots in OAM are free.
    oam_occupied_list: Box<[bool; 128]>,
    /// See `hide_all_push()` and `show_all_pop()` docs for details
    sprite_visibility_stack: Vec<Box<[bool; 128]>>,
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

        let entries = Box::new([(0, SpriteBlockState::Unused); 1024]);
        let pal = Box::new(palette_as_gba_colors);
        let oam_occupied_list = Box::new([false; 128]);

        let hashmap: HashMap<u64, usize, BuildHasherDefault<XxHash64>> = Default::default();
        let hasher_builder: BuildHasherDefault<XxHash64> = Default::default();
        let hasher = hasher_builder.build_hasher();
        let sprite_visibility_stack: Vec<Box<[bool; 128]>> = Vec::new();
        return HWSpriteAllocator {
            allocation_map: entries,
            allocation_hashmap: hashmap,
            hasher,
            palette: pal,
            oam_occupied_list,
            sprite_visibility_stack,
        };
    }

    /// Initialize the allocator by copying the palette into VRAM.
    ///  
    /// Note that you're still required to manually enable object display in DISPCNT in order to see the sprites.
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

    /// Allocate the given sprite in VRAM from data contained in the file with the given name.
    /// A sprite size has to be supplied because several different shapes of sprites share the same size.
    /// TODO: Store sprite size info in the filesystem, so there's no need to keep track of it in code
    pub fn alloc_from_fs_file(
        &mut self,
        filename: &str,
        sprite_size: HWSpriteSize,
    ) -> Result<HWSpriteHandle, HWSpriteAllocError> {
        match crate::FS.get_file_data_by_name_as_u32_slice(filename) {
            Ok(sprite_data) => return self.alloc(sprite_data, sprite_size),
            Err(gbfs_err) => return Err(HWSpriteAllocError::File(gbfs_err)),
        }
    }

    /// Allocate the given sprite in VRAM.
    pub fn alloc(
        &mut self,
        sprite_data: &[u32],
        sprite_size: HWSpriteSize,
    ) -> Result<HWSpriteHandle, HWSpriteAllocError> {
        // Check whether the sprite is already in VRAM by comparing it's hash
        // FIXME: I think we don't correctly understand the Hasher interface, as identical
        // sprites return different hashes. Maybe using by_address is part of the solution.
        sprite_data.hash(&mut self.hasher);
        let sprite_hash = self.hasher.finish();
        let starting_vram_tile_id: usize;
        debug_log!(
            Subsystems::HWSprite,
            "Allocating sprite with hash {:?}",
            sprite_hash
        );
        if self.allocation_hashmap.contains_key(&sprite_hash) {
            debug_log!(
                Subsystems::HWSprite,
                "Sprite already present, not actually allocating"
            );
            starting_vram_tile_id = *self.allocation_hashmap.get(&sprite_hash).unwrap();
            self.allocation_map[starting_vram_tile_id].0 += 1;
        } else {
            debug_log!(
                Subsystems::HWSprite,
                "Sprite not present, actually allocating"
            );

            // Find first spot with enough contiguous free blocks to hold the sprite
            let num_32b_blocks = sprite_size.to_num_of_32_byte_blocks();
            starting_vram_tile_id = self.find_contiguous_free_blocks(num_32b_blocks)?;

            debug_log!(
                Subsystems::HWSprite,
                "Beginning allocation at block #{} for {} block sprite",
                starting_vram_tile_id,
                num_32b_blocks
            );

            // Copy the sprite into VRAM using DMA
            sprite_dma::dma_copy_sprite(sprite_data, starting_vram_tile_id, sprite_size);

            // Mark blocks as occupied, with a reference count of 1
            self.allocation_map[starting_vram_tile_id] = (1, SpriteBlockState::Used);
            for i in 1..num_32b_blocks {
                self.allocation_map[starting_vram_tile_id + i] = (1, SpriteBlockState::Continue);
            }

            // Insert new sprite's hash into hashmap
            self.allocation_hashmap
                .insert(sprite_hash, starting_vram_tile_id);
        }

        // Assign a slot in OAM
        let oam_slot = self.find_free_oam_slot()?;
        self.oam_occupied_list[oam_slot] = true;
        let (size, shape) = sprite_size.to_obj_size_and_shape();

        self.prepare_oam_slot(
            (starting_vram_tile_id * 2).try_into().unwrap(),
            oam_slot,
            size,
            shape,
        );
        return Ok(HWSpriteHandle {
            sprite_size,
            starting_block: starting_vram_tile_id,
            oam_slot,
            data_hash: sprite_hash,
        });
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
        ); // Identity matrix, ensures that the last use's affine transform is wiped
        oam::write_obj_attributes(
            oam_slot,
            oam::ObjectAttributes {
                attr0: oam::OBJAttr0::new()
                    .with_obj_rendering(oam::ObjectRender::Disabled)
                    .with_obj_shape(obj_shape)
                    .with_is_8bpp(true),
                attr1: oam::OBJAttr1::new().with_obj_size(obj_size),
                attr2: oam::OBJAttr2::new()
                    .with_tile_id(starting_vram_tile_id)
                    .with_priority(1),
            },
        );
    }
    /// Find a free slot in OAM.
    /// If none are available, panic.
    fn find_free_oam_slot(&self) -> Result<usize, HWSpriteAllocError> {
        match self.oam_occupied_list.iter().position(|&x| !x) {
            Some(pos) => Ok(pos),
            None => return Err(HWSpriteAllocError::OAMFull),
        }
    }

    /// Return the index of the beginning of the first area in the allocation map
    /// with sufficient space.
    fn find_contiguous_free_blocks(&self, num_blocks: usize) -> Result<usize, HWSpriteAllocError> {
        for (i, (_refcount, block)) in self.allocation_map.iter().enumerate() {
            if *block == SpriteBlockState::Unused {
                let mut free_blocks: usize = 1;
                for j in 1..num_blocks {
                    if self.allocation_map[i + j].1 == SpriteBlockState::Unused {
                        free_blocks += 1;
                    } else {
                        break;
                    }
                }
                if free_blocks >= num_blocks {
                    return Ok(i);
                }
            }
        }
        return Err(HWSpriteAllocError::VRAMFull);
    }

    /// Drop the allocation of the given sprite.
    /// Note that the sprite still exists in VRAM until overwritten (or reused if the refcount is not 0),
    /// but is marked as inactive in OAM and therefore not displayed.
    pub fn free(&mut self, handle: HWSpriteHandle) {
        handle.set_visibility(false);
        self.oam_occupied_list[handle.oam_slot] = false;

        // Decrease refcount of first block
        self.allocation_map[handle.starting_block].0 -= 1;
        // Decrease refcount of  all blocks that are marked CONTINUE after the first block
        // (therefore part of this sprite)
        let mut i = 1;
        loop {
            if self.allocation_map[handle.starting_block + i].1 == SpriteBlockState::Continue {
                self.allocation_map[handle.starting_block + i].0 -= 1;
            } else {
                break;
            }
            i += 1;
        }

        // Mark starting block with refcount of 0 as free
        if self.allocation_map[handle.starting_block].0 == 0 {
            debug_log!(Subsystems::HWSprite, "Refcount reached 0, freeing sprite");
            self.allocation_map[handle.starting_block] = (0, SpriteBlockState::Unused);
            self.allocation_hashmap.remove(&handle.data_hash);
        }
        // Mark all CONTINUE blocks with refcount of 0 as free as well
        let mut i = 1;
        loop {
            if self.allocation_map[handle.starting_block + i].1 == SpriteBlockState::Continue {
                if self.allocation_map[handle.starting_block + i].0 == 0 {
                    self.allocation_map[handle.starting_block + i] = (0, SpriteBlockState::Unused);
                }
            } else {
                break;
            }
            i += 1;
        }
    }

    /// Record which sprites are live on an internal stack and hide all sprites.
    ///
    /// This is very useful to, for example, display a sprite-based menu
    /// without interference from game sprites and then return
    /// to the main game, restoring game sprites on screen without having to reinitialize each sprite.
    pub fn hide_sprites_push(&mut self) {
        self.sprite_visibility_stack
            .push(self.oam_occupied_list.clone());
        for (slot, is_occupied) in self.oam_occupied_list.iter().enumerate() {
            if *is_occupied {
                let mut attrs = oam::read_obj_attributes(slot).unwrap();
                attrs.attr0 = attrs.attr0.with_obj_rendering(oam::ObjectRender::Disabled);
                oam::write_obj_attributes(slot, attrs);
            }
        }
    }

    /// Restore which sprites are live on an internal stack.
    /// If `hide_sprites_push()` was not called beforehand this will error.
    ///
    /// This is very useful to, for example, display a sprite-based menu
    /// without interference from game sprites and then return
    /// to the main game, restoring game sprites on screen without having to reinitialize each sprite.
    pub fn show_sprites_pop(&mut self) -> Result<(), HWSpriteAllocError> {
        match self.sprite_visibility_stack.pop() {
            Some(list) => self.oam_occupied_list = list,
            None => return Err(HWSpriteAllocError::SpriteVisibilityStackEmpty),
        }
        for (slot, is_occupied) in self.oam_occupied_list.iter().enumerate() {
            if *is_occupied {
                let mut attrs = oam::read_obj_attributes(slot).unwrap();
                attrs.attr0 = attrs.attr0.with_obj_rendering(oam::ObjectRender::Normal);
                oam::write_obj_attributes(slot, attrs);
            } else {
                let mut attrs = oam::read_obj_attributes(slot).unwrap();
                attrs.attr0 = attrs.attr0.with_obj_rendering(oam::ObjectRender::Disabled);
                oam::write_obj_attributes(slot, attrs);
            }
        }
        return Ok(());
    }
}
