use crate::shared_types::Position;

pub struct PositionComponent(pub Position);

impl PositionComponent {
    // Returns the value before the comma.
    pub fn floor(&self) -> (u32, u32) {
        return ((self.0).0.int().to_num(), (self.0).1.int().to_num());
    }

    pub fn with_pos(pos: Position) -> PositionComponent {
        return PositionComponent(pos);
    }
}
