//! What a run is built from. `content` writes it from `data/*.json`, a replay
//! carries it, and `sim` never asks which mission it is running (D13).
//!
//! The seams of PLANNING-BRIEF 0.7 are fields here from the first milestone
//! that has spouts: `Spout::rail`, `Spout::nozzle`, `Belt::schedule`, and a
//! second `Line`. Adding a condition on an existing seam is a data change.

use crate::balance;
use crate::fx::Fx;
use serde::{Deserialize, Serialize};

/// A flavor's number: its place in `data/flavors.json`.
pub type Flavor = u8;

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Setup {
    pub seed: u64,
    /// Which of `balance::TUNINGS` (D6).
    pub tuning: u8,
    pub physics: Physics,
    /// Seat 0 runs line 0 and seat 1 runs line 1. The MVP has one line
    /// (PLAN.md §8 Q5); the second is the seam for two seats.
    pub lines: [Option<Line>; 2],
    /// How many flavors exist, so a cup's counts have a fixed length.
    pub flavors: u8,
}

/// The constants a test may need to change, carried in the setup so a
/// replay keeps them (Vagrancy DECISIONS.md, "A replay carries its Setup").
#[derive(Copy, Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Physics {
    pub gravity: Fx,
    pub cap: Fx,
    pub drag: Fx,
}

impl Default for Physics {
    fn default() -> Physics {
        Physics { gravity: balance::GRAVITY, cap: balance::CAP, drag: balance::DRAG }
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Line {
    pub belt: Belt,
    pub cup: CupSpec,
    /// Up to four, in belt order: key 1 pulls the first.
    pub spouts: Vec<Spout>,
    /// One order per cup, in the order the cups come.
    pub orders: Vec<Order>,
    /// Where the first cup's center starts, and the distance between cups.
    pub first_x: Fx,
    pub spacing: Fx,
    /// Where a cup's center is judged.
    pub lid_x: Fx,
    /// Where the line ends: a unit or a cup past it has left the world.
    pub end_x: Fx,
    /// The top of the belt, where a cup's floor sits.
    pub belt_y: Fx,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Belt {
    /// cm per tick, toward +x.
    pub speed: Fx,
    /// Changes of speed at given ticks (0.7's seam: a belt that pauses). Empty
    /// for a steady belt.
    pub schedule: Vec<(u32, Fx)>,
}

impl Belt {
    pub fn speed_at(&self, tick: u32) -> Fx {
        let mut s = self.speed;
        for &(t, v) in &self.schedule {
            if tick >= t {
                s = v;
            }
        }
        s
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct CupSpec {
    /// Half the inside width, and the inside height, in cm.
    pub inner_half: Fx,
    pub inner_height: Fx,
    /// Units the order's shares divide (D9); measured in M1.0.
    pub capacity: u32,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Spout {
    /// Where the nozzle is when the rail, if any, is at its center.
    pub x: Fx,
    pub y: Fx,
    /// The flavors it pours, in their repeating order: one entry for a single
    /// flavor, the blend's cycle for a blend (D10).
    pub pours: Vec<Flavor>,
    pub rail: Option<Rail>,
    pub nozzle: Nozzle,
}

/// A spout that slides back and forth over the belt at a constant speed,
/// turning at each end (0.7; H8).
#[derive(Copy, Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Rail {
    /// cm either side of the spout's `x`.
    pub amplitude: i32,
    /// Ticks for one whole trip out and back. A multiple of four.
    pub period: u32,
}

impl Rail {
    /// The offset at `tick`: 0 at tick 0, moving downstream, `+amplitude` a
    /// quarter period in, `-amplitude` three quarters in. A triangle, not a
    /// sine (H8: a sine gives 21 at tick 45 where this gives 15).
    pub fn offset(&self, tick: u32) -> Fx {
        let q = (self.period / 4).max(1) as i32;
        let ph = (tick % (q as u32 * 4)) as i32;
        let a = Fx::int(self.amplitude);
        let num = if ph < q {
            ph
        } else if ph < 3 * q {
            2 * q - ph
        } else {
            ph - 4 * q
        };
        a.scale(num as i64, q as i64)
    }
}

/// How slush leaves the spout. Only `Fall` exists in the MVP; the rocket
/// nozzle (0.7, `Jet`) arrives in M6 with a `SIM_VERSION` bump.
#[derive(Copy, Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum Nozzle {
    Fall,
}

/// The flavors a cup should hold and their parts (0.4). Always single
/// flavors: a blend counts as its parts (Q3).
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Order {
    pub parts: Vec<(Flavor, u32)>,
}

impl Order {
    pub fn total_parts(&self) -> u32 {
        self.parts.iter().map(|p| p.1).sum()
    }
    /// Units of `flavor` that count toward the score: `C·t/T`, which the
    /// content lint keeps whole (`every_share_is_a_whole_number_of_units`).
    pub fn share(&self, flavor: Flavor, capacity: u32) -> u32 {
        let t: u32 = self.parts.iter().filter(|p| p.0 == flavor).map(|p| p.1).sum();
        let total = self.total_parts().max(1);
        capacity * t / total
    }
}
