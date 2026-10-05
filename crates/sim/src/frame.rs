//! What the page draws: positions and counts, and nothing it would have to
//! work out (D2). Positions are raw fixed point (1/4096 cm); the shim's
//! `numbers()` gives the page the scale.
//!
//! The units go separately, as a flat array (`units_flat`), because there are
//! hundreds of them and sixty frames a second.

use crate::balance;
use crate::world::{Event, World};
use serde::Serialize;

#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
pub struct Frame {
    pub tick: u32,
    pub done: bool,
    pub lines: Vec<LineView>,
    pub events: Vec<Event>,
}

#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
pub struct LineView {
    pub seat: u8,
    pub belt_y: i32,
    pub floor: i32,
    pub rim: i32,
    pub inner_half: i32,
    pub wall_half: i32,
    pub lid_x: i32,
    pub end_x: i32,
    pub travel: i32,
    pub capacity: u32,
    pub spouts: Vec<SpoutView>,
    pub cups: Vec<CupView>,
    pub emitted: u32,
    pub wasted: u32,
    pub judged: u32,
}

#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
pub struct SpoutView {
    pub x: i32,
    pub y: i32,
    pub pivot: [i32; 2],
    pub tip: [i32; 2],
    /// The valve's opening, raw fixed point: 4096 is fully open.
    pub opening: i32,
    /// The flavors it pours, in their repeating order.
    pub pours: Vec<u8>,
    pub rail: bool,
}

#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
pub struct CupView {
    pub x: i32,
    pub order: u16,
    pub judged: bool,
    /// Each ordered flavor and the units its share is.
    pub shares: Vec<[u32; 2]>,
    /// Units of each flavor in the cup now, by flavor number. Zero once
    /// judged; the result carries what it held.
    pub counts: Vec<u32>,
    /// For each flavor in the cup, the middle of its units: flavor, x, y.
    /// The page labels the slush there when Settings asks it to.
    pub centers: Vec<[i32; 3]>,
    pub score: Option<u32>,
}

pub fn frame(w: &World) -> Frame {
    let mut lines = Vec::new();
    for seat in 0..2 {
        let (Some(line), Some(ls)) = (&w.setup.lines[seat], &w.lines[seat]) else { continue };
        let counts = w.in_cup_counts(seat);
        let (px, py) = balance::HANDLE_PIVOT;
        lines.push(LineView {
            seat: seat as u8,
            belt_y: line.belt_y.0,
            floor: w.floor(seat).0,
            rim: w.rim(seat).0,
            inner_half: line.cup.inner_half.0,
            wall_half: balance::WALL_HALF.0,
            lid_x: line.lid_x.0,
            end_x: line.end_x.0,
            travel: ls.travel.0,
            capacity: line.cup.capacity,
            spouts: line
                .spouts
                .iter()
                .enumerate()
                .map(|(i, s)| {
                    let x = w.spout_x(seat, i);
                    let pivot = [x.0 + crate::fx::Fx::int(px).0, s.y.0 + crate::fx::Fx::int(py).0];
                    let h = &ls.handles[i];
                    SpoutView {
                        x: x.0,
                        y: s.y.0,
                        pivot,
                        tip: [pivot[0] + h.r.x.0, pivot[1] + h.r.y.0],
                        opening: h.opening().0,
                        pours: s.pours.clone(),
                        rail: s.rail.is_some(),
                    }
                })
                .collect(),
            cups: ls
                .cups
                .iter()
                .enumerate()
                .map(|(ci, c)| {
                    let order = &line.orders[c.order as usize];
                    CupView {
                        x: c.x.0,
                        order: c.order,
                        judged: c.judged,
                        shares: order.parts.iter().map(|&(f, _)| [f as u32, order.share(f, line.cup.capacity)]).collect(),
                        counts: counts[ci].clone(),
                        centers: centers(w, seat, ci),
                        score: ls.results.iter().find(|r| r.cup == c.order).map(|r| r.score.score),
                    }
                })
                .collect(),
            emitted: ls.emitted,
            wasted: ls.wasted,
            judged: ls.judged,
        });
    }
    Frame { tick: w.tick, done: w.done(), lines, events: w.events.clone() }
}

fn centers(w: &World, seat: usize, ci: usize) -> Vec<[i32; 3]> {
    let c = &w.lines[seat].as_ref().unwrap().cups[ci];
    let mut sum = vec![(0i64, 0i64, 0i64); w.setup.flavors as usize];
    for u in w.units.iter().filter(|u| u.line as usize == seat && !c.judged && w.inside(u, c)) {
        let s = &mut sum[u.flavor as usize];
        s.0 += u.p.x.0 as i64;
        s.1 += u.p.y.0 as i64;
        s.2 += 1;
    }
    sum.iter()
        .enumerate()
        .filter(|(_, s)| s.2 > 0)
        .map(|(f, s)| [f as i32, (s.0 / s.2) as i32, (s.1 / s.2) as i32])
        .collect()
}

/// Every unit as five numbers: id, x, y, radius (raw), and flavor + 256 ×
/// line. The id lets the page interpolate a unit between two frames.
pub fn units_flat(w: &World) -> Vec<i32> {
    let mut out = Vec::with_capacity(w.units.len() * 5);
    for u in &w.units {
        out.extend_from_slice(&[u.id as i32, u.p.x.0, u.p.y.0, u.r.0, u.flavor as i32 + 256 * u.line as i32]);
    }
    out
}
