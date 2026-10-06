//! The world, and the one door it changes through: `World::step`.
//!
//! Per tick, in this order: the handles (servo, spring, stops), the valves
//! (opening × rate into an accumulator, one unit per whole number), the belt
//! and the rails, the units (velocity first, then position: H1), the contact
//! list, the thickness rule, then `balance::PASSES` passes of contacts and
//! walls, then waste, the lid and the counts. Every unit is in one place
//! (H5): emitted = loose + in cups + judged + wasted, at every tick.

use crate::balance::{self, Tuning};
use crate::fx::{cos_deg, sin_deg, Fx, ONE, V2};
use crate::input::Input;
use crate::rng::Rng;
use crate::score::{self, CupScore};
use crate::setup::{Line, Setup};
use crate::slush::{self, Unit};
use serde::{Deserialize, Serialize};

/// A spout's handle (D7): a stick from an anchored pivot. Held as the vector
/// from the pivot to the tip, now and last tick, so a spout that moves on a
/// rail carries its handle without pushing it.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Handle {
    pub r: V2,
    pub r_prev: V2,
}

impl Handle {
    pub fn closed() -> Handle {
        let r = V2::new(Fx(0), balance::HANDLE_LEN);
        Handle { r, r_prev: r }
    }
    /// The handle at full travel, still.
    pub fn full() -> Handle {
        let r = full_travel();
        Handle { r, r_prev: r }
    }
    /// The valve's opening, `0..=ONE`: the handle's travel as a share of full
    /// travel, read from how far its tip has swung downstream (D7). No
    /// trigonometry at run time.
    pub fn opening(&self) -> Fx {
        let full = full_travel().x;
        Fx::ratio(self.r.x.raw().max(0) as i64, full.raw() as i64).clamp(Fx(0), ONE)
    }
}

/// Where the tip sits at full travel, relative to the pivot.
pub fn full_travel() -> V2 {
    let a = balance::HANDLE_TRAVEL_DEG;
    V2::new(balance::HANDLE_LEN * sin_deg(a), balance::HANDLE_LEN * cos_deg(a))
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Valve {
    /// Opening × rate, summed; a unit leaves for each whole number.
    pub acc: Fx,
    /// How many units this spout has poured: picks the flavor from the
    /// spout's cycle (D10) and the place across the nozzle.
    pub poured: u32,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Cup {
    /// The center of the cup, and where it was last tick.
    pub x: Fx,
    pub prev_x: Fx,
    /// Index into the line's orders.
    pub order: u16,
    /// Scored and closed by the lid.
    pub judged: bool,
}

/// What the lid found in one cup.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct CupResult {
    pub cup: u16,
    /// Units of each flavor, by flavor number.
    pub counts: Vec<u32>,
    pub score: CupScore,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct LineState {
    pub handles: Vec<Handle>,
    pub valves: Vec<Valve>,
    pub cups: Vec<Cup>,
    pub results: Vec<CupResult>,
    /// How far the belt has moved, for drawing its marks.
    pub travel: Fx,
    /// The throttle: the share of the mission's belt speed the belt runs at.
    pub belt_factor: Fx,
    pub emitted: u32,
    pub judged: u32,
    pub wasted: u32,
}

/// What happened this tick, for the page to draw and later to sound.
#[derive(Copy, Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum Event {
    /// A unit landed somewhere other than a cup.
    Waste { line: u8, x: Fx, y: Fx },
    /// The lid closed on a cup.
    Judged { line: u8, cup: u16, score: u32 },
}

/// The whole state of a run. `checksum()` hashes all of it.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct World {
    pub setup: Setup,
    pub tick: u32,
    pub rng: Rng,
    pub units: Vec<Unit>,
    pub lines: [Option<LineState>; 2],
    pub events: Vec<Event>,
}

/// Which side of each wall of a cup a unit is on this tick, decided once
/// from its path (D9). Transient: rebuilt every tick.
#[derive(Copy, Clone, Debug, Default)]
struct Sides {
    cup: Option<u16>,
    left: i32,
    right: i32,
}

impl World {
    pub fn new(setup: Setup) -> World {
        let lines = [0, 1].map(|i| setup.lines[i].as_ref().map(|l| LineState {
            handles: l.spouts.iter().map(|_| Handle::closed()).collect(),
            valves: l.spouts.iter().map(|_| Valve { acc: Fx(0), poured: 0 }).collect(),
            cups: (0..l.orders.len())
                .map(|k| {
                    let x = l.first_x - l.spacing * k as i32;
                    Cup { x, prev_x: x, order: k as u16, judged: false }
                })
                .collect(),
            results: Vec::new(),
            travel: Fx(0),
            belt_factor: ONE,
            emitted: 0,
            judged: 0,
            wasted: 0,
        }));
        World { rng: Rng::new(setup.seed), setup, tick: 0, units: Vec::new(), lines, events: Vec::new() }
    }

