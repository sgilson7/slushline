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
    /// The swell (D6 rule 3): the radius grows from birth to full size over
    /// `swell_ticks`, then holds. It grows with the square of the unit's age,
    /// slowly at first, so most of the swell happens after the unit has
    /// landed, where a player can see the cup rise. Linear growth spent most
    /// of itself in the air, and the solver hid the rest (SECOND-ORDER-M1
    /// row 4, M2 row 3).
    pub fn grow(&mut self, t: &Tuning) {
        // The age keeps counting past the swell, for slush that melts
        // (2026-10-08); the swell reads it only while it grows.
        self.age = self.age.saturating_add(1);
        if self.age <= t.swell_ticks {
            let span = balance::R_FULL - balance::R_BIRTH;
            let (a, n) = (self.age as i64, t.swell_ticks.max(1) as i64);
            self.r = balance::R_BIRTH + span.scale(a * a, n * n);
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
pub fn contact(a: &mut Unit, b: &mut Unit, t: &Tuning, ma: i64, mb: i64) {
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
        let part = Fx(crate::fx::narrow(reach.raw() as i64 * mb / (ma + mb)));
        a.p.x -= part;
        b.p.x += reach - part;
        return;
    }
    let dist = d.len();
    let overlap = reach - dist;
    let push = d.scale(overlap.raw() as i64, dist.raw().max(1) as i64);
    // Each moves in proportion to the other's mass: a heavy unit is moved
    // less, so it sinks through light slush (2026-10-06). Equal masses split
    // the push in half, as before.
    let share = |v: V2| V2::new(Fx(crate::fx::narrow(v.x.raw() as i64 * mb / (ma + mb))), Fx(crate::fx::narrow(v.y.raw() as i64 * mb / (ma + mb))));
    let part = share(push);
    a.p -= part;
    b.p += push - part;
    // Slip: how far a moved against b this tick, less its part along d.
    let rel = (a.p - a.q) - (b.p - b.q);
    let along = rel.dot_raw(d);
    let slip = rel - d.scale(along, dsq);
    // Heavy slush against light slush has no grip: it slides down through
    // it (2026-10-06, "heavy slush settles to the bottom";
    // `heavy_slush_sinks_through_lighter_slush`).
    if ma != mb {
        return;
    }
    let Some(cancel) = friction(slip, overlap, t) else { return };
    let h = share(cancel);
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

/// Contacts stop slush; they do not throw it. In a Verlet solver every push
/// a contact makes becomes velocity, so a fast unit driven deep into a pile
/// was pushed out faster than it came in and splashed over the cup's walls
/// (M2.0: a standing cup lost 12 of 44 units). After the passes, the speed a
/// unit has along the way it was pushed is held to what it had that way
/// before the passes, and to no more than zero if it was moving against the
/// push: the push separates, and the bounce is gone. `before` is where the
/// unit was before the passes.
pub fn inelastic(u: &mut Unit, before: V2) {
    let push = u.p - before;
    let psq = push.len_sq_raw();
    if psq == 0 {
        return;
    }
    let v_pre = before - u.q;
    let v_now = u.p - u.q;
    // Along the push, as raw dot products over |push|²: comparing
    // v·push directly keeps it to one division.
    // A small allowance is kept, so the gentle pushes of swelling slush
    // still carry it upward through a deep pile in six passes; only a
    // bounce bigger than the allowance is taken away.
    let allow = balance::BOUNCE.raw() as i64 * crate::fx::isqrt(psq as u64) as i64;
    let pre = v_pre.dot_raw(push).max(0) + allow;
    let now = v_now.dot_raw(push);
    if now > pre {
        let excess = push.scale(now - pre, psq);
        u.q += excess;
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

