//! This module contains ECS components.
mod input_component;
mod movement_component;
mod position_component;
mod sprite_component;
pub(crate) use input_component::InputComponent;
pub(crate) use movement_component::MovementComponent;
pub(crate) use position_component::PositionComponent;
pub(crate) use sprite_component::SpriteComponent;
