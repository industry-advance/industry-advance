use crate::components::{InputComponent, MovementComponent, SpriteComponent};
use crate::map::Map;
use crate::shared_types::{Velocity, ZERO_VELOCITY};

use core::convert::TryInto;

/// Maximum player speed, in pixels per frame
/// A value of 1 means the player can move at most 60 pixels or 7.5 tiles a second.
const PLAYER_MAX_VELOCITY: Velocity =
    Velocity::from_bits(0b0000_0000_0000_0000_0000_0001_0000_0000); // 1
const PLAYER_MIN_VELOCITY: Velocity =
    Velocity::from_bits(-0b0000_0000_0000_0000_0000_0001_0000_0001); // -1

/// How much the player's speed changes for each frame the button is held down, in pixels per second.
const VELOCITY_DELTA_PER_FRAME: Velocity =
    Velocity::from_bits(0b0000_0000_0000_0000_0000_0000_01101); // 0.1

use tiny_ecs::{ECSError, Entities};
/// An ECS system which moves entity sprites based on their velocity
pub struct MovementSystem {}

impl MovementSystem {
    /// For each entity that is live,
    /// check whether it has a sprite and move it if it does.
    ///
    /// If the camera should stay focused on the entity, move the map instead of the entity.
    pub fn tick(
        ecs: &mut Entities,
        live_entities: &[usize],
        map: &mut Map,
    ) -> Result<(), ECSError> {
        let mut movables = ecs.borrow_mut::<MovementComponent>().unwrap();
        let inputables = ecs.borrow_mut::<InputComponent>().unwrap();
        let mut sprites = ecs.borrow_mut::<SpriteComponent>().unwrap();
        for id in live_entities {
            // Process position updates caused by input
            if ecs.entity_contains::<MovementComponent>(*id)
                && ecs.entity_contains::<InputComponent>(*id)
            {
                let e_movement: &mut MovementComponent = movables.get_mut(*id).unwrap();
                if ecs.entity_contains::<InputComponent>(*id) {
                    let e_input: &InputComponent = inputables.get(*id).unwrap();
                    update_movement_based_on_input(e_input, e_movement);
                }

                // Process scrolling the map around entities which the camera's centered on
                if e_movement.keep_camera_centered_on {
                    if e_movement.pending_movement_delta_x != ZERO_VELOCITY
                        || e_movement.pending_movement_delta_y != ZERO_VELOCITY
                    {
                        // Subtract the number of whole pixels we can scroll from the accumulated movement
                        let (map_delta_x, map_delta_y) = e_movement.reset_pending_movement_delta();
                        // Map scrolling happens in the opposite direction to where the player's moving
                        map.scroll(map_delta_x, map_delta_y);
                    }
                }

                // Process updating the sprite position on screen
                if ecs.entity_contains::<SpriteComponent>(*id)
                    && !e_movement.keep_camera_centered_on
                {
                    let e_sprite: &mut SpriteComponent = sprites.get_mut(*id).unwrap();
                    if e_movement.pending_movement_delta_x != ZERO_VELOCITY
                        || e_movement.pending_movement_delta_y != ZERO_VELOCITY
                    {
                        let (sprite_delta_x, sprite_delta_y) =
                            e_movement.reset_pending_movement_delta();
                        let mut sprite_attrs = e_sprite.get_handle().read_obj_attributes();

                        // Modify sprite X position based on velocity
                        let new_row_coord: u16 = (sprite_attrs.attr0.row_coordinate() as i32
                            + sprite_delta_x)
                            .try_into()
                            .unwrap();
                        sprite_attrs.attr0 = sprite_attrs.attr0.with_row_coordinate(new_row_coord);
                        // Modify sprite Y position based on velocity
                        let new_col_coord: u16 = (sprite_attrs.attr1.col_coordinate() as i32
                            + sprite_delta_y)
                            .try_into()
                            .unwrap();
                        sprite_attrs.attr1 =
                            sprite_attrs.attr1.with_col_coordinate(new_col_coord as u16);

                        e_sprite.get_handle().write_obj_attributes(sprite_attrs);
                    }
                }
            }
        }
        return Ok(());
    }
}

/// Update an entity's position based on it's input component.
fn update_movement_based_on_input(ic: &InputComponent, mc: &mut MovementComponent) {
    // If the button is pressed, accelerate
    if ic.left_pressed {
        gba::debug!("Should Only Move Left");
        if mc.x_velocity > PLAYER_MIN_VELOCITY {
            mc.x_velocity -= VELOCITY_DELTA_PER_FRAME;
        }
        mc.pending_movement_delta_x += mc.x_velocity;
    // If the button isn't pressed and we aren't moving in the opposite direction, decelerate
    } else if !ic.left_pressed && !ic.right_pressed && mc.x_velocity < ZERO_VELOCITY {
        if mc.x_velocity < -VELOCITY_DELTA_PER_FRAME {
            mc.x_velocity += VELOCITY_DELTA_PER_FRAME;
        // Make sure we don't overshoot and cause a drift into positive X velocity
        } else {
            mc.x_velocity = ZERO_VELOCITY;
        }
        mc.pending_movement_delta_x += mc.x_velocity;
    }

    if ic.right_pressed {
        gba::debug!("Should Only Move Right");
        if mc.x_velocity < PLAYER_MAX_VELOCITY {
            mc.x_velocity += VELOCITY_DELTA_PER_FRAME;
        }
        mc.pending_movement_delta_x += mc.x_velocity;
    } else if !ic.right_pressed && !ic.left_pressed && mc.x_velocity > ZERO_VELOCITY {
        if mc.x_velocity > VELOCITY_DELTA_PER_FRAME {
            mc.x_velocity -= VELOCITY_DELTA_PER_FRAME;
        } else {
            mc.x_velocity = ZERO_VELOCITY;
        }
        mc.pending_movement_delta_x += mc.x_velocity;
    }

    // If no buttons causing movement on the X axis are pressed, decelerate towards 0

    if ic.up_pressed {
        gba::debug!("Should Only Move Up");
        if mc.y_velocity > PLAYER_MIN_VELOCITY {
            mc.y_velocity -= VELOCITY_DELTA_PER_FRAME;
        }
        mc.pending_movement_delta_y += mc.y_velocity;
    } else if !ic.up_pressed && !ic.down_pressed && mc.y_velocity < ZERO_VELOCITY {
        gba::debug!("Seems i have veloc up left");
        if mc.y_velocity < -VELOCITY_DELTA_PER_FRAME {
            mc.y_velocity += VELOCITY_DELTA_PER_FRAME;
        } else {
            mc.y_velocity = ZERO_VELOCITY;
        }
        mc.pending_movement_delta_y += mc.y_velocity;
    }

    if ic.down_pressed {
        gba::debug!("Should Only Move Down");
        if mc.y_velocity < PLAYER_MAX_VELOCITY{
            mc.y_velocity += VELOCITY_DELTA_PER_FRAME;
        }
        gba::debug!("Y_Velocity Now By {}", mc.y_velocity);
        mc.pending_movement_delta_y += mc.y_velocity;
    } else if !ic.down_pressed && !ic.up_pressed && mc.y_velocity > ZERO_VELOCITY {
        gba::debug!("Seems i have veloc down left");
        if mc.y_velocity > VELOCITY_DELTA_PER_FRAME {
            mc.y_velocity -= VELOCITY_DELTA_PER_FRAME;
        } else {
            mc.y_velocity = ZERO_VELOCITY;
        }
        mc.pending_movement_delta_y += mc.y_velocity;
    }
}
