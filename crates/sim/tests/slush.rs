//! Slush holds a slope, swells, comes to rest, and stays on its side of a
//! cup wall (D5, D6, D9). Each test has a number in it, and each number was
//! measured by `lab recon-m1` first (analysis/recon-m1.md).

use content::setup::{line, order, setup_of, standing};
use sim::fx::{Fx, V2};
use sim::rng::Rng;
use sim::setup::Physics;
use sim::slush::Unit;
use sim::{Input, World};

/// A wide floor with one spout over its middle.
fn wide(tuning: u8) -> World {
    let mut l = line(&["cola"], "regular", "steady", vec![order(&[("cola", 1)])]);
    l.belt.speed = Fx(0);
    l.first_x = l.spouts[0].x;
    l.cup.inner_half = Fx::int(120);
    l.lid_x = Fx::int(10_000);
    World::new(setup_of(1, tuning, l))
}

/// Pour about 60 units from one point, wait until tick 600, and measure the
/// pile: its height over the half-width of its base.
fn pile_slope(tuning: u8) -> (f64, usize) {
    let mut w = wide(tuning);
    let mut t = 0;
    while w.lines[0].as_ref().unwrap().emitted < 52 {
        w.step([Input(Input::SPOUT[0]), Input::NONE]);
        t += 1;
    }
    while t < 600 {
        w.step([Input::NONE; 2]);
        t += 1;
    }
    let floor = w.floor(0);
    let x0 = w.spout_x(0, 0);
    let height = w.units.iter().map(|u| u.p.y + u.r - floor).max().unwrap();
    let half = w.units.iter().filter(|u| u.p.y - u.r - floor < Fx::int(2)).map(|u| (u.p.x - x0).abs() + u.r).max().unwrap();
    (height.0 as f64 / half.0 as f64, w.units.len())
}

#[test]
fn a_pile_of_slush_keeps_its_slope() {
    // M1.0: tunings 0, 1 and 2 hold 0.36, 0.38 and 0.34; water (the
    // control, no slope rule and no thickness) spreads to the walls at 0.06.
    for tuning in 0..3u8 {
        let (slope, n) = pile_slope(tuning);
        assert!(n >= 52, "the pour made {n} units");
        assert!(slope >= 0.25, "tuning {tuning}: the pile's slope is {slope:.2}, flatter than 0.25");
    }
    let (water, _) = pile_slope(sim::balance::CONTROL);
    assert!(water < 0.12, "the control should level like water, and holds {water:.2}");
}

/// The level of a cup's slush: the mean top of the five highest units
/// inside its walls (a unit resting on a rim is not in the cup).
fn level(w: &World) -> Fx {
    let c = &w.lines[0].as_ref().unwrap().cups[0];
    let mut tops: Vec<Fx> = w.units.iter().filter(|u| w.inside(u, c)).map(|u| u.p.y + u.r).collect();
    tops.sort();
    tops.iter().rev().take(5).fold(Fx(0), |a, &b| a + b) / 5
}

#[test]
fn a_cup_keeps_rising_after_the_last_unit_lands() {
    // The swell (D6 rule 3): units leave small and grow for `swell_ticks`.
    // Pour, let go, and wait for the last unit to land (nothing moving faster
    // than half a cm a tick); the level then still rises by at least a cm.
    let mut w = World::new(standing(1, sim::balance::DEFAULT_TUNING));
    for _ in 0..30 {
        w.step([Input(Input::SPOUT[0]), Input::NONE]);
    }
    let fast = |w: &World| w.units.iter().any(|u| (u.p - u.q).len() > Fx::ratio(1, 2));
    let mut t = 0;
    while (fast(&w) || w.lines[0].as_ref().unwrap().handles[0].opening().0 > 0) && t < 400 {
        w.step([Input::NONE; 2]);
        t += 1;
    }
    assert!(t < 400, "the pour never landed");
    let at_landing = level(&w);
    let young = w.units.iter().filter(|u| u.age < w.tuning().swell_ticks).count();
    assert!(young > 0, "every unit had finished swelling before the last one landed");
    for _ in 0..60 {
        w.step([Input::NONE; 2]);
    }
    let later = level(&w);
    assert!(
        later - at_landing >= Fx::int(1),
        "the level went from {:.2} to {:.2} cm after the last unit landed",
        at_landing.0 as f64 / 4096.0,
        later.0 as f64 / 4096.0
    );
}

#[test]
fn a_filled_cup_comes_to_rest() {
    // The brief's guess: one tuning comes to rest within two seconds. The
    // default must; the others are printed for the recon table.
    let mut settle = Vec::new();
    for tuning in 0..3u8 {
        let mut w = World::new(standing(1, tuning));
        // Filled, not overfilled: let go when what is out plus the tail
        // (15 units at the default spring, analysis/recon-m2.md) is the
        // cup's capacity.
        let cap = w.line(0).cup.capacity;
        while w.lines[0].as_ref().unwrap().emitted + 15 < cap {
            w.step([Input(Input::SPOUT[0]), Input::NONE]);
        }
        for _ in 0..60 {
            w.step([Input::NONE; 2]);
        }
        let mut k = 0;
        while w.units.iter().any(|u| (u.p - u.q).len() > Fx::ratio(1, 20)) && k < 600 {
            w.step([Input::NONE; 2]);
            k += 1;
        }
        println!("tuning {tuning}: at rest {k} ticks after the pour landed");
        settle.push(k);
    }
    let d = settle[sim::balance::DEFAULT_TUNING as usize];
    assert!(d < 2 * sim::balance::TICKS_PER_SECOND, "the default tuning is still moving {d} ticks after the pour landed");
}

