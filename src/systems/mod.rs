/// This module contains ECS systems.
mod input_system;
pub(crate) use input_system::InputSystem;
mod movement_system;
pub(crate) use movement_system::MovementSystem;
pub mod building_system;
pub mod item_movement_system;
pub mod mining_system;
