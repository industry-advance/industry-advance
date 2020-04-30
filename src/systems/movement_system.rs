use crate::components::{MovementComponent, SpriteComponent};
use crate::map::Map;
use alloc::vec::Vec;
use core::convert::TryInto;

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
        live_entities: &Vec<usize>,
        map: &mut Map,
    ) -> Result<(), ECSError> {
        let movables = ecs.borrow_mut::<MovementComponent>().unwrap();
        let mut sprites = ecs.borrow_mut::<SpriteComponent>().unwrap();
        for id in live_entities {
            if ecs.entity_contains::<MovementComponent>(*id) {
                let e_movement: &MovementComponent = movables.get(*id).unwrap();
                if ecs.entity_contains::<SpriteComponent>(*id) {
                    // Move the sprite if the camera should not stay centered on it, otherwise move the map background.
                    if !e_movement.keep_camera_centered_on {
                        let e_sprite: &mut SpriteComponent = sprites.get_mut(*id).unwrap();
                        if e_movement.x_velocity != 0 || e_movement.y_velocity != 0 {
                            let mut sprite_attrs = e_sprite.get_handle().read_obj_attributes();
                            // Modify X position based on velocity
                            // Requires some type voodoo because of signed + unsigned
                            let new_row_coord: u16 = ((sprite_attrs.attr0.row_coordinate() as i32)
                                + e_movement.x_velocity)
                                .try_into()
                                .unwrap();
                            sprite_attrs.attr0 =
                                sprite_attrs.attr0.with_row_coordinate(new_row_coord);
                            // Modify Y position based on velocity
                            let new_col_coord: u16 = ((sprite_attrs.attr1.col_coordinate() as i32)
                                + e_movement.y_velocity)
                                .try_into()
                                .unwrap();
                            sprite_attrs.attr1 =
                                sprite_attrs.attr1.with_col_coordinate(new_col_coord);

                            e_sprite.get_handle().write_obj_attributes(sprite_attrs);
                        }
                    }
                    //  Scroll the map/background.
                    else {
                        if e_movement.x_velocity != 0 || e_movement.y_velocity != 0 {
                            map.scroll(e_movement.x_velocity, e_movement.y_velocity);
                        }
                    }
                }
            }
        }
        return Ok(());
    }
}
