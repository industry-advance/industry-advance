use crate::shared_types::{Position, ZERO_POSITION};

pub struct PositionComponent(pub Position);

impl PositionComponent {
    pub fn new() -> PositionComponent {
        return PositionComponent(ZERO_POSITION);
    }

    // Returns the value before the comma.
    pub fn floor(&self) -> (u32, u32) {
        return ((self.0).0.int().to_num(), (self.0).1.int().to_num());
    }
}
