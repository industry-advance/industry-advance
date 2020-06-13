//! This module implements a system which ticks miners and causes them to produce resources.

use crate::components::MinerComponent;

use tiny_ecs::{ECSError, Entities};

pub fn tick(ecs: &mut Entities, live_entities: &[usize]) {
    let miners = ecs.borrow_mut::<MinerComponent>().unwrap();
    for id in live_entities {
        if ecs.entity_contains::<MinerComponent>(*id) {
            let mut e_miner = miners.get_mut(*id).unwrap();
            if !e_miner.backed_up {
                // Increment the amount of item produced
                e_miner.item_progress += e_miner.speed;
            }
            // TODO: Else branch checking whether output is now possible
        }
    }
}
