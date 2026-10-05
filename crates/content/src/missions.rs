//! The missions (PLANNING-BRIEF 0.6; D12), the conditions that change a
//! mission's setup (0.7; D13), and the result a mission ends with.
//!
//! The shape is Vagrancy's `data/tutorial.json` (`id`, `chapter`, `teaches`,
//! `builds_on`, `requires`) plus the line, the orders, the conditions and the
//! pass mark. `requires` is a list of requirement objects, as in Vagrancy's
//! `data/road.json`, so the path can become a tree later without a change of
//! format; in the MVP a lint holds it to a chain.

use crate::copy::fill;
use crate::setup::{self, flavor};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sim::score::{average, Cause};
use sim::setup::{Order, Setup};
use sim::World;

pub const MISSIONS_JSON: &str = include_str!("../../../data/missions.json");
pub const CONDITIONS_JSON: &str = include_str!("../../../data/conditions.json");

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Requirement {
    pub pass: String,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct LineSpec {
    pub belt: String,
    pub cup: String,
    pub spouts: Vec<String>,
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
pub struct Mission {
    pub id: String,
    pub chapter: String,
    pub teaches: Vec<String>,
    pub builds_on: Vec<String>,
    pub requires: Vec<Requirement>,
    pub line: LineSpec,
    pub orders: Vec<OrderSpec>,
    pub conditions: Vec<String>,
    pub pass: Pass,
}

#[derive(Deserialize)]
struct File {
    missions: Vec<Mission>,
    held: Vec<Mission>,
}

/// The path, in order.
pub fn missions() -> Vec<Mission> {
    serde_json::from_str::<File>(MISSIONS_JSON).expect("data/missions.json is valid").missions
}

/// Missions designed and not yet built (mission 12, after the MVP).
pub fn held() -> Vec<Mission> {
    serde_json::from_str::<File>(MISSIONS_JSON).expect("data/missions.json is valid").held
}

pub fn mission(id: &str) -> Option<Mission> {
    missions().into_iter().find(|m| m.id == id)
}

pub fn conditions() -> Value {
    serde_json::from_str(CONDITIONS_JSON).expect("data/conditions.json is valid")
}

impl Mission {
    /// One order per cup, in the order the cups come.
    pub fn cup_orders(&self) -> Vec<Order> {
        self.orders.iter().flat_map(|o| std::iter::repeat_n(o.order(), o.count as usize)).collect()
    }

    pub fn cup_count(&self) -> u32 {
        self.orders.iter().map(|o| o.count).sum()
    }

    /// The world this mission is played in, its conditions applied.
    pub fn setup(&self, seed: u64, tuning: u8) -> Setup {
        let spouts: Vec<&str> = self.line.spouts.iter().map(String::as_str).collect();
        let mut line = setup::line(&spouts, &self.line.cup, &self.line.belt, self.cup_orders());
        let conds = conditions();
        for c in &self.conditions {
            let def = &conds[c];
            assert!(def.get("held").is_none(), "{} uses the held condition {c}", self.id);
            match c.as_str() {
                "rail" => {
                    let spout = def["spout"].as_u64().unwrap() as usize;
                    line = setup::with_rail(line, spout, def["amplitude"].as_i64().unwrap() as i32, def["period"].as_u64().unwrap() as u32);
                }
                other => panic!("no code applies the condition {other}"),
            }
        }
        setup::setup_of(seed, tuning, line)
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

/// What the mission's card says: its name, its order line and its pass line.
/// The thing to try is the page's to fill, because it names keys.
pub fn card(m: &Mission) -> Value {
    let order = if m.orders.len() == 1 {
        fill("missions.order.same", &json!({ "count": m.cup_count(), "recipe": recipe(&m.orders[0].parts()) }))
    } else {
        fill("missions.order.mixed", &json!({ "count": m.cup_count() }))
    };
    let max = sim::balance::MAX_SCORE;
    let pass = match m.pass.waste_pct {
        Some(w) => fill("missions.pass", &json!({ "pass": m.pass.average, "max_score": max, "waste_limit_pct": w })),
        None => fill("missions.pass_no_limit", &json!({ "pass": m.pass.average, "max_score": max })),
    };
    let conditions: Vec<Value> = m
        .conditions
        .iter()
        .map(|c| json!({ "name": fill(&format!("conditions.{c}.name"), &json!({})), "desc": fill(&format!("conditions.{c}.desc"), &json!({})) }))
        .collect();
    json!({
        "id": m.id,
        "name": fill(&format!("missions.list.{}.name", m.id), &json!({})),
        "order": order,
        "pass": pass,
        "try_key": format!("missions.list.{}.try", m.id),
        "conditions": conditions,
        "cups": m.cup_count(),
        "spouts": m.line.spouts.len(),
    })
}

/// The labels of a line's spouts, in belt order (`spout.single`, `spout.blend`).
pub fn spout_labels(m: &Mission) -> Vec<String> {
    let look = crate::look::flavors();
    m.line
        .spouts
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
}

/// The waste share in whole percent, floored; and whether it is under a
/// limit, decided exactly (`wasted · 100 < limit · poured`).
pub fn waste(wasted: u32, emitted: u32) -> u32 {
    if emitted == 0 {
        0
    } else {
        (wasted as u64 * 100 / emitted as u64) as u32
    }
}

pub fn outcome(m: &Mission, w: &World) -> Outcome {
    let ls = w.lines[0].as_ref().expect("a mission has a line");
    let mut results = ls.results.clone();
    results.sort_by_key(|r| r.cup);
    let scores: Vec<u32> = results.iter().map(|r| r.score.score).collect();
    let avg = average(&scores);
    let waste_pct = waste(ls.wasted, ls.emitted);
    let under = m.pass.waste_pct.is_none_or(|lim| (ls.wasted as u64) * 100 < lim as u64 * ls.emitted.max(1) as u64);
    let passed = avg >= m.pass.average && under && scores.len() as u32 == m.cup_count();
    let ids: Vec<String> = crate::look::flavors().into_iter().map(|f| f.id).collect();
    let max = sim::balance::MAX_SCORE;
    let mut lines = Vec::new();
    let (mut empty, mut over, mut wrong) = (0u32, 0u32, 0u32);
    for r in &results {
        let s = &r.score;
        lines.push(fill("results.cup.line", &json!({ "n": r.cup + 1, "score": s.score, "max_score": max })));
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
    Outcome { scores, average: avg, waste_pct, passed, cause: cause.map(str::to_string), lines }
}

fn mid(id: &str) -> String {
    fill(&format!("flavors.{id}.name_mid"), &json!({}))
}

/// What the HUD shows: the next cup to be judged, its order, the average so
/// far, and the waste so far. Numbers and filled sentences; the page adds
/// the keys.
pub fn hud(w: &World) -> Value {
    let Some(ls) = w.lines[0].as_ref() else { return json!(null) };
    let line = w.line(0);
    let ids: Vec<String> = crate::look::flavors().into_iter().map(|f| f.id).collect();
    let next = ls.cups.iter().find(|c| !c.judged);
    let scores: Vec<u32> = ls.results.iter().map(|r| r.score.score).collect();
    let order = next.map(|c| {
        let o = &line.orders[c.order as usize];
        let parts: Vec<(String, u32)> = o.parts.iter().map(|&(f, n)| (ids[f as usize].clone(), n)).collect();
        fill("hud.order", &json!({ "recipe": recipe(&parts) }))
    });
    json!({
        "cup": next.map(|c| fill("hud.cup", &json!({ "n": c.order + 1, "count": ls.cups.len() }))),
        "order": order,
        "score": (!scores.is_empty()).then(|| fill("hud.score", &json!({ "score": average(&scores), "max_score": sim::balance::MAX_SCORE }))),
        "waste": fill("hud.waste", &json!({ "waste_pct": waste(ls.wasted, ls.emitted) })),
    })
}
