use crate::shared_types::Position;
use crate::sprite::HWSpriteAllocator;

use tiny_ecs::{ECSError, Entities};

/// All objects that can be built in the world.
pub trait Buildable {
    /// Creates a new instance of the entity in the world.
    ///
    /// Returns the ECS ID of the constructed entity.
    fn build(
        &self,
        pos: Position,
        entities: &mut Entities,
        sprite_alloc: &mut HWSpriteAllocator,
    ) -> Result<usize, ECSError>;
}
