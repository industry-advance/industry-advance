use crate::components::{InputComponent, MovementComponent, PositionComponent, SpriteComponent};
use crate::debug_log::*;
use crate::shared_constants::{SCREEN_HEIGHT, SCREEN_WIDTH};
use crate::shared_types::Coordinate;
use crate::sprite::{HWSpriteAllocator, HWSpriteSize};
use tiny_ecs::{ECSError, Entities};

/// Middle of the screen should be middle of sprite as well
pub const INITIAL_CURSOR_ONSCREEN_POS_X: u16 = (SCREEN_WIDTH / 2) as u16;
pub const INITIAL_CURSOR_ONSCREEN_POS_Y: u16 = (SCREEN_HEIGHT / 2) as u16;
/// Adds a cursor to the ECS.
/// The cursor accepts user input and the camera stays centered on it's sprite.
pub(crate) fn add_cursor(
    entities: &mut Entities,
    sprite_alloc: &mut HWSpriteAllocator,
) -> Result<usize, ECSError> {
    let entity_id = entities
        .new_entity()
        .with(SpriteComponent::with_pos(
            sprite_alloc,
            "cursorTiles",
            HWSpriteSize::SixteenBySixteen,
            INITIAL_CURSOR_ONSCREEN_POS_X,
            INITIAL_CURSOR_ONSCREEN_POS_Y,
            false,
        ))?
        .finalise()?;
    debug_log!(Subsystems::Entity, "Created cursor");

    return Ok(entity_id);
}
