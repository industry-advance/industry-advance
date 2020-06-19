//! This module contains various helper functions for dealing with ECS entities.
//! It should perhaps be upstreamed to `tiny_ecs`.

use core::option::NoneError;

use tiny_ecs::{ECSError, Entities};

/// Move a component of type `T` from `src` to `dest`.
pub fn move_component<T>(
    src: usize,
    dest: usize,
    entities: &mut Entities,
) -> Result<(), MoveComponentError>
where
    T: Clone + 'static,
{
    let components = entities.borrow_mut::<T>()?;
    let src_component = components.get(src)?.clone();
    drop(components);
    entities.add_component(dest, src_component)?;
    // Remove component from old entity
    entities.rm_component::<T>(src)?;
    return Ok(());
}

/// Clone a component of type `T` from `src` to `dest`.
pub fn clone_component<T>(
    src: usize,
    dest: usize,
    entities: &mut Entities,
) -> Result<(), MoveComponentError>
where
    T: Clone + 'static,
{
    let components = entities.borrow_mut::<T>()?;
    let src_component = components.get(src)?.clone();
    drop(components);
    entities.add_component(dest, src_component)?;
    return Ok(());
}

#[derive(Debug)]
pub enum MoveComponentError {
    ECSError(ECSError),
    NoSuchEntityError,
}

impl From<ECSError> for MoveComponentError {
    fn from(error: ECSError) -> Self {
        MoveComponentError::ECSError(error)
    }
}

impl From<NoneError> for MoveComponentError {
    fn from(_: NoneError) -> Self {
        MoveComponentError::NoSuchEntityError
    }
}
