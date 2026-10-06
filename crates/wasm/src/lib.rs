//! The shim. It moves bytes across the boundary and decides nothing: an `if`
//! here is a rule that belongs in `sim` or `content`, where the test suite
//! can reach it.

use serde_json::json;
use sim::replay::{self, Playback, Recording};
use sim::{balance, frame, fx, Input};
use wasm_bindgen::prelude::*;

/// The copy file, shipped inside the module so the strings and the build that
/// uses them cannot drift apart.
#[wasm_bindgen]
pub fn copy_json() -> String {
    content::copy::COPY_JSON.to_string()
}

#[wasm_bindgen]
pub fn palette_json() -> String {
    content::look::PALETTE_JSON.to_string()
}

#[wasm_bindgen]
pub fn flavors_json() -> String {
    content::look::FLAVORS_JSON.to_string()
}

#[wasm_bindgen]
pub fn controls_json() -> String {
    include_str!("../../../data/controls.json").to_string()
}

/// Every constant the page needs, from where it is decided.
#[wasm_bindgen]
pub fn numbers() -> String {
    json!({
        "ticks_per_second": balance::TICKS_PER_SECOND,
        "frac_bits": fx::FRAC_BITS,
        "sim_version": sim::SIM_VERSION,
        "actions": Input::ACTIONS.iter().map(|(n, b)| json!([n, b])).collect::<Vec<_>>(),
        "tunings": balance::TUNINGS.len(),
        "default_tuning": balance::DEFAULT_TUNING,
        "max_score": balance::MAX_SCORE,
        "r_full": balance::R_FULL.0,
    })
    .to_string()
}

/// The fixed script's checksum after `ticks`, computed in the browser. The
/// gate compares it with `lab script-checksum`, the same code run natively.
#[wasm_bindgen]
pub fn script_checksum(ticks: u32) -> String {
    replay::script_checksum_of(content::setup::standing(2026, balance::DEFAULT_TUNING), ticks)
}

/// Every mission on the path, as its card says it, with its spouts' labels,
/// and whether the save given opens it.
#[wasm_bindgen]
pub fn path_json(save_json: &str) -> String {
    let save = content::save::Save::load(save_json.as_bytes()).unwrap_or_default();
    let ms = content::missions::missions();
    let list: Vec<serde_json::Value> = ms
        .iter()
        .enumerate()
        .map(|(i, m)| {
            let mut card = content::missions::card(m);
            card["spout_labels"] = json!(content::missions::spout_labels(m));
            card["open"] = json!(save.open(m));
            card["passed"] = json!(save.passed.contains(&m.id));
            card["best"] = json!(save.best.get(&m.id));
            card["locked_by"] = json!(m.requires.first().map(|r| content::copy::fill(&format!("missions.list.{}.name", r.pass), &json!({}))));
            card["next"] = json!(ms.get(i + 1).map(|n| n.id.clone()));
            card
        })
        .collect();
    serde_json::to_string(&list).unwrap()
}

/// How long slush swells under a tuning, in seconds to one place, for
/// `how.swell.body`'s `{swell_s}`.
#[wasm_bindgen]
pub fn swell_seconds(tuning: u8) -> String {
    let t = balance::tuning(tuning).swell_ticks as u32 * 10 / balance::TICKS_PER_SECOND;
    format!("{}.{}", t / 10, t % 10)
}

// --- the save file: every rule is in content::save ------------------------

#[wasm_bindgen]
pub fn save_default() -> String {
    content::save::Save::default().to_json()
}

/// A save file's bytes as the save's JSON, or the sentence that refuses it.
#[wasm_bindgen]
pub fn save_load(bytes: &[u8]) -> Result<String, String> {
    content::save::Save::load(bytes).map(|s| s.to_json()).map_err(|e| e.sentence().to_string())
}

#[wasm_bindgen]
pub fn save_record(save_json: &str, mission: &str, average: u32, passed: bool) -> String {
    let mut s = content::save::Save::load(save_json.as_bytes()).unwrap_or_default();
    s.record(mission, average, passed);
    s.to_json()
}

/// The save with `action` bound to `code`, or the refusal naming the action
/// that already has the key.
#[wasm_bindgen]
pub fn save_rebind(save_json: &str, action: &str, code: &str) -> Result<String, String> {
    let mut s = content::save::Save::load(save_json.as_bytes()).unwrap_or_default();
    s.rebind(action, code).map(|_| s.to_json()).map_err(|e| e.to_string())
}

#[wasm_bindgen]
pub fn save_reset_keys(save_json: &str) -> String {
    let mut s = content::save::Save::load(save_json.as_bytes()).unwrap_or_default();
    s.reset_keys();
    s.to_json()
}

#[wasm_bindgen]
pub fn save_set_volume(save_json: &str, volume: u32) -> String {
    let mut s = content::save::Save::load(save_json.as_bytes()).unwrap_or_default();
    s.options.sound_volume = volume.min(100);
    s.to_json()
}

#[wasm_bindgen]
pub fn save_set_short_codes(save_json: &str, on: bool) -> String {
    let mut s = content::save::Save::load(save_json.as_bytes()).unwrap_or_default();
    s.options.short_codes = on;
    s.to_json()
}

