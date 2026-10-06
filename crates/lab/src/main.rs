//! Recon, the ladder and the golden replays. Not shipped.
//!
//!     cargo run --release -p lab -- recon-m1
//!     cargo run --release -p lab -- golden

use sim::fx::{Fx, V2};
use sim::slush::Unit;
use sim::{Input, World};
use std::time::Instant;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("recon-m1") => recon_m1(),
        Some("recon-m2") => recon_m2(),
        Some("ladder") => {
            let runs: u64 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(200);
            ladder(runs);
        }
        Some("splash") => {
            for hold in [20u32, 30, 40] {
                let mut w = World::new(content::setup::standing(1, sim::balance::DEFAULT_TUNING));
                for t in 0..400 {
                    w.step([Input(if t < hold { Input::SPOUT[0] } else { 0 }), Input::NONE]);
                }
                let ls = w.lines[0].as_ref().unwrap();
                let inside: u32 = w.in_cup_counts(0)[0].iter().sum();
                println!("held {hold}: emitted {} inside {inside} wasted {} loose {}", ls.emitted, ls.wasted, w.units.len() as u32 - inside);
            }
            // Where the pilot's waste lands, on the first mission.
            let m = content::missions::mission("m_first_pour").unwrap();
            let mut w = World::new(m.setup(1, sim::balance::DEFAULT_TUNING));
            let mut p = pilot::Pilot::new(pilot::Kind::Timer);
            let mut xs = Vec::new();
            let mut held = 0;
            while !w.done() {
                let i = p.input(&w, 0);
                if i.0 != 0 { held += 1; }
                w.step([i, Input::NONE]);
                for e in &w.events {
                    if let sim::world::Event::Waste { x, .. } = e {
                        let c = w.lines[0].as_ref().unwrap().cups.iter().map(|c| (x.0 - c.x.0) / 4096).min_by_key(|d| d.abs()).unwrap();
                        xs.push(c);
                    }
                }
            }
            xs.sort();
            println!("held {held} ticks; waste offsets from nearest cup center (cm): {:?}", xs);
        }
        Some("fields") => {
            // Where one unit from each spout lands with and without its
            // line's fields, over the first few seconds.
            for m in content::missions::missions().into_iter().filter(|m| m.lines.iter().any(|l| !l.conditions.is_empty())) {
                for (seat, spec) in m.lines.iter().enumerate() {
                    if spec.conditions.iter().all(|c| c == "rail") { continue; }
                    let s = m.setup(1, 1);
                    let line = s.lines[seat].clone().unwrap();
                    for i in 0..line.spouts.len() {
                        let mut offs = Vec::new();
                        for t0 in [0u32, 150, 300] {
                            let drop = |fields: bool| {
                                let ph = s.physics;
                                let (mut p, mut v) = (V2::new(line.spouts[i].x, line.spouts[i].y), V2::new(Fx(0), -sim::balance::SPOUT_SPEED));
                                let target = line.belt_y + Fx::int(14);
                                let mut t = 0;
                                while p.y > target && t < 300 {
                                    v -= v * ph.drag; v.y -= ph.gravity;
                                    if fields { v += line.field_accel(p, t0 + t); }
                                    if v.len() > ph.cap { v = v.with_len(ph.cap); }
                                    p += v; t += 1;
                                }
                                p.x
                            };
                            offs.push(((drop(true) - drop(false)).0 as f64 / 4096.0).round() as i32);
                        }
                        if offs.iter().any(|&o| o != 0) {
                            println!("{:20} line {seat} spout {} ({}): lands {:?} cm from where it would without the field, released at ticks 0, 150, 300", m.id, i + 1, spec.spouts[i], offs);
                        }
                    }
                }
            }
        }
        Some("play") => {
            for m in content::missions::missions() {
                let mut w = World::new(m.setup(1, sim::balance::DEFAULT_TUNING));
                let mut p = [pilot::Pilot::new(pilot::Kind::Timer), pilot::Pilot::new(pilot::Kind::Timer)];
                while !w.done() && w.tick < 20_000 {
                    let i = [p[0].input(&w, 0), p[1].input(&w, 1)];
                    w.step(i);
                }
                let o = content::missions::outcome(&m, &w);
                println!("{:22} ticks {:5} scores {:?} avg {:3} waste {:2}% passed {} cause {:?}", m.id, w.tick, o.scores, o.average, o.waste_pct, o.passed, o.cause);
            }
        }
        Some("golden") => golden(),
        Some("bench") => {
            let mut w = loaded(2000, 200, sim::balance::DEFAULT_TUNING);
            let t0 = Instant::now();
            for _ in 0..1500 {
                w.step([Input::NONE; 2]);
            }
            println!("{:.3} ms/tick", t0.elapsed().as_secs_f64() * 1000.0 / 1500.0);

        }
        Some("script-checksum") => {
            let ticks: u32 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(600);
            println!("{}", sim::replay::script_checksum_of(content::setup::standing(2026, sim::balance::DEFAULT_TUNING), ticks));
        }
        _ => eprintln!("usage: lab recon-m1 | golden | script-checksum <ticks>"),
    }
}