/// Which cup a unit is inside, if any, with a margin so a unit pressed
/// against a wall counts as inside.
fn cup_of(w: &World, u: &Unit) -> Option<usize> {
    let ls = w.lines[0].as_ref().unwrap();
    let half = w.line(0).cup.inner_half;
    ls.cups.iter().position(|c| !c.judged && (u.p.x - c.x).abs() < half && u.p.y < w.rim(0) - u.r)
}

fn outside_below_rim(w: &World, u: &Unit) -> bool {
    let ls = w.lines[0].as_ref().unwrap();
    let half = w.line(0).cup.inner_half + sim::balance::WALL_HALF * 2;
    u.p.y < w.rim(0) - u.r * 2 && ls.cups.iter().all(|c| (u.p.x - c.x).abs() > half)
}

#[test]
fn no_unit_passes_through_a_cup_wall() {
    // Fast belts and a high cap, over seeded pours: a unit inside a cup below
    // the rim on one tick is never outside every cup below the rim on the
    // next, and the other way round. The brief's trap: a blade through a limb,
    // a ball through rock, here a unit through a wall (Part E).
    let mut moves = 0u64;
    for seed in 0..12u64 {
        let mut rng = Rng::new(seed + 100);
        let mut l = line(&["cola", "lemon"], "regular", "steady", (0..4).map(|_| order(&[("cola", 1)])).collect());
        l.belt.speed = Fx::ratio(rng.range(20, 240) as i64, 60);
        let mut s = setup_of(seed, (seed % 3) as u8, l);
        s.physics = Physics { gravity: sim::balance::GRAVITY * 2, cap: Fx::int(6), drag: sim::balance::DRAG };
        let mut w = World::new(s);
        let mut prev: Vec<(u32, Option<usize>, bool)> = Vec::new();
        for _ in 0..1_500 {
            let bits = if rng.chance(3) { rng.below(4) as u16 } else { prev.len() as u16 % 2 };
            w.step([Input(bits & 0x3), Input::NONE]);
            let now: Vec<(u32, Option<usize>, bool)> = w.units.iter().map(|u| (u.id, cup_of(&w, u), outside_below_rim(&w, u))).collect();
            for &(id, cup, out) in &now {
                if let Some(&(_, was_cup, was_out)) = prev.iter().find(|p| p.0 == id) {
                    assert!(!(was_cup.is_some() && out), "seed {seed}, tick {}: unit {id} left cup {was_cup:?} through a wall", w.tick);
                    assert!(!(was_out && cup.is_some()), "seed {seed}, tick {}: unit {id} entered cup {cup:?} through a wall", w.tick);
                    moves += 1;
                }
            }
            prev = now;
        }
    }
    assert!(moves > 100_000, "only {moves} unit-ticks were checked");
}

#[test]
fn units_of_one_size_would_not_swell() {
    // The swell's own control: a unit at full size stays at full size, and
    // one at birth size grows to full size in exactly `swell_ticks`.
    let t = sim::balance::tuning(sim::balance::DEFAULT_TUNING);
    let mut u = Unit { id: 0, p: V2::ZERO, q: V2::ZERO, r: sim::balance::R_BIRTH, age: 0, flavor: 0, line: 0 };
    for _ in 0..t.swell_ticks {
        assert!(u.r < sim::balance::R_FULL);
        u.grow(&t);
    }
    assert_eq!(u.r, sim::balance::R_FULL);
    u.grow(&t);
    assert_eq!(u.r, sim::balance::R_FULL);
}

#[test]
fn heavy_slush_sinks_through_lighter_slush() {
    // Sam, 2026-10-06: "a slush type that is super heavy". Lemon is poured
    // first and raspberry on top of it; once settled, the raspberry's units
    // sit lower on average than the lemon's.
    let rasp = content::setup::flavor("raspberry");
    let lemon = content::setup::flavor("lemon");
    let mut l = line(&["lemon", "raspberry"], "regular", "steady", vec![order(&[("lemon", 1)])]);
    l.belt.speed = Fx(0);
    l.spouts[1].x = l.spouts[0].x;
    l.first_x = l.spouts[0].x;
    l.lid_x = Fx::int(10_000);
    let mut w = World::new(setup_of(1, sim::balance::DEFAULT_TUNING, l));
    for t in 0..260u32 {
        let bits = if t < 30 { Input::SPOUT[0] } else if (90..120).contains(&t) { Input::SPOUT[1] } else { 0 };
        w.step([Input(bits), Input::NONE]);
    }
    let mean = |f: u8| {
        let ys: Vec<i64> = w.units.iter().filter(|u| u.flavor == f).map(|u| u.p.y.0 as i64).collect();
        assert!(ys.len() > 20, "only {} units of flavor {f}", ys.len());
        ys.iter().sum::<i64>() as f64 / ys.len() as f64 / 4096.0
    };
    let (r, le) = (mean(rasp), mean(lemon));
    assert!(r + 2.0 < le, "raspberry poured last sits at {r:.1} cm on average and lemon at {le:.1}: it did not sink");
}
