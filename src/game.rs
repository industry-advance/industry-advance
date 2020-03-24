use crate::assets::sprites::sprites::DART_SHIP_TILES;
use crate::map::Map;
use crate::sprite::{HWSprite, HWSpriteAllocator, HWSpriteSize};

pub(crate) struct Game {
    sprite_alloc: HWSpriteAllocator,
    map: Map<'static>,
}

impl Game {
    pub(crate) fn new(sprite_alloc: HWSpriteAllocator) -> Game {
        return Game {
            sprite_alloc: sprite_alloc,
            map: Map::new_testmap(),
        };
    }
    pub(crate) fn run(&mut self) {
        self.init();
    }
    fn init(&mut self) {
        // Draw the map for 1st time
        self.map.draw_initial();
        // Initialize player sprite
        let _player_sprite_handle = self.sprite_alloc.alloc(&HWSprite::from_u32_slice(
            &DART_SHIP_TILES,
            HWSpriteSize::SixtyFourBySixtyFour,
        ));

        // TODO: player_sprite_handle.display();
    }
    fn update(&mut self) {
        // Process key events

        // Simulate

        // Update display
        //self.map.draw();
    }
}