/// A standing cup `inner_half` wide, no lid, with `n` units dropped in a
/// lattice above it: a load for timing and settling.
fn loaded(n: usize, inner_half: i32, tuning: u8) -> World {
    let mut l = content::setup::line(&["cola"], "regular", "steady", vec![content::setup::order(&[("cola", 1)])]);
    l.belt.speed = Fx(0);
    l.first_x = Fx::int(250);
    l.cup.inner_half = Fx::int(inner_half);
    l.cup.inner_height = Fx::int(120);
    l.lid_x = Fx::int(10_000);
    l.end_x = Fx::int(10_000);
    let mut w = World::new(content::setup::setup_of(1, tuning, l));
    let floor = w.floor(0);
    let per_row = ((inner_half * 2 - 4) / 5).max(1) as usize;
    for k in 0..n {
        let (cx, cy) = ((k % per_row) as i32, (k / per_row) as i32);
        let p = V2::new(Fx::int(250 - inner_half + 3 + cx * 5 + (cy % 2)), floor + Fx::int(4 + cy * 5));
        w.units.push(Unit { id: k as u32, p, q: p, r: sim::balance::R_FULL, age: 999, flavor: 0, line: 0 });
    }
    w
}

fn max_speed(w: &World) -> f64 {
    w.units.iter().map(|u| (u.p - u.q).len().0 as f64 / 4096.0).fold(0.0, f64::max)
}