    pub fn tuning(&self) -> Tuning {
        balance::tuning(self.setup.tuning)
    }

    /// The only way the world changes.
    pub fn step(&mut self, inputs: [Input; 2]) {
        self.events.clear();
        let t = self.tuning();
        for seat in 0..2 {
            if self.lines[seat].is_some() {
                self.handles(seat, inputs[seat], &t);
                self.belt(seat, inputs[seat]);
                self.valves(seat);
            }
        }
        self.integrate(&t);
        let sides = self.sides();
        let pairs = slush::pairs(&self.units, balance::CONTACT_MARGIN);
        slush::thicken(&mut self.units, &pairs, t.thick_k);
        let before: Vec<V2> = self.units.iter().map(|u| u.p).collect();
        let mut touched = vec![false; self.units.len()];
        for _ in 0..balance::PASSES {
            for &(i, j) in &pairs {
                let (i, j) = (i as usize, j as usize);
                let (lo, hi) = self.units.split_at_mut(j);
                slush::contact(&mut lo[i], &mut hi[0], &t);
            }
            self.walls(&sides, &t, &mut touched);
        }
        for (u, &b) in self.units.iter_mut().zip(&before) {
            slush::inelastic(u, b);
        }
        let cap = self.setup.physics.cap;
        for &(i, j) in &pairs {
            let (a, b) = (&self.units[i as usize], &self.units[j as usize]);
            let reach = (a.r + b.r + balance::CONTACT_MARGIN / 4).raw() as i64;
            if (b.p - a.p).len_sq_raw() <= reach * reach {
                touched[i as usize] = true;
                touched[j as usize] = true;
            }
        }
        for (u, &t) in self.units.iter_mut().zip(&touched) {
            slush::hold_to_cap(u, cap);
            if t {
                slush::rest(u);
            }
        }
        self.waste();
        self.lid();
        self.tick += 1;
    }

    /// 64-bit FNV-1a over the `postcard` encoding of the whole world, so a
    /// field is covered the day it is added (Floodline, by way of Vagrancy).
    pub fn checksum(&self) -> u64 {
        fnv1a(&postcard::to_allocvec(self).expect("a world always encodes"))
    }

    pub fn line(&self, i: usize) -> &Line {
        self.setup.lines[i].as_ref().expect("a line that exists")
    }

    // --- the handles (D7) -------------------------------------------------

    /// Servo while the key is held, spring always, then the stick and the
    /// two stops. Only `Input::pulls` reads the input here, so no other bit
    /// can move a handle (`only_spout_bits_move_a_handle`).
    fn handles(&mut self, seat: usize, input: Input, t: &Tuning) {
        let full = full_travel();
        let len = balance::HANDLE_LEN;
        let ls = self.lines[seat].as_mut().unwrap();
        for (i, h) in ls.handles.iter_mut().enumerate() {
            let len_sq = h.r.len_sq_raw().max(1);
            // Angular speed, rad/tick, counterclockwise positive.
            let omega = Fx(crate::fx::narrow(h.r_prev.cross_raw(h.r) * ONE.raw() as i64 / len_sq));
            // The spring pulls toward closed (counterclockwise) in proportion
            // to the opening.
            let mut alpha = t.spring * h.opening();
            if input.pulls(i) {
                // Clockwise is open. The servo pushes as hard as
                // HANDLE_ACCEL allows until the handle turns at HANDLE_SPEED.
                let room = balance::HANDLE_SPEED + omega;
                if room.raw() > 0 {
                    alpha -= room.min(balance::HANDLE_ACCEL);
                }
            }
            let mut v = h.r - h.r_prev;
            v -= v * balance::HANDLE_DRAG;
            v += h.r.perp() * alpha;
            h.r_prev = h.r;
            h.r = (h.r + v).with_len(len);
            // The stops: closed points straight up, full travel is `full`.
            if h.r.x.raw() < 0 || h.r.y.raw() < 0 && h.r.x.raw() <= 0 {
                h.r = V2::new(Fx(0), len);
                h.r_prev = h.r;
            } else if h.r.x > full.x || h.r.y < full.y {
                h.r = full;
                h.r_prev = h.r;
            }
        }
    }

