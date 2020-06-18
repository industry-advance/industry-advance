use crate::shared_types::{Velocity, ZERO_VELOCITY};
/// This component controls an entity's movement.
/// Velocity is given in pixels per tick.
#[derive(Clone)]
pub(crate) struct MovementComponent {
    /// Whether the entity's position depends on player input.
    pub input_controlled: bool,
    /// Whether the entity is a "perspective character", meaning the world should move instead of the entity on screen.
    pub keep_camera_centered_on: bool,
    /// Current velocity among the X axis
    pub x_velocity: Velocity,
    /// Current velocity among the Y axis
    pub y_velocity: Velocity,
    /// The amount of difference in position between what it should be and what it is on screen among the X axis.
    /// This is needed because velocity is fractional, but pixels on a screen are absolute values.
    /// This should be adjusted whenever the display is adjusted appropriately.
    pub pending_movement_delta_x: Velocity,
    /// The amount of difference in position between what it should be and what it is on screen among the Y axis.
    /// This is needed because velocity is fractional, but pixels on a screen are absolute values.
    /// This should be adjusted whenever the display is adjusted appropriately.
    pub pending_movement_delta_y: Velocity,
}

impl MovementComponent {
    pub fn new() -> MovementComponent {
        return MovementComponent {
            input_controlled: false,
            keep_camera_centered_on: false,
            x_velocity: ZERO_VELOCITY,
            y_velocity: ZERO_VELOCITY,
            pending_movement_delta_x: ZERO_VELOCITY,
            pending_movement_delta_y: ZERO_VELOCITY,
        };
    }

    /// Resets the wholes part of the pending movement delta and returns it.
    /// The fractional part stays the same.
    ///
    /// For example, if the current pending X movement delta is 1.2, we subtract 1 and return it, and 0.2 remains.
    pub fn reset_pending_movement_delta(&mut self) -> (i32, i32) {
        let int_x = self.pending_movement_delta_x.int();
        let int_y = self.pending_movement_delta_y.int();
        self.pending_movement_delta_x = self.pending_movement_delta_x.frac();
        self.pending_movement_delta_y = self.pending_movement_delta_y.frac();
        return (int_x.to_num(), int_y.to_num());
    }
}
