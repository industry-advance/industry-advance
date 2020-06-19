use crate::components::{
    component_utils::*, InputComponent, MovementComponent, PositionComponent, SpriteComponent,
};
use crate::debug_log::*;
use crate::entities;
use crate::entities::player;
use crate::map::{Map, Maps};
use crate::sprite::HWSpriteAllocator;
use crate::systems::{item_movement_system, mining_system, InputSystem, MovementSystem};
use crate::text::TextEngine;

use crate::FS;

use alloc::boxed::Box;
use alloc::vec::Vec;

use gba::io::display::{DISPCNT, VBLANK_SCANLINE, VCOUNT};
use gbfs_rs::Filename;
use tiny_ecs::Entities;

#[derive(Debug, PartialEq)]
enum GameMode {
    // State in which the player controls the avatar directly and all game systems are ticked.
    TimeRunning,
    // State in which the player does not control the avatar, but navigates menus and builds while
    // the rest of the game logic is paused.
    TimeStopped,
}
#[allow(dead_code)] // Some struct fields not used after construction right now, remove once changed
pub(crate) struct Game {
    sprite_alloc: HWSpriteAllocator,
    map: Box<Map>,
    entities: Entities,
    player_id: usize,
    cursor_id: usize,
    live_entity_ids: Vec<usize>,
    input_system: InputSystem,
    text_engine: TextEngine,
    game_mode: GameMode,
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

        let cursor_id = entities::add_cursor(&mut e, &mut sprite_allocator)
            .expect("Failed to initialize cursor entity");
        live_entity_ids.push(cursor_id);

        // Initialize a copper wall for testing
        let copper_wall_id = entities::add_copper_wall(&mut e, &mut sprite_allocator)
            .expect("Failed to initialize copper wall entity");
        live_entity_ids.push(copper_wall_id);

        // Initialize a miner and deposit container for testing
        let container_id = entities::add_container(&mut e, &mut sprite_allocator)
            .expect("Failed to initialize container entity");
        live_entity_ids.push(container_id);
        let miner_id = entities::add_mechanical_drill(&mut e, &mut sprite_allocator, container_id)
            .expect("Failed to initialize mechanical drill entity");
        live_entity_ids.push(miner_id);

        // Put the player at the center of the screen
        let mut components = e.borrow_mut::<SpriteComponent>().unwrap();
        let player_sprite_handle = components.get_mut(player_id).unwrap().get_handle();
        player_sprite_handle.set_visibility(true);
        drop(components);

        // Initialize the text engine
        let text_engine = TextEngine::with_default_font_and_screenblock();

        return Game {
            sprite_alloc: sprite_allocator,
            map,
            entities: e,
            player_id,
            cursor_id,
            live_entity_ids,
            input_system: InputSystem::init(),
            text_engine,
            game_mode: GameMode::TimeRunning,
        };
    }

    fn update(&mut self) {
        // Process player input
        let start_pressed = self
            .input_system
            .tick(&mut self.entities, &self.live_entity_ids)
            .expect("Failed to tick input system");
        // Start is the button for switching between game modes
        if start_pressed {
            use GameMode::*;
            match self.game_mode {
                // If we're in stopped mode, we want the camera to follow the cursor,
                // and we want to take away the player's InputComponent and give it to the cursor.
                TimeRunning => {
                    self.game_mode = GameMode::TimeStopped;
                    // Copy the movement component to the cursor
                    clone_component::<MovementComponent>(
                        self.player_id,
                        self.cursor_id,
                        &mut self.entities,
                    )
                    .unwrap();

                    // Move the input component to the cursor
                    move_component::<InputComponent>(
                        self.player_id,
                        self.cursor_id,
                        &mut self.entities,
                    )
                    .unwrap();

                    // Copy the position component to the cursor
                    clone_component::<PositionComponent>(
                        self.player_id,
                        self.cursor_id,
                        &mut self.entities,
                    )
                    .unwrap();

                    // Ensure that the camera doesn't try to follow the player anymore
                    let mut movement_components =
                        self.entities.borrow_mut::<MovementComponent>().unwrap();
                    let mut player_movement_component =
                        movement_components.get_mut(self.player_id).unwrap();
                    player_movement_component.input_controlled = false;
                    player_movement_component.keep_camera_centered_on = false;

                    // Cursor has to be made visible
                    let mut sprite_components =
                        self.entities.borrow_mut::<SpriteComponent>().unwrap();
                    let mut cursor_sprite_component =
                        sprite_components.get_mut(self.cursor_id).unwrap();
                    let mut handle = cursor_sprite_component.get_handle();
                    handle.set_visibility(true);
                }
                TimeStopped => {
                    self.game_mode = GameMode::TimeRunning;

                    // Move the input component back to the player
                    move_component::<InputComponent>(
                        self.cursor_id,
                        self.player_id,
                        &mut self.entities,
                    )
                    .unwrap();

                    // We don't want the player teleporting to the cursor's position,
                    // therefore we don't copy the PositionComponent back.
                    self.entities
                        .rm_component::<PositionComponent>(self.cursor_id)
                        .unwrap();
                    // We also don't want the camera to follow the cursor anymore, either.
                    self.entities
                        .rm_component::<MovementComponent>(self.cursor_id)
                        .unwrap();

                    // Recenter player on the screen
                    let mut sprite_components =
                        self.entities.borrow_mut::<SpriteComponent>().unwrap();
                    let mut player_sprite_component =
                        sprite_components.get_mut(self.cursor_id).unwrap();
                    let mut handle = player_sprite_component.get_handle();
                    handle.set_x_pos(player::INITIAL_PLAYER_ONSCREEN_POS_X);
                    handle.set_y_pos(player::INITIAL_PLAYER_ONSCREEN_POS_X);

                    // Make camera follow the player again
                    let mut movement_components =
                        self.entities.borrow_mut::<MovementComponent>().unwrap();
                    let mut player_movement_component =
                        movement_components.get_mut(self.player_id).unwrap();
                    player_movement_component.input_controlled = true;
                    player_movement_component.keep_camera_centered_on = true;

                    // Cursor has to be made invisible again
                    let mut cursor_sprite_component =
                        sprite_components.get_mut(self.cursor_id).unwrap();
                    let mut handle = cursor_sprite_component.get_handle();
                    handle.set_visibility(false);
                }
            }
        }

        // Simulate all game systems
        if self.game_mode == GameMode::TimeRunning {
            // Simulate
            MovementSystem::tick(&mut self.entities, &self.live_entity_ids, &mut self.map)
                .expect("Failed to tick movement system");

            // Update display

            // Update miners
            mining_system::tick(&mut self.entities, &self.live_entity_ids);

            // Perform inventory transfers
            item_movement_system::tick(&mut self.entities);
        // Only simulate systems needed for moving the cursor and building
        } else {
            MovementSystem::tick(&mut self.entities, &self.live_entity_ids, &mut self.map)
                .expect("Failed to tick movement system");
        }
    }
}
