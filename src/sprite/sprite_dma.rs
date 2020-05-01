use super::HWSpriteSize;
use core::convert::TryInto;
use gba::io::dma;

const SPRITE_CHARBLOCK_BASE_ADDR: usize = 0x0601_0000;
const NUM_BYTES_PER_SPRITE_TILE_SLOT: usize = 64;

/// Copies a sprite into the given sprite slot using DMA.
/// The start slot can range from 0 to 1024, and represents an offset into charblocks 4 and 5 in VRAM,
/// which are usable for sprite tiles.
pub fn dma_copy_sprite(sprite_tile_data: &[u32], start_slot: usize, sprite_size: HWSpriteSize) {
    let expected_num_sprite_u32s: usize = (sprite_size.to_size_in_bytes() / 4).try_into().unwrap();
    // A single sprite tile consists of 16 u32's worth of data
    let num_occupied_slots: usize = expected_num_sprite_u32s / 16;
    // Make sure we don't DMA into random memory
    if (start_slot + num_occupied_slots) >= 1024 {
        panic!(
            "Attempt to write to invalid sprite start slot (Start slot: {}), sprite size: {}",
            start_slot, num_occupied_slots
        );
    }
    // Ensure we were given enough data
    if sprite_tile_data.len() != expected_num_sprite_u32s {
        panic!("Attempt to create hardware sprite with incorrect amount of data for given size");
    }
    // Perform transfer
    let dest_addr = SPRITE_CHARBLOCK_BASE_ADDR + (start_slot * NUM_BYTES_PER_SPRITE_TILE_SLOT);
    unsafe {
        dma::DMA3::set_source(sprite_tile_data.as_ptr());
        dma::DMA3::set_dest((dest_addr) as *mut u32);
        dma::DMA3::set_count(sprite_tile_data.len().try_into().unwrap());
        dma::DMA3::set_control(
            dma::DMAControlSetting::new()
                .with_enabled(true)
                .with_use_32bit(true),
        );
    }
}
