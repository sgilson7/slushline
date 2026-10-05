//! The content lints for the path (PLANNING-BRIEF 0.6), and the cases H4
//! and H6, which are about what a line can pour.

use content::missions::{conditions, held, mission, missions, outcome, Mission};
use content::setup::{flavor, line, order, pours, setup_of};
use sim::score::score;
use sim::setup::{Line, Order};
use sim::{Input, World};

/// The best score an order can reach from a line: every way of splitting a
/// cup's room among the spouts, each spout pouring its cycle from the start.
/// Brute force over the first spouts; the last takes the rest of the room,
/// because more slush never lowers a score.
fn best(line: &Line, o: &Order) -> u32 {
    let c = line.cup.capacity;
    let n = line.spouts.len();
    let flavors = content::setup::flavor_count() as usize;
    let counts = |amounts: &[u32]| {
        let mut k = vec![0u32; flavors];
        for (s, &a) in line.spouts.iter().zip(amounts) {
            for j in 0..a as usize {
                k[s.pours[j % s.pours.len()] as usize] += 1;
            }
        }
        k
    };
    let mut top = 0;
    let mut amounts = vec![0u32; n];
    fn walk(i: usize, left: u32, amounts: &mut Vec<u32>, f: &mut dyn FnMut(&[u32])) {
        if i + 1 == amounts.len() {
            amounts[i] = left;
            f(amounts);
            return;
        }
        for a in 0..=left {
            amounts[i] = a;
            walk(i + 1, left - a, amounts, f);
        }
    }
    walk(0, c, &mut amounts, &mut |a| top = top.max(score(&counts(a), o, c).score));
    top
}

/// H6 — an order the line cannot make. A cola spout and a cherry cola spout,
/// capacity 120: cherry alone tops out at 50 (120 units of the blend hold 60
/// cherry); cola and cherry at 1 to 3 want 90 cherry and top out at 75; at
/// 3 to 1 they reach 100 (H4).
#[test]
fn h6_an_order_the_line_cannot_make_is_found() {
    let l = line(&["cola", "cherry_cola"], "regular", "steady", vec![]);
    assert_eq!(l.cup.capacity, 120);
    assert_eq!(best(&l, &order(&[("cherry", 1)])), 50);
    assert_eq!(best(&l, &order(&[("cola", 1), ("cherry", 3)])), 75);
    assert_eq!(best(&l, &order(&[("cola", 3), ("cherry", 1)])), 100);
}

#[test]
fn every_order_can_reach_full_marks_from_its_line() {
    let mut checked = 0;
    for m in missions() {
        let s = m.setup(1, 1);
        let l = s.lines[0].as_ref().unwrap();
        for o in &m.orders {
            let got = best(l, &o.order());
            assert_eq!(got, 100, "{}: the order {:?} tops out at {got} from the spouts {:?}", m.id, o.parts(), m.line.spouts);
            checked += 1;
        }
    }
    assert!(checked >= 15, "only {checked} orders checked");
    // missions.list.m_more_cola.try says cherry reaches a cup only through
    // the blend: that mission's line has no cherry spout.
    let more = mission("m_more_cola").unwrap();
    assert!(!more.line.spouts.iter().any(|s| s == "cherry"), "m_more_cola's line has a cherry spout");
    assert!(more.line.spouts.iter().any(|s| s == "cherry_cola"));
}

/// H4 — a blend counts as its parts. Capacity 120, cola and cherry at 3 to 1:
/// shares 90 and 30. Sixty units from the cherry cola spout are 30 cherry and
/// 30 cola; sixty more from the cola spout make 90 cola. The score is 100.
/// Filed as a flavor of its own, the blend would count for nothing and the
/// cup would score 50.
#[test]
fn h4_a_blend_counts_as_its_parts() {
    let cycle = pours("cherry_cola");
    let (ch, co) = (flavor("cherry"), flavor("cola"));
    assert_eq!(cycle, vec![ch, co], "cherry cola pours cherry, then cola");
    let first_six: Vec<u8> = (0..6).map(|k| cycle[k % cycle.len()]).collect();
    assert_eq!(first_six, vec![ch, co, ch, co, ch, co]);
    let mut counts = vec![0u32; 3];
    for k in 0..60 {
        counts[cycle[k % 2] as usize] += 1;
    }
    counts[co as usize] += 60;
    assert_eq!((counts[co as usize], counts[ch as usize]), (90, 30));
    let o = order(&[("cola", 3), ("cherry", 1)]);
    assert_eq!(score(&counts, &o, 120).score, 100);
    // The other reading: 60 cola counted, the blend's 60 counted as nothing.
    let mut as_own = vec![0u32; 3];
    as_own[co as usize] = 60;
    assert_eq!(score(&as_own, &o, 120).score, 50);
}

