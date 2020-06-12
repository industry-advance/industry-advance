use core::fmt;

/// An item is something that can exist in an inventory and has an associated sprite.
/// These are taken from the original game's `Items.java`.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
// A lot of items are never used
#[allow(dead_code)]
pub enum Item {
    Scrap,
    Copper,
    Lead,
    Graphite,
    Coal,
    Titanium,
    Thorium,
    Silicon,
    Plastanium,
    PhaseFabric,
    SurgeAlloy,
    SporePod,
    Sand,
    BlastCompound,
    Pyratite,
    Metaglass,
}

impl Item {
    /// Returns the filename of the item's sprite.
    pub fn to_sprite_name(self) -> &'static str {
        use Item::*;
        match self {
            Scrap => "item_scrapTiles",
            Copper => "item_copperTiles",
            Lead => "item_leadTiles",
            Graphite => "item_graphiteTiles",
            BlastCompound => "item_blast_compoundTiles",
            Coal => "item_coalTiles",
            Metaglass => "item_metaglassTiles",
            PhaseFabric => "item_phase_fabricTiles",
            Plastanium => "item_plastaniumTiles",
            Pyratite => "item_pyratiteTiles",
            Sand => "item_sandTiles",
            Silicon => "item_siliconTiles",
            SporePod => "item_spore_podTiles",
            SurgeAlloy => "item_surge_alloyTiles",
            Thorium => "item_thoriumTiles",
            Titanium => "item_titaniumTiles",
        }
    }
}

impl fmt::Display for Item {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Item::*;
        match self {
            Scrap => write!(f, "Scrap"),
            Copper => write!(f, "Copper"),
            Lead => write!(f, "Lead"),
            Graphite => write!(f, "Graphite"),
            BlastCompound => write!(f, "Blast Compound"),
            Coal => write!(f, "Coal"),
            Metaglass => write!(f, "Metaglass"),
            PhaseFabric => write!(f, "Phase Fabric"),
            Plastanium => write!(f, "Plastanium"),
            Pyratite => write!(f, "Pyratite"),
            Sand => write!(f, "Sand"),
            Silicon => write!(f, "Silicon"),
            SporePod => write!(f, "Spore Pod"),
            SurgeAlloy => write!(f, "Surge Alloy"),
            Thorium => write!(f, "Thorium"),
            Titanium => write!(f, "Titanium"),
        }
    }
}
