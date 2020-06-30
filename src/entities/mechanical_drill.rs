use crate::components::miner_component::MiningProgress;
use crate::components::ItemSourceComponent;
use crate::components::{MinerComponent, PositionComponent, SpriteComponent};
use crate::debug_log::*;
use crate::item::Item;
use crate::shared_types::*;

use crate::sprite::{HWSpriteAllocator, HWSpriteSize};
use core::convert::TryInto;
use tiny_ecs::{ECSError, Entities};

/// Adds a mechanical drill to the ECS.
pub(crate) fn add_mechanical_drill(
    entities: &mut Entities,
    sprite_alloc: &mut HWSpriteAllocator,
    target_inventory_entity_id: usize,
    x_pos: usize,
    y_pos: usize,
) -> Result<usize, ECSError> {
    let entity_id = entities
        .new_entity()
        .with(SpriteComponent::with_pos(
            sprite_alloc,
            "mechanical_drillTiles",
            HWSpriteSize::ThirtyTwoByThirtyTwo,
            x_pos.try_into().unwrap(),
            y_pos.try_into().unwrap(),
            true,
        ))?
        .with(PositionComponent::with_pos((
            Coordinate::from_num(x_pos),
            Coordinate::from_num(y_pos),
        )))?
        // TODO: Correct resource type and speed
        .with(MinerComponent::new(
            Item::Copper,
            MiningProgress::from_num(1),
        ))?
        .with(ItemSourceComponent::new(
            Item::Copper,
            [Some(target_inventory_entity_id), None, None, None],
        ))?
        .finalise()?;
    debug_log!(Subsystems::Entity, "Created mechanical drill");

    return Ok(entity_id);
}
