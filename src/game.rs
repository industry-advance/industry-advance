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
use gbfs_rs::FilenameString;
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
        gba::debug!("Before FS");

        let mut b = Box::new(4);
        gba::debug!("Box {:?}", b);
        b = Box::new(4);
        gba::debug!("Box {:?}", b);

        // Initialize hardware sprite management

        gba::debug!("FilenameString::try_from_str('sprite_sharedPal').unwrap()");
        let d1 = FilenameString::try_from_str("sprite_sharedPal");

        gba::debug!("DATA: {:?}", d1);
        let d2 = d1.unwrap();
        //panic!("??????");
        gba::debug!(".unwrap() {:?}", d2);

        //panic!(">>>>>>>>>");

        //let d3 = ;

        gba::debug!("DFSERGERGEGERREG {:?}", FS.get_file_by_name(d2));

        let mut sprite_allocator = HWSpriteAllocator::new(
            &FS.get_file_by_name(FilenameString::try_from_str("sprite_sharedPal").unwrap())
                .unwrap()
                .to_u16_vec(),
        );

        //panic!("::::::::");

        sprite_allocator.init();

        //panic!("::::::::");

        gba::debug!("After FS");

        let map_0: &'static [u8] = FS
            .get_file_data_by_name(FilenameString::try_from_str("testmap_0Map").unwrap())
            .unwrap();
        //panic!("::::::::");
        let map_1: &'static [u8] = FS
            .get_file_data_by_name(FilenameString::try_from_str("testmap_1Map").unwrap())
            .unwrap();
        //panic!("::::::::");
        let map_2: &'static [u8] = FS
            .get_file_data_by_name(FilenameString::try_from_str("testmap_2Map").unwrap())
            .unwrap();
        let map_3: &'static [u8] = FS
            .get_file_data_by_name(FilenameString::try_from_str("testmap_3Map").unwrap())
            .unwrap();
        //panic!("TEstdfsfef");
        let mut tilemaps: Vec<&'static [u8]> = Vec::new();

        tilemaps.push(map_0);
        tilemaps.push(map_1);
        tilemaps.push(map_2);
        tilemaps.push(map_3);

        gba::debug!("Tilemaps: {:?}", tilemaps);
        // Read palette from FS and convert to u16's
        let pal: Vec<u16> = FS
            .get_file_by_name(FilenameString::try_from_str("map_sharedPal").unwrap())
            .unwrap()
            .to_u16_vec();
        let tiles: Vec<u32> = FS
            .get_file_by_name(FilenameString::try_from_str("map_sharedTiles").unwrap())
            .unwrap()
            .to_u32_vec();
        let map = Map::new_map(pal, 2, 2, tiles, tilemaps);

        // Initialize the ECS
        let mut e = Entities::new(Some(256), Some(24));
        let mut live_entity_ids = Vec::new();

        // Initialize the player entity
        let player_id = entities::add_player(
            &mut e,
            &mut sprite_allocator,
            &FS.get_file_by_name(FilenameString::try_from_str("dart_shipTiles").unwrap())
                .unwrap()
                .to_u32_vec(),
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
