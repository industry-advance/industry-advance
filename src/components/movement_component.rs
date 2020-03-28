/// This component controls an entity's movement.
/// Velocity is given in pixels per tick.
pub(crate) struct MovementComponent {
    pub input_controlled: bool,
    pub x_velocity: i32,
    pub y_velocity: i32,
}

impl MovementComponent {
    pub fn new() -> MovementComponent {
        return MovementComponent {
            input_controlled: false,
            x_velocity: 0,
            y_velocity: 0,
        };
    }
}
