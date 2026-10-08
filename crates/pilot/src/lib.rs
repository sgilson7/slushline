//! Players that are not people (D14). A pilot reads the world as a player
//! sees it and returns an `Input`, and nothing else, so it can do nothing a
//! player cannot.
//!
//! Three kinds:
//! - **idle** holds no key;
//! - **timer** plans how much each spout should pour into each cup from the
//!   cup's order, holds a spout's key while the cup will be under the falling
//!   slush and the plan is not met, and lets go early by the tail it measured
//!   on its own first pour;
//! - **yardstick** is the timer with a seeded error in when it lets go: the
//!   one fixed player every mission is measured against (`make ladder`).

use sim::fx::Fx;
use sim::rng::Rng;
use sim::setup::Flavor;
use sim::{Input, World};

/// Bumped when a pilot plays differently, so `analysis/ladder.md` is known to
/// be stale.
pub const VERSION: u32 = 4;

/// The yardstick's error on each let-go, in units either way.
pub const YARDSTICK_ERROR: u32 = 10;

#[derive(Clone, Debug)]
pub enum Kind {
    Idle,
    Timer,
    /// Seeded error, in units of slush either way, on each let-go.
    Yardstick { seed: u64, error: u32 },
}

/// A pour in progress or done: which cup, and the valve's count when it began.
#[derive(Clone, Debug, Default)]
struct Pour {
    /// The line the cup is on (the line below, on a mirror line) and its order.
    cup: Option<(usize, u16)>,
    start: u32,
    done: bool,
    /// The valve's count when the key was let go, to measure the tail.
    released_at: Option<u32>,
    /// This pour's error, in units, drawn once.
    slop: i32,
}

#[derive(Clone, Debug)]
pub struct Pilot {
    pub kind: Kind,
    pours: Vec<Pour>,
    /// The tail it expects, per spout, in units: guessed, then measured.
    tail: Vec<i32>,
    rng: Rng,
}

impl Pilot {
    pub fn new(kind: Kind) -> Pilot {
        let seed = match kind {
            Kind::Yardstick { seed, .. } => seed,
            _ => 1,
        };
        Pilot { kind, pours: Vec::new(), tail: Vec::new(), rng: Rng::new(seed) }
    }

    /// The input for `seat` this tick.
    pub fn input(&mut self, w: &World, seat: usize) -> Input {
        if matches!(self.kind, Kind::Idle) {
            return Input::NONE;
        }
        let Some(line) = w.setup.lines[seat].as_ref() else { return Input::NONE };
        let ls = w.lines[seat].as_ref().unwrap();
        let n = line.spouts.len();
        if self.pours.len() != n {
            self.pours = vec![Pour::default(); n];
            self.tail = vec![14; n];
        }
        let speed = w.belt_speed(seat);
        let mut bits = 0u16;
        for i in 0..n {
            let poured = ls.valves[i].poured;
            // Measure the tail: units out between letting go and the valve
            // shutting.
            if let Some(at) = self.pours[i].released_at {
                if ls.handles[i].opening().0 == 0 {
                    self.tail[i] = (poured - at) as i32;
                    self.pours[i].released_at = None;
                }
            }
            // Where and when slush leaving now lands, through the line's
            // fields, and the cup that will be there then.
            let (x, fall) = landing(w, seat, i);
            let lead = speed * fall as i32;
            // An upper cup this spout has nothing to pour into does not stop
            // it reaching a lower cup through the open belt (the mirror line).
            let here = aimed(w, seat, line, &ls.cups, x, lead).map(|c| (seat, c.order));
            let wants = |o: u16| plan(line, &line.orders[o as usize], i) > 0;
            let target = match here {
                Some((_, o)) if wants(o) => here,
                _ => below(w, seat, i).or(here),
            };
            let Some((cup_seat, order)) = target else {
                if self.pours[i].cup.is_some() && !self.pours[i].done {
                    self.let_go(i, poured);
                }
                continue;
            };
            if self.pours[i].cup != Some((cup_seat, order)) {
                let slop = match self.kind {
                    Kind::Yardstick { error, .. } => self.rng.range(-(error as i32), error as i32),
                    _ => 0,
                };
                self.pours[i] = Pour { cup: Some((cup_seat, order)), start: poured, done: false, released_at: None, slop };
            }
            if self.pours[i].done {
                continue;
            }
            // Aim a little short: slush heaps under the stream and spills
            // over one wall before the cup is level (the slope rule), so a
            // full share poured at one point wastes some of it.
            // A cup on the line below is filled to its own order and size
            // from this line's spouts.
            let plan = if cup_seat == seat {
                plan(line, &line.orders[order as usize], i)
            } else {
                let mut via = line.clone();
                via.cup = w.line(cup_seat).cup;
                plan(&via, &w.line(cup_seat).orders[order as usize], i)
            } as i32
                * AIM.0
                / AIM.1;
            let out = (poured - self.pours[i].start) as i32;
            if plan > 0 && out + self.tail[i] + self.pours[i].slop < plan {
                bits |= Input::SPOUT[i];
            } else if out > 0 || plan == 0 {
                self.let_go(i, poured);
            }
        }
        // Slush that melts (2026-10-08) is carried to the lid sooner on a
        // faster belt, so the pilot holds the belt's faster key, as the
        // mission's card tells a player to.
        if line.melt.is_some() {
            bits |= Input::BELT_FASTER;
        }
        Input(bits)
    }

