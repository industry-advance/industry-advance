/// This component contains the player input relevant to an entity.
/// Velocity is given in pixels per tick.
#[derive(Clone)]
pub struct InputComponent {
    pub left_pressed: bool,
    pub right_pressed: bool,
    pub up_pressed: bool,
    pub down_pressed: bool,
}

impl InputComponent {
    pub fn new() -> InputComponent {
        return InputComponent {
            left_pressed: false,
            right_pressed: false,
            up_pressed: false,
            down_pressed: false,
        };
    }
}
