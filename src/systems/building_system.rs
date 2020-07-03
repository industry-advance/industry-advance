//! This system is responsible for placing a miner at the position specified by the BuilderComponent.
//! TODO: Support building other stuff, probably requires defining costs for entities and a universal interface for their creation.

use crate::components::BuilderComponent;
use crate::entities::add_mechanical_drill;
use crate::sprite::HWSpriteAllocator;
use crate::{debug_log, debug_log::Subsystems};

use alloc::vec::Vec;

use tiny_ecs::Entities;

/// Tick the system.
pub fn tick(
    ecs: &mut Entities,
    live_entities: &mut Vec<usize>,
    sprite_alloc: &mut HWSpriteAllocator,
) {
    for id in live_entities.clone() {
        if ecs.entity_contains::<BuilderComponent>(id) {
            let builders = ecs.borrow::<BuilderComponent>().unwrap();
            let e_builder = builders.get(id).unwrap();
            let builder = e_builder.clone();
            // Gotta make borrow checker happy here
            drop(builders);
            if builder.build {
                debug_log!(Subsystems::BuilderSystem, "Build!");
                // Create a new miner
                let miner_id =
                    add_mechanical_drill(ecs, sprite_alloc, 0xDEAD, builder.x_pos, builder.y_pos)
                        .unwrap();
                live_entities.push(miner_id);
            }
            // Ensure nothing gets built next tick
            let mut builders = ecs.borrow_mut::<BuilderComponent>().unwrap();
            let e_builder = builders.get_mut(id).unwrap();
            e_builder.build = false;
        }
    }
}
