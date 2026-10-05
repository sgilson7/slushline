//! Save v1 round-trips, refuses what it cannot read, and keeps the rules
//! about progress and keys (D16).

use content::missions::missions;
use content::save::{Save, SaveError};

#[test]
fn a_save_round_trips() {
    let mut s = Save::default();
    s.record("m_first_pour", 84, true);
    s.record("m_tail", 51, false);
    s.options.short_codes = true;
    s.rebind("spout_1", "KeyA").unwrap();
    let back = Save::load(s.to_json().as_bytes()).expect("its own save loads");
    assert_eq!(back, s);
}

#[test]
fn a_file_that_is_not_a_save_is_refused() {
    assert_eq!(Save::load(b"not json"), Err(SaveError::Format));
    assert_eq!(Save::load(br#"{"format": "vagrancy.save", "version": 1}"#), Err(SaveError::Format));
    assert_eq!(Save::load(b""), Err(SaveError::Format));
}

#[test]
fn a_save_from_a_newer_version_is_refused() {
    let mut v: serde_json::Value = serde_json::from_str(&Save::default().to_json()).unwrap();
    v["version"] = 2.into();
    assert_eq!(Save::load(v.to_string().as_bytes()), Err(SaveError::Newer { theirs: 2, ours: 1 }));
}

#[test]
fn a_damaged_save_is_refused_rather_than_half_loaded() {
    let good = Save::default().to_json();
    let cut = &good.as_bytes()[..good.len() / 2];
    assert_eq!(Save::load(cut), Err(SaveError::Format), "a truncated file is not JSON at all");
    for (path, bad) in [
        ("best", serde_json::json!({"m_nowhere": 50})),
        ("best", serde_json::json!({"m_first_pour": 101})),
        ("passed", serde_json::json!(["m_nowhere"])),
        ("keys", serde_json::json!({"spout_1": "KeyD"})),
        ("keys", serde_json::json!({"spout_1": "KeyD", "spout_2": "KeyD", "spout_3": "KeyJ", "spout_4": "KeyK"})),
        ("extra", serde_json::json!(1)),
    ] {
        let mut v: serde_json::Value = serde_json::from_str(&good).unwrap();
        v[path] = bad.clone();
        assert_eq!(Save::load(v.to_string().as_bytes()), Err(SaveError::Damaged), "{path} = {bad}");
    }
}

#[test]
fn two_actions_cannot_share_a_key() {
    let mut s = Save::default();
    let taken = s.keys["spout_2"].clone();
    let err = s.rebind("spout_1", &taken).unwrap_err();
    assert_eq!(err["other"], "spout_2");
    assert_ne!(s.keys["spout_1"], taken, "a refused binding changes nothing");
    s.rebind("spout_1", "KeyQ").unwrap();
    s.rebind("spout_1", "KeyQ").expect("rebinding an action to its own key is not a conflict");
    s.reset_keys();
    assert_eq!(s, Save::default());
}

#[test]
fn a_pass_opens_the_next_mission_and_a_worse_run_keeps_the_best() {
    let ms = missions();
    let mut s = Save::default();
    assert!(s.open(&ms[0]) && !s.open(&ms[1]));
    s.record(&ms[0].id, 70, true);
    assert!(s.open(&ms[1]));
    s.record(&ms[0].id, 40, false);
    assert_eq!(s.best[&ms[0].id], 70);
    assert_eq!(s.passed, vec![ms[0].id.clone()]);
}
