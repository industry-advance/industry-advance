use crate::components::{component_utils::*, *};
use crate::debug_log::*;
use crate::entities;
use crate::entities::{cursor, player};
use crate::map::{Map, Maps};
use crate::sprite::HWSpriteAllocator;
use crate::systems::{
    building_system, item_movement_system, mining_system, InputSystem, MovementSystem,
};
use crate::window::Window;

use crate::FS;

use alloc::{boxed::Box, vec::Vec};

use gba::io::display::{DISPCNT, VBLANK_SCANLINE, VCOUNT};
use tiny_ecs::Entities;

/// Data which is needed to perform game mode switches.
#[derive(Copy, Clone, Debug)]
struct ModePersist {
    map_scroll_pos_x: u32,
    map_scroll_pos_y: u32,
}

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
    game_mode: GameMode,
    mode_persist: Option<ModePersist>,
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
            &FS.get_file_data_by_name_as_u16_slice("sprite_sharedPal")
                .unwrap(),
        );
        sprite_allocator.init();

        // Stop blanking the screen so that menus are visible
        DISPCNT.write(DISPCNT.read().with_force_vblank(false));

        // Ask the player which map they'd like
        let maps = Maps::read_map_data();
        let map_names: Vec<&str> = maps.maps.iter().map(|x| x.name.as_str()).collect();
        let mut win_menu = Window::new();
        win_menu.show();
        let choice_idx = win_menu.make_text_menu("Choose a map", &map_names);
        drop(win_menu);
        let map_entry = &maps.maps[choice_idx];
        // Create a map
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

        // Put the player at the center of the screen
        let mut components = e.borrow_mut::<SpriteComponent>().unwrap();
        let player_sprite_handle = components.get_mut(player_id).unwrap().get_handle();
        player_sprite_handle.set_visibility(true);
        drop(components);

        debug_log!(Subsystems::Game, "Init done. Starting game loop");

        return Game {
            sprite_alloc: sprite_allocator,
            map,
            entities: e,
            player_id,
            cursor_id,
            live_entity_ids,
            input_system: InputSystem::init(),
            game_mode: GameMode::TimeRunning,
            mode_persist: None,
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
            self.toggle_game_mode();
        }

        // Simulate all game systems
        if self.game_mode == GameMode::TimeRunning {
            // Simulate
            MovementSystem::tick(&mut self.entities, &self.live_entity_ids, &mut self.map)
                .expect("Failed to tick movement system");

            // Update miners
            mining_system::tick(&mut self.entities, &self.live_entity_ids);

            // Perform inventory transfers
            item_movement_system::tick(&mut self.entities);

        // Only simulate systems needed for moving the cursor and building
        } else {
            MovementSystem::tick(&mut self.entities, &self.live_entity_ids, &mut self.map)
                .expect("Failed to tick movement system");
            building_system::tick(
                &mut self.entities,
                &mut self.live_entity_ids,
                &mut self.sprite_alloc,
            );
        }
    }

    /// Switch between game modes.
    fn toggle_game_mode(&mut self) {
        use GameMode::*;
        match self.game_mode {
            // If we're in stopped mode, we want the camera to follow the cursor,
            // and we want to take away the player's InputComponent and give it to the cursor.
            TimeRunning => {
                self.game_mode = GameMode::TimeStopped;
                // Move the movement component to the cursor
                move_component::<MovementComponent>(
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

                // Move the inventory component to the cursor
                move_component::<InventoryComponent>(
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

                // We want to restore the map view to the player's position
                // once cursor mode is left. Therefore, we have to store the current position of the map.
                let (map_scroll_pos_x, map_scroll_pos_y) = self.map.get_top_left_corner_coords();

                self.mode_persist = Some(ModePersist {
                    map_scroll_pos_x,
                    map_scroll_pos_y,
                });

                // The cursor is unique in that it can build.
                self.entities
                    .add_component(self.cursor_id, BuilderComponent::new())
                    .unwrap();

                // Recenter cursor on the screen
                let mut sprite_components = self.entities.borrow_mut::<SpriteComponent>().unwrap();
                let cursor_sprite_component = sprite_components.get_mut(self.cursor_id).unwrap();
                let handle = cursor_sprite_component.get_handle();
                handle.set_x_pos(cursor::INITIAL_CURSOR_ONSCREEN_POS_X);
                handle.set_y_pos(cursor::INITIAL_CURSOR_ONSCREEN_POS_Y);
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

                // Move the inventory component back to the player
                move_component::<InventoryComponent>(
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
                // Or the cursor to be able to build
                self.entities
                    .rm_component::<BuilderComponent>(self.cursor_id)
                    .unwrap();

                // Recenter player on the screen
                let mut sprite_components = self.entities.borrow_mut::<SpriteComponent>().unwrap();
                let player_sprite_component = sprite_components.get_mut(self.cursor_id).unwrap();
                let handle = player_sprite_component.get_handle();
                handle.set_x_pos(player::INITIAL_PLAYER_ONSCREEN_POS_X);
                handle.set_y_pos(player::INITIAL_PLAYER_ONSCREEN_POS_Y);

                // Scroll map back to the player's position
                let mode_persist = self.mode_persist.unwrap();
                self.map
                    .scroll_abs(mode_persist.map_scroll_pos_x, mode_persist.map_scroll_pos_y);
                // Cursor has to be made invisible again
                let cursor_sprite_component = sprite_components.get_mut(self.cursor_id).unwrap();
                let handle = cursor_sprite_component.get_handle();
                handle.set_visibility(false);
                drop(sprite_components);

                // Make camera follow the player again
                let mut player_movement = MovementComponent::new();
                player_movement.input_controlled = true;
                player_movement.keep_camera_centered_on = true;
                self.entities
                    .add_component(self.player_id, player_movement)
                    .unwrap();
            }
        }
    }
}
