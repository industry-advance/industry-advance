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
use core::hash::{BuildHasher, BuildHasherDefault, Hash, Hasher};

use super::{sprite_dma, HWSpriteSize};
use alloc::boxed::Box;
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
pub(crate) struct HWSpriteAllocator {
    /// List of 32 byte regions in object VRAM (1 tile per region),
    /// as well as how many OAM sprites use that particular tile.
    allocation_map: Box<[(u16, SpriteBlockState); 1024]>,
    /// Maps the hash of a sprite's tile data to a slot, if any.
    /// Note that we don't use the sprite data directly here, in order to avoid dealing with lifetimes.
    allocation_hashmap: HashMap<u64, usize, BuildHasherDefault<XxHash64>>,
    /// The hasher used for initially hashing the sprite data. This hash is what's stored in the HashMap.
    hasher: XxHash64,
    palette: Box<[Color; 256]>,
    oam_occupied_list: Box<[bool; 128]>, // List keeping track of which slots in OAM are free
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
        return HWSpriteAllocator {
            allocation_map: entries,
            allocation_hashmap: hashmap,
            hasher,
            palette: pal,
            oam_occupied_list,
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
    ///
    /// Panics if the file doesn't exist.
    pub fn alloc_from_fs_file(
        &mut self,
        filename: &str,
        sprite_size: HWSpriteSize,
    ) -> HWSpriteHandle {
        let sprite_data = crate::FS
            .get_file_data_by_name_as_u32_slice(filename.try_into().unwrap())
            .expect("Failed to find sprite with given name in filesystem");
        return self.alloc(sprite_data, sprite_size);
    }

    /// Allocate the given sprite in VRAM.
    ///
    /// This will panic if insufficient space is available or too many sprites are already active.
    pub fn alloc(&mut self, sprite_data: &[u32], sprite_size: HWSpriteSize) -> HWSpriteHandle {
        // Check whether the sprite is already in VRAM by comparing it's hash
        sprite_data.hash(&mut self.hasher);
        let sprite_hash = self.hasher.finish();
        let starting_vram_tile_id: usize;
        gba::info!(
            "[HW_SPRITE_ALLOC] Allocating sprite with hash {:?}",
            sprite_hash
        );
        if self.allocation_hashmap.contains_key(&sprite_hash) {
            gba::info!("[ALLOC] Sprite already present, not actually allocating");
            starting_vram_tile_id = *self.allocation_hashmap.get(&sprite_hash).unwrap();
            self.allocation_map[starting_vram_tile_id].0 += 1;
        } else {
            gba::info!("[HW_SPRITE_ALLOC] Sprite not present, actually allocating");

            // Find first spot with enough contiguous free blocks to hold the sprite
            let num_32b_blocks = sprite_size.to_num_of_32_byte_blocks();
            starting_vram_tile_id = self
                .find_contiguous_free_blocks(num_32b_blocks)
                .expect("No contiguous free block of VRAM available to allocate hardware sprite");

            gba::info!(
                "[HW_SPRITE_ALLOC] Beginning allocation at block #{} for {} block sprite",
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
        let oam_slot = self.find_free_oam_slot();
        self.oam_occupied_list[oam_slot] = true;
        let (size, shape) = sprite_size.to_obj_size_and_shape();

        self.allocate_oam_slot(
            starting_vram_tile_id.try_into().unwrap(),
            oam_slot,
            size,
            shape,
        );
        return HWSpriteHandle {
            starting_block: starting_vram_tile_id,
            oam_slot,
            data_hash: sprite_hash,
        };
    }

    /// Prepares a slot in OAM for the sprite.
    fn allocate_oam_slot(
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
        match self.oam_occupied_list.iter().position(|&x| !x) {
            Some(pos) => pos,
            None => panic!("Attempt to create sprite when OAM is full"),
        }
    }

    /// Return the index of the beginning of the first area in the allocation map
    /// with sufficient space.
    fn find_contiguous_free_blocks(&self, num_blocks: usize) -> Option<usize> {
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
                    return Some(i);
                }
            }
        }
        return None;
    }

    /// Drop the allocation of the given sprite.
    /// Note that the sprite still exists in VRAM until overwritten (or reused if the refcount is not 0),
    /// but is marked as inactive in OAM and therefore not displayed.
    #[allow(dead_code)]
    pub fn free(&mut self, handle: HWSpriteHandle) {
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
            gba::info!("[SPRITE] Refcount reached 0, freeing sprite");
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

        // Set the sprite to not render in OAM
        let mut attrs = oam::read_obj_attributes(handle.oam_slot).unwrap();
        attrs.attr0 = attrs.attr0.with_obj_rendering(oam::ObjectRender::Disabled);
        oam::write_obj_attributes(handle.oam_slot, attrs);

        // Mark slot in OAM as available
        self.oam_occupied_list[handle.oam_slot] = false;
    }
}

/// A handle to a hardware sprite allocated in VRAM/OAM.
/// Also provides some wrappers to avoid the tedium of having to get an object, modify it, and write it back
/// for commonly used object attributes.
pub(crate) struct HWSpriteHandle {
    starting_block: usize,
    data_hash: u64,
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
