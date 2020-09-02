use crate::components::{
    InputComponent, InventoryComponent, MovementComponent, PositionComponent, SpriteComponent,
};
use crate::debug_log::*;
use crate::shared_constants::{SCREEN_HEIGHT, SCREEN_WIDTH};
use crate::shared_types::Coordinate;
use crate::sprite::{HWSpriteAllocator, HWSpriteSize};
use tiny_ecs::{ECSError, Entities};

/// Middle of the screen should be middle of sprite as well
pub const INITIAL_PLAYER_ONSCREEN_POS_X: u16 = (SCREEN_WIDTH / 2 - 32 / 2) as u16;
pub const INITIAL_PLAYER_ONSCREEN_POS_Y: u16 = (SCREEN_HEIGHT / 2 - 32 / 2) as u16;
const PLAYER_INVENTORY_CAPACITY: usize = 64;
/// Adds a player to the ECS.
/// The player accepts user input and the camera stays centered on it's sprite.
pub fn add_player(
    entities: &mut Entities,
    sprite_alloc: &mut HWSpriteAllocator,
) -> Result<usize, ECSError> {
    let mut movement_component = MovementComponent::new();
    movement_component.input_controlled = true;
    movement_component.keep_camera_centered_on = true;
    let entity_id = entities
        .new_entity()
        .with(SpriteComponent::with_pos(
            sprite_alloc,
            "dart_shipTiles",
            HWSpriteSize::ThirtyTwoByThirtyTwo,
            INITIAL_PLAYER_ONSCREEN_POS_X,
            INITIAL_PLAYER_ONSCREEN_POS_Y,
            true,
        ))?
        .with(movement_component)?
        .with(InputComponent::new())?
        .with(InventoryComponent::new(PLAYER_INVENTORY_CAPACITY))?
        // Place player in the middle of the screen
        .with(PositionComponent::with_pos((
            Coordinate::from_num(INITIAL_PLAYER_ONSCREEN_POS_X),
            Coordinate::from_num(INITIAL_PLAYER_ONSCREEN_POS_Y),
        )))?
        .finalise()?;
    debug_log!(Subsystems::Entity, "Created player");

    return Ok(entity_id);
}
