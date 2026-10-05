//! The hand-computed cases of PLANNING-BRIEF A.4 that land in `sim`, each
//! run on the real integrator and the real line, not on a formula.
//! `C4 Verify against hand-computed cases`. PLAN.md restates each with the
//! agent's own arithmetic.

use content::setup::{line, order, setup_of, standing};
use sim::fx::{Fx, V2};
use sim::score::{score, Cause};
use sim::setup::{Order, Physics, Rail};
use sim::slush::Unit;
use sim::world::{Event, Handle};
use sim::{Input, World};

/// A world with no drag, gravity 4 cm/tick², and a cap out of reach, so a
/// falling unit follows H1's sum exactly.
fn h1_world() -> World {
    let mut s = standing(1, sim::balance::DEFAULT_TUNING);
    s.physics = Physics { gravity: Fx::int(4), cap: Fx::int(100_000), drag: Fx(0) };
    World::new(s)
}

fn drop_unit(w: &mut World, at: V2, down: i32) {
    let id = w.units.len() as u32;
    w.units.push(Unit { id, p: at, q: V2::new(at.x, at.y + Fx::int(down)), r: sim::balance::R_BIRTH, age: 0, flavor: 0, line: 0 });
}

/// H1 — the order of the integrator. Downward speed 1, gravity 4, speed
/// first and then position: it falls 5, then 9, then 13, and n(2n + 3) in
/// all: 14 at n = 2, 230 at n = 10. Position first would give 6 at n = 2,
/// and the familiar n(n + 1)/2 gives 3.
#[test]
fn h1_a_unit_falls_n_times_2n_plus_3() {
    let mut w = h1_world();
    let start = V2::cm(-200, 4000);
    drop_unit(&mut w, start, 1);
    let fallen = |w: &World| (start.y - w.units[0].p.y).trunc();
    let mut got = Vec::new();
    for n in 1..=10 {
        w.step([Input::NONE; 2]);
        got.push(fallen(&w));
        assert_eq!(fallen(&w), n * (2 * n + 3), "after {n} ticks");
    }
    assert_eq!(got[1], 14);
    assert_eq!(got[9], 230);
}

/// H2 — the cup moves while the slush falls. A unit leaves 230 above the
/// rim with downward speed 1 under gravity 4, so it reaches the rim on tick
/// 10 (H1). The belt moves 3 a tick, 30 in all. A cup of inner half-width 20
/// centered under the spout at release has its upstream wall 10 past the
/// spout on tick 10, and the unit lands 10 outside the cup. Released when the
/// cup's center is 30 short of the spout, it lands in the cup.
#[test]
fn h2_the_cup_moves_while_the_slush_falls() {
    for (start_offset, lands_in) in [(0, false), (-30, true)] {
        let mut l = line(&["cola"], "regular", "steady", vec![order(&[("cola", 1)])]);
        l.belt.speed = Fx::int(3);
        l.cup.inner_half = Fx::int(20);
        let x0 = l.spouts[0].x;
        l.first_x = x0 + Fx::int(start_offset);
        let mut s = setup_of(1, sim::balance::DEFAULT_TUNING, l);
        s.physics = Physics { gravity: Fx::int(4), cap: Fx::int(100_000), drag: Fx(0) };
        let mut w = World::new(s);
        let rim = w.rim(0);
        drop_unit(&mut w, V2::new(x0, rim + Fx::int(230)), 1);
        for _ in 0..10 {
            w.step([Input::NONE; 2]);
        }
        assert_eq!(w.units[0].p.y, rim, "the unit reaches the rim on tick 10");
        let cup_x = w.lines[0].as_ref().unwrap().cups[0].x;
        assert_eq!(cup_x - x0, Fx::int(start_offset + 30), "the belt carried the cup 30");
        // Let it land.
        for _ in 0..30 {
            w.step([Input::NONE; 2]);
        }
        let ls = w.lines[0].as_ref().unwrap();
        let inside: u32 = w.in_cup_counts(0)[0].iter().sum();
        if lands_in {
            assert_eq!((inside, ls.wasted), (1, 0), "released 30 short, the unit lands in the cup");
        } else {
            assert_eq!((inside, ls.wasted), (0, 1), "released over the cup, the unit lands outside it");
        }
    }
}

