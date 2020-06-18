use crate::components::{InventoryComponent, PositionComponent, SpriteComponent};
use crate::debug_log::*;
use crate::shared_types::*;

use crate::sprite::{HWSpriteAllocator, HWSpriteSize};
use tiny_ecs::{ECSError, Entities};

/// Adds a container to the ECS.
pub(crate) fn add_container(
    entities: &mut Entities,
    sprite_alloc: &mut HWSpriteAllocator,
) -> Result<usize, ECSError> {
    let entity_id = entities
        .new_entity()
        .with(SpriteComponent::with_pos(
            sprite_alloc,
            "containerTiles",
            HWSpriteSize::ThirtyTwoByThirtyTwo,
            128,
            128,
            true
        ))?
        .with(PositionComponent::with_pos((
            Coordinate::from_num(128),
            Coordinate::from_num(128),
        )))?
        .with(InventoryComponent::new(1000))?
        .finalise()?;
    debug_log!(Subsystems::Entity, "Created container");

    return Ok(entity_id);
}
