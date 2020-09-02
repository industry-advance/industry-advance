//! This system is responsible for executing the orders of `BuilderComponent`.

use crate::components::BuilderComponent;
use crate::sprite::HWSpriteAllocator;
use crate::{debug_log, debug_log::Subsystems};

use alloc::vec::Vec;

use tiny_ecs::Entities;

/// Tick the system by placing the object to be built into the world, if any.
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
            if builder.buildable.is_some() {
                debug_log!(Subsystems::BuilderSystem, "Building");
                // Create a new miner
                let built_entity_id = builder
                    .buildable
                    .unwrap()
                    .build(builder.pos.unwrap(), ecs, sprite_alloc)
                    .unwrap();
                live_entities.push(built_entity_id);
            }
            // Ensure nothing gets built next tick
            let mut builders = ecs.borrow_mut::<BuilderComponent>().unwrap();
            let e_builder = builders.get_mut(id).unwrap();
            e_builder.buildable = None;
        }
    }
}