fn recon_m1() {
    println!("# M1.0 recon\n");
    println!("## Tick cost, native release, by loose units (tuning {})\n", sim::balance::DEFAULT_TUNING);
    println!("| units | ms per tick (mean of 300 ticks) |\n|---|---|");
    for n in [500usize, 1000, 2000] {
        let mut w = loaded(n, 200, sim::balance::DEFAULT_TUNING);
        for _ in 0..30 {
            w.step([Input::NONE; 2]);
        }
        let t0 = Instant::now();
        for _ in 0..300 {
            w.step([Input::NONE; 2]);
        }
        println!("| {n} | {:.3} |", t0.elapsed().as_secs_f64() * 1000.0 / 300.0);
    }

    println!("\n## Units that fill each cup to its rim, by tuning\n");
    println!("Hold spout 1 over a standing cup long enough to overfill it, let go, wait 600 ticks, count the units below the rim.\n");
    println!("| cup | tuning | units inside at rest | emitted | wasted |\n|---|---|---|---|---|");
    for cup in ["regular", "small"] {
        for tuning in 0..3u8 {
            let mut l = content::setup::line(&["cola"], cup, "steady", vec![content::setup::order(&[("cola", 1)])]);
            l.belt.speed = Fx(0);
            l.first_x = l.spouts[0].x;
            l.lid_x = Fx::int(10_000);
            let mut w = World::new(content::setup::setup_of(1, tuning, l));
            let cap = w.line(0).cup.capacity;
            for _ in 0..(cap * 3 / 4 + 20) {
                w.step([Input(Input::SPOUT[0]), Input::NONE]);
            }
            for _ in 0..600 {
                w.step([Input::NONE; 2]);
            }
            let inside: u32 = w.in_cup_counts(0)[0].iter().sum();
            let ls = w.lines[0].as_ref().unwrap();
            println!("| {cup} | {tuning} | {inside} | {} | {} |", ls.emitted, ls.wasted);
        }
    }

    println!("\n## A pile of about 60 units poured from one point, after 600 ticks\n");
    println!("Poured onto the floor of a cup 240 cm wide. Slope is height over half-width of the base, as a rise per run.\n");
    println!("| tuning | units | height cm | half-width cm | slope | max speed at 600, cm/tick |\n|---|---|---|---|---|---|");
    for tuning in [0u8, 1, 2, 99] {
        let mut l = content::setup::line(&["cola"], "regular", "steady", vec![content::setup::order(&[("cola", 1)])]);
        l.belt.speed = Fx(0);
        l.first_x = l.spouts[0].x;
        l.cup.inner_half = Fx::int(120);
        l.lid_x = Fx::int(10_000);
        // 99 is the control: tuning 1 with no slope rule and no thickness,
        // which is what water would do here.
        let mut w = World::new(content::setup::setup_of(1, tuning.min(2), l));
        if tuning == 99 {
            w.setup.tuning = 200;
        }
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
        let height = w.units.iter().map(|u| u.p.y + u.r - floor).max().unwrap_or(Fx(0));
        let base: Vec<&Unit> = w.units.iter().filter(|u| u.p.y - u.r - floor < Fx::int(2)).collect();
        let half = base.iter().map(|u| (u.p.x - x0).abs() + u.r).max().unwrap_or(Fx(1));
        println!(
            "| {} | {} | {:.1} | {:.1} | {:.2} | {:.3} |",
            if tuning == 99 { "control: no slope rule".to_string() } else { tuning.to_string() },
            w.units.len(),
            height.0 as f64 / 4096.0,
            half.0 as f64 / 4096.0,
            height.0 as f64 / half.0.max(1) as f64,
            max_speed(&w)
        );
    }

    println!("\n## Settling: ticks after the last unit lands until no unit moves faster than 0.05 cm/tick\n");
    println!("| tuning | ticks to rest (from a filled cup) |\n|---|---|");
    for tuning in 0..3u8 {
        let mut w = World::new(content::setup::standing(1, tuning));
        for _ in 0..70 {
            w.step([Input(Input::SPOUT[0]), Input::NONE]);
        }
        // Wait for the last unit to leave the air: 60 ticks is more than the fall.
        for _ in 0..60 {
            w.step([Input::NONE; 2]);
        }
        let mut k = 0;
        while max_speed(&w) > 0.05 && k < 3000 {
            w.step([Input::NONE; 2]);
            k += 1;
        }
        println!("| {tuning} | {k}{} |", if k >= 3000 { " (did not settle)" } else { "" });
    }
}

fn recon_m2() {
    use sim::world::Handle;
    println!("# M2.0 recon\n");
    println!("## The tail: units out after the key is let go, from full travel held for a second\n");
    println!("| tuning | spring rad/tick² | units after release | ticks to close |\n|---|---|---|---|");
    for tuning in 0..3u8 {
        let mut w = World::new(content::setup::standing(1, tuning));
        w.lines[0].as_mut().unwrap().handles[0] = Handle::full();
        for _ in 0..60 {
            w.step([Input(Input::SPOUT[0]), Input::NONE]);
        }
        let before = w.lines[0].as_ref().unwrap().emitted;
        let mut k = 0;
        while w.lines[0].as_ref().unwrap().handles[0].opening().0 > 0 && k < 600 {
            w.step([Input::NONE; 2]);
            k += 1;
        }
        let after = w.lines[0].as_ref().unwrap().emitted;
        let spring = sim::balance::tuning(tuning).spring;
        println!("| {tuning} | {:.4} | {} | {k} |", spring.0 as f64 / 4096.0, after - before);
    }

    println!("\n## Spill from a full cup carried by the belt\n");
    println!("A standing regular cup overfilled for 110 ticks and left 300 to settle, then carried 400 cm. Units lost on the way.\n");
    println!("| belt | cm/s | units in the cup before | units lost |\n|---|---|---|---|");
    let d = content::setup::line_def();
    for belt in ["slow", "steady", "quick"] {
        let mut l = content::setup::line(&["cola"], "regular", belt, vec![content::setup::order(&[("cola", 1)])]);
        let speed = l.belt.speed;
        l.belt.speed = Fx(0);
        l.belt.schedule = vec![(410, speed)];
        l.first_x = l.spouts[0].x;
        l.lid_x = Fx::int(10_000);
        l.end_x = Fx::int(10_000);
        let mut w = World::new(content::setup::setup_of(1, sim::balance::DEFAULT_TUNING, l));
        for t in 0..410 {
            w.step([Input(if t < 110 { Input::SPOUT[0] } else { 0 }), Input::NONE]);
        }
        let before: u32 = w.in_cup_counts(0)[0].iter().sum();
        let wasted0 = w.lines[0].as_ref().unwrap().wasted;
        let ticks = (400 * 4096 / speed.0.max(1)) as u32;
        for _ in 0..ticks {
            w.step([Input::NONE; 2]);
        }
        let after: u32 = w.in_cup_counts(0)[0].iter().sum();
        let lost = w.lines[0].as_ref().unwrap().wasted - wasted0;
        println!("| {belt} | {} | {before} | {lost} (in cup after: {after}) |", d.belts[belt]);
    }

    println!("\n## Encoding and hashing the world\n");
    println!("| units | µs per checksum (mean of 200) |\n|---|---|");
    for n in [500usize, 2000] {
        let w = loaded(n, 200, sim::balance::DEFAULT_TUNING);
        let t0 = Instant::now();
        let mut x = 0u64;
        for _ in 0..200 {
            x ^= w.checksum();
        }
        println!("| {n} | {:.1} |{}", t0.elapsed().as_secs_f64() * 1e6 / 200.0, if x == 1 { " " } else { "" });
    }
}

