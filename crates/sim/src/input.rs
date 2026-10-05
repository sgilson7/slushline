//! One input per seat per tick (D8). The only thing a player, a pilot or a
//! replay can give the world.

use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
pub struct Input(pub u16);

impl Input {
    pub const NONE: Input = Input(0);
    /// Bits 0 to 3 pull the handles of spouts one to four, in belt order.
    pub const SPOUT: [u16; 4] = [1 << 0, 1 << 1, 1 << 2, 1 << 3];
    /// Bits 4 and 5 are reserved for moving or aiming a spout (0.7's seam).
    /// Nothing reads them yet, and a replay that sets one is refused.
    pub const RESERVED: u16 = (1 << 4) | (1 << 5);
    /// "Start", from the page's button or a pilot.
    pub const READY: u16 = 1 << 6;
    /// Every bit that may be set: the spouts and ready.
    pub const VALID: u16 = 0x0F | Self::READY;

    /// The actions a key can be bound to, in the order Settings lists them.
    /// The page reads these names and bits from here rather than keeping a
    /// copy (CLAUDE.md: the page keeps no constant of its own).
    pub const ACTIONS: [(&'static str, u16); 4] = [
        ("spout_1", Self::SPOUT[0]),
        ("spout_2", Self::SPOUT[1]),
        ("spout_3", Self::SPOUT[2]),
        ("spout_4", Self::SPOUT[3]),
    ];

    pub const fn has(self, bit: u16) -> bool {
        self.0 & bit != 0
    }

    /// Whether the key for spout `i` (0-based, belt order) is held.
    pub const fn pulls(self, i: usize) -> bool {
        i < 4 && self.has(Self::SPOUT[i])
    }

    /// Whether every set bit is one a run may carry.
    pub const fn is_valid(self) -> bool {
        self.0 & !Self::VALID == 0
    }
}
