use crate::components::SpriteComponent;
use crate::debug_log::*;
use crate::entities;
use crate::map::{Map, Maps};
use crate::sprite::HWSpriteAllocator;
use crate::systems::{mining_system, InputSystem, MovementSystem};
use crate::text::TextEngine;

use crate::FS;

use alloc::boxed::Box;
use alloc::vec::Vec;

use gba::io::display::{DISPCNT, VBLANK_SCANLINE, VCOUNT};
use gbfs_rs::Filename;
use tiny_ecs::Entities;

#[allow(dead_code)] // Some struct fields not used after construction right now, remove once changed
pub(crate) struct Game {
    sprite_alloc: HWSpriteAllocator,
    map: Box<Map>,
    entities: Entities,
    live_entity_ids: Vec<usize>,
    input_system: InputSystem,
    text_engine: TextEngine,
}

impl Game {
    pub(crate) fn run(&mut self) {
        loop {
            self.update();
            // For now, tick once every vblank
            // TODO: More power efficiency w/ interrupt
            while VCOUNT.read() >= VBLANK_SCANLINE {}
        }
    }

    /// Creates and initializes a new game.
    pub fn init() -> Game {
        debug_log!(Subsystems::Game, "Loading game data from FS");

        // Initialize hardware sprite management
        debug_log!(Subsystems::Game, "Initializing sprite allocator");
        let mut sprite_allocator = HWSpriteAllocator::new(
            &FS.get_file_data_by_name_as_u16_slice(
                Filename::try_from_str("sprite_sharedPal").unwrap(),
            )
            .unwrap(),
        );
        sprite_allocator.init();

        // Create a map
        let maps = Maps::read_map_data();
        let map_entry = maps.get_by_name("craters").unwrap();
        debug_log!(Subsystems::Game, "Loading map {}", map_entry.name);
        let map = map_entry.get_map();

        // Ensure sprites are visible
        DISPCNT.write(DISPCNT.read().with_obj(true).with_oam_memory_1d(true));

        // Initialize the ECS
        let mut e = Entities::new(Some(256), Some(24));
        let mut live_entity_ids = Vec::new();

        // Initialize the player entity
        let player_id = entities::add_player(&mut e, &mut sprite_allocator)
            .expect("Failed to initialize player entity");
        live_entity_ids.push(player_id);

        // Initialize a copper wall for testing
        let copper_wall_id = entities::add_copper_wall(&mut e, &mut sprite_allocator)
            .expect("Failed to initialize copper wall entity");
        live_entity_ids.push(copper_wall_id);

        // Initialize a miner for testing
        let miner_id = entities::add_mechanical_drill(&mut e, &mut sprite_allocator)
            .expect("Failed to initialize mechanical drill entity");
        live_entity_ids.push(miner_id);

        // Put the player at the center of the screen
        let mut components = e.borrow_mut::<SpriteComponent>().unwrap();
        let player_sprite_handle = components.get_mut(player_id).unwrap().get_handle();
        player_sprite_handle.set_visibility(true);
        drop(components);

        // Initialize the text engine
        let text_engine = TextEngine::with_default_font();
        return Game {
            sprite_alloc: sprite_allocator,
            map,
            entities: e,
            live_entity_ids,
            input_system: InputSystem::init(),
            text_engine,
        };
    }

    fn update(&mut self) {
        // Process player input
        self.input_system
            .tick(&mut self.entities, &self.live_entity_ids)
            .expect("Failed to tick input system");

        // Simulate
        MovementSystem::tick(&mut self.entities, &self.live_entity_ids, &mut self.map)
            .expect("Failed to tick movement system");

        // Update display

        // Update miners
        mining_system::tick(&mut self.entities, &self.live_entity_ids);
    }
}
