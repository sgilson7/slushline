//! The missions (PLANNING-BRIEF 0.6; D12), the conditions that change a
//! line's setup (0.7; D13), and the result a mission ends with.
//!
//! The shape is Vagrancy's `data/tutorial.json` (`id`, `chapter`, `teaches`,
//! `builds_on`, `requires`) plus the lines and the pass mark. The missions are
//! a tree after Vagrancy's `data/road.json` (Sam, 2026-10-05): a mission
//! opens when every requirement is met, and its row is how many it has.

use crate::copy::fill;
use crate::setup::{self, flavor};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sim::fx::{Fx, V2};
use sim::score::{average, Cause};
use sim::setup::{Field, Line, Order, Setup};
use sim::World;

pub const MISSIONS_JSON: &str = include_str!("../../../data/missions.json");
pub const CONDITIONS_JSON: &str = include_str!("../../../data/conditions.json");

/// How far above the lower line the upper one sits when a mission has two,
/// in cm.
pub const UPPER_LINE_RISE: i32 = 170;

/// One thing a player must have done before a mission opens.
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum Requirement {
    /// Passed that mission.
    Pass(String),
    /// Averaged at least `average` on it, in one run, passed or not.
    Mark { mission: String, average: u32 },
    /// Passed it with waste under `waste_pct` percent.
    Clean { mission: String, waste_pct: u32 },
}

impl Requirement {
    pub fn mission(&self) -> &str {
        match self {
            Requirement::Pass(m) | Requirement::Mark { mission: m, .. } | Requirement::Clean { mission: m, .. } => m,
        }
    }

