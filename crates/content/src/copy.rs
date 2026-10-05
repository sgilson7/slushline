//! The copy file: every string a player reads (PLANNING-BRIEF 0.9).
//!
//! The strings are implemented exactly as written. This module only walks the
//! file; it never changes a string.

use serde_json::Value;

/// The copy file, compiled in so the shim and the tests read the same bytes.
pub const COPY_JSON: &str = include_str!("../../../data/copy.en.json");

pub fn copy() -> Value {
    parsed().clone()
}

/// The copy file parsed once.
fn parsed() -> &'static Value {
    static C: std::sync::OnceLock<Value> = std::sync::OnceLock::new();
    C.get_or_init(|| serde_json::from_str(COPY_JSON).expect("data/copy.en.json is valid JSON"))
}

/// Every player-facing string as `(dotted.key, text)`, in file order.
///
/// Keys that begin with an underscore are instructions and are never shown,
/// so they are skipped at every depth.
pub fn player_strings(v: &Value) -> Vec<(String, String)> {
    let mut out = Vec::new();
    walk(v, String::new(), &mut out);
    out
}

fn walk(v: &Value, path: String, out: &mut Vec<(String, String)>) {
    match v {
        Value::Object(m) => {
            for (k, child) in m {
                // `_` keys are instructions; `review` marks a string the agent
                // wrote and Sam has not yet accepted (PLANNING-BRIEF 0.9).
                if k.starts_with('_') || k == "review" {
                    continue;
                }
                let p = if path.is_empty() { k.clone() } else { format!("{path}.{k}") };
                walk(child, p, out);
            }
        }
        Value::String(s) => out.push((path, s.clone())),
        _ => {}
    }
}

/// The placeholder names inside one string: `{a}` and `{key.b}`.
pub fn placeholders(s: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut rest = s;
    while let Some(i) = rest.find('{') {
        match rest[i..].find('}') {
            Some(j) => {
                out.push(&rest[i + 1..i + j]);
                rest = &rest[i + j + 1..];
            }
            None => break,
        }
    }
    out
}

/// A copy string by dotted key, with its `{placeholders}` filled from
/// `values` and `{game}` from `game.name`. Panics on a missing key or an
/// unfilled placeholder: a sentence half-filled is a bug, not a fallback.
pub fn fill(key: &str, values: &Value) -> String {
    let c = parsed();
    let mut v = c;
    for k in key.split('.') {
        v = &v[k];
    }
    let s = v.as_str().unwrap_or_else(|| panic!("no copy string {key}"));
    let mut out = String::new();
    let mut rest = s;
    while let Some(i) = rest.find('{') {
        out.push_str(&rest[..i]);
        let j = rest[i..].find('}').unwrap_or_else(|| panic!("an open brace in {key}"));
        let name = &rest[i + 1..i + j];
        let val = if name == "game" {
            c["game"]["name"].as_str().unwrap().to_string()
        } else {
            match values.get(name) {
                Some(Value::String(s)) => s.clone(),
                Some(other) => other.to_string(),
                None => panic!("no value for {{{name}}} in {key}"),
            }
        };
        out.push_str(&val);
        rest = &rest[i + j + 1..];
    }
    out.push_str(rest);
    out
}
