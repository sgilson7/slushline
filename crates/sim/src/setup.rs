//! What a run is built from. `content` writes it from `data/*.json`, a replay
//! carries it, and `sim` never asks which mission it is running (D13).
//!
//! The seams of PLANNING-BRIEF 0.7 are fields here from the first milestone
//! that has spouts: `Spout::rail`, `Spout::nozzle`, `Belt::schedule`, and a
//! second `Line`. Adding a condition on an existing seam is a data change.

use crate::balance;
use crate::fx::{Fx, V2};
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
    /// Per flavor, by flavor number: how heavy its slush is (Sam,
    /// 2026-10-06: "a slush type that is super heavy").
    pub flavor_physics: Vec<FlavorPhysics>,
}

/// How one flavor's slush moves.
#[derive(Copy, Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct FlavorPhysics {
    /// Its mass in contacts, against 1 for ordinary slush: a heavier unit is
    /// moved less by a push, so it sinks through lighter slush.
    pub mass: u8,
    /// Gravity on it, in percent of the world's.
    pub gravity_pct: u16,
}

impl Default for FlavorPhysics {
    fn default() -> FlavorPhysics {
        FlavorPhysics { mass: 1, gravity_pct: 100 }
    }
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
    /// Force fields that push the line's falling slush (Sam, 2026-10-05:
    /// "gravity fields that act almost like voltage fields to deflect the
    /// slurpee"). Empty on most lines.
    pub fields: Vec<Field>,
    /// Rows of cups (Sam, 2026-10-06: "multiple rows of cups"): cup k rides
    /// `rows[k % rows.len()]` cm above the belt, on a rail if it is above
    /// it. Empty is one row, on the belt.
    pub rows: Vec<Fx>,
    /// Cups that ride up and down as they go ("cups that move up and down as
    /// you pour them"): every cup's height moves by this triangle wave.
    pub bob: Option<Rail>,
}

/// A field that pushes slush, as charged plates or a charge push a beam.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum Field {
    /// A uniform push inside a box, as between two charged plates: every
    /// unit of the line inside it gains `accel` each tick. With `flip`, the
    /// push reverses every `flip` ticks, as an alternating voltage does.
    Plates { x0: Fx, x1: Fx, y0: Fx, y1: Fx, accel: V2, flip: Option<u32> },
    /// A charge at `at`: inside `radius` it pushes a unit straight away from
    /// itself (or, with a negative `strength`, pulls it in), by `strength`
    /// at the center falling linearly to nothing at the radius.
    Charge { at: V2, strength: Fx, radius: Fx },
}

impl Field {
    /// The push this field gives a unit at `p` on `tick`, cm per tick².
    pub fn accel(&self, p: V2, tick: u32) -> V2 {
        match *self {
            Field::Plates { x0, x1, y0, y1, accel, flip } => {
                if p.x < x0 || p.x > x1 || p.y < y0 || p.y > y1 {
                    return V2::ZERO;
                }
                match flip {
                    Some(n) if n > 0 && (tick / n) % 2 == 1 => -accel,
                    _ => accel,
                }
            }
            Field::Charge { at, strength, radius } => {
                let d = p - at;
                let r = radius.raw() as i64;
                let dsq = d.len_sq_raw();
                if dsq >= r * r || dsq == 0 {
                    return V2::ZERO;
                }
                let dist = d.len();
                let mag = strength.scale((radius - dist).raw() as i64, radius.raw().max(1) as i64);
                d.with_len(mag.abs()) * mag.signum()
            }
        }
    }

    /// Whether the field's push is reversed on `tick` (for drawing).
    pub fn flipped(&self, tick: u32) -> bool {
        matches!(*self, Field::Plates { flip: Some(n), .. } if n > 0 && (tick / n) % 2 == 1)
    }
}

impl Line {
    /// The sum of every field's push at `p` on `tick`.
    pub fn field_accel(&self, p: V2, tick: u32) -> V2 {
        self.fields.iter().fold(V2::ZERO, |a, f| a + f.accel(p, tick))
    }
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
    /// How much wider each side is at the rim than halfway up, and narrower
    /// at the floor: the flower pot's lean.
    pub flare: Fx,
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
    /// The rocket nozzle (PLANNING-BRIEF 0.7; Sam, 2026-10-06: "a nozzle
    /// that fires slush downwards in a small beam at high pressure"): each
    /// unit is traced straight down to the first thing under the nozzle and
    /// put there moving down at `speed`, so there is no fall time and it
    /// pushes what it hits.
    Jet { speed: Fx },
}

/// The flavors a cup should hold and their parts (0.4). Always single
/// flavors: a blend counts as its parts (Q3).
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Order {
    pub parts: Vec<(Flavor, u32)>,
    /// The parts are layers, from the bottom up (Sam, 2026-10-06: "levels
    /// where the order of the slushy in the cup matters").
    pub layered: bool,
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