    // --- the belt and the rails -------------------------------------------

    /// The belt keys move the throttle; then the belt carries the cups at
    /// the mission's speed times the throttle. Only the belt bits touch it.
    fn belt(&mut self, seat: usize, input: Input) {
        let ls = self.lines[seat].as_mut().unwrap();
        if input.has(Input::BELT_FASTER) && !input.has(Input::BELT_SLOWER) {
            ls.belt_factor = (ls.belt_factor + balance::BELT_STEP).min(balance::BELT_MAX);
        } else if input.has(Input::BELT_SLOWER) && !input.has(Input::BELT_FASTER) {
            ls.belt_factor = (ls.belt_factor - balance::BELT_STEP).max(balance::BELT_MIN);
        }
        let speed = self.belt_speed(seat);
        let ls = self.lines[seat].as_mut().unwrap();
        ls.travel += speed;
        for c in &mut ls.cups {
            c.prev_x = c.x;
            c.x += speed;
        }
    }

    /// How fast the belt runs this tick, cm per tick: the mission's speed
    /// (and its schedule) times the throttle.
    pub fn belt_speed(&self, seat: usize) -> Fx {
        let base = self.line(seat).belt.speed_at(self.tick);
        base * self.lines[seat].as_ref().unwrap().belt_factor
    }

    /// Where spout `i`'s nozzle is this tick: its place, moved by its rail.
    pub fn spout_x(&self, seat: usize, i: usize) -> Fx {
        let s = &self.line(seat).spouts[i];
        s.x + s.rail.map(|r| r.offset(self.tick)).unwrap_or(Fx(0))
    }

    // --- the valves -------------------------------------------------------

    fn valves(&mut self, seat: usize) {
        let n = self.line(seat).spouts.len();
        for i in 0..n {
            let x = self.spout_x(seat, i);
            let line = self.setup.lines[seat].as_ref().unwrap();
            let spout = &line.spouts[i];
            let (y, pours) = (spout.y, spout.pours.clone());
            let ls = self.lines[seat].as_mut().unwrap();
            let opening = ls.handles[i].opening();
            let valve = &mut ls.valves[i];
            valve.acc += opening * balance::VALVE_RATE;
            while valve.acc >= ONE {
                valve.acc -= ONE;
                let k = valve.poured as usize;
                valve.poured += 1;
                let off = Fx::int(balance::NOZZLE_OFFSETS[k % balance::NOZZLE_OFFSETS.len()]);
                let p = V2::new(x + off, y);
                // Moving straight down at the spout's speed: `q` is above `p`.
                let q = V2::new(p.x, p.y + balance::SPOUT_SPEED);
                let flavor = pours[k % pours.len()];
                self.units.push(Unit { id: ls.emitted, p, q, r: balance::R_BIRTH, age: 0, flavor, line: seat as u8 });
                ls.emitted += 1;
            }
        }
    }

    // --- the units ----------------------------------------------------------

    /// Velocity first, then position: H1's order. Then the swell.
    fn integrate(&mut self, t: &Tuning) {
        let ph = self.setup.physics;
        let cap = ph.cap.raw() as i64;
        let tick = self.tick;
        let lines = &self.setup.lines;
        for u in &mut self.units {
            let mut v = u.p - u.q;
            v -= v * ph.drag;
            v.y -= ph.gravity;
            // The line's fields push its slush, and only its slush.
            if let Some(l) = lines[u.line as usize].as_ref() {
                if !l.fields.is_empty() {
                    v += l.field_accel(u.p, tick);
                }
            }
            if v.len_sq_raw() > cap * cap {
                v = v.with_len(ph.cap);
            }
            u.q = u.p;
            u.p += v;
            u.grow(t);
        }
    }

    // --- the cups' walls (D9) ---------------------------------------------