    fn let_go(&mut self, i: usize, poured: u32) {
        self.pours[i].done = true;
        if self.pours[i].released_at.is_none() {
            self.pours[i].released_at = Some(poured);
        }
    }
}

/// The first cup on `line` that slush landing at `x` reaches, `lead` ahead:
/// inside the narrower of the cup's middle and its mouth, so a jar's
/// shoulders are not poured on (cup shapes, 2026-10-08).
fn aimed<'a>(_w: &World, _seat: usize, line: &sim::setup::Line, cups: &'a [sim::world::Cup], x: Fx, lead: Fx) -> Option<&'a sim::world::Cup> {
    let aim = line.cup.inner_half.min(line.cup.mouth());
    let room = (aim - Fx::int(8)).max(aim / 2);
    cups.iter().find(|c| !c.judged && (c.x + lead - x).abs() <= room)
}

/// On a mirror line (2026-10-08): the lower line's cup that slush from
/// spout `i` falls into through the open upper belt, if no upper cup is in
/// its way as it passes the belt.
fn below(w: &World, seat: usize, i: usize) -> Option<(usize, u16)> {
    let line = w.line(seat);
    if !line.open || seat != 0 || w.lines[1].is_none() {
        return None;
    }
    let (x, fall, cross) = landing_below(w, i)?;
    let up = w.lines[0].as_ref().unwrap();
    let speed_up = w.belt_speed(0);
    let clear = line.cup.widest() + sim::balance::WALL_HALF * 2 + sim::balance::R_FULL * 2;
    // Where the slush passes the upper belt, no upper cup may stand.
    let (cx, ct) = cross;
    if up.cups.iter().any(|c| !c.judged && (c.x + speed_up * ct as i32 - cx).abs() < clear) {
        return None;
    }
    let low = w.line(1);
    let lead = w.belt_speed(1) * fall as i32;
    aimed(w, 1, low, &w.lines[1].as_ref().unwrap().cups, x, lead).map(|c| (1, c.order))
}

