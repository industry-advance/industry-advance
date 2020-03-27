use crate::assets::sprites::palette::PAL;

// TODO: Write tests

/// Ensure initializing the object works
#[test_case]
fn test_init() {
    let mut sprite_allocator = super::HWSpriteAllocator::new(&PAL);
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
