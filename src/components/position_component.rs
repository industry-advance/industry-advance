use crate::shared_types::{Position, ZERO_POSITION};

pub type PositionComponent = Position;

impl PositionComponent {
    pub fn new() -> PositionComponent {
        return PositionComponent(ZERO_POSITION);
    }
}