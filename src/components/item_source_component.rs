use crate::item::Item;

/// This component is designed for entities which produce/transport items
/// and want to dump them into some other entity with an `InventoryComponent`.
pub struct ItemSourceComponent {
    // IDs of item sink entities to dump items into if possible.
    // The exact distribution of items to these targets is determined by the item movement system.
    pub targets: [Option<usize>; 4],
    // Item to dump
    pub dump_item: Item,
    // Whether to dump items continuously starting on the next tick.
    // Dumping does not occur until this is set.
    pub dump_enabled: bool,
    // Whether a transfer was successful on the previous tick
    pub did_transfer: bool,
}

impl ItemSourceComponent {
    pub fn new(dump_item: Item, targets: [Option<usize>; 4]) -> ItemSourceComponent {
        return ItemSourceComponent {
            targets,
            dump_item,
            dump_enabled: false,
            did_transfer: false,
        };
    }
}
