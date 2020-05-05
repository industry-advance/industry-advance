use crate::components::{InputComponent, MovementComponent, PositionComponent, SpriteComponent};
use crate::sprite::{HWSpriteAllocator, HWSpriteSize};
use tiny_ecs::{ECSError, Entities};

/// Adds a player to the ECS.
/// The player accepts user input and the camera stays centered on it's sprite.
pub(crate) fn add_player(
    entities: &mut Entities,
    sprite_alloc: &mut HWSpriteAllocator,
) -> Result<usize, ECSError> {
    let mut movement_component = MovementComponent::new();
    movement_component.input_controlled = true;
    movement_component.keep_camera_centered_on = true;
    let entity_id = entities
        .new_entity()
        .with(SpriteComponent::init(
            sprite_alloc,
            "dart_shipTiles",
            HWSpriteSize::ThirtyTwoByThirtyTwo,
        ))?
        .with(movement_component)?
        .with(InputComponent::new())?
        .with(PositionComponent::new())?
        .finalise()?;
    gba::info!("[ENTITY] Created player");

    return Ok(entity_id);
}
