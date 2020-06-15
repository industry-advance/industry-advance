//! This module contains ECS components.
mod input_component;
mod inventory_component;
mod item_source_component;
pub mod miner_component;
mod movement_component;
mod position_component;
mod sprite_component;
pub(crate) use input_component::InputComponent;
pub(crate) use inventory_component::InventoryComponent;
pub(crate) use item_source_component::ItemSourceComponent;
pub(crate) use miner_component::MinerComponent;
pub(crate) use movement_component::MovementComponent;
pub(crate) use position_component::PositionComponent;
pub(crate) use sprite_component::SpriteComponent;
