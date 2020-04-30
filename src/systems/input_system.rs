use crate::components::MovementComponent;
use alloc::vec::Vec;
use gba::io::keypad;
use tiny_ecs::{ECSError, Entities};

/// This system reads and processes player input.
pub(crate) struct InputSystem {
    // Tracks the key state of the last update to allow checking for differences w/ current state
    last_keys: keypad::KeyInput,
}
impl InputSystem {
    /// Initializes the system.
    pub fn init() -> InputSystem {
        return InputSystem {
            last_keys: keypad::KeyInput::new(),
        };
    }

    /// Updates the input-related components of entities.
    pub fn tick(&mut self, ecs: &mut Entities, live_entities: &Vec<usize>) -> Result<(), ECSError> {
        // Read the current state of the keypad
        let keys = keypad::read_key_input();

        let mut movables = ecs.borrow_mut::<MovementComponent>().unwrap();
        for id in live_entities {
            if ecs.entity_contains::<MovementComponent>(*id) {
                let mut e_movement_component: &mut MovementComponent =
                    movables.get_mut(*id).unwrap();
                if e_movement_component.input_controlled {
                    // Pass D-Pad movement onto objects
                    if keys.left() {
                        gba::info!("[INPUT] D-Pad left pressed");
                        e_movement_component.x_velocity = e_movement_component.x_velocity - 1;
                    }
                    if keys.right() {
                        gba::info!("[INPUT] D-Pad right pressed");
                        e_movement_component.x_velocity = e_movement_component.x_velocity + 1;
                    }
                    if keys.up() {
                        gba::info!("[INPUT] D-Pad up pressed");
                        e_movement_component.y_velocity = e_movement_component.y_velocity - 1;
                    }
                    if keys.down() {
                        gba::info!("[INPUT] D-Pad down pressed");
                        e_movement_component.y_velocity = e_movement_component.y_velocity + 1;
                    }
                }
            }
        }

        // Store the keypad state for next call
        self.last_keys = keys;

        return Ok(());
    }
}
