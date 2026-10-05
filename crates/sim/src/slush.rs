//! Slush: units, the grid that finds their neighbors, and the three rules
//! that make it slush rather than water (D5, D6).
//!
//! A unit is one Verlet particle: where it is, where it was last tick, a
//! radius and a flavor. Units have no sticks. Each tick the pairs that touch
//! are found once through a uniform grid held in vectors, visited cell by
//! cell and then by index, so every machine sees the pairs in one order; then
//! each pass pushes every touching pair apart as an exact pair shift.

use crate::balance::{self, Tuning};
use crate::fx::{Fx, V2};
use crate::setup::Flavor;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Unit {
    /// Its number on its line, in the order units left the spouts. Lets a
    /// test, or the page's interpolation, follow one unit across ticks.
    pub id: u32,
    pub p: V2,
    /// Where it was last tick. `p - q` is its velocity.
    pub q: V2,
    pub r: Fx,
    /// Ticks since it left the spout, up to the swell time.
    pub age: u16,
    pub flavor: Flavor,
    /// Which line poured it. A unit never meets the other line's slush.
    pub line: u8,
}

impl Unit {
    /// The swell (D6 rule 3): the radius grows linearly from birth to full
    /// size over `swell_ticks`, then holds.
    pub fn grow(&mut self, t: &Tuning) {
        if self.age < t.swell_ticks {
            self.age += 1;
            let span = balance::R_FULL - balance::R_BIRTH;
            self.r = balance::R_BIRTH + span.scale(self.age as i64, t.swell_ticks.max(1) as i64);
        }
    }
}

/// Every pair of units of one line closer than their radii plus the margin,
/// each pair once, in grid order.
pub fn pairs(units: &[Unit], margin: Fx) -> Vec<(u32, u32)> {
    let n = units.len();
    if n < 2 {
        return Vec::new();
    }
    let cell = (balance::R_FULL * 2 + margin).raw() as i64;
    let (mut minx, mut miny, mut maxx, mut maxy) = (i64::MAX, i64::MAX, i64::MIN, i64::MIN);
    for u in units {
        let (x, y) = (u.p.x.raw() as i64, u.p.y.raw() as i64);
        minx = minx.min(x);
        miny = miny.min(y);
        maxx = maxx.max(x);
        maxy = maxy.max(y);
    }
    let w = ((maxx - minx) / cell + 1) as usize;
    let h = ((maxy - miny) / cell + 1) as usize;
    let cell_of = |u: &Unit| -> (usize, usize) {
        (((u.p.x.raw() as i64 - minx) / cell) as usize, ((u.p.y.raw() as i64 - miny) / cell) as usize)
    };
    // Counting sort by cell: `start[c]..start[c+1]` in `order` are the units
    // of cell c, in index order, because the sort is stable.
    let mut start = vec![0u32; w * h + 1];
    let cells: Vec<usize> = units.iter().map(|u| {
        let (cx, cy) = cell_of(u);
        cy * w + cx
    }).collect();
    for &c in &cells {
        start[c + 1] += 1;
    }
    for c in 0..w * h {
        start[c + 1] += start[c];
    }
    let mut fill = start.clone();
    let mut order = vec![0u32; n];
    for (i, &c) in cells.iter().enumerate() {
        order[fill[c] as usize] = i as u32;
        fill[c] += 1;
    }
    let mut out = Vec::new();
    let touching = |a: &Unit, b: &Unit| {
        let reach = (a.r + b.r + margin).raw() as i64;
        a.line == b.line && (b.p - a.p).len_sq_raw() < reach * reach
    };
    // Half the neighborhood, so each pair is seen once: this cell (later
    // units only), and the cells right, down-left, down and down-right.
    const NEAR: [(i64, i64); 4] = [(1, 0), (-1, 1), (0, 1), (1, 1)];
    for cy in 0..h {
        for cx in 0..w {
            let c = cy * w + cx;
            let here = &order[start[c] as usize..start[c + 1] as usize];
            for (k, &i) in here.iter().enumerate() {
                for &j in &here[k + 1..] {
                    if touching(&units[i as usize], &units[j as usize]) {
                        out.push((i, j));
                    }
                }
                for (dx, dy) in NEAR {
                    let (nx, ny) = (cx as i64 + dx, cy as i64 + dy);
                    if nx < 0 || ny < 0 || nx >= w as i64 || ny >= h as i64 {
                        continue;
                    }
                    let nc = ny as usize * w + nx as usize;
                    for &j in &order[start[nc] as usize..start[nc + 1] as usize] {
                        if touching(&units[i as usize], &units[j as usize]) {
                            out.push((i.min(j), i.max(j)));
                        }
                    }
                }
            }
        }
    }
    out
}

