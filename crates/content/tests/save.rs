//! Save v1 round-trips, refuses what it cannot read, and keeps the rules
//! about progress and keys (D16).

use content::missions::missions;
use content::save::{Save, SaveError};

#[test]
fn a_save_round_trips() {
    let mut s = Save::default();
    s.record("m_first_pour", 84, true, 9);
    s.record("m_tail", 51, false, 40);
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
    v["version"] = 3.into();
    assert_eq!(Save::load(v.to_string().as_bytes()), Err(SaveError::Newer { theirs: 3, ours: 2 }));
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
    let tail = ms.iter().find(|m| m.id == "m_tail").unwrap();
    let mut s = Save::default();
    assert!(s.open(&ms[0]) && !s.open(tail));
    s.record(&ms[0].id, 70, true, 12);
    assert!(s.open(tail));
    s.record(&ms[0].id, 40, false, 50);
    assert_eq!(s.best[&ms[0].id], 70);
    assert_eq!(s.passed, vec![ms[0].id.clone()]);
}

#[test]
fn a_save_from_before_the_belt_keys_loads_with_their_defaults() {
    // The MVP's first deploy wrote saves with four spout keys and no belt.
    let old = r#"{"format": "slushline.save", "version": 1, "best": {"m_first_pour": 80},
        "passed": ["m_first_pour"], "options": {"short_codes": false},
        "keys": {"spout_1": "KeyD", "spout_2": "KeyF", "spout_3": "KeyJ", "spout_4": "KeyK"}}"#;
    let s = Save::load(old.as_bytes()).expect("an old save still loads");
    assert_eq!(s.keys["spout_1"], "KeyD", "its own bindings are kept");
    assert_eq!(s.keys["belt_slower"], "ArrowLeft");
    assert_eq!(s.keys["belt_faster"], "ArrowRight");
    assert_eq!((s.keys["lower_1"].as_str(), s.keys["lower_2"].as_str()), ("KeyL", "Semicolon"), "J and K are taken, so the next free keys stand in");
}

#[test]
fn each_kind_of_requirement_is_met_by_what_it_asks_and_nothing_less() {
    use content::missions::Requirement::*;
    let mut s = Save::default();
    let mark = Mark { mission: "m_first_pour".into(), average: 75 };
    let clean = Clean { mission: "m_tail".into(), waste_pct: 15 };
    s.record("m_first_pour", 74, true, 5);
    assert!(s.met(&Pass("m_first_pour".into())) && !s.met(&mark), "74 is not a mark of 75");
    s.record("m_first_pour", 75, false, 50);
    assert!(s.met(&mark), "a failed run's average still counts toward a mark");
    s.record("m_tail", 90, false, 3);
    assert!(!s.met(&clean), "a clean run that did not pass does not count");
    s.record("m_tail", 80, true, 15);
    assert!(!s.met(&clean), "15 percent is not under 15");
    s.record("m_tail", 70, true, 14);
    assert!(s.met(&clean));
}

#[test]
fn a_version_1_save_loads_as_version_2() {
    let old = r#"{"format": "slushline.save", "version": 1, "best": {"m_first_pour": 80},
        "passed": ["m_first_pour"], "options": {"short_codes": false},
        "keys": {"spout_1": "KeyA", "spout_2": "KeyS", "spout_3": "KeyD", "spout_4": "KeyF", "belt_slower": "ArrowLeft", "belt_faster": "ArrowRight"}}"#;
    let s = Save::load(old.as_bytes()).expect("a version 1 save loads");
    assert_eq!(s.version, 2);
    assert!(s.clean.is_empty());
    assert_eq!(s.keys["lower_1"], "KeyJ", "the lower line's keys take their defaults");
}

#[test]
fn the_map_shows_a_mission_once_a_mission_it_requires_is_passed() {
    // Sam, 2026-10-07: clean up the map after Vagrancy's chart, which shows a
    // fight only once it is open, won, or a prerequisite is won.
    let ms = content::missions::missions();
    let mut save = content::save::Save::default();
    let known = |s: &content::save::Save| ms.iter().filter(|m| s.known(m)).map(|m| m.id.clone()).collect::<Vec<_>>();
    assert_eq!(known(&save), vec!["m_first_pour".to_string()]);
    save.record("m_first_pour", 70, true, 10);
    let after = known(&save);
    for id in ["m_first_pour", "m_tail", "m_two_spouts", "m_push"] {
        assert!(after.iter().any(|k| k == id), "{id} is not shown after passing the first mission: {after:?}");
    }
    // A mission two steps on stays hidden.
    assert!(!after.iter().any(|k| k == "m_half"), "{after:?}");
    // Each passed mission is shown, and so is each open one.
    for m in &ms {
        assert!(!(save.open(m) || save.passed.contains(&m.id)) || save.known(m), "{}", m.id);
    }
}

#[test]
fn a_save_from_before_touch_loads_holding_and_keeps_a_tap_choice() {
    // Sam, 2026-10-10: on the iPad a nozzle opens while held, or one tap
    // opens and the next closes. A save written before touch has no
    // choice and loads as holding; a save that chose tapping keeps it.
    let mut v: serde_json::Value = serde_json::from_str(&Save::default().to_json()).unwrap();
    v["options"].as_object_mut().unwrap().remove("touch_mode");
    let old = Save::load(v.to_string().as_bytes()).expect("a save from before touch loads");
    assert_eq!(old.options.touch_mode, content::save::TouchMode::Hold);
    let mut tap = Save::default();
    tap.options.touch_mode = content::save::TouchMode::Tap;
    let back = Save::load(tap.to_json().as_bytes()).unwrap();
    assert_eq!(back.options.touch_mode, content::save::TouchMode::Tap);
    let written: serde_json::Value = serde_json::from_str(&tap.to_json()).unwrap();
    assert_eq!(written["options"]["touch_mode"], "tap", "the file names the choice");
}
