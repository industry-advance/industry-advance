//! This module contains a macro and related types which allows for enabling debug output
//! for particular subsystems at compile time.
//!
//! If a subsystem's debugging is disabled, debug print invocations cause no performance penalty.
//!
//! Please disable all subsystems you're not currently working on before publishing a release binary!
//!
//! # WARNING
//! Do not attempt to integrate this logging system into ewram_alloc! It causes mysterious writes to wrong
//! addresses, and we don't know why.

/// List of subsystems logging can be enabled for.
#[derive(Debug, PartialEq)]
pub enum Subsystems {
    Game,
    Main,
    Entity,
    HWSprite,
    InputSystem,
    InventorySystem,
    MovementSystem,
    BuilderSystem,
    Text,
    Map,
    Menu,
}

impl Subsystems {
    pub fn to_str(&self) -> &str {
        use Subsystems::*;
        match self {
            Game => "GAME",
            Main => "MAIN",
            Entity => "ENTITY",
            HWSprite => "HW SPRITE",
            InputSystem => "INPUT SYSTEM",
            InventorySystem => "INVENTORY SYSTEM",
            MovementSystem => "MOVEMENT SYSTEM",
            BuilderSystem => "BUILDER SYSTEM",
            Text => "TEXT",
            Map => "MAP",
            Menu => "MENU",
        }
    }
}
/// List of subsystems to enable logging for
pub const ENABLED_SUBSYSTEMS: [Subsystems; 3] = [
    Subsystems::Main,
    Subsystems::Game,
    Subsystems::InventorySystem,
];

#[macro_export]
macro_rules! debug_log {
  ($subsystem:expr ,$($arg:tt)*) => {{
    if $crate::debug_log::ENABLED_SUBSYSTEMS.contains(&$subsystem) {
        use gba::mgba::{MGBADebug, MGBADebugLevel};
        use core::fmt::Write;
        if let Some(mut mgba) = MGBADebug::new() {
          let _ = write!(mgba, "{}", format!("[{}] {}", $subsystem.to_str(), format!($($arg)*)));
          mgba.send(MGBADebugLevel::Debug);
        }
    }
  }};
}
