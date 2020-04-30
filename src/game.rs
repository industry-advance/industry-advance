use crate::background::SCREENBLOCK_SIZE_IN_U8;
use crate::components::SpriteComponent;
use crate::entities;
use crate::map::Map;
use crate::sprite::HWSpriteAllocator;
use crate::systems::{InputSystem, MovementSystem};
use crate::FS;

use alloc::boxed::Box;
use alloc::{vec, vec::Vec};
use core::convert::TryInto; // One is a macro, the other a namespace

use crate::ewram_alloc;

use gba::io::display::{VBLANK_SCANLINE, VCOUNT};
use gbfs_rs::Filename;
use gbfs_rs::*;
use tiny_ecs::Entities;

#[allow(dead_code)] // Some struct fields not used after construction right now, remove once changed
pub(crate) struct Game<'a> {
    sprite_alloc: HWSpriteAllocator,
    map: Map<'a>,
    entities: Entities,
    live_entity_ids: Vec<usize>,
    input_system: InputSystem,
}

impl<'a> Game<'a> {
    pub(crate) fn run(&mut self) {
        loop {
            self.update();
            // For now, tick once every vblank
            // TODO: More power efficiency w/ interrupt
            while VCOUNT.read() >= VBLANK_SCANLINE {}
        }
    }

    /// Creates and initializes a new game.
    pub fn init() -> Game<'a> {
        gba::debug!("Loading game data from FS");

        // Initialize hardware sprite management

        gba::debug!("Initializing sprite allocator");
        let mut sprite_allocator = HWSpriteAllocator::new(
            &FS.get_file_data_by_name_as_u16_slice(
                Filename::try_from_str("sprite_sharedPal").unwrap(),
            )
            .unwrap(),
        );
        sprite_allocator.init();

        let map_0: &'static [u8] = FS
            .get_file_data_by_name(Filename::try_from_str("testmap_0Map").unwrap())
            .unwrap();
        let map_1: &'static [u8] = FS
            .get_file_data_by_name(Filename::try_from_str("testmap_1Map").unwrap())
            .unwrap();
        let map_2: &'static [u8] = FS
            .get_file_data_by_name(Filename::try_from_str("testmap_2Map").unwrap())
            .unwrap();
        let map_3: &'static [u8] = FS
            .get_file_data_by_name(Filename::try_from_str("testmap_3Map").unwrap())
            .unwrap();
        let mut tilemaps: Vec<&'static [u8]> = Vec::new();

        tilemaps.push(map_0);
        tilemaps.push(map_1);
        tilemaps.push(map_2);
        tilemaps.push(map_3);

        let pal: &'a [u16] = FS
            .get_file_data_by_name_as_u16_slice(Filename::try_from_str("map_sharedPal").unwrap())
            .unwrap();
        let tiles: &'a [u32] = FS
            .get_file_data_by_name_as_u32_slice(Filename::try_from_str("map_sharedTiles").unwrap())
            .unwrap();
        let map = Map::new_map(pal, 2, 2, tiles, tilemaps);

        // Initialize the ECS
        let mut e = Entities::new(Some(256), Some(24));
        let mut live_entity_ids = Vec::new();

        // Initialize the player entity
        let player_id = entities::add_player(
            &mut e,
            &mut sprite_allocator,
            &FS.get_file_data_by_name_as_u32_slice(
                Filename::try_from_str("dart_shipTiles").unwrap(),
            )
            .unwrap(),
        )
        .expect("Failed to initialize player entity");
        live_entity_ids.push(player_id);
        // Put the player at the center of the screen
        let mut components = e.borrow_mut::<SpriteComponent>().unwrap();
        let player_sprite_handle = components.get_mut(player_id).unwrap().get_handle();
        player_sprite_handle.set_visibility(true);
        player_sprite_handle.set_x_pos(96);
        player_sprite_handle.set_y_pos(64);
        drop(components);

        return Game {
            sprite_alloc: sprite_allocator,
            map: map,
            entities: e,
            live_entity_ids: live_entity_ids,
            input_system: InputSystem::init(),
        };
    }

    fn update(&mut self) {
        // Process player input
        self.input_system
            .tick(&mut self.entities, &self.live_entity_ids)
            .expect("Failed to tick input system");

        // Simulate
        MovementSystem::tick(&mut self.entities, &self.live_entity_ids)
            .expect("Failed to tick movement system");

        // Update display
    }
}