/// H3 — the score, at a capacity of 120 with cola and lemon at 1 to 1:
/// 100, 50, 50, 91, 50. The fourth is ⌊100 × 110 / 120⌋. Averaging a fill
/// score and a ratio score would give the cup of 120 cola 75.
#[test]
fn h3_the_score_follows_the_one_rule() {
    let (cola, lemon, cherry) = (content::setup::flavor("cola"), content::setup::flavor("lemon"), content::setup::flavor("cherry"));
    let o = Order { parts: vec![(cola, 1), (lemon, 1)] };
    let cup = |pairs: &[(u8, u32)]| {
        let mut c = vec![0u32; 3];
        for &(f, n) in pairs {
            c[f as usize] = n;
        }
        score(&c, &o, 120)
    };
    let rows = [
        (cup(&[(cola, 60), (lemon, 60)]), 100, 120, Cause::None),
        (cup(&[(cola, 30), (lemon, 30)]), 50, 60, Cause::Empty),
        (cup(&[(cola, 120)]), 50, 60, Cause::Over),
        (cup(&[(cola, 70), (lemon, 50)]), 91, 110, Cause::Over),
        (cup(&[(cola, 60), (cherry, 60)]), 50, 60, Cause::Wrong),
    ];
    for (i, (s, want, counted, cause)) in rows.iter().enumerate() {
        assert_eq!((s.score, s.counted, s.cause), (*want, *counted, *cause), "row {}", i + 1);
    }
}

#[test]
fn slush_past_its_share_scores_nothing() {
    // how.score.body: "Slush past that share … adds nothing." Ten more cola
    // past its share does not move the score by a point.
    let cola = content::setup::flavor("cola");
    let o = Order { parts: vec![(cola, 1)] };
    let at_share = score(&[60, 0, 0], &o, 60).score;
    for extra in 1..40 {
        assert_eq!(score(&[60 + extra, 0, 0], &o, 60).score, at_share);
    }
}

/// H5 — every unit is in one place. A fully open spout emits 2 a tick, 20 in
/// 10 ticks; at every later tick loose + in cups + judged + wasted equals
/// what was emitted, including the tick a cup crosses the lid and the ticks
/// units spill off a rim. The brief's sentence says "= 20"; with a lever the
/// handle takes some ticks to close after the key is let go, so the total is
/// 20 plus the tail (SECOND-ORDER-M1 row 6). The identity is checked against
/// the counted total.
#[test]
fn h5_every_unit_is_in_one_place() {
    let l = line(&["cola"], "regular", "steady", vec![order(&[("cola", 1)]), order(&[("cola", 1)])]);
    let mut w = World::new(setup_of(3, sim::balance::DEFAULT_TUNING, l));
    w.lines[0].as_mut().unwrap().handles[0] = Handle::full();
    for _ in 0..10 {
        w.step([Input(Input::SPOUT[0]), Input::NONE]);
    }
    assert_eq!(w.lines[0].as_ref().unwrap().emitted, 20, "a fully open spout emits 2 a tick");
    let (mut saw_lid, mut saw_spill) = (false, false);
    for t in 0..2_400u32 {
        // A long overfilling pour while the first cup passes, so slush goes
        // over a rim.
        let hold = (140..320).contains(&t);
        w.step([Input(if hold { Input::SPOUT[0] } else { 0 }), Input::NONE]);
        let ls = w.lines[0].as_ref().unwrap();
        let in_cups: u32 = w.in_cup_counts(0).iter().flatten().sum();
        let loose = w.units.len() as u32 - in_cups;
        assert_eq!(loose + in_cups + ls.judged + ls.wasted, ls.emitted, "tick {}", w.tick);
        saw_lid |= w.events.iter().any(|e| matches!(e, Event::Judged { .. }));
        saw_spill |= w.events.iter().any(|e| matches!(e, Event::Waste { .. }));
    }
    assert!(saw_lid && saw_spill, "the run must include a judged cup ({saw_lid}) and a spill ({saw_spill})");
    assert!(w.done());
}

/// H8 — a spout on a rail. Amplitude 30, period 120, starting at the center
/// and moving downstream at 1 a tick: 0, +30, +15, 0, −30, 0 at ticks 0, 30,
/// 45, 60, 90, 120. A sine would give 21 at tick 45.
#[test]
fn h8_a_spout_on_a_rail_moves_as_a_triangle() {
    let r = Rail { amplitude: 30, period: 120 };
    for (t, want) in [(0, 0), (30, 30), (45, 15), (60, 0), (90, -30), (120, 0), (15, 15), (31, 29)] {
        assert_eq!(r.offset(t), Fx::int(want), "tick {t}");
    }
    // And on the line: the nozzle is where the rail says, tick by tick.
    let l = content::setup::with_rail(line(&["cola", "lemon"], "regular", "steady", vec![order(&[("cola", 1)])]), 1, 30, 120);
    let mut w = World::new(setup_of(1, 1, l));
    let x0 = w.line(0).spouts[1].x;
    for _ in 0..45 {
        w.step([Input::NONE; 2]);
    }
    assert_eq!(w.spout_x(0, 1) - x0, Fx::int(15));
}

#[test]
fn a_mission_s_score_is_the_floored_average_of_its_cups() {
    assert_eq!(sim::score::average(&[100, 50, 91]), 80);
    assert_eq!(sim::score::average(&[]), 0);
}
