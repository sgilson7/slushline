//! Save v2 (D16): each mission's best average, whether it was passed and its
//! lowest waste in a passing run (for the tree's requirements), the key
//! bindings, and the options. Version 1 files load, with no waste recorded. JSON, so a player can read their own file.
//! Refused with a sentence when it cannot be read, never half-loaded.
//!
//! The rules about a save live here, not in the page: which missions are
//! open, what a new result changes, and that two actions cannot share a key.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::BTreeMap;

pub const FORMAT: &str = "slushline.save";
pub const VERSION: u32 = 2;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Save {
    pub format: String,
    pub version: u32,
    /// Mission id to its best average.
    pub best: BTreeMap<String, u32>,
    /// Missions passed at least once.
    pub passed: Vec<String>,
    /// Mission id to the lowest waste, in whole percent, of a run that
    /// passed it. A version 1 save has none.
    #[serde(default)]
    pub clean: BTreeMap<String, u32>,
    /// Action (`spout_1`…) to `KeyboardEvent.code`.
    pub keys: BTreeMap<String, String>,
    pub options: Options,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Options {
    /// Draw each flavor's short code on its slush in the cups (0.5).
    pub short_codes: bool,
    /// Sound effects, 0 to 100. A save from before sound has none, and gets
    /// the default.
    #[serde(default = "default_volume")]
    pub sound_volume: u32,
    /// How a finger on a nozzle works (Sam, 2026-10-10, for the iPad): held
    /// open while the finger stays down, or opened by one tap and closed by
    /// the next. A save from before touch has none, and gets `Hold`.
    #[serde(default)]
    pub touch_mode: TouchMode,
}

/// See `Options::touch_mode`.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum TouchMode {
    #[default]
    Hold,
    Tap,
}

fn default_volume() -> u32 {
    70
}

impl Default for Options {
    fn default() -> Options {
        Options { short_codes: false, sound_volume: default_volume(), touch_mode: TouchMode::Hold }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SaveError {
    /// Not a save file of this game. `settings.save.error.format`.
    Format,
    /// A newer save version. `settings.save.error.newer`.
    Newer { theirs: u32, ours: u32 },
    /// Incomplete, damaged or inconsistent. `settings.save.error.damaged`.
    Damaged,
}

impl SaveError {
    /// The sentence that refuses the file, as a copy key and its values.
    pub fn sentence(self) -> Value {
        match self {
            SaveError::Format => json!({ "key": "settings.save.error.format", "values": {} }),
            SaveError::Newer { theirs, ours } => json!({ "key": "settings.save.error.newer", "values": { "theirs": theirs, "ours": ours } }),
            SaveError::Damaged => json!({ "key": "settings.save.error.damaged", "values": {} }),
        }
    }
}

/// The default bindings, from `data/controls.json`.
pub fn default_keys() -> BTreeMap<String, String> {
    let v: Value = serde_json::from_str(include_str!("../../../data/controls.json")).expect("data/controls.json is valid");
    v["solo"].as_object().unwrap().iter().map(|(k, c)| (k.clone(), c.as_str().unwrap().to_string())).collect()
}

impl Default for Save {
    fn default() -> Save {
        Save { format: FORMAT.into(), version: VERSION, best: BTreeMap::new(), passed: Vec::new(), clean: BTreeMap::new(), keys: default_keys(), options: Options::default() }
    }
}

impl Save {
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).expect("a save always encodes")
    }

