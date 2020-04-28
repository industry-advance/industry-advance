use crate::FS;
use gbfs_rs::FilenameString;

// TODO: Write tests

/// Ensure initializing the object works
#[test_case]
fn test_init() {
    // Load palette from filesystem
    let pal = FS
        .get_file_by_name(FilenameString::try_from_str("sprite_sharedPal").unwrap())
        .unwrap()
        .to_u16_vec();
    let mut sprite_allocator = super::HWSpriteAllocator::new(&pal);
    sprite_allocator.init();
}

/// Ensure that full OAM causes panic
#[test_case]
fn test_exhaust_oam() {}

/// Ensure that reclaiming OAM works
#[test_case]
fn test_reclaim_oam() {}

/// Ensure panic on VRAM exhaustion
#[test_case]
fn test_exhaust_vram() {}

/// Ensure reclaiming VRAM works
#[test_case]
fn test_reclaim_vram() {}
