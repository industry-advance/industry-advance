use crate::components::miner_component::MiningProgress;
use crate::components::ItemSourceComponent;
use crate::components::{MinerComponent, PositionComponent, SpriteComponent};
use crate::debug_log::*;
use crate::item::Item;
use crate::shared_types::*;

use crate::sprite::{HWSpriteAllocator, HWSpriteSize};
use tiny_ecs::{ECSError, Entities};

/// Adds a mechanical drill to the ECS.
pub(crate) fn add_mechanical_drill(
    entities: &mut Entities,
    sprite_alloc: &mut HWSpriteAllocator,
    target_inventory_entity_id: usize,
) -> Result<usize, ECSError> {
    let entity_id = entities
        .new_entity()
        .with(SpriteComponent::with_pos(
            sprite_alloc,
            "mechanical_drillTiles",
            HWSpriteSize::ThirtyTwoByThirtyTwo,
            64,
            64,
            true
        ))?
        .with(PositionComponent::with_pos((
            Coordinate::from_num(64),
            Coordinate::from_num(64),
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