/// Play the yardstick on every mission, `runs` times each with its own
/// seed, and write `analysis/ladder.md` (PLANNING-BRIEF M4.0).
fn ladder(runs: u64) {
    let ms = content::missions::missions();
    let handles: Vec<_> = ms
        .iter()
        .cloned()
        .map(|m| {
            std::thread::spawn(move || {
                let mut avgs = Vec::new();
                let mut passes = 0;
                for seed in 0..runs {
                    let mut w = World::new(m.setup(seed, sim::balance::DEFAULT_TUNING));
                    let mut p = [
                        pilot::Pilot::new(pilot::Kind::Yardstick { seed: seed + 1, error: pilot::YARDSTICK_ERROR }),
                        pilot::Pilot::new(pilot::Kind::Yardstick { seed: seed + 1001, error: pilot::YARDSTICK_ERROR }),
                    ];
                    while !w.done() && w.tick < 30_000 {
                        let i = [p[0].input(&w, 0), p[1].input(&w, 1)];
                        w.step(i);
                    }
                    let o = content::missions::outcome(&m, &w);
                    avgs.push(o.average as f64);
                    passes += o.passed as u32;
                }
                let n = avgs.len() as f64;
                let mean = avgs.iter().sum::<f64>() / n;
                let var = avgs.iter().map(|a| (a - mean).powi(2)).sum::<f64>() / (n - 1.0).max(1.0);
                (m.id.clone(), mean, (var / n).sqrt(), passes)
            })
        })
        .collect();
    let rows: Vec<(String, f64, f64, u32)> = handles.into_iter().map(|h| h.join().unwrap()).collect();
    let mut out = String::new();
    out.push_str("# The ladder\n\n");
    out.push_str(&format!(
        "The yardstick pilot (the timer with up to {} units of error on each let-go, seeded) on every mission, {runs} runs each, tuning {}. Written by `make ladder`; read by `the_path_gets_no_easier`.\n\n",
        pilot::YARDSTICK_ERROR,
        sim::balance::DEFAULT_TUNING
    ));
    out.push_str(&format!("fingerprint: {}\n\n", content::missions::fingerprint(pilot::VERSION)));
    out.push_str("| # | mission | runs | average | standard error | passed |\n|---|---|---|---|---|---|\n");
    for (i, (id, mean, se, passes)) in rows.iter().enumerate() {
        out.push_str(&format!("| {} | {id} | {runs} | {mean:.2} | {se:.2} | {passes} |\n", i + 1));
    }
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../analysis/ladder.md");
    std::fs::write(path, &out).unwrap();
    print!("{out}");
}

/// Record the golden replay: the fixed script over the standing pour.
fn golden() {
    let mut rec = sim::replay::Recording::new(content::setup::standing(2026, sim::balance::DEFAULT_TUNING));
    for t in 0..1500 {
        rec.step(sim::replay::script(t));
    }
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../testing/replays/golden.replay");
    std::fs::write(path, rec.bytes()).unwrap();
    println!("wrote {path}: {} ticks, checksum {:016x}", rec.world.tick, rec.world.checksum());
}