    /// The cup a unit is near, and which side of each of its two walls the
    /// unit is on. A wall is a vertical capsule from the belt to the rim; in
    /// the wall's own frame a unit whose path crossed the wall's line at or
    /// below the rim stays on the side it started, and one that crossed above
    /// the rim went over. This is the swept contact, exact because the walls
    /// are axis-aligned: no speed passes a unit through a wall.
    fn sides(&self) -> Vec<Sides> {
        let mut out = vec![Sides::default(); self.units.len()];
        for (k, u) in self.units.iter().enumerate() {
            let seat = u.line as usize;
            let line = self.line(seat);
            let ls = self.lines[seat].as_ref().unwrap();
            let reach = line.cup.inner_half + balance::WALL_HALF * 2 + balance::R_FULL * 3;
            let Some(ci) = ls.cups.iter().position(|c| (u.p.x - c.x).abs() <= reach || (u.q.x - c.prev_x).abs() <= reach) else {
                continue;
            };
            let c = &ls.cups[ci];
            let rim = self.rim(seat);
            let side_of = |wall_now: Fx, wall_then: Fx| -> i32 {
                let a = u.q.x - wall_then;
                let b = u.p.x - wall_now;
                let s0 = if a.raw() != 0 { a.signum() } else if b.raw() != 0 { b.signum() } else { 1 };
                if b.signum() == s0 || b.raw() == 0 {
                    return s0;
                }
                // Crossed the line between ticks: where?
                let f = Fx::ratio(a.raw() as i64, (a - b).raw() as i64);
                let y = u.q.y + (u.p.y - u.q.y) * f;
                if y <= rim + balance::WALL_HALF { s0 } else { -s0 }
            };
            let off = line.cup.inner_half + balance::WALL_HALF;
            out[k] = Sides {
                cup: Some(ci as u16),
                left: side_of(c.x - off, c.prev_x - off),
                right: side_of(c.x + off, c.prev_x + off),
            };
        }
        out
    }

    /// The top of a cup's walls, and the top of its floor.
    pub fn rim(&self, seat: usize) -> Fx {
        let l = self.line(seat);
        self.floor(seat) + l.cup.inner_height
    }
    pub fn floor(&self, seat: usize) -> Fx {
        self.line(seat).belt_y + balance::WALL_HALF * 2
    }

    fn walls(&mut self, sides: &[Sides], t: &Tuning, touched: &mut [bool]) {
        let w = balance::WALL_HALF;
        for k in 0..self.units.len() {
            let s = sides[k];
            let Some(ci) = s.cup else { continue };
            let seat = self.units[k].line as usize;
            let (rim, floor) = (self.rim(seat), self.floor(seat));
            let half = self.line(seat).cup.inner_half;
            let c = self.lines[seat].as_ref().unwrap().cups[ci as usize].clone();
            let moved = V2::new(c.x - c.prev_x, Fx(0));
            let u = &mut self.units[k];
            for (wx, side) in [(c.x - half - w, s.left), (c.x + half + w, s.right)] {
                let reach = u.r + w;
                if u.p.y <= rim {
                    let gap = (u.p.x - wx) * side;
                    if gap < reach {
                        let push = reach - gap;
                        u.p.x = wx + reach * side;
                        // The walls are slick: no slope rule against them, so
                        // swelling slush rises up a cup's sides rather than
                        // sticking to them (M1.0: with the slope rule here the
                        // swell was hidden; SECOND-ORDER-M1).
                        let _ = push;
                        touched[k] = true;
                    }
                } else {
                    // The rounded rim: a circle at the wall's top.
                    let d = u.p - V2::new(wx, rim);
                    let rr = reach.raw() as i64;
                    if d.len_sq_raw() < rr * rr {
                        let dist = d.len();
                        let n = if dist.raw() == 0 { V2::new(Fx(0), ONE) } else { d };
                        let push = reach - dist;
                        u.p = V2::new(wx, rim) + n.with_len(reach);
                        let _ = push;
                        touched[k] = true;
                    }
                }
            }
            // The floor, between the walls. Nothing reaches the underside: the
            // belt is there.
            let (xl, xr) = (c.x - half - w * 2, c.x + half + w * 2);
            if u.p.x > xl && u.p.x < xr {
                let top = floor + u.r;
                if u.p.y < top {
                    let push = top - u.p.y;
                    u.p.y = top;
                    slush::rub(u, moved, V2::new(Fx(0), ONE), push, t);
                    touched[k] = true;
                }
                // A judged cup is closed: its lid is a floor at the rim.
                if c.judged && u.p.y > rim - u.r {
                    let top = rim + w + u.r;
                    if u.p.y < top {
                        let push = top - u.p.y;
                        u.p.y = top;
                        slush::rub(u, moved, V2::new(Fx(0), ONE), push, t);
                    }
                }
            }
        }
    }

    /// Is the unit inside cup `c`'s walls, below the rim?
    pub fn inside(&self, u: &Unit, c: &Cup) -> bool {
        let seat = u.line as usize;
        let half = self.line(seat).cup.inner_half;
        (u.p.x - c.x).abs() < half && u.p.y < self.rim(seat) && u.p.y > self.floor(seat)
    }

