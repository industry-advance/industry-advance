/// This component controls an entity's movement.
/// Velocity is given in pixels per tick.
pub(crate) struct MovementComponent {
    /// Whether the entity's position depends on player input.
    pub input_controlled: bool,
    /// Whether the entity is a "perspective character", meaning the world should move instead of the entity on screen.
    pub keep_camera_centered_on: bool,
    pub x_velocity: i32,
    pub y_velocity: i32,
}

impl MovementComponent {
    pub fn new() -> MovementComponent {
        return MovementComponent {
            input_controlled: false,
            keep_camera_centered_on: false,
            x_velocity: 0,
            y_velocity: 0,
        };
    }
}
