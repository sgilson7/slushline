//! `data/*.json` into a `sim::Setup` (D13). Every number that reaches `sim`
//! is a whole number or a ratio of whole numbers: no float crosses into a
//! world, even here where floats are allowed.

use serde::Deserialize;
use serde_json::Value;
use sim::fx::Fx;
use sim::setup::{Belt, CupSpec, Flavor, Line, Nozzle, Order, Physics, Rail, Setup, Spout};
use std::collections::BTreeMap;

pub const CUPS_JSON: &str = include_str!("../../../data/cups.json");
pub const LINE_JSON: &str = include_str!("../../../data/line.json");
pub const BLENDS_JSON: &str = include_str!("../../../data/blends.json");

#[derive(Deserialize, Clone, Copy, Debug)]
pub struct CupDef {
    pub inner_half: i32,
    pub inner_height: i32,
    pub capacity: u32,
}

#[derive(Deserialize, Debug)]
pub struct LineDef {
    pub belt_y: i32,
    pub spout_y: i32,
    pub first_x: i32,
    pub spacing: i32,
    pub lid_x: i32,
    pub end_x: i32,
    pub spout_x: BTreeMap<String, Vec<i32>>,
    pub belts: BTreeMap<String, i32>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct BlendDef {
    pub id: String,
    pub parts: Vec<(String, u32)>,
}

pub fn cups() -> BTreeMap<String, CupDef> {
    let v: Value = serde_json::from_str(CUPS_JSON).expect("data/cups.json is valid");
    v.as_object()
        .unwrap()
        .iter()
        .filter(|(k, _)| !k.starts_with('_'))
        .map(|(k, c)| (k.clone(), serde_json::from_value(c.clone()).expect("a cup definition")))
        .collect()
}

pub fn line_def() -> LineDef {
    serde_json::from_str(LINE_JSON).expect("data/line.json is valid")
}

pub fn blends() -> Vec<BlendDef> {
    #[derive(Deserialize)]
    struct F {
        blends: Vec<BlendDef>,
    }
    serde_json::from_str::<F>(BLENDS_JSON).expect("data/blends.json is valid").blends
}

/// A flavor's number in `sim`: its place in `data/flavors.json`.
pub fn flavor(id: &str) -> Flavor {
    crate::look::flavors().iter().position(|f| f.id == id).unwrap_or_else(|| panic!("no flavor {id}")) as Flavor
}

pub fn flavor_count() -> u8 {
    crate::look::flavors().len() as u8
}

/// What a spout named `id` pours, in its repeating order: a flavor's one
/// entry, or a blend's cycle built from its parts in the order listed (D10).
/// Parts cherry 1, cola 1 give cherry, cola (H4).
pub fn pours(id: &str) -> Vec<Flavor> {
    if let Some(b) = blends().into_iter().find(|b| b.id == id) {
        let mut left: Vec<(Flavor, u32)> = b.parts.iter().map(|(f, n)| (flavor(f), *n)).collect();
        let mut out = Vec::new();
        // Round robin over the parts until each has poured its count: 2 to 1
        // gives a, b, a.
        while left.iter().any(|p| p.1 > 0) {
            for p in left.iter_mut() {
                if p.1 > 0 {
                    out.push(p.0);
                    p.1 -= 1;
                }
            }
        }
        out
    } else {
        vec![flavor(id)]
    }
}

pub fn is_blend(id: &str) -> bool {
    blends().iter().any(|b| b.id == id)
}

/// An order from `(flavor id, parts)`.
pub fn order(parts: &[(&str, u32)]) -> Order {
    Order { parts: parts.iter().map(|(f, n)| (flavor(f), *n)).collect() }
}

/// A line from data: its spouts by id in belt order, its cup size, its belt,
/// and one order per cup.
pub fn line(spouts: &[&str], cup: &str, belt: &str, orders: Vec<Order>) -> Line {
    let d = line_def();
    let c = cups()[cup];
    let xs = &d.spout_x[&spouts.len().to_string()];
    Line {
        belt: Belt { speed: Fx::ratio(d.belts[belt] as i64, sim::balance::TICKS_PER_SECOND as i64), schedule: Vec::new() },
        cup: CupSpec { inner_half: Fx::int(c.inner_half), inner_height: Fx::int(c.inner_height), capacity: c.capacity },
        spouts: spouts
            .iter()
            .zip(xs)
            .map(|(id, &x)| Spout { x: Fx::int(x), y: Fx::int(d.spout_y), pours: pours(id), rail: None, nozzle: Nozzle::Fall })
            .collect(),
        orders,
        first_x: Fx::int(d.first_x),
        spacing: Fx::int(d.spacing),
        lid_x: Fx::int(d.lid_x),
        end_x: Fx::int(d.end_x),
        belt_y: Fx::int(d.belt_y),
        fields: Vec::new(),
    }
}

pub fn setup_of(seed: u64, tuning: u8, line: Line) -> Setup {
    Setup { seed, tuning, physics: Physics::default(), lines: [Some(line), None], flavors: flavor_count() }
}

/// The pour of M1: one cola spout over one regular cup that stands still
/// under it, with a lid nobody reaches.
pub fn standing(seed: u64, tuning: u8) -> Setup {
    let mut l = line(&["cola"], "regular", "steady", vec![order(&[("cola", 1)])]);
    l.belt.speed = Fx(0);
    l.first_x = l.spouts[0].x;
    l.lid_x = Fx::int(10_000);
    setup_of(seed, tuning, l)
}

/// A spout on a rail (`data/conditions.json`, `rail`).
pub fn with_rail(mut l: Line, spout: usize, amplitude: i32, period: u32) -> Line {
    l.spouts[spout].rail = Some(Rail { amplitude, period });
    l
}
