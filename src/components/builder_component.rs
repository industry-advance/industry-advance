use crate::entities::Buildable;
use crate::shared_types::Position;

use alloc::boxed::Box;

/// Entities which posess this component are permitted to request structures to be built.
#[derive(Clone)]
pub struct BuilderComponent<'a> {
    pub buildable: Option<Box<&'a dyn Buildable>>,
    pub pos: Option<Position>,
}

impl<'a> BuilderComponent<'a> {
    pub fn new() -> BuilderComponent<'a> {
        return BuilderComponent {
            buildable: None,
            pos: None,
        };
    }
}
