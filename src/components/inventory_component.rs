use crate::item::Item;
use crate::{debug_log, debug_log::Subsystems};

use hashbrown::hash_map::HashMap;
use twox_hash::XxHash64;

use alloc::vec::Vec;
use core::fmt;
use core::hash::BuildHasherDefault;

/// Describes an inventory.
#[derive(Debug, Clone)]
pub struct InventoryComponent {
    /* Kind of items to accept. If `None`, all items are accepted.
    If empty, no kind of item is accepted. */
    item_whitelist: Option<Vec<Item>>,
    // Amount of free space remaining
    free: usize,
    // Actual inventory contents
    contents: HashMap<Item, usize, BuildHasherDefault<XxHash64>>,
}

// Metadata of ItemTransaction
pub struct InventoryPartialTransfer {
    // Amount of items accepted
    amount: usize,
}

impl InventoryComponent {
    /// Create a new empty inventory with the given capacity.
    pub fn new(cap: usize) -> InventoryComponent {
        debug_log!(Subsystems::InventorySystem, "Created new inventory");
        let contents: HashMap<Item, usize, BuildHasherDefault<XxHash64>> = Default::default();
        return InventoryComponent {
            free: cap,
            item_whitelist: None,
            contents,
        };
    }

    /// Gets a listing of the inventory contents without removing anything.
    pub fn peek(&self) -> &HashMap<Item, usize, BuildHasherDefault<XxHash64>> {
        return &self.contents;
    }

    /// Inserts the given quantity of given item.
    /// Returns an `InventoryError` if space is insufficient.
    pub fn insert(&mut self, item: Item, quantity: usize) -> Result<(), InventoryError> {
        // Check that we have space
        if (self.free as i32 - quantity as i32) < 0 {
            if self.free > 0 {
                return Err(InventoryError::PartialTransfer(self.free));
            }
            return Err(InventoryError::Full);
        }
        // Check that the item is on the whitelist, if it exists
        if self.item_whitelist != None && !self.item_whitelist.as_ref().unwrap().contains(&item) {
            return Err(InventoryError::RejectedItemType);
        }

        // Actually insert the item by increasing quantity if it's in the map or creating a new entry if not.
        if self.contents.contains_key(&item) {
            let new_quantity = self.contents.get(&item).unwrap() + quantity;
            self.contents.insert(item, new_quantity);
        } else {
            self.contents.insert(item, quantity);
        }
        self.free -= quantity;

        debug_log!(
            Subsystems::InventorySystem,
            "Inserted {} {}",
            quantity,
            item
        );
        return Ok(());
    }

    /// Retrieves the given quantity of given item from the inventory.
    /// Returns an `InventoryError` if inventory does not contain a sufficient amount of the item.
    pub fn retrieve(&mut self, item: Item, quantity: usize) -> Result<(), InventoryError> {
        if !self.contents.contains_key(&item) {
            return Err(InventoryError::InsufficientItems);
        }

        let contained_quantity = *self.contents.get(&item).unwrap();

        if contained_quantity < quantity {
            return Err(InventoryError::InsufficientItems);
        }

        self.contents.insert(item, contained_quantity - quantity);
        self.free += quantity;

        return Ok(());
    }

    /// Returns whether the given item and quantity can be accepted.
    pub fn check_item_accept(&self, item: Item, quantity: usize) -> bool {
        // We accept if we have space and the item is whitelisted (if it exists)
        if self.item_whitelist != None && !self.item_whitelist.as_ref().unwrap().contains(&item) {
            return false;
        }
        return self.free >= quantity;
    }
}

/// Describes error conditions related to inventory handling.
pub enum InventoryError {
    PartialTransfer(usize),
    Full,
    RejectedItemType,
    InsufficientItems,
}

impl fmt::Display for InventoryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use InventoryError::*;
        match self {
            PartialTransfer(amount) => write!(
                f,
                "InventoryError: Only {} items could be transferred",
                amount
            ),
            Full => write!(f, "InventoryError: Inventory is full"),
            RejectedItemType => write!(f, "InventoryError: Item is not in whitelist"),
            InsufficientItems => write!(
                f,
                "InventoryError: Inventory does not have the required items"
            ),
        }
    }
}

impl fmt::Debug for InventoryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        return <InventoryError as fmt::Display>::fmt(&self, f);
    }
}
