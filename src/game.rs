use gba::{
    fatal,
    io::{
        display::{DisplayControlSetting, DisplayMode, DISPCNT, VBLANK_SCANLINE, VCOUNT},
        keypad, *,
    },
    palram::index_palram_bg_4bpp,
    vram::{get_4bpp_character_block, CHAR_BASE_BLOCKS},
    Color,
};

const BLACK: Color = Color::from_rgb(0, 0, 0);
const WHITE: Color = Color::from_rgb(31, 31, 31);

#[derive(Clone, Copy)]
struct Entity {}
pub(crate) struct World {
    entities: [Entity; 32],
}

impl World {
    fn new() -> World {
        return World {
            entities: [Entity {}; 32],
        };
    }
}

pub(crate) struct Game {
    world: World,
}

impl Game {
    pub(crate) fn new() -> Game {
        return Game {
            world: World::new(),
        };
    }
    fn update(&mut self) {}

    fn draw(&self) {}

    fn draw_bg(&self) {
        fn set_bg_tile_4bpp(charblock: usize, index: usize, tile: gba::vram::Tile4bpp) {
            assert!(charblock < 4);
            assert!(index < 512);
            unsafe {
                CHAR_BASE_BLOCKS
                    .index(charblock)
                    .cast::<gba::vram::Tile4bpp>()
                    .offset(index as isize)
                    .write(tile)
            }
        }

        // We use mode 0 because it allows for the most layers
        let mode_0 = DisplayControlSetting::new().with_mode(DisplayMode::Mode0);
        display::set_display_control(mode_0);

        // Paint the background
        // Load the background palette
        load_bg_palette();
        // Load the background tilemap (checkerboard)
        load_bg_tilemap();
    }
}

fn load_bg_palette() {
    // Load the background palette
    const PALBANK_0: u8 = 0;
    index_palram_bg_4bpp(PALBANK_0, 0).write(BLACK);
    index_palram_bg_4bpp(PALBANK_0, 1).write(WHITE);
}

fn load_bg_tilemap() {
    // Load the background tilemap (checkerboard)
    //
}
