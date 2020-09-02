use crate::components::{PositionComponent, SpriteComponent};
use crate::debug_log::*;
use crate::shared_types::*;
use crate::sprite::{HWSpriteAllocator, HWSpriteSize};

use tiny_ecs::{ECSError, Entities};

#[derive(Debug, Clone)]
pub struct CopperWall {}

impl super::Buildable for CopperWall {
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
                "copper_wallTiles",
                HWSpriteSize::SixteenBySixteen,
                pos.0.to_num(),
                pos.1.to_num(),
                true,
            ))?
            .with(PositionComponent::with_pos(pos))?
            .finalise()?;
        debug_log!(Subsystems::Entity, "Created copper wall");

        return Ok(entity_id);
    }
}
