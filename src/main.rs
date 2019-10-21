#![no_std]
#![feature(start)]

use gba::{
	fatal,
	io::{
		display::{DisplayControlSetting, DisplayMode, DISPCNT, VBLANK_SCANLINE, VCOUNT},
		keypad, *,
	},
	palram::index_palram_bg_4bpp,
	vram::{get_4bpp_character_block},
	Color,
};

const BLACK: Color = Color::from_rgb(0, 0, 0);
const WHITE: Color = Color::from_rgb(31, 31, 31);

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
	// This kills the emulation with a message if we're running within mGBA.
	fatal!("{}", info);
	// If we're _not_ running within mGBA then we still need to not return, so
	// loop forever doing nothing.
	loop {}
}

#[start]
fn main(_argc: isize, _argv: *const *const u8) -> isize {
	let mut game = Game::new();
	game.draw_bg(); // Draw a 1024*1024 tiled background
	loop {
		// Simulate
		game.update();
		// Wait for VBlank
		// TODO: Optimize via interrupts
		spin_until_vblank();
		// Draw changed tiles/sprites
		game.draw();
	}
}

fn spin_until_vblank() {
	while VCOUNT.read() < VBLANK_SCANLINE {}
}

#[derive(Clone, Copy)]
struct Entity {}
struct World {
	entities: [Entity; 32],
}

impl World {
	fn new() -> World {
		return World {
			entities: [Entity {}; 32],
		};
	}
}

struct Game {
	world: World,
}

impl Game {
	fn new() -> Game {
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
  unsafe { CHAR_BASE_BLOCKS.index(charblock).cast::<gba::vram::Tile4bpp>().offset(index as isize).write(tile) }
}

		// We use mode 0 because it allows for the most layers
		let mode_0 = DisplayControlSetting::new().with_mode(DisplayMode::Mode0);
		display::set_display_control(mode_0);

		// Paint the background
		// Load the background palette
		const PALBANK_0: u8 = 0;
		index_palram_bg_4bpp(PALBANK_0, 0).write(BLACK);
		index_palram_bg_4bpp(PALBANK_0, 1).write(WHITE);
		// Load the background tilemap (checkerboard)
		gba::vram::
	}
}
