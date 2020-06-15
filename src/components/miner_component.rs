//! Component describing entities which can dig resources out of ore patches.

use crate::item::Item;
use fixed::{types::extra::U8, FixedU32};

pub type MiningProgress = FixedU32<U8>;
pub const ZERO_PROGRESS: MiningProgress = MiningProgress::from_bits(0b0);
pub const ONE_PROGRESS: MiningProgress =
    MiningProgress::from_bits(0b0000_0000_0000_0000_0000_0001_0000_0000);

/// A miner. Note that the kind of item to be extracted is not checked to coincide with
/// the tile below the miner, so you must call the constructor with the correct item.
/// The `mining_system` assumes that all entities which posess this component also have a
/// `ItemSourceComponent`
pub struct MinerComponent {
    pub obtained_resource: Item,
    // Fixed-point value describing how many resources are extracted per tick.
    pub speed: MiningProgress,
    // Fixed-point value describing how many items are complete
    pub item_progress: MiningProgress,
    // Whether the miner is backed up due to inability to emit items
    pub backed_up: bool,
}

impl MinerComponent {
    /// Creates a new MinerComponent with given speed and item type.
    pub fn new(obtained_resource: Item, speed: MiningProgress) -> MinerComponent {
        return MinerComponent {
            obtained_resource,
            speed,
            item_progress: ZERO_PROGRESS,
            backed_up: false,
        };
    }
}