#[test]
fn a_blend_spout_emits_its_flavors_in_its_fixed_order() {
    // On the real line: hold the cherry cola spout and read the flavors in
    // the order the units left it.
    let m = mission("m_cherry_cola").unwrap();
    let mut w = World::new(m.setup(1, 1));
    for _ in 0..40 {
        w.step([Input(Input::SPOUT[1]), Input::NONE]);
    }
    let mut out: Vec<(u32, u8)> = w.units.iter().map(|u| (u.id, u.flavor)).collect();
    out.sort();
    let (ch, co) = (flavor("cherry"), flavor("cola"));
    assert!(out.len() >= 6);
    for (k, &(_, f)) in out.iter().enumerate() {
        assert_eq!(f, if k % 2 == 0 { ch } else { co }, "unit {k} of the blend");
    }
}

#[test]
fn every_share_is_a_whole_number_of_units() {
    for m in missions().into_iter().chain(held()) {
        let cap = content::setup::cups()[&m.line.cup].capacity;
        for o in &m.orders {
            let t: u32 = o.parts().iter().map(|p| p.1).sum();
            for (f, n) in o.parts() {
                assert_eq!(cap * n % t, 0, "{}: {f}'s share is {cap}·{n}/{t}, not whole", m.id);
            }
        }
    }
}

#[test]
fn the_path_is_a_chain() {
    let ms = missions();
    assert_eq!(ms.len(), 11, "the MVP path has eleven missions (PLAN.md §8 Q5)");
    assert!(ms[0].requires.is_empty(), "the first mission is open from the start");
    for w in ms.windows(2) {
        let req: Vec<&str> = w[1].requires.iter().map(|r| r.pass.as_str()).collect();
        assert_eq!(req, vec![w[0].id.as_str()], "{} must require exactly {}", w[1].id, w[0].id);
    }
    let mut ids: Vec<&str> = ms.iter().map(|m| m.id.as_str()).collect();
    ids.sort();
    ids.dedup();
    assert_eq!(ids.len(), ms.len(), "two missions share an id");
}

#[test]
fn every_component_is_taught_after_the_ones_it_builds_on() {
    // `builds_on` lists edges `A_B` of data/kc_graph.json: B is what the
    // mission teaches, and A must have been taught by an earlier one.
    let g: serde_json::Value = serde_json::from_str(include_str!("../../../data/kc_graph.json")).unwrap();
    let nodes: Vec<&str> = g["nodes"].as_array().unwrap().iter().map(|n| n["id"].as_str().unwrap()).collect();
    let edges: Vec<&str> = g["edges"].as_array().unwrap().iter().map(|e| e.as_str().unwrap()).collect();
    let copy = content::copy::copy();
    let mut taught: Vec<String> = Vec::new();
    for m in missions().into_iter().chain(held()) {
        for t in &m.teaches {
            assert!(nodes.contains(&t.as_str()), "{} teaches {t}, which is not in the graph", m.id);
            for part in ["name", "when", "then"] {
                assert!(copy["kc"][t][part].is_string(), "kc.{t}.{part} has no sentence");
            }
        }
        for b in &m.builds_on {
            let (from, to) = b.split_once('_').unwrap_or_else(|| panic!("{}: {b} is not an edge", m.id));
            assert!(edges.contains(&b.as_str()), "{}: the edge {b} is not in the graph", m.id);
            assert!(copy["kc_edge"][b].is_string(), "kc_edge.{b} has no sentence");
            assert!(m.teaches.iter().any(|t| t == to), "{} builds on {b} but does not teach {to}", m.id);
            assert!(taught.iter().any(|t| t == from), "{} builds on {from}, which no earlier mission teaches", m.id);
        }
        taught.extend(m.teaches.iter().cloned());
    }
    for e in &edges {
        assert!(missions().into_iter().chain(held()).any(|m| m.builds_on.iter().any(|b| b == e)), "the edge {e} is used by no mission");
    }
}

#[test]
fn every_condition_a_mission_names_exists() {
    let c = conditions();
    for m in missions() {
        for k in &m.conditions {
            assert!(c.get(k).is_some(), "{} names the condition {k}, which data/conditions.json lacks", m.id);
            assert!(c[k].get("held").is_none(), "{} uses {k}, which is held", m.id);
        }
    }
}

