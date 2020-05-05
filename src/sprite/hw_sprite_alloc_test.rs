use crate::FS;
use gbfs_rs::Filename;

// TODO: Write tests

/// Ensure initializing the object works
#[test_case]
fn test_init() {
    gba::debug!("[TEST] Testing HW sprite alloc init");
    // Load palette from filesystem
    let pal = FS
        .get_file_data_by_name_as_u16_slice(Filename::try_from_str("sprite_sharedPal").unwrap())
        .unwrap();
    let mut sprite_allocator = super::HWSpriteAllocator::new(&pal);
    sprite_allocator.init();
    gba::debug!("[TEST] Test passed");
}

/// Ensure that full OAM causes panic
#[test_case]
fn test_exhaust_oam() {}

/// Ensure that reclaiming OAM works
#[test_case]
fn test_reclaim_oam() {
    // Alloc the maximum number of sprites
}

/// Ensure panic on VRAM exhaustion
#[test_case]
fn test_exhaust_vram() {}

/// Ensure reclaiming VRAM works
#[test_case]
fn test_reclaim_vram() {}
