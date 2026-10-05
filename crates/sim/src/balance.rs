//! Every number that decides how a pour feels, in one place.
//!
//! The page reads these through the shim; nothing outside `sim` keeps a copy.
//! Values marked *(guess)* have not been measured yet. Each recon replaces a
//! guess with a number and says which command produced it.

use crate::fx::Fx;
use serde::{Deserialize, Serialize};

/// Ticks per second (D4). The page owns the clock and steps this often.
pub const TICKS_PER_SECOND: u32 = 60;

/// 981 cm/s² at 60 ticks a second: 981 / 3600 = 0.2725 cm per tick², 1,116
/// raw (Vagrancy's, by the same arithmetic).
pub const GRAVITY: Fx = Fx::ratio(981, 3600);
/// No unit falls faster than this, cm per tick. Below a unit's diameter, so
/// two units cannot pass through each other between ticks. *(guess)*
pub const CAP: Fx = Fx::int(3);
/// The share of a unit's speed lost each tick to the air. *(guess)*
pub const DRAG: Fx = Fx::ratio(1, 100);

/// A unit at full size, and its radius as it leaves the spout. Areas of 4 and
/// 2.2 cm² × π: the unit grows by 80 % of its area, the trade's figure for
/// overrun (PLANNING-BRIEF 0.2). *(guess; M1.0)*
pub const R_FULL: Fx = Fx::int(2);
pub const R_BIRTH: Fx = Fx::ratio(149, 100);

/// Relaxation passes per tick, in a fixed order (D5). *(guess; M1.0)*
pub const PASSES: u32 = 6;
/// Pairs closer than the sum of their radii plus this are put on the tick's
/// contact list, so a pass can still find a pair the last one pushed
/// together. *(guess)*
pub const CONTACT_MARGIN: Fx = Fx::ratio(1, 2);

/// A unit touching something and moving slower than this, cm per tick, is
/// set still (`slush::rest`). *(guess; M1.0)*
pub const REST_SPEED: Fx = Fx::ratio(1, 10);

/// Half the thickness of a cup's walls and floor, cm.
pub const WALL_HALF: Fx = Fx::int(1);

/// How fast a unit leaves the spout, cm per tick downward. *(guess)*
pub const SPOUT_SPEED: Fx = Fx::int(1);
/// Units leave across the nozzle's width at these offsets, in this order, so
/// a pour does not stack in one column (Part C, M1.0). In cm.
pub const NOZZLE_OFFSETS: [i32; 5] = [0, -2, 1, -1, 2];
/// Units per tick from a fully open valve (H5 uses 2). *(guess; M1.0)*
pub const VALVE_RATE: Fx = Fx::int(2);

/// The handle (D7): a stick from an anchored pivot. Closed, it points
/// straight up; pulled, it turns clockwise toward the belt's downstream end
/// by up to `HANDLE_TRAVEL_DEG`.
pub const HANDLE_LEN: Fx = Fx::int(14);
pub const HANDLE_TRAVEL_DEG: i32 = 60;
/// The servo (Vagrancy `world.rs:421-458`): the held key accelerates the
/// handle by up to this, rad/tick², until it turns at `HANDLE_SPEED`.
pub const HANDLE_ACCEL: Fx = Fx::ratio(1, 25);
pub const HANDLE_SPEED: Fx = Fx::ratio(1, 10);
/// The share of the handle's speed lost each tick, so the spring settles.
pub const HANDLE_DRAG: Fx = Fx::ratio(1, 20);

/// What makes it slush (D6), and the return spring (D7). Sam chooses among
/// the tunings by playing; the page takes `?tuning=0|1|2`.
#[derive(Copy, Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Tuning {
    /// A contact's sideways slip is cancelled outright while it is under this
    /// many times the overlap: the slope rule.
    pub static_k: Fx,
    /// Past that, the slip is reduced by this many times the overlap.
    pub kinetic_k: Fx,
    /// Each tick a unit's velocity moves this share of the way toward the
    /// mean velocity of the units touching it: the thickness rule.
    pub thick_k: Fx,
    /// Ticks for a unit to grow from `R_BIRTH` to `R_FULL`: the swell.
    pub swell_ticks: u16,
    /// The return spring: angular acceleration toward closed at full travel,
    /// rad/tick², in proportion to the opening.
    pub spring: Fx,
}

/// Three candidate tunings (M1.0, M2.0). *(guesses until recon)*
pub const TUNINGS: [Tuning; 3] = [
    Tuning { static_k: Fx::int(2), kinetic_k: Fx::int(1), thick_k: Fx::ratio(1, 10), swell_ticks: 60, spring: Fx::ratio(1, 80) },
    Tuning { static_k: Fx::int(4), kinetic_k: Fx::int(2), thick_k: Fx::ratio(1, 5), swell_ticks: 90, spring: Fx::ratio(1, 55) },
    Tuning { static_k: Fx::int(8), kinetic_k: Fx::int(3), thick_k: Fx::ratio(1, 3), swell_ticks: 120, spring: Fx::ratio(1, 40) },
];
pub const DEFAULT_TUNING: u8 = 1;

/// Tuning `i`. Index 200 is the recon's control: tuning 1 with the slope
/// rule and the thickness off, which is water (M1.0). No mission uses it.
pub fn tuning(i: u8) -> Tuning {
    if i == CONTROL {
        return Tuning { static_k: Fx(0), kinetic_k: Fx(0), thick_k: Fx(0), ..TUNINGS[1] };
    }
    TUNINGS[(i as usize).min(TUNINGS.len() - 1)]
}
pub const CONTROL: u8 = 200;

/// The lid scores a cup out of this.
pub const MAX_SCORE: u32 = 100;

/// Where a handle's pivot sits relative to its spout's nozzle, cm. It is
/// drawn there; nothing in the world touches it.
pub const HANDLE_PIVOT: (i32, i32) = (0, 16);
