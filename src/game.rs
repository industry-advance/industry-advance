use crate::map::Map;

pub(crate) struct Game {
    map: Map<'static>,
}

impl Game {
    pub(crate) fn new() -> Game {
        return Game {
            map: Map::new_testmap(),
        };
    }
    pub(crate) fn run(&mut self) {
        self.init();
    }
   
    fn init(&mut self) {
        // Draw the map for 1st time
        self.map.draw_initial();
    }
    fn update(&mut self) {
        // Process key events

        // Simulate

        // Update display
        //self.map.draw();
    }
}
