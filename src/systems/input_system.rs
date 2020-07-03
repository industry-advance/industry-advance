use crate::components::{BuilderComponent, InputComponent};
use crate::debug_log::*;

use gba::io::keypad;
use tiny_ecs::{ECSError, Entities};

/// This system reads and processes player input.
pub(crate) struct InputSystem {
    // Tracks the key state of the last update to allow checking for differences w/ current state
    last_keys: keypad::KeyInput,
    start_held: bool,
}
impl InputSystem {
    /// Initializes the system.
    pub fn init() -> InputSystem {
        return InputSystem {
            last_keys: keypad::KeyInput::new(),
            start_held: false,
        };
    }

    /// Updates the input-related components of entities.
    pub fn tick(&mut self, ecs: &mut Entities, live_entities: &[usize]) -> Result<bool, ECSError> {
        // Read the current state of the keypad
        let keys = keypad::read_key_input();
        // If the new state is different than the old one, do the updating
        if keys != self.last_keys {
            for id in live_entities {
                // TODO: Consider refactoring this ugly hack
                if ecs.entity_contains::<BuilderComponent>(*id) {
                    let mut builders = ecs.borrow_mut::<BuilderComponent>().unwrap();
                    let mut e_builder_component: &mut BuilderComponent =
                        builders.get_mut(*id).unwrap();
                    // Pass A button press to builder component, if available
                    // TODO: May need to debounce
                    if keys.a() {
                        debug_log!(Subsystems::InputSystem, "A pressed");
                        e_builder_component.build = true;
                    }
                }
                if ecs.entity_contains::<InputComponent>(*id) {
                    let mut movables = ecs.borrow_mut::<InputComponent>().unwrap();
                    let mut e_input_component: &mut InputComponent = movables.get_mut(*id).unwrap();
                    // Pass D-Pad movement onto entity movement components
                    if keys.left() {
                        debug_log!(Subsystems::InputSystem, "D-Pad left pressed");
                        e_input_component.left_pressed = true;
                    } else if e_input_component.left_pressed {
                        e_input_component.left_pressed = false;
                        debug_log!(Subsystems::InputSystem, "D-Pad left released");
                    }
                    if keys.right() {
                        debug_log!(Subsystems::InputSystem, "D-Pad right pressed");
                        e_input_component.right_pressed = true;
                    } else if e_input_component.right_pressed {
                        e_input_component.right_pressed = false;
                        debug_log!(Subsystems::InputSystem, "D-Pad right released");
                    }
                    if keys.up() {
                        debug_log!(Subsystems::InputSystem, "D-Pad up pressed");
                        e_input_component.up_pressed = true;
                    } else if e_input_component.up_pressed {
                        e_input_component.up_pressed = false;
                        debug_log!(Subsystems::InputSystem, "D-Pad up released");
                    }
                    if keys.down() {
                        debug_log!(Subsystems::InputSystem, "D-Pad down pressed");
                        e_input_component.down_pressed = true;
                    } else if e_input_component.down_pressed {
                        e_input_component.down_pressed = false;
                        debug_log!(Subsystems::InputSystem, "D-Pad down released");
                    }
                    /* The state of the start key doesn't concern any entity directly,
                    but it does concern the game loop. Therefore, we return the value here. */
                    let start_pressed: bool;
                    if keys.start() && !self.start_held {
                        debug_log!(Subsystems::InputSystem, "Start pressed");
                        start_pressed = true;
                        self.start_held = true;
                    } else {
                        start_pressed = false;
                        self.start_held = false;
                    }
                    self.last_keys = keys;
                    return Ok(start_pressed);
                }
            }
        }
        // Store the keypad state for next call
        self.last_keys = keys;
        return Ok(false);
    }
}
