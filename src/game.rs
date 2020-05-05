use crate::components::SpriteComponent;
use crate::entities;
use crate::map::Map;
use crate::sprite::HWSpriteAllocator;
use crate::systems::{InputSystem, MovementSystem};

use crate::FS;

use alloc::vec::Vec;

use gba::io::display::{DISPCNT, VBLANK_SCANLINE, VCOUNT};
use gbfs_rs::Filename;
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
        gba::debug!("[GAME] Loading game data from FS");

        // Initialize hardware sprite management
        gba::debug!("Initializing sprite allocator");
        let mut sprite_allocator = HWSpriteAllocator::new(
            &FS.get_file_data_by_name_as_u16_slice(
                Filename::try_from_str("sprite_sharedPal").unwrap(),
            )
            .unwrap(),
        );
        sprite_allocator.init();

        // Create a map
        let map = Map::load_test_map_from_fs();

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

        // Put the player at the center of the screen
        let mut components = e.borrow_mut::<SpriteComponent>().unwrap();
        let player_sprite_handle = components.get_mut(player_id).unwrap().get_handle();
        player_sprite_handle.set_visibility(true);
        drop(components);

        return Game {
            sprite_alloc: sprite_allocator,
            map,
            entities: e,
            live_entity_ids,
            input_system: InputSystem::init(),
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
    }
}
