use crate::components::{PositionComponent, SpriteComponent};
use crate::sprite::{HWSpriteAllocator, HWSpriteSize};
use tiny_ecs::{ECSError, Entities};

/// Adds a copper wall to the ECS.
pub(crate) fn add_copper_wall(
    entities: &mut Entities,
    sprite_alloc: &mut HWSpriteAllocator,
) -> Result<usize, ECSError> {
    let entity_id = entities
        .new_entity()
        .with(SpriteComponent::init(
            sprite_alloc,
            "copper_wallTiles",
            HWSpriteSize::SixteenBySixteen,
        ))?
        .with(PositionComponent::new())?
        .finalise()?;
    gba::info!("[ENTITY] Created copper wall");

    return Ok(entity_id);
}
