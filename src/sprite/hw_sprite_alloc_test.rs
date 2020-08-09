use super::*;
use crate::test::test;
use crate::FS;

// TODO: Write tests

// Generic test setup code
#[cfg(test)]
fn test_setup() -> HWSpriteAllocator {
    let pal = FS
        .get_file_data_by_name_as_u16_slice("sprite_sharedPal")
        .unwrap();
    let mut sprite_allocator = super::HWSpriteAllocator::new(&pal);
    sprite_allocator.init();
    return sprite_allocator;
}
/// Ensure initializing the object works
#[test_case]
fn test_sprite_alloc_init() {
    test(
        &|| {
            test_setup();
        },
        "test_sprite_alloc_init",
        "ensure HW sprite alloc init works",
    );
}

/// Ensure that OAM fits exactly 128 sprites
#[test_case]
fn test_sprite_alloc_fill_oam() {
    test(
        &|| {
            let mut alloc = test_setup();
            for _ in 0..128 {
                alloc
                    .alloc_from_fs_file("copper_wallTiles", HWSpriteSize::SixteenBySixteen)
                    .unwrap();
            }
        },
        "test_sprite_alloc_fill_oam",
        "ensure all OAM slots can be filled",
    );
}

/// Ensure that when OAM is full adding a new sprite fails
#[test_case]
fn test_sprite_alloc_overfill_oam() {
    test(
        &|| {
            let mut alloc = test_setup();
            for _ in 0..128 {
                alloc
                    .alloc_from_fs_file("copper_wallTiles", HWSpriteSize::SixteenBySixteen)
                    .unwrap();
            }
            match alloc.alloc_from_fs_file("copper_wallTiles", HWSpriteSize::SixteenBySixteen) {
                Ok(_) => panic!("Expected allocation to fail with full OAM, but it didn't"),
                Err(_) => return,
            };
        },
        "test_sprite_alloc_overfill_oam",
        "ensure that when OAM is full adding a new sprite fails",
    );
}

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