/// The thickness rule (D6 rule 2): each unit's velocity moves `thick_k` of
/// the way toward the mean velocity of the units it touches. Every mean is
/// taken from the velocities before any of them changes.
pub fn thicken(units: &mut [Unit], pairs: &[(u32, u32)], k: Fx) {
    if k.raw() == 0 {
        return;
    }
    let mut sum = vec![(0i64, 0i64, 0i64); units.len()];
    for &(i, j) in pairs {
        let (a, b) = (&units[i as usize], &units[j as usize]);
        let near = (a.r + b.r).raw() as i64;
        if (b.p - a.p).len_sq_raw() > near * near {
            continue;
        }
        let (va, vb) = (a.p - a.q, b.p - b.q);
        sum[i as usize].0 += vb.x.raw() as i64;
        sum[i as usize].1 += vb.y.raw() as i64;
        sum[i as usize].2 += 1;
        sum[j as usize].0 += va.x.raw() as i64;
        sum[j as usize].1 += va.y.raw() as i64;
        sum[j as usize].2 += 1;
    }
    for (u, &(sx, sy, n)) in units.iter_mut().zip(&sum) {
        if n == 0 {
            continue;
        }
        let mean = V2::new(Fx(crate::fx::narrow(sx / n)), Fx(crate::fx::narrow(sy / n)));
        let v = u.p - u.q;
        u.p = u.q + v + (mean - v) * k;
    }
}

/// One contact: push the pair apart along the line between them, then the
/// slope rule (D6 rule 1): the sideways part of how far they slid against
/// each other this tick is cancelled outright while it is under
/// `static_k × overlap`, and reduced by `kinetic_k × overlap` past it.
/// Both shifts are split exactly between the two, so neither rule moves the
/// pair's middle.
pub fn contact(a: &mut Unit, b: &mut Unit, t: &Tuning) {
    let d = b.p - a.p;
    let reach = a.r + b.r;
    let dsq = d.len_sq_raw();
    let rr = reach.raw() as i64;
    if dsq >= rr * rr {
        return;
    }
    if dsq == 0 {
        // Exactly on top of each other: part them along x, the lower index
        // to the left, so the choice does not depend on anything else.
        let half = Fx(reach.raw() / 2);
        a.p.x -= half;
        b.p.x += reach - half;
        return;
    }
    let dist = d.len();
    let overlap = reach - dist;
    let push = d.scale(overlap.raw() as i64, dist.raw().max(1) as i64);
    let half = V2::new(Fx(push.x.raw() / 2), Fx(push.y.raw() / 2));
    a.p -= half;
    b.p += push - half;
    // Slip: how far a moved against b this tick, less its part along d.
    let rel = (a.p - a.q) - (b.p - b.q);
    let along = rel.dot_raw(d);
    let slip = rel - d.scale(along, dsq);
    let Some(cancel) = friction(slip, overlap, t) else { return };
    let h = V2::new(Fx(cancel.x.raw() / 2), Fx(cancel.y.raw() / 2));
    a.p -= h;
    b.p += cancel - h;
}

/// The slope rule against something that is not a unit: a wall, a floor or a
/// rim moving by `surface` this tick. `normal_overlap` is how far the unit
/// was pushed out; the unit alone takes the correction.
pub fn rub(u: &mut Unit, surface: V2, normal: V2, normal_overlap: Fx, t: &Tuning) {
    let rel = (u.p - u.q) - surface;
    let nsq = normal.len_sq_raw();
    if nsq == 0 || normal_overlap.raw() <= 0 {
        return;
    }
    let slip = rel - normal.scale(rel.dot_raw(normal), nsq);
    if let Some(cancel) = friction(slip, normal_overlap, t) {
        u.p -= cancel;
    }
}

/// How much of a slip the slope rule takes back: all of it under
/// `static_k × overlap`, else `kinetic_k × overlap` of it. `None` for no slip.
/// The static test compares squares, so the common case takes no root.
fn friction(slip: V2, overlap: Fx, t: &Tuning) -> Option<V2> {
    let sl2 = slip.len_sq_raw();
    if sl2 == 0 {
        return None;
    }
    let lim = (t.static_k * overlap).raw() as i64;
    if sl2 < lim * lim {
        return Some(slip);
    }
    let sl = slip.len();
    let k = (t.kinetic_k * overlap).min(sl);
    Some(slip.scale(k.raw() as i64, sl.raw().max(1) as i64))
}

/// A unit that touched something this tick and moves slower than
/// `balance::REST_SPEED` stops. Without this, gravity and the contacts trade
/// a little speed back and forth every tick and a full cup never comes to
/// rest (M1.0: tuning 0 had not settled after 3,000 ticks).
pub fn rest(u: &mut Unit) {
    let v = u.p - u.q;
    let s = balance::REST_SPEED.raw() as i64;
    if v.len_sq_raw() < s * s {
        u.q = u.p;
    }
}

/// A whole unit's speed held to the cap, by moving where it was rather than
/// where it is (Vagrancy's `hold_to_cap`).
pub fn hold_to_cap(u: &mut Unit, cap: Fx) {
    let v = u.p - u.q;
    let c = cap.raw() as i64;
    if v.len_sq_raw() > c * c {
        u.q = u.p - v.with_len(cap);
    }
}

