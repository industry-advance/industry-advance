
use crate::map::Map;

pub(crate) struct Game {
    map: Map<'static>,
}

impl Game {
    pub(crate) fn new() -> Game {
        return Game {
            map: Map::new_empty(),
        };
    }
    pub(crate) fn run(&mut self) {}
    fn update(&mut self) {
        // Process key events

        // Simulate

        // Update display
        self.map.draw();
    }
}
