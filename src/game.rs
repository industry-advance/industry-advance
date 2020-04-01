use crate::assets::maps::*;
use crate::assets::sprites;

use crate::background::SCREENBLOCK_SIZE_IN_U16;
use crate::components::SpriteComponent;
use crate::entities;
use crate::map::Map;
use crate::sprite::HWSpriteAllocator;
use crate::systems::{InputSystem, MovementSystem};

use alloc::{vec, vec::Vec}; // One is a macro, the other a namespace

use gba::io::display::{VBLANK_SCANLINE, VCOUNT};
use tiny_ecs::Entities;

#[allow(dead_code)] // Some struct fields not used after construction right now, remove once changed
pub(crate) struct Game {
    sprite_alloc: HWSpriteAllocator,
    map: Map<'static>,
    entities: Entities,
    live_entity_ids: Vec<usize>,
    input_system: InputSystem,
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
        // Initialize hardware sprite management
        let mut sprite_allocator = HWSpriteAllocator::new(&sprites::palette::PAL);
        sprite_allocator.init();

        // Initialize the background
        let tilemaps: Vec<&[u16; SCREENBLOCK_SIZE_IN_U16]> = vec![&maps::TESTMAP_MAP];
        let map = Map::new_map(&palette::TESTMAP_PAL, 8, 8, &maps::TESTMAP_TILES, tilemaps);

        // Initialize the ECS
        let mut e = Entities::new(Some(256), Some(24));
        let mut live_entity_ids = Vec::new();

        // Initialize the player entity
        let player_id = entities::add_player(&mut e, &mut sprite_allocator)
            .expect("Failed to initialize player entity");
        live_entity_ids.push(player_id);
        // Put the player at the center of the screen
        let mut components = e.borrow_mut::<SpriteComponent>().unwrap();
        let mut player_sprite_handle = components.get_mut(player_id).unwrap().get_handle();
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
