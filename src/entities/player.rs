use crate::assets::sprites::sprites::DART_SHIP_TILES;
use crate::components::{MovementComponent, SpriteComponent};
use crate::sprite::{HWSpriteAllocator, HWSpriteSize};
use tiny_ecs::{ECSError, Entities};

/// Adds a player to the ECS.
pub(crate) fn add_player(
    entities: &mut Entities,
    sprite_alloc: &mut HWSpriteAllocator,
) -> Result<usize, ECSError> {
    let mut movement_component = MovementComponent::new();
    movement_component.input_controlled = true;
    let entity_id = entities
        .new_entity()
        .with(SpriteComponent::init(
            sprite_alloc,
            &DART_SHIP_TILES,
            HWSpriteSize::ThirtyTwoByThirtyTwo,
        ))?
        .with(movement_component)?
        .finalise()?;

    return Ok(entity_id);
}
