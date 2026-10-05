//! Save v1 (D16): each mission's best average and whether it was passed, the
//! key bindings, and the options. JSON, so a player can read their own file.
//! Refused with a sentence when it cannot be read, never half-loaded.
//!
//! The rules about a save live here, not in the page: which missions are
//! open, what a new result changes, and that two actions cannot share a key.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::BTreeMap;

pub const FORMAT: &str = "slushline.save";
pub const VERSION: u32 = 1;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Save {
    pub format: String,
    pub version: u32,
    /// Mission id to its best average.
    pub best: BTreeMap<String, u32>,
    /// Missions passed at least once.
    pub passed: Vec<String>,
    /// Action (`spout_1`…) to `KeyboardEvent.code`.
    pub keys: BTreeMap<String, String>,
    pub options: Options,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Default)]
#[serde(deny_unknown_fields)]
pub struct Options {
    /// Draw each flavor's short code on its slush in the cups (0.5).
    pub short_codes: bool,
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
        Save { format: FORMAT.into(), version: VERSION, best: BTreeMap::new(), passed: Vec::new(), keys: default_keys(), options: Options::default() }
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
        let s: Save = serde_json::from_value(v).map_err(|_| SaveError::Damaged)?;
        let ids: Vec<String> = crate::missions::missions().into_iter().map(|m| m.id).collect();
        let actions: Vec<&str> = sim::Input::ACTIONS.iter().map(|a| a.0).collect();
        let consistent = s.best.keys().all(|k| ids.contains(k))
            && s.best.values().all(|&b| b <= sim::balance::MAX_SCORE)
            && s.passed.iter().all(|k| ids.contains(k))
            && s.keys.keys().all(|k| actions.contains(&k.as_str()))
            && actions.iter().all(|a| s.keys.contains_key(*a))
            && distinct(&s.keys);
        if !consistent {
            return Err(SaveError::Damaged);
        }
        Ok(s)
    }

    /// What a finished run changes: the best average, and a pass once made.
    pub fn record(&mut self, mission: &str, average: u32, passed: bool) {
        let b = self.best.entry(mission.to_string()).or_insert(0);
        *b = (*b).max(average);
        if passed && !self.passed.iter().any(|p| p == mission) {
            self.passed.push(mission.to_string());
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

    /// A mission is open when every requirement it names has been passed.
    pub fn open(&self, mission: &crate::missions::Mission) -> bool {
        mission.requires.iter().all(|r| self.passed.contains(&r.pass))
    }
}

fn distinct(keys: &BTreeMap<String, String>) -> bool {
    let mut v: Vec<&String> = keys.values().collect();
    v.sort();
    v.windows(2).all(|w| w[0] != w[1])
}