    /// The sentence that states it, with the mission's name.
    pub fn sentence(&self) -> String {
        let name = fill(&format!("missions.list.{}.name", self.mission()), &json!({}));
        match self {
            Requirement::Pass(_) => fill("missions.req.pass", &json!({ "mission": name })),
            Requirement::Mark { average, .. } => {
                fill("missions.req.mark", &json!({ "mission": name, "avg": average, "max_score": sim::balance::MAX_SCORE }))
            }
            Requirement::Clean { waste_pct, .. } => fill("missions.req.clean", &json!({ "mission": name, "waste_limit_pct": waste_pct })),
        }
    }
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct LineSpec {
    pub belt: String,
    pub cup: String,
    pub spouts: Vec<String>,
    pub orders: Vec<OrderSpec>,
    #[serde(default)]
    pub conditions: Vec<String>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct OrderSpec {
    pub count: u32,
    /// Flavor id to parts, in the order the sentence names them.
    pub parts: serde_json::Map<String, Value>,
}

impl OrderSpec {
    pub fn parts(&self) -> Vec<(String, u32)> {
        self.parts.iter().map(|(k, v)| (k.clone(), v.as_u64().expect("parts are whole numbers") as u32)).collect()
    }
    pub fn order(&self) -> Order {
        Order { parts: self.parts().iter().map(|(f, n)| (flavor(f), *n)).collect() }
    }
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Pass {
    pub average: u32,
    /// Waste must stay under this share of what was poured, in percent.
    pub waste_pct: Option<u32>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct Mission {
    pub id: String,
    pub chapter: String,
    pub teaches: Vec<String>,
    pub builds_on: Vec<String>,
    pub requires: Vec<Requirement>,
    pub lines: Vec<LineSpec>,
    pub pass: Pass,
}

#[derive(Deserialize)]
struct File {
    missions: Vec<Mission>,
    #[serde(default)]
    held: Vec<Mission>,
}

/// Every mission, in the file's order.
pub fn missions() -> Vec<Mission> {
    serde_json::from_str::<File>(MISSIONS_JSON).expect("data/missions.json is valid").missions
}

/// Missions designed and not yet built.
pub fn held() -> Vec<Mission> {
    serde_json::from_str::<File>(MISSIONS_JSON).expect("data/missions.json is valid").held
}

pub fn mission(id: &str) -> Option<Mission> {
    missions().into_iter().find(|m| m.id == id)
}

pub fn conditions() -> Value {
    serde_json::from_str(CONDITIONS_JSON).expect("data/conditions.json is valid")
}

/// A thousandth of a cm per tick², as data writes field strengths.
fn milli(n: i64) -> Fx {
    Fx::ratio(n, 1000)
}

/// Apply a condition from `data/conditions.json` to a line.
fn apply(line: &mut Line, id: &str, def: &Value) {
    assert!(def.get("held").is_none(), "the held condition {id} is in use");
    if id == "second_line" {
        return;
    }
    let spout = def["spout"].as_u64().unwrap_or_else(|| panic!("{id} names no spout")) as usize;
    let int = |k: &str| def[k].as_i64().unwrap_or_else(|| panic!("{id} has no {k}"));
    let rim = line.belt_y + sim::balance::WALL_HALF * 2 + line.cup.inner_height;
    let (sx, sy) = (line.spouts[spout].x, line.spouts[spout].y);
    if id == "rail" {
        line.spouts[spout].rail = Some(sim::setup::Rail { amplitude: int("amplitude") as i32, period: int("period") as u32 });
        return;
    }
    match def["field"].as_str() {
        Some("plates") => {
            let a = def["accel"].as_array().unwrap();
            line.fields.push(Field::Plates {
                x0: sx + Fx::int(int("x0") as i32),
                x1: sx + Fx::int(int("x1") as i32),
                y0: rim + Fx::int(int("from_rim") as i32),
                y1: sy - Fx::int(int("below_spout") as i32),
                accel: V2::new(milli(a[0].as_i64().unwrap()), milli(a[1].as_i64().unwrap())),
                flip: def.get("flip").and_then(Value::as_u64).map(|f| f as u32),
            });
        }
        Some("charge") => line.fields.push(Field::Charge {
            at: V2::new(sx + Fx::int(int("dx") as i32), sy + Fx::int(int("dy") as i32)),
            strength: milli(int("strength")),
            radius: Fx::int(int("radius") as i32),
        }),
        _ => panic!("no code applies the condition {id}"),
    }
}

impl LineSpec {
    pub fn cup_orders(&self) -> Vec<Order> {
        self.orders.iter().flat_map(|o| std::iter::repeat_n(o.order(), o.count as usize)).collect()
    }
    pub fn cup_count(&self) -> u32 {
        self.orders.iter().map(|o| o.count).sum()
    }
    /// The line this spec builds, `rise` cm above the usual height, its
    /// conditions applied.
    pub fn build(&self, rise: i32) -> Line {
        let spouts: Vec<&str> = self.spouts.iter().map(String::as_str).collect();
        let mut line = setup::line(&spouts, &self.cup, &self.belt, self.cup_orders());
        line.belt_y += Fx::int(rise);
        for s in &mut line.spouts {
            s.y += Fx::int(rise);
        }
        let conds = conditions();
        for c in &self.conditions {
            apply(&mut line, c, &conds[c]);
        }
        line
    }
}

impl Mission {
    pub fn cup_count(&self) -> u32 {
        self.lines.iter().map(LineSpec::cup_count).sum()
    }

    /// The row of the tree it sits in: how many requirements it has.
    pub fn level(&self) -> usize {
        self.requires.len()
    }

    /// Every condition on any of its lines, once, and `second_line` when it
    /// has two.
    pub fn condition_ids(&self) -> Vec<String> {
        let mut out: Vec<String> = Vec::new();
        if self.lines.len() > 1 {
            out.push("second_line".into());
        }
        for c in self.lines.iter().flat_map(|l| l.conditions.iter()) {
            if !out.contains(c) {
                out.push(c.clone());
            }
        }
        out
    }

    /// The world this mission is played in. With two lines, the first is the
    /// upper one and is seat 0; the second, lower, is seat 1.
    pub fn setup(&self, seed: u64, tuning: u8) -> Setup {
        assert!((1..=2).contains(&self.lines.len()), "{} has {} lines", self.id, self.lines.len());
        let mut s = setup::setup_of(seed, tuning, self.lines[0].build(if self.lines.len() == 2 { UPPER_LINE_RISE } else { 0 }));
        if let Some(lower) = self.lines.get(1) {
            s.lines[1] = Some(lower.build(0));
        }
        s
    }
}

/// An order as the sentence `recipe.*` names it: "cola", "equal shares of
/// cola and lemon", "cola and lemon, 2 to 1".
pub fn recipe(parts: &[(String, u32)]) -> String {
    let mid = |f: &str| fill(&format!("flavors.{f}.name_mid"), &json!({}));
    match parts {
        [(a, _)] => fill("recipe.one", &json!({ "flavor_mid": mid(a) })),
        [(a, x), (b, y)] if x == y => fill("recipe.two_equal", &json!({ "a": mid(a), "b": mid(b) })),
        [(a, x), (b, y)] => fill("recipe.two", &json!({ "a": mid(a), "b": mid(b), "a_parts": x, "b_parts": y })),
        [(a, x), (b, y), (c, z)] => fill(
            "recipe.three",
            &json!({ "a": mid(a), "b": mid(b), "c": mid(c), "a_parts": x, "b_parts": y, "c_parts": z }),
        ),
        _ => panic!("an order of {} flavors has no recipe sentence", parts.len()),
    }
}

/// A blend's recipe, for its spout label (`spout.blend`).
pub fn blend_recipe(id: &str) -> String {
    let b = setup::blends().into_iter().find(|b| b.id == id).unwrap_or_else(|| panic!("no blend {id}"));
    recipe(&b.parts)
}

/// What the mission's card says: its name, order line, pass line,
/// conditions, and what opens it.
pub fn card(m: &Mission) -> Value {
    let specs: Vec<&OrderSpec> = m.lines.iter().flat_map(|l| l.orders.iter()).collect();
    let same = specs.windows(2).all(|w| w[0].parts() == w[1].parts());
    let order = if same {
        fill("missions.order.same", &json!({ "count": m.cup_count(), "recipe": recipe(&specs[0].parts()) }))
    } else {
        fill("missions.order.mixed", &json!({ "count": m.cup_count() }))
    };
    let max = sim::balance::MAX_SCORE;
    let pass = match m.pass.waste_pct {
        Some(w) => fill("missions.pass", &json!({ "pass": m.pass.average, "max_score": max, "waste_limit_pct": w })),
        None => fill("missions.pass_no_limit", &json!({ "pass": m.pass.average, "max_score": max })),
    };
    let conditions: Vec<Value> = m
        .condition_ids()
        .iter()
        .map(|c| json!({ "name": fill(&format!("conditions.{c}.name"), &json!({})), "desc": fill(&format!("conditions.{c}.desc"), &json!({})) }))
        .collect();
    json!({
        "id": m.id,
        "chapter": m.chapter,
        "level": m.level(),
        "name": fill(&format!("missions.list.{}.name", m.id), &json!({})),
        "order": order,
        "pass": pass,
        "try_key": format!("missions.list.{}.try", m.id),
        "conditions": conditions,
        "cups": m.cup_count(),
        "lines": m.lines.len(),
        "spouts": m.lines.iter().map(|l| l.spouts.len()).collect::<Vec<_>>(),
    })
}

/// The labels of each line's spouts, in belt order (`spout.single`,
/// `spout.blend`).
pub fn spout_labels(m: &Mission) -> Vec<Vec<String>> {
    let look = crate::look::flavors();
    m.lines
        .iter()
        .map(|l| {
            l.spouts
                .iter()
                .map(|id| {
                    if setup::is_blend(id) {
                        fill("spout.blend", &json!({ "blend": fill(&format!("blends.{id}.name"), &json!({})), "recipe": blend_recipe(id) }))
                    } else {
                        let pattern = &look.iter().find(|f| &f.id == id).unwrap().pattern;
                        fill(
                            "spout.single",
                            &json!({
                                "flavor": fill(&format!("flavors.{id}.name"), &json!({})),
                                "short": fill(&format!("flavors.{id}.short"), &json!({})),
                                "pattern": fill(&format!("patterns.{pattern}"), &json!({})),
                            }),
                        )
                    }
                })
                .collect()
        })
        .collect()
}

/// How a finished mission went: the numbers, and the sentences for them.
#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub struct Outcome {
    pub scores: Vec<u32>,
    pub average: u32,
    pub waste_pct: u32,
    pub passed: bool,
    /// The cause that cost the most points over every cup, if any did.
    pub cause: Option<String>,
    /// One line per cup, then the mission's line, then the cause's line.
    pub lines: Vec<String>,
    /// The longest run of the streak's word on any line.
    pub best_streak: u32,
}

/// The waste share in whole percent, floored.
pub fn waste(wasted: u32, emitted: u32) -> u32 {
    if emitted == 0 {
        0
    } else {
        (wasted as u64 * 100 / emitted as u64) as u32
    }
}

pub fn outcome(m: &Mission, w: &World) -> Outcome {
    let mut results = Vec::new();
    let (mut wasted, mut emitted) = (0u32, 0u32);
    let mut best_streak = 0;
    for ls in w.lines.iter().flatten() {
        let mut r = ls.results.clone();
        r.sort_by_key(|r| r.cup);
        let word = streak_word();
        let mut run = 0;
        for x in &r {
            run = if judgement(x.score.score) == word { run + 1 } else { 0 };
            best_streak = best_streak.max(run);
        }
        results.extend(r);
        wasted += ls.wasted;
        emitted += ls.emitted;
    }
    let scores: Vec<u32> = results.iter().map(|r| r.score.score).collect();
    let avg = average(&scores);
    let waste_pct = waste(wasted, emitted);
    let under = m.pass.waste_pct.is_none_or(|lim| (wasted as u64) * 100 < lim as u64 * emitted.max(1) as u64);
    let passed = avg >= m.pass.average && under && scores.len() as u32 == m.cup_count();
    let ids: Vec<String> = crate::look::flavors().into_iter().map(|f| f.id).collect();
    let max = sim::balance::MAX_SCORE;
    let mut lines = Vec::new();
    let (mut empty, mut over, mut wrong) = (0u32, 0u32, 0u32);
    for (k, r) in results.iter().enumerate() {
        let s = &r.score;
        lines.push(fill("results.cup.line", &json!({ "n": k + 1, "score": s.score, "max_score": max })));
        let why = match s.cause {
            Cause::None => fill("results.cup.full", &json!({})),
            Cause::Empty => fill("results.cup.short", &json!({ "fill_pct": s.fill_pct })),
            Cause::Over => fill("results.cup.over", &json!({ "flavor_mid": mid(&ids[s.worst.unwrap() as usize]) })),
            Cause::Wrong => fill("results.cup.wrong", &json!({ "flavor_mid": mid(&ids[s.worst.unwrap() as usize]) })),
        };
        lines.push(why);
        empty += s.empty;
        over += s.over;
        wrong += s.wrong;
    }
    if passed {
        lines.push(fill("results.mission.pass", &json!({ "avg": avg, "max_score": max, "pass": m.pass.average })));
    } else if avg < m.pass.average || m.pass.waste_pct.is_none() {
        lines.push(fill("results.mission.fail_score", &json!({ "avg": avg, "max_score": max, "pass": m.pass.average })));
    } else {
        lines.push(fill(
            "results.mission.fail_waste",
            &json!({ "waste_pct": waste_pct, "waste_limit_pct": m.pass.waste_pct.unwrap() }),
        ));
    }
    let cause = if avg >= max {
        None
    } else if empty >= over && empty >= wrong && empty > 0 {
        Some("empty")
    } else if over >= wrong && over > 0 {
        Some("over")
    } else if wrong > 0 {
        Some("wrong")
    } else {
        None
    };
    if let Some(c) = cause {
        lines.push(fill("results.mission.cause", &json!({ "cause": fill(&format!("results.cause.{c}"), &json!({})) })));
    }
    Outcome { scores, average: avg, waste_pct, passed, cause: cause.map(str::to_string), lines, best_streak }
}

fn mid(id: &str) -> String {
    fill(&format!("flavors.{id}.name_mid"), &json!({}))
}

/// What the HUD shows: for each line, the next cup to be judged and its
/// order; then the average so far, the waste so far and the belt.
pub fn hud(w: &World) -> Value {
    let ids: Vec<String> = crate::look::flavors().into_iter().map(|f| f.id).collect();
    let two = w.lines[1].is_some();
    let mut per_line = Vec::new();
    let (mut scores, mut wasted, mut emitted) = (Vec::new(), 0, 0);
    let mut belt = 0;
    for seat in 0..2 {
        let Some(ls) = w.lines[seat].as_ref() else { continue };
        let line = w.line(seat);
        scores.extend(ls.results.iter().map(|r| r.score.score));
        wasted += ls.wasted;
        emitted += ls.emitted;
        belt = ((ls.belt_factor.0 as i64 * 100 + 2048) >> 12) as u32;
        let next = ls.cups.iter().find(|c| !c.judged);
        per_line.push(json!({
            "name": two.then(|| fill(if seat == 0 { "lines.upper.name" } else { "lines.lower.name" }, &json!({}))),
            "cup": next.map(|c| fill("hud.cup", &json!({ "n": c.order + 1, "count": ls.cups.len() }))),
            "order": next.map(|c| {
                let o = &line.orders[c.order as usize];
                let parts: Vec<(String, u32)> = o.parts.iter().map(|&(f, n)| (ids[f as usize].clone(), n)).collect();
                fill("hud.order", &json!({ "recipe": recipe(&parts) }))
            }),
        }));
    }
    json!({
        "lines": per_line,
        "score": (!scores.is_empty()).then(|| fill("hud.score", &json!({ "score": average(&scores), "max_score": sim::balance::MAX_SCORE }))),
        "waste": fill("hud.waste", &json!({ "waste_pct": waste(wasted, emitted) })),
        "belt": fill("hud.belt", &json!({ "belt_pct": belt })),
    })
}

fn streak_word() -> String {
    let v: Value = serde_json::from_str(JUDGEMENTS_JSON).expect("data/judgements.json is valid");
    v["streak"]["word"].as_str().unwrap().to_string()
}

/// What the ladder's numbers depend on: the simulation, the data that builds
/// a mission, and the pilots' version. `analysis/ladder.md` records it, and
/// the ladder test refuses a ladder made from anything else.
pub fn fingerprint(pilot_version: u32) -> String {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(JUDGEMENTS_JSON.as_bytes());
    bytes.extend_from_slice(&UPPER_LINE_RISE.to_le_bytes());
    bytes.extend_from_slice(&sim::SIM_VERSION.to_le_bytes());
    bytes.extend_from_slice(&pilot_version.to_le_bytes());
    for f in [MISSIONS_JSON, CONDITIONS_JSON, setup::CUPS_JSON, setup::LINE_JSON, setup::BLENDS_JSON, crate::look::FLAVORS_JSON] {
        bytes.extend_from_slice(f.as_bytes());
    }
    format!("{:016x}", sim::world::fnv1a(&bytes))
}

pub const JUDGEMENTS_JSON: &str = include_str!("../../../data/judgements.json");

/// The word the lid's score earns (`data/judgements.json`): the first row,
/// from the top, whose mark the score reaches. The copy key is
/// `judge.<word>`.
pub fn judgement(score: u32) -> String {
    let v: Value = serde_json::from_str(JUDGEMENTS_JSON).expect("data/judgements.json is valid");
    for row in v["judgements"].as_array().unwrap() {
        if score >= row["at"].as_u64().unwrap() as u32 {
            return row["word"].as_str().unwrap().to_string();
        }
    }
    unreachable!("the last judgement is at 0")
}

/// How many cups in a row, ending with the last one judged on `seat`'s line,
/// earned the streak's word; and whether that run has just reached a
/// multiple of the streak's length, which plays the reward (Sam,
/// 2026-10-05: "3 of the highest tier in the row").
pub fn streak(w: &World, seat: usize) -> (u32, bool) {
    let v: Value = serde_json::from_str(JUDGEMENTS_JSON).expect("data/judgements.json is valid");
    let word = v["streak"]["word"].as_str().unwrap();
    let length = v["streak"]["length"].as_u64().unwrap() as u32;
    let Some(ls) = w.lines[seat].as_ref() else { return (0, false) };
    let run = ls.results.iter().rev().take_while(|r| judgement(r.score.score) == word).count() as u32;
    (run, run > 0 && run % length == 0)
}