/// Where slush from spout `i` of the upper line lands on the line below,
/// how many ticks it takes, and where and when it passes the upper belt.
fn landing_below(w: &World, i: usize) -> Option<(Fx, u32, (Fx, u32))> {
    let ph = w.setup.physics;
    let line = w.line(0);
    let target = w.floor(1) + Fx::int(12);
    let mut p = sim::fx::V2::new(w.spout_x(0, i), line.spouts[i].y);
    let mut v = sim::fx::V2::new(Fx(0), -sim::balance::SPOUT_SPEED);
    let cap = ph.cap.raw() as i64;
    let mut t = 0u32;
    let mut cross = None;
    while p.y > target && t < 400 {
        v -= v * ph.drag;
        v.y -= ph.gravity;
        let fields = if p.y > line.belt_y { line.field_accel(p, w.tick + t) } else { w.line(1).field_accel(p, w.tick + t) };
        v += fields;
        if v.len_sq_raw() > cap * cap {
            v = v.with_len(ph.cap);
        }
        p += v;
        t += 1;
        if cross.is_none() && p.y <= line.belt_y {
            cross = Some((p.x, t));
        }
    }
    Some((p.x, t, cross?))
}

/// The share of the plan the timer pours, as a fraction.
pub const AIM: (i32, i32) = (11, 12);

/// Where slush leaving spout `i` now lands, a little above its cup's floor,
/// and how many ticks it takes: one particle stepped by the rule the world
/// steps (speed first, drag, gravity, the line's fields, the cap). Reading
/// the setup and stepping a copy is what a player does by eye.
fn landing(w: &World, seat: usize, i: usize) -> (Fx, u32) {
    let ph = w.setup.physics;
    let line = w.line(seat);
    let target = w.floor(seat) + Fx::int(12);
    let mut p = sim::fx::V2::new(w.spout_x(seat, i), line.spouts[i].y);
    let mut v = sim::fx::V2::new(Fx(0), -sim::balance::SPOUT_SPEED);
    let cap = ph.cap.raw() as i64;
    let mut t = 0u32;
    while p.y > target && t < 300 {
        v -= v * ph.drag;
        v.y -= ph.gravity;
        v += line.field_accel(p, w.tick + t);
        if v.len_sq_raw() > cap * cap {
            v = v.with_len(ph.cap);
        }
        p += v;
        t += 1;
    }
    (p.x, t)
}

/// How many units spout `i` should pour into a cup with this order: a blend
/// pours enough to fill the share of a flavor only it supplies; a single
/// flavor tops up what the blends left of its share (H4; mission 9).
pub fn plan(line: &sim::setup::Line, order: &sim::setup::Order, i: usize) -> u32 {
    let cap = line.cup.capacity;
    let share = |f: Flavor| order.share(f, cap);
    let frac = |pours: &[Flavor], f: Flavor| (pours.iter().filter(|&&p| p == f).count() as u32, pours.len() as u32);
    let single = |f: Flavor| line.spouts.iter().position(|s| s.pours.len() == 1 && s.pours[0] == f);
    let blend_amount = |s: &sim::setup::Spout| -> u32 {
        let mut amt = 0;
        for &f in &s.pours {
            if single(f).is_none() {
                let (k, n) = frac(&s.pours, f);
                amt = amt.max(share(f) * n / k.max(1));
            }
        }
        amt
    };
    let s = &line.spouts[i];
    let distinct: Vec<Flavor> = {
        let mut v = s.pours.clone();
        v.sort();
        v.dedup();
        v
    };
    if distinct.len() > 1 {
        return blend_amount(s);
    }
    let f = s.pours[0];
    if single(f) != Some(i) {
        return 0;
    }
    let from_blends: u32 = line
        .spouts
        .iter()
        .filter(|b| b.pours.len() > 1 && b.pours.iter().collect::<std::collections::BTreeSet<_>>().len() > 1)
        .map(|b| {
            let (k, n) = frac(&b.pours, f);
            blend_amount(b) * k / n.max(1)
        })
        .sum();
    share(f).saturating_sub(from_blends)
}

/// One step of a world under a pilot on seat 0, for tests and the lab.
pub fn drive(w: &mut World, p: &mut Pilot) {
    let i = p.input(w, 0);
    w.step([i, Input::NONE]);
}