    /// Read a save file, refusing rather than half-loading.
    pub fn load(bytes: &[u8]) -> Result<Save, SaveError> {
        let v: Value = serde_json::from_slice(bytes).map_err(|_| SaveError::Format)?;
        if v.get("format").and_then(Value::as_str) != Some(FORMAT) {
            return Err(SaveError::Format);
        }
        let version = v.get("version").and_then(Value::as_u64).ok_or(SaveError::Damaged)? as u32;
        if version > VERSION {
            return Err(SaveError::Newer { theirs: version, ours: VERSION });
        }
        let mut s: Save = serde_json::from_value(v).map_err(|_| SaveError::Damaged)?;
        // A version 1 save reads as version 2 with no waste recorded.
        s.version = VERSION;
        // A save from before the belt keys (2026-10-05) has none: they take
        // their defaults where the key is free, rather than refusing a file
        // that was right when it was written.
        // Where a default is taken (an older save had the spouts on J and
        // K, which are now the lower line's), the first free key of a short
        // list stands in.
        const SPARE: [&str; 12] = ["KeyJ", "KeyK", "KeyL", "Semicolon", "KeyU", "KeyI", "KeyO", "KeyP", "KeyM", "Comma", "Period", "Slash"];
        for (action, code) in default_keys() {
            let added = action.starts_with("belt_") || action.starts_with("lower_");
            if s.keys.contains_key(&action) || !added {
                continue;
            }
            let free = std::iter::once(code.as_str()).chain(SPARE).find(|c| !s.keys.values().any(|v| v == c));
            if let Some(c) = free {
                s.keys.insert(action, c.to_string());
            }
        }
        let ids: Vec<String> = crate::missions::missions().into_iter().map(|m| m.id).collect();
        let actions: Vec<&str> = sim::Input::ACTIONS.iter().map(|a| a.0).collect();
        let actions: Vec<String> = actions.iter().map(|a| a.to_string()).chain((1..=4).map(|i| format!("lower_{i}"))).collect();
        let actions: Vec<&str> = actions.iter().map(String::as_str).collect();
        let consistent = s.best.keys().all(|k| ids.contains(k))
            && s.best.values().all(|&b| b <= sim::balance::MAX_SCORE)
            && s.options.sound_volume <= 100
            && s.passed.iter().all(|k| ids.contains(k))
            && s.clean.keys().all(|k| s.passed.contains(k))
            && s.clean.values().all(|&w| w <= 100)
            && s.keys.keys().all(|k| actions.contains(&k.as_str()))
            && actions.iter().all(|a| s.keys.contains_key(*a))
            && distinct(&s.keys);
        if !consistent {
            return Err(SaveError::Damaged);
        }
        Ok(s)
    }

    /// What a finished run changes: the best average, a pass once made, and
    /// the lowest waste of a passing run.
    pub fn record(&mut self, mission: &str, average: u32, passed: bool, waste_pct: u32) {
        let b = self.best.entry(mission.to_string()).or_insert(0);
        *b = (*b).max(average);
        if passed {
            if !self.passed.iter().any(|p| p == mission) {
                self.passed.push(mission.to_string());
            }
            let c = self.clean.entry(mission.to_string()).or_insert(waste_pct);
            *c = (*c).min(waste_pct);
        }
    }

    /// Whether one requirement of the tree is met.
    pub fn met(&self, r: &crate::missions::Requirement) -> bool {
        use crate::missions::Requirement::*;
        match r {
            Pass(m) => self.passed.contains(m),
            Mark { mission, average } => self.best.get(mission).is_some_and(|b| b >= average),
            Clean { mission, waste_pct } => self.clean.get(mission).is_some_and(|w| w < waste_pct),
        }
    }

    /// Bind `action` to `code`. Two actions cannot share a key: the refusal
    /// names the action that already has it.
    pub fn rebind(&mut self, action: &str, code: &str) -> Result<(), Value> {
        if let Some((other, _)) = self.keys.iter().find(|(a, c)| a.as_str() != action && c.as_str() == code) {
            return Err(json!({ "key": "settings.keys.conflict", "other": other }));
        }
        self.keys.insert(action.to_string(), code.to_string());
        Ok(())
    }

    pub fn reset_keys(&mut self) {
        self.keys = default_keys();
    }

    /// A mission is open when every requirement it has is met.
    pub fn open(&self, mission: &crate::missions::Mission) -> bool {
        mission.requires.iter().all(|r| self.met(r))
    }
    /// Whether the map shows a mission yet (Sam, 2026-10-07, after
    /// Vagrancy's chart, which hides "the fights a player cannot yet see
    /// coming"): it is open, or passed, or a mission it requires is passed.
    pub fn known(&self, mission: &crate::missions::Mission) -> bool {
        self.open(mission) || self.passed.contains(&mission.id) || mission.requires.iter().any(|r| self.passed.iter().any(|p| p == r.mission()))
    }
}

fn distinct(keys: &BTreeMap<String, String>) -> bool {
    let mut v: Vec<&String> = keys.values().collect();
    v.sort();
    v.windows(2).all(|w| w[0] != w[1])
}