    // --- waste and the lid ------------------------------------------------

    /// A unit on the belt outside a cup, on the floor, or past either end of
    /// the line is waste, and leaves the world.
    fn waste(&mut self) {
        let mut keep = Vec::with_capacity(self.units.len());
        let units = std::mem::take(&mut self.units);
        for u in units {
            let seat = u.line as usize;
            let line = self.line(seat);
            let ls = self.lines[seat].as_ref().unwrap();
            let in_cup_span = ls.cups.iter().any(|c| (u.p.x - c.x).abs() < line.cup.inner_half + balance::WALL_HALF * 2);
            let on_belt = u.p.y - u.r <= line.belt_y && !in_cup_span;
            let gone = u.p.y - u.r <= Fx(0) || u.p.x > line.end_x || u.p.x < -line.end_x;
            if on_belt || gone {
                self.events.push(Event::Waste { line: u.line, x: u.p.x, y: u.p.y });
                self.lines[seat].as_mut().unwrap().wasted += 1;
            } else {
                keep.push(u);
            }
        }
        self.units = keep;
    }

    /// A cup whose center reaches the lid is counted by flavor, scored, and
    /// its units leave the world. Units heaped above the rim are scraped off
    /// as waste. A cup is judged once, on the tick its center reaches the lid.
    fn lid(&mut self) {
        for seat in 0..2 {
            let Some(line) = self.setup.lines[seat].clone() else { continue };
            let n = self.lines[seat].as_ref().unwrap().cups.len();
            for ci in 0..n {
                let c = self.lines[seat].as_ref().unwrap().cups[ci].clone();
                if c.judged || c.x < line.lid_x {
                    continue;
                }
                let mut counts = vec![0u32; self.setup.flavors as usize];
                let (mut judged, mut scraped) = (0u32, 0u32);
                let units = std::mem::take(&mut self.units);
                let mut keep = Vec::with_capacity(units.len());
                let span = line.cup.inner_half + balance::WALL_HALF * 2;
                for u in units {
                    if u.line as usize == seat && self.inside(&u, &c) {
                        counts[u.flavor as usize] += 1;
                        judged += 1;
                    } else if u.line as usize == seat && (u.p.x - c.x).abs() < span && u.p.y >= self.rim(seat) {
                        scraped += 1;
                        self.events.push(Event::Waste { line: seat as u8, x: u.p.x, y: u.p.y });
                    } else {
                        keep.push(u);
                    }
                }
                self.units = keep;
                let order = &line.orders[c.order as usize];
                let sc = score::score(&counts, order, line.cup.capacity);
                self.events.push(Event::Judged { line: seat as u8, cup: c.order, score: sc.score });
                let ls = self.lines[seat].as_mut().unwrap();
                ls.cups[ci].judged = true;
                ls.judged += judged;
                ls.wasted += scraped;
                ls.results.push(CupResult { cup: c.order, counts, score: sc });
            }
        }
    }

    // --- reading the world --------------------------------------------------

    /// Units of a line inside each of its cups, by flavor.
    pub fn in_cup_counts(&self, seat: usize) -> Vec<Vec<u32>> {
        let ls = self.lines[seat].as_ref().unwrap();
        let mut out = vec![vec![0u32; self.setup.flavors as usize]; ls.cups.len()];
        for u in self.units.iter().filter(|u| u.line as usize == seat) {
            if let Some(ci) = ls.cups.iter().position(|c| !c.judged && self.inside(u, c)) {
                out[ci][u.flavor as usize] += 1;
            }
        }
        out
    }

    /// Every cup on the line has been judged.
    pub fn done(&self) -> bool {
        self.lines.iter().flatten().all(|l| l.cups.iter().all(|c| c.judged))
    }
}

/// 64-bit FNV-1a (Floodline `world.rs:2180`, by way of Vagrancy).
pub fn fnv1a(bytes: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for &b in bytes {
        h ^= b as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fnv1a_matches_the_published_vectors() {
        assert_eq!(fnv1a(b""), 0xcbf2_9ce4_8422_2325);
        assert_eq!(fnv1a(b"a"), 0xaf63_dc4c_8601_ec8c);
        assert_eq!(fnv1a(b"foobar"), 0x8594_4171_f739_67e8);
    }

    #[test]
    fn a_closed_handle_opens_nothing_and_a_full_one_opens_all_the_way() {
        assert_eq!(Handle::closed().opening(), Fx(0));
        assert_eq!(Handle::full().opening(), ONE);
    }
}
