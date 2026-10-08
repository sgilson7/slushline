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
pub const VERSION: u32 = 3;

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
    cup: Option<u16>,
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
            // Aim inside the narrower of the middle and the mouth, so a jar's
            // shoulders are not poured on (cup shapes, 2026-10-08).
            let aim = line.cup.inner_half.min(line.cup.mouth());
            let room = (aim - Fx::int(8)).max(aim / 2);
            let target = ls.cups.iter().find(|c| !c.judged && (c.x + lead - x).abs() <= room);
            let Some(cup) = target else {
                if self.pours[i].cup.is_some() && !self.pours[i].done {
                    self.let_go(i, poured);
                }
                continue;
            };
            if self.pours[i].cup != Some(cup.order) {
                let slop = match self.kind {
                    Kind::Yardstick { error, .. } => self.rng.range(-(error as i32), error as i32),
                    _ => 0,
                };
                self.pours[i] = Pour { cup: Some(cup.order), start: poured, done: false, released_at: None, slop };
            }
            if self.pours[i].done {
                continue;
            }
            // Aim a little short: slush heaps under the stream and spills
            // over one wall before the cup is level (the slope rule), so a
            // full share poured at one point wastes some of it.
            let plan = plan(line, &line.orders[cup.order as usize], i) as i32 * AIM.0 / AIM.1;
            let out = (poured - self.pours[i].start) as i32;
            if plan > 0 && out + self.tail[i] + self.pours[i].slop < plan {
                bits |= Input::SPOUT[i];
            } else if out > 0 || plan == 0 {
                self.let_go(i, poured);
            }
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