#[test]
fn every_condition_is_used_by_a_mission_or_marked_held() {
    let c = conditions();
    let used: Vec<String> = missions().iter().flat_map(|m| m.conditions.clone()).collect();
    for (k, v) in c.as_object().unwrap() {
        if k.starts_with('_') {
            continue;
        }
        assert!(used.contains(k) || v.get("held").is_some(), "the condition {k} is neither used nor held");
    }
}

#[test]
fn every_mission_s_card_and_spouts_read_from_the_copy_file() {
    for m in missions() {
        let card = content::missions::card(&m);
        assert!(card["order"].as_str().unwrap().ends_with('.'));
        assert_eq!(content::missions::spout_labels(&m).len(), m.line.spouts.len());
        let copy = content::copy::copy();
        let tk = card["try_key"].as_str().unwrap();
        let mut v = &copy;
        for k in tk.split('.') {
            v = &v[k];
        }
        assert!(v.is_string(), "{} has no {tk}", m.id);
    }
}

fn run(m: &Mission, mut input: impl FnMut(&World) -> Input) -> World {
    let mut w = World::new(m.setup(1, sim::balance::DEFAULT_TUNING));
    while !w.done() && w.tick < 30_000 {
        let i = input(&w);
        w.step([i, Input::NONE]);
    }
    assert!(w.done(), "{} never finished", m.id);
    w
}

#[test]
fn every_mission_can_be_passed() {
    // By the timer pilot, which returns an Input and nothing else.
    for m in missions() {
        let mut p = pilot::Pilot::new(pilot::Kind::Timer);
        let w = run(&m, |w| p.input(w, 0));
        let o = outcome(&m, &w);
        assert!(o.passed, "{}: the timer averaged {} with {}% waste", m.id, o.average, o.waste_pct);
    }
}

#[test]
fn no_mission_after_the_first_is_passed_with_every_key_held() {
    // The brief's trap: a goal met by standing still, here by holding every
    // key down (Part E). The waste limit is what stops it.
    for m in missions().iter().skip(1) {
        let w = run(m, |_| Input(0x0F));
        let o = outcome(m, &w);
        assert!(!o.passed, "{} was passed with every key held: average {}, waste {}%", m.id, o.average, o.waste_pct);
    }
}

#[test]
fn the_result_names_what_cost_the_most_points() {
    // The named cause is the largest of the three losses summed over the
    // cups, worked out here from the cups' own counts, not from the score's
    // fields; and the last line of the result says it.
    let m = mission("m_half").unwrap();
    let cases: Vec<(&str, Box<dyn FnMut(&World) -> Input>)> = vec![
        ("idle", Box::new(|_: &World| Input::NONE)),
        ("cola only", Box::new(|w: &World| {
            let ls = w.lines[0].as_ref().unwrap();
            Input(if ls.cups.iter().any(|c| !c.judged && (c.x - w.spout_x(0, 0)).abs() < sim::fx::Fx::int(8)) { Input::SPOUT[0] } else { 0 })
        })),
    ];
    let o_spec = order(&[("cola", 1), ("lemon", 1)]);
    let mut seen = Vec::new();
    for (name, f) in cases {
        let w = run(&m, f);
        let o = outcome(&m, &w);
        let (mut empty, mut over, mut wrong) = (0u32, 0u32, 0u32);
        for r in &w.lines[0].as_ref().unwrap().results {
            let total: u32 = r.counts.iter().sum();
            empty += 120u32.saturating_sub(total);
            for (fl, &n) in r.counts.iter().enumerate() {
                let share = o_spec.share(fl as u8, 120);
                if o_spec.parts.iter().any(|p| p.0 as usize == fl) {
                    over += n.saturating_sub(share);
                } else {
                    wrong += n;
                }
            }
        }
        let want = if empty >= over && empty >= wrong { "empty" } else if over >= wrong { "over" } else { "wrong" };
        assert_eq!(o.cause.as_deref(), Some(want), "{name}: losses empty {empty}, over {over}, wrong {wrong}; {:?}", o.lines);
        let cause_text = content::copy::fill(&format!("results.cause.{want}"), &serde_json::json!({}));
        assert!(o.lines.last().unwrap().contains(&cause_text), "{name}: {:?}", o.lines);
        seen.push(want);
    }
    assert_eq!(seen, vec!["empty", "over"], "the two runs should name two different causes");
    let c = flavor("cola");
    let mut counts = vec![0u32; 3];
    counts[c as usize] = 120;
    let s = score(&counts, &o_spec, 120);
    assert_eq!((s.cause, s.worst), (sim::score::Cause::Over, Some(c)));
}

