use crate::components::{PositionComponent, SpriteComponent};
use crate::debug_log::*;
use crate::shared_types::*;
use crate::sprite::{HWSpriteAllocator, HWSpriteSize};
use tiny_ecs::{ECSError, Entities};

/// Adds a copper wall to the ECS.
pub(crate) fn add_copper_wall(
    entities: &mut Entities,
    sprite_alloc: &mut HWSpriteAllocator,
) -> Result<usize, ECSError> {
    let entity_id = entities
        .new_entity()
        .with(SpriteComponent::with_pos(
            sprite_alloc,
            "copper_wallTiles",
            HWSpriteSize::SixteenBySixteen,
            32,
            32,
            true,
        ))?
        .with(PositionComponent::with_pos((
            Coordinate::from_num(32),
            Coordinate::from_num(32),
        )))?
        .finalise()?;
    debug_log!(Subsystems::Entity, "Created copper wall");

    return Ok(entity_id);
}
