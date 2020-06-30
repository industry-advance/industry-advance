/// This component marks that the entity wishes to create a building at it's current location.
/// Currently only places miners.
/// TODO: Actually support building anything (blocked on UI)
use crate::shared_types::Coordinate;
#[derive(Clone)]
pub struct BuilderComponent {
    pub build: bool,
    pub x_pos: usize,
    pub y_pos: usize,
}

impl BuilderComponent {
    pub fn new() -> BuilderComponent {
        return BuilderComponent {
            build: false,
            x_pos: 0,
            y_pos: 0,
        };
    }
}
