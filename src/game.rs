use crate::components::SpriteComponent;
use crate::entities;
use crate::map::Map;
use crate::sprite::{HWSprite, HWSpriteAllocator, HWSpriteSize};
use crate::systems::{InputSystem, MovementSystem};
use alloc::vec::Vec;

use gba::{
    io::background::{BGSize, BackgroundControlSetting, BG0CNT},
    io::display::{DisplayControlSetting, DisplayMode, DISPCNT, VBLANK_SCANLINE, VCOUNT},
    mgba, vram,
};

use tiny_ecs::Entities;

pub(crate) struct Game {
    sprite_alloc: HWSpriteAllocator,
    map: Map<'static>,
    entities: Entities,
    live_entity_ids: Vec<usize>,
    input_system: InputSystem,
}

impl Game {
    pub fn new(sprite_alloc: HWSpriteAllocator) -> Game {
        // Create a new ECS
        let e = Entities::new(Some(256), Some(24));
        return Game {
            sprite_alloc: sprite_alloc,
            map: Map::new_testmap(),
            entities: e,
            live_entity_ids: Vec::with_capacity(128),
            input_system: InputSystem::init(),
        };
    }
    pub(crate) fn run(&mut self) {
        self.init();
        loop {
            self.update();
            // For now, tick once every vblank
            // TODO: More power efficiency w/ interrupt
            while VCOUNT.read() >= VBLANK_SCANLINE {}
        }
    }
    pub fn init(&mut self) {
        // Configure video mode
        const MODE: DisplayControlSetting = DisplayControlSetting::new()
            .with_mode(DisplayMode::Mode0)
            .with_bg0(true)
            .with_obj(true)
            .with_oam_memory_1d(true);
        DISPCNT.write(MODE);

        // Initialize the player entity
        let player_id = entities::add_player(&mut self.entities, &mut self.sprite_alloc)
            .expect("Failed to initialize player entity");
        self.live_entity_ids.push(player_id);
        // Put the player at the center of the screen
        let mut components = self.entities.borrow_mut::<SpriteComponent>().unwrap();
        let mut player_sprite_handle = components.get_mut(player_id).unwrap().get_handle();
        player_sprite_handle.set_visibility(true);
        player_sprite_handle.set_x_pos(96);
        player_sprite_handle.set_y_pos(64);
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