#[test]
fn the_belt_moves_one_way() {
    // missions.list.m_three.try: "the belt does not go back".
    for m in missions() {
        let s = m.setup(1, 1);
        let l = s.lines[0].as_ref().unwrap();
        assert!(l.belt.speed.0 > 0 && l.belt.schedule.iter().all(|&(_, v)| v.0 >= 0), "{}", m.id);
    }
}

#[test]
fn a_cup_is_judged_once_on_the_tick_it_crosses_the_lid() {
    let m = mission("m_first_pour").unwrap();
    let mut w = World::new(m.setup(1, 1));
    let lid = w.line(0).lid_x;
    let mut judged_at = vec![None; m.cup_count() as usize];
    while !w.done() {
        let before: Vec<bool> = w.lines[0].as_ref().unwrap().cups.iter().map(|c| c.x >= lid).collect();
        w.step([Input(Input::SPOUT[0]), Input::NONE]);
        for e in &w.events {
            if let sim::world::Event::Judged { cup, .. } = e {
                assert!(judged_at[*cup as usize].is_none(), "cup {cup} judged twice");
                judged_at[*cup as usize] = Some(w.tick);
                let c = &w.lines[0].as_ref().unwrap().cups[*cup as usize];
                assert!(c.x >= lid && !before[*cup as usize], "cup {cup} was judged a tick late or early");
            }
        }
    }
    assert!(judged_at.iter().all(Option::is_some));
}

#[test]
fn judged_units_leave_the_world() {
    let m = mission("m_first_pour").unwrap();
    let mut p = pilot::Pilot::new(pilot::Kind::Timer);
    let mut w = World::new(m.setup(1, 1));
    while !w.done() {
        let i = p.input(&w, 0);
        w.step([i, Input::NONE]);
        let ls = w.lines[0].as_ref().unwrap();
        for c in ls.cups.iter().filter(|c| c.judged) {
            assert!(!w.units.iter().any(|u| w.inside(u, c)), "a unit is still inside a judged cup at tick {}", w.tick);
        }
    }
    assert!(w.lines[0].as_ref().unwrap().judged > 200, "the pilot poured too little to check");
}

#[test]
fn the_waste_limit_is_decided_exactly() {
    // 20 of 100 is not under a 20 % limit; 19 of 100 is.
    assert_eq!(content::missions::waste(20, 100), 20);
    let m = mission("m_tail").unwrap();
    assert_eq!(m.pass.waste_pct, Some(20));
    let _ = setup_of;
}

/// The ladder's rows: mission, average, standard error.
fn ladder() -> (String, Vec<(String, f64, f64)>) {
    let text = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../../analysis/ladder.md"))
        .expect("analysis/ladder.md exists; run `make ladder`");
    let fp = text.lines().find_map(|l| l.strip_prefix("fingerprint: ")).expect("the ladder records its fingerprint").trim().to_string();
    let rows = text
        .lines()
        .filter(|l| l.starts_with("| ") && !l.starts_with("| #"))
        .map(|l| {
            let c: Vec<&str> = l.split('|').map(str::trim).collect();
            (c[2].to_string(), c[4].parse().unwrap(), c[5].parse().unwrap())
        })
        .collect();
    (fp, rows)
}

#[test]
fn the_path_gets_no_easier_within_a_chapter() {
    // PLANNING-BRIEF 0.6 names this lint `the_path_gets_no_easier`. The
    // ladder (yardstick averages, 200 seeded runs a mission) rises within
    // every chapter and falls where a chapter opens on a new idea, and the
    // path cannot be reordered past what each mission builds on. So this
    // checks the claim the measurement supports, within chapters, and the
    // whole-path claim is SECOND-ORDER-M4 row 2, for Sam's play to settle.
    let (fp, rows) = ladder();
    assert_eq!(fp, content::missions::fingerprint(pilot::VERSION), "analysis/ladder.md is stale; run `make ladder`");
    let ms = missions();
    assert_eq!(rows.iter().map(|r| r.0.clone()).collect::<Vec<_>>(), ms.iter().map(|m| m.id.clone()).collect::<Vec<_>>());
    let mut steps = 0;
    for (w, pair) in ms.windows(2).zip(rows.windows(2)) {
        if w[0].chapter != w[1].chapter {
            continue;
        }
        let ((_, a, sa), (_, b, sb)) = (&pair[0], &pair[1]);
        let noise = 2.0 * (sa * sa + sb * sb).sqrt();
        assert!(b <= &(a + noise), "{} ({b:.2}) is easier than {} ({a:.2}) for the yardstick, beyond the noise of {noise:.2}", w[1].id, w[0].id);
        steps += 1;
    }
    assert!(steps >= 5, "only {steps} steps within chapters were checked");
}