/// A run being played and recorded, or a replay being watched.
#[wasm_bindgen]
pub struct Game {
    rec: Option<Recording>,
    play: Option<Playback>,
    /// The mission it plays, for its result; empty for the pour.
    mission: String,
    /// The timer pilot, made the first time the gate asks it to play.
    pilot: Option<pilot::Pilot>,
    /// Lids that closed since the page last asked, so a lid is heard and
    /// seen even when several ticks pass between two drawn frames.
    judged: Vec<serde_json::Value>,
}

#[wasm_bindgen]
impl Game {
    /// One spout over one standing cup (M1's pour).
    pub fn standing(seed: u32, tuning: u8) -> Game {
        Game { rec: Some(Recording::new(content::setup::standing(seed as u64, tuning))), play: None, mission: String::new(), pilot: None, judged: Vec::new() }
    }
    /// A mission from the path, or `None` if there is no such mission.
    pub fn mission(id: &str, seed: u32, tuning: u8) -> Option<Game> {
        let m = content::missions::mission(id)?;
        Some(Game { rec: Some(Recording::new(m.setup(seed as u64, tuning))), play: None, mission: id.to_string(), pilot: None, judged: Vec::new() })
    }
    /// A replay file, or the copy key and values of the sentence that refuses it.
    pub fn load_replay(bytes: &[u8]) -> Result<Game, String> {
        replay::load(bytes)
            .map(|r| {
                // A replay names no mission; content finds the one whose
                // setup it carries, so its result can be shown.
                let mission = content::missions::missions()
                    .into_iter()
                    .find(|m| m.setup(r.setup.seed, r.setup.tuning) == r.setup)
                    .map(|m| m.id)
                    .unwrap_or_default();
                Game { rec: None, play: Some(Playback::new(r)), mission, pilot: None, judged: Vec::new() }
            })
            .map_err(|e| content::messages::replay_error(e).to_string())
    }
    pub fn step(&mut self, a: u16, b: u16) {
        match (&mut self.rec, &mut self.play) {
            (Some(r), _) => r.step([Input(a), Input(b)]),
            (_, Some(p)) => {
                p.step();
            }
            _ => {}
        }
        self.collect();
    }
    fn collect(&mut self) {
        let events = self.world().events.clone();
        for e in events {
            if let sim::world::Event::Judged { cup, score, .. } = e {
                self.judged.push(json!({ "cup": cup, "score": score, "word": content::missions::judgement(score) }));
            }
        }
    }
    /// The lids that closed since the last call: cup, score and the word
    /// its score earns.
    pub fn take_judged(&mut self) -> String {
        serde_json::to_string(&std::mem::take(&mut self.judged)).unwrap()
    }
    fn world(&self) -> &sim::World {
        match (&self.rec, &self.play) {
            (Some(r), _) => &r.world,
            (_, Some(p)) => &p.world,
            _ => unreachable!("a Game is always one or the other"),
        }
    }
    pub fn frame(&self) -> String {
        serde_json::to_string(&frame::frame(self.world())).unwrap()
    }
    /// Every unit: id, x, y, radius, flavor + 256 × line.
    pub fn units(&self) -> Vec<i32> {
        frame::units_flat(self.world())
    }
    pub fn tick(&self) -> u32 {
        self.world().tick
    }
    pub fn checksum(&self) -> String {
        format!("{:016x}", self.world().checksum())
    }
    pub fn is_replay(&self) -> bool {
        self.play.is_some()
    }
    /// A replay that has played to its end.
    pub fn replay_done(&self) -> bool {
        self.play.as_ref().is_some_and(|p| p.done())
    }
    pub fn replay_bytes(&self) -> Vec<u8> {
        match (&self.rec, &self.play) {
            (Some(r), _) => r.bytes(),
            (_, Some(p)) => replay::encode(&p.replay),
            _ => Vec::new(),
        }
    }
    /// Up to `ticks` steps with the timer pilot's input, as a player's
    /// would be recorded. For the gate, which plays a mission to its result.
    pub fn autoplay(&mut self, ticks: u32) {
        let Some(r) = self.rec.as_mut() else { return };
        let p = self.pilot.get_or_insert_with(|| pilot::Pilot::new(pilot::Kind::Timer));
        for _ in 0..ticks {
            if r.world.done() {
                break;
            }
            let i = p.input(&r.world, 0);
            r.step([i, Input::NONE]);
            for e in r.world.events.clone() {
                if let sim::world::Event::Judged { cup, score, .. } = e {
                    self.judged.push(json!({ "cup": cup, "score": score, "word": content::missions::judgement(score) }));
                }
            }
        }
    }
    pub fn mission_id(&self) -> String {
        self.mission.clone()
    }
    /// Every cup has been judged.
    pub fn done(&self) -> bool {
        self.world().done()
    }
    /// The HUD's numbers, as content works them out.
    pub fn hud(&self) -> String {
        content::missions::hud(self.world()).to_string()
    }
    /// The result of a finished mission: its numbers and its sentences.
    pub fn outcome(&self) -> String {
        match content::missions::mission(&self.mission) {
            Some(m) => serde_json::to_string(&content::missions::outcome(&m, self.world())).unwrap(),
            None => "null".into(),
        }
    }
    /// The checksum the replay recorded, for a watcher to compare at the end.
    pub fn recorded_checksum(&self) -> String {
        self.play.as_ref().map(|p| format!("{:016x}", p.replay.checksum)).unwrap_or_default()
    }
}
