use crate::item::Item;

use hashbrown::hash_map::HashMap;
use twox_hash::XxHash64;

use core::hash::BuildHasherDefault;

/// Describes an inventory.
pub struct InventoryComponent {
    // Maximum capacity of an inventory
    cap: usize,
    // Amount of free space remaining
    free: usize,
    pub contents: HashMap<Item, usize, BuildHasherDefault<XxHash64>>,
}

impl InventoryComponent {
    /// Create a new empty inventory with the given capacity.
    pub fn new(cap: usize) -> InventoryComponent {
        let contents: HashMap<Item, usize, BuildHasherDefault<XxHash64>> = Default::default();
        return InventoryComponent {
            cap,
            free: cap,
            contents,
        };
    }

    /// Inserts the given quantity of given item.
    /// Returns an `InventoryError` if space is insufficient.
    pub fn insert(&mut self, item: Item, quantity: usize) -> Result<(), InventoryError> {
        if (self.free - quantity) < 0 {
            return Err(InventoryError {});
        }
        // Actually insert the item by increasing quantity if it's in the map or creating a new entry if not.
        if self.contents.contains_key(&item) {
            let new_quantity = self.contents.get(&item).unwrap() + quantity;
            self.contents.insert(item, new_quantity);
        } else {
            self.contents.insert(item, quantity);
            self.free -= quantity;
        }
        return Ok(());
    }

    /// Retrieves the given quantity of given item from the inventory.
    /// Returns an `InventoryError` if inventory does not contain a sufficient amount of the item.
    pub fn retrieve(&mut self, item: Item, quantity: usize) -> Result<(), InventoryError> {
        if !self.contents.contains_key(&item) {
            return Err(InventoryError {});
        }

        let contained_quantity = *self.contents.get(&item).unwrap();

        if contained_quantity < quantity {
            return Err(InventoryError {});
        }

        self.contents.insert(item, contained_quantity - quantity);
        self.free += quantity;

        return Ok(());
    }
}

/// Describes error conditions related to inventory handling.
pub struct InventoryError {}
