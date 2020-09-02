use super::Buildable;
use crate::components::miner_component::MiningProgress;
use crate::components::ItemSourceComponent;
use crate::components::{MinerComponent, PositionComponent, SpriteComponent};
use crate::debug_log::*;
use crate::item::Item;
use crate::shared_types::*;
use crate::sprite::{HWSpriteAllocator, HWSpriteSize};

use tiny_ecs::{ECSError, Entities};

pub struct MechanicalDrill {
    // Where the builder should deposit items into.
    target_inventory_entity_id: usize,
}

impl MechanicalDrill {
    // Create a new mechanical drill object that deposits the mined material into the given inventory.
    //
    // Note that this miner is not actually active until placed into the world by calling `build()`.
    pub fn new(target_inventory_entity_id: usize) -> MechanicalDrill {
        return MechanicalDrill {
            target_inventory_entity_id,
        };
    }
}

impl Buildable for MechanicalDrill {
    // Places the mechanical drill into the world.
    fn build(
        &self,
        pos: Position,
        entities: &mut Entities,
        sprite_alloc: &mut HWSpriteAllocator,
    ) -> Result<usize, ECSError> {
        let entity_id = entities
            .new_entity()
            .with(SpriteComponent::with_pos(
                sprite_alloc,
                "mechanical_drillTiles",
                HWSpriteSize::ThirtyTwoByThirtyTwo,
                pos.0.ceil().to_num(),
                pos.1.ceil().to_num(),
                true,
            ))?
            .with(PositionComponent::with_pos(pos))?
            // TODO: Correct resource type and speed
            .with(MinerComponent::new(
                Item::Copper,
                MiningProgress::from_num(1),
            ))?
            .with(ItemSourceComponent::new(
                Item::Copper,
                [Some(self.target_inventory_entity_id), None, None, None],
            ))?
            .finalise()?;
        debug_log!(Subsystems::BuilderSystem, "Placed mechanical drill");

        return Ok(entity_id);
    }
}
