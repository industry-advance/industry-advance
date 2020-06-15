//! This module implements a system which ticks miners and causes them to produce resources.

use crate::components::{
    miner_component::{MiningProgress, ONE_PROGRESS},
    ItemSourceComponent, MinerComponent,
};

use tiny_ecs::Entities;

/// How many items can be stuck in the miner
const MAX_BACKUP: MiningProgress =
    MiningProgress::from_bits(0b0000_0000_0000_0000_0000_0001_0000_0000);

pub fn tick(ecs: &mut Entities, live_entities: &[usize]) {
    let mut miners = ecs.borrow_mut::<MinerComponent>().unwrap();
    let mut item_sources = ecs.borrow_mut::<ItemSourceComponent>().unwrap();
    for id in live_entities {
        if ecs.entity_contains::<MinerComponent>(*id) {
            let mut e_miner = miners.get_mut(*id).unwrap();
            // Each miner has to have an ItemSourceComponent which regulates item transfer.
            let mut e_item_source = item_sources.get_mut(*id).unwrap();

            // Try to output an item if possible
            if e_miner.item_progress >= ONE_PROGRESS {
                e_item_source.dump_enabled = true;
            } else {
                // If not, stop dumping
                e_item_source.dump_enabled = false;
            }
            if e_miner.backed_up {
                // Check whether an item was transferred out last tick,
                // meaning we've got to remove it from the backlog
                if e_item_source.did_transfer {
                    e_miner.item_progress -= ONE_PROGRESS;
                }
                // We may not be backed up anymore
                if e_miner.item_progress < MAX_BACKUP {
                    e_miner.backed_up = false;
                }
            } else {
                // Produce
                e_miner.item_progress += e_miner.speed;
                // Not backed up, check whether we should be
                if e_miner.item_progress >= MAX_BACKUP {
                    e_miner.backed_up = true;
                }
            }
        }
    }
}
