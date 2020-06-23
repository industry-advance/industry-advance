//! System responsible for moving items between item sources and their item sinks.

use crate::components::{InventoryComponent, ItemSourceComponent};
use crate::{debug_log, debug_log::Subsystems};

use alloc::vec::Vec;

use tiny_ecs::Entities;
fn have_sources(ecs: &mut Entities) -> bool {
    match ecs.borrow_mut::<ItemSourceComponent>() {
        Ok(_) => return true,
        Err(_) => return false,
    }
}
pub fn tick(ecs: &mut Entities) {
    if have_sources(ecs) {
        let mut sources = ecs.borrow_mut::<ItemSourceComponent>().unwrap();
        /* TODO: We want to support sinking items into entities that don't have an inventory,
         but tiny_ecs unfortunately does not seem to like trait objects
         (it doesn't realize that it should extract all objects that implement the trait, not
         the trait itself).
        */
        let mut inventories = ecs.borrow_mut::<InventoryComponent>().unwrap();
        // We're only interested in sources which have targets and are dumping
        let mut active_sources: Vec<&mut ItemSourceComponent> = sources
            .iter_mut()
            .filter(|(_i, source)| source.dump_enabled && source.targets != [None; 4])
            .map(|(_i, source)| source)
            .collect();

        for source in active_sources.iter_mut() {
            // TODO: Let source component decide which of the targets gets the item.
            // TODO: For now, it's always the 1st one that's not None.
            let target: usize = source
                .targets
                .iter()
                .filter(|&target| *target != None)
                .map(|&target| target.unwrap())
                .next()
                .unwrap();
            // Check whether the target can accept (meaning it has an InventoryComponent and accepts the item type)
            match inventories.get_mut(target) {
                Some(inventory) => {
                    if inventory.check_item_accept(source.dump_item, 1) {
                        inventory.insert(source.dump_item, 1).unwrap(); //.expect("Couldn't insert item into inventory!");
                        source.did_transfer = true;
                    } else {
                        debug_log!(
                            Subsystems::InventorySystem,
                            "Inventory doesn't accept item, not inserting"
                        );
                        source.did_transfer = false;
                    }
                }
                None => panic!("Attempt to transfer items to target without InventoryComponent"),
            };
        }
    }
}
