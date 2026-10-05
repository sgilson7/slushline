//! Every string a player reads is checked against `TONE.md`, the brand
//! deny-list and the name rule (PLANNING-BRIEF 0.9; TONE.md).
//!
//! Each check names the TONE.md rule it implements. A rule that needs a
//! reviewer (sentence case, a list of three made for rhythm) is not pretended
//! here. What a player sees on the page is checked again by
//! `testing/drive.py`, from rendered text, so this file is not the only guard.
//! Shape after `vagrancy/crates/content/tests/copy.rs`.

use content::copy::{copy, placeholders, player_strings};
use serde_json::Value;
use std::path::{Path, PathBuf};

fn root() -> PathBuf {
    Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../..")).canonicalize().unwrap()
}

/// Lowercase words, keeping hyphens and apostrophes inside a word.
fn words(s: &str) -> Vec<String> {
    s.split(|c: char| !(c.is_alphanumeric() || c == '-' || c == '\'' || c == '’'))
        .map(|w| w.trim_matches(|c| c == '-' || c == '\'' || c == '’').to_lowercase())
        .map(|w| w.strip_suffix("'s").or_else(|| w.strip_suffix("’s")).map(str::to_string).unwrap_or(w))
        .filter(|w| !w.is_empty())
        .collect()
}

/// A phrase as a whole-word match, case-insensitive.
fn has_phrase(s: &str, phrase: &str) -> bool {
    let w = words(s).join(" ");
    let p = words(phrase).join(" ");
    format!(" {w} ").contains(&format!(" {p} "))
}

/// The string with every `{placeholder}` removed: what a person typed.
fn typed(s: &str) -> String {
    let mut out = String::new();
    let mut depth = 0;
    for c in s.chars() {
        match c {
            '{' => depth += 1,
            '}' => depth -= 1,
            _ if depth == 0 => out.push(c),
            _ => {}
        }
    }
    out
}

// TONE.md, "Word lists for the lint".
const OPENERS: &[&str] = &["you can", "you should", "try to", "remember to", "it is important to", "i think"];
/// Rule 4. The draft list from TONE.md (Part I question 2: Sam chose generic
/// names). A match anywhere in `data/` or `web/` fails, in any case.
pub const BRANDS: &[&str] = &["slurpee", "icee", "slush puppie", "coke", "coca-cola", "pepsi", "7-eleven"];
const UNIVERSALS: &[&str] = &["every", "always", "never", "only", "cannot", "nothing", "impossible", "guarantees", "instantly"];
const PROMISES: &[&str] = &["will", "the trick is", "the way to"];
const GUIDE_WORDS: &[&str] = &[
    "delve", "tapestry", "crucially", "robust", "leverage", "underscore", "landscape", "realm", "deep dive",
    "key takeaway", "not only",
];
const PRAISE: &[&str] = &[
    "perfect", "awesome", "amazing", "epic", "ultimate", "insane", "legendary", "delicious", "great job", "nice work",
];
const FILLER: &[&str] = &["basically", "essentially", "just", "simply", "really", "actually"];
const CONTRAST: &[&str] = &["not just", "more than just", "isn't just", "rather than"];
const COLOR_WORDS: &[&str] = &[
    "red", "orange", "yellow", "green", "blue", "purple", "pink", "brown", "black", "white", "gray", "dark", "light",
];
// Rule 5: no digit, and no number word from three up. "One" and "two" stay.
const NUMBER_WORDS: &[&str] = &[
    "three", "four", "five", "six", "seven", "eight", "nine", "ten", "eleven", "twelve", "thirteen", "fourteen",
    "fifteen", "sixteen", "seventeen", "eighteen", "nineteen", "twenty", "thirty", "forty", "fifty", "sixty",
    "seventy", "eighty", "ninety", "hundred", "thousand", "million", "dozen",
];
const BRITISH: &[&str] = &[
    "colour", "colours", "favour", "behaviour", "centre", "metre", "organise", "recognise", "travelling", "grey",
    "flavour", "flavours",
];
/// Rule 3: strings for two players, where "you" is two people.
const TWO_PLAYER_PREFIXES: &[&str] = &["results.versus.", "local.", "lines."];
const YOU: &[&str] = &["you", "your", "yours", "yourself"];
/// Rule 6: strings that give advice, where a promise does not belong.
fn is_advice(key: &str) -> bool {
    (key.starts_with("missions.list.") && key.ends_with(".try")) || (key.starts_with("how.") && key.ends_with(".body"))
}
/// Glossary exceptions, each with its reason. `conditions.jet.*` is "the one
/// place the word nozzle appears" (Game text, Missions).
const GLOSSARY_ALLOWED: &[(&str, &str)] = &[("conditions.jet.name", "nozzle")];

/// Every TONE.md failure in one string, as sentences naming the rule.
fn tone_failures(key: &str, s: &str, universals: &Value) -> Vec<String> {
    let t = typed(s);
    let w = words(&t);
    let mut f = Vec::new();
    // Rule 1: the string, and each sentence in it, opens with what to do.
    for sentence in t.split(['.', '?']).map(str::trim).filter(|x| !x.is_empty()) {
        let lower = sentence.to_lowercase();
        for o in OPENERS {
            if lower.starts_with(o) && lower[o.len()..].starts_with(' ') {
                f.push(format!("rule 1: opens with `{o}`"));
            }
        }
    }
    if TWO_PLAYER_PREFIXES.iter().any(|p| key.starts_with(p)) {
        for y in YOU {
            if w.iter().any(|x| x == y) {
                f.push(format!("rule 3: `{y}` in a two-player string; use the lines' names"));
            }
        }
    }
    for b in BRANDS {
        if has_phrase(&t, b) {
            f.push(format!("rule 4: brand name `{b}`"));
        }
    }
    for n in NUMBER_WORDS {
        if w.iter().any(|x| x == n) {
            f.push(format!("rule 5: number word `{n}`; fill it from core"));
        }
    }
    if t.chars().any(|c| c.is_ascii_digit()) {
        f.push("rule 5: a digit outside a placeholder; fill it from core".into());
    }
    for u in UNIVERSALS {
        if has_phrase(&t, u) && universals.get(key).is_none() {
            f.push(format!("rule 6: `{u}` with no test named in _universals"));
        }
    }
    if is_advice(key) {
        for p in PROMISES {
            if has_phrase(&t, p) {
                f.push(format!("rule 6: advice promises with `{p}`"));
            }
        }
    }
    for list in [GUIDE_WORDS, PRAISE, FILLER, CONTRAST] {
        for x in list {
            if has_phrase(&t, x) {
                f.push(format!("rule 9: `{x}`"));
            }
        }
    }
    if t.contains('!') {
        f.push("rule 9: an exclamation mark".into());
    }
    for c in COLOR_WORDS {
        if w.iter().any(|x| x == c) {
            f.push(format!("rule 10: color word `{c}`"));
        }
    }
    for b in BRITISH {
        if w.iter().any(|y| y == b) {
            f.push(format!("mechanics: British spelling `{b}`"));
        }
    }
    if (t.contains('…') || t.contains("...")) && key != "game.loading" {
        f.push("mechanics: an ellipsis outside a loading line".into());
    }
    f
}

fn glossary_failures(key: &str, s: &str, glossary: &Value) -> Vec<String> {
    let t = typed(s);
    let mut f = Vec::new();
    if let Some(m) = glossary.as_object() {
        for (term, entry) in m {
            for syn in entry["not"].as_array().into_iter().flatten().filter_map(|v| v.as_str()) {
                if has_phrase(&t, syn) && !GLOSSARY_ALLOWED.contains(&(key, syn)) {
                    f.push(format!("rule 4: `{syn}` renames the glossary term `{term}`"));
                }
            }
        }
    }
    f
}

#[test]
fn no_string_breaks_the_tone_file() {
    let c = copy();
    let strings = player_strings(&c);
    assert!(strings.len() > 120, "only {} strings found; is the file being walked?", strings.len());
    let mut problems = Vec::new();
    for (k, s) in &strings {
        for why in tone_failures(k, s, &c["_universals"]).into_iter().chain(glossary_failures(k, s, &c["_glossary"])) {
            problems.push(format!("{k}: {why}\n    \"{s}\""));
        }
    }
    assert!(problems.is_empty(), "\n{}\n", problems.join("\n"));
}

#[test]
fn the_tone_lint_catches_each_rule() {
    let none = Value::Null;
    let caught = |k: &str, s: &str| !tone_failures(k, s, &none).is_empty();
    assert!(caught("x", "You can pull the handle."), "rule 1");
    assert!(caught("x", "Pull it. Try to stop early."), "rule 1, second sentence");
    assert!(caught("local.x", "You pour first."), "rule 3");
    assert!(!caught("missions.x", "You passed."), "rule 3 is for two-player strings");
    assert!(caught("x", "Pour a Slurpee."), "rule 4");
    assert!(caught("x", "Fill three cups."), "rule 5, a word");
    assert!(caught("x", "Fill 3 cups."), "rule 5, a digit");
    assert!(!caught("x", "Fill {count} cups."), "a placeholder is not a typed number");
    assert!(caught("x", "Every cup scores."), "rule 6");
    assert!(caught("how.x.body", "Pouring early will help."), "rule 6, a promise");
    assert!(caught("x", "A perfect pour."), "rule 9, praise");
    assert!(caught("x", "Just pour."), "rule 9, filler");
    assert!(caught("x", "Pour now!"), "rule 9, exclamation");
    assert!(caught("x", "Pour the dark one."), "rule 10");
    assert!(caught("x", "Loading..."), "an ellipsis");
    let g = copy();
    assert!(!glossary_failures("x", "Fill the glass.", &g["_glossary"]).is_empty(), "rule 4, a glossary synonym");
}

#[test]
fn every_universal_names_a_test_that_exists() {
    // A universal is allowed only beside the test that establishes it, so the
    // test must be real: its name is searched for in the workspace's tests.
    let c = copy();
    let mut tests = String::new();
    for crate_dir in std::fs::read_dir(root().join("crates")).unwrap() {
        let dir = crate_dir.unwrap().path();
        for sub in ["tests", "src"] {
            let mut files = Vec::new();
            rs_files(&dir.join(sub), &mut files);
            for f in files {
                tests.push_str(&std::fs::read_to_string(f).unwrap());
            }
        }
    }
    let mut missing = Vec::new();
    for (key, entry) in c["_universals"].as_object().unwrap() {
        let name = entry["test"].as_str().unwrap();
        if !tests.contains(&format!("fn {name}(")) {
            missing.push(format!("{key}: `{name}`"));
        }
    }
    // Carried as SECOND-ORDER rows until the milestone that writes the test.
    // Each name leaves this list in the commit that writes its test.
    let pending: &[&str] = &[
        "every_order_can_reach_full_marks_from_its_line", // M3
        "two_actions_cannot_share_a_key",                 // M3
        "a_damaged_save_is_refused_rather_than_half_loaded", // M4
        "a_file_that_is_not_a_save_is_refused",           // M4
        "a_save_from_a_newer_version_is_refused",         // M4
    ];
    missing.retain(|m| !pending.iter().any(|p| m.contains(p)));
    assert!(missing.is_empty(), "\nthese universals name a test that does not exist yet:\n{}\n", missing.join("\n"));
}

fn rs_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(rd) = std::fs::read_dir(dir) else { return };
    for e in rd {
        let p = e.unwrap().path();
        if p.is_dir() {
            rs_files(&p, out);
        } else if p.extension().is_some_and(|e| e == "rs") {
            out.push(p);
        }
    }
}

#[test]
fn every_placeholder_is_one_the_copy_file_explains() {
    let c = copy();
    let explained: Vec<String> = c["_placeholders"]
        .as_object()
        .unwrap()
        .keys()
        .flat_map(|k| k.split(',').map(|p| p.trim().to_string()).collect::<Vec<_>>())
        .collect();
    let mut problems = Vec::new();
    for (k, s) in player_strings(&c) {
        for p in placeholders(&s) {
            if !explained.iter().any(|e| e == p) {
                problems.push(format!("{k}: {{{p}}} is not in _placeholders"));
            }
        }
    }
    assert!(problems.is_empty(), "\n{}\n", problems.join("\n"));
}

fn text_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(rd) = std::fs::read_dir(dir) else { return };
    let mut v: Vec<_> = rd.map(|e| e.unwrap().path()).collect();
    v.sort();
    for p in v {
        let name = p.file_name().unwrap().to_string_lossy().to_string();
        if p.is_dir() {
            if name != "vendor" {
                text_files(&p, out);
            }
        } else if ["html", "js", "css", "json", "md", "txt"].iter().any(|e| name.ends_with(&format!(".{e}"))) {
            out.push(p);
        }
    }
}

#[test]
fn no_string_names_a_brand() {
    let root = root();
    let mut files = Vec::new();
    text_files(&root.join("data"), &mut files);
    text_files(&root.join("web"), &mut files);
    assert!(files.len() >= 5, "found {} files to scan", files.len());
    let mut problems = Vec::new();
    for f in &files {
        let text = std::fs::read_to_string(f).unwrap();
        for b in BRANDS {
            if has_phrase(&text, b) {
                problems.push(format!("{}: `{b}`", f.strip_prefix(&root).unwrap().display()));
            }
        }
    }
    assert!(problems.is_empty(), "\na brand name appears in:\n{}\n", problems.join("\n"));
}

#[test]
fn the_brand_check_sees_a_brand_inside_a_sentence() {
    // The scan above would pass over an empty directory; this one cannot.
    assert!(has_phrase("pour a Slurpee now", "slurpee"));
    assert!(has_phrase("Cherry Coke.", "coke"));
    assert!(!has_phrase("cola", "coke"));
}

#[test]
fn the_game_name_is_spelled_only_in_game_name() {
    let c = copy();
    let name = c["game"]["name"].as_str().unwrap();
    let holders: Vec<String> = player_strings(&c).into_iter().filter(|(_, s)| s.contains(name)).map(|(k, _)| k).collect();
    assert_eq!(holders, vec!["game.name".to_string()], "the name is spelled in {holders:?}");
    let mut files = Vec::new();
    text_files(&root().join("web"), &mut files);
    text_files(&root().join("data"), &mut files);
    for f in files {
        if f.ends_with("copy.en.json") {
            continue;
        }
        let text = std::fs::read_to_string(&f).unwrap();
        assert!(!text.contains(name), "{} spells the game's name; use {{game}}", f.display());
    }
}

#[test]
fn every_copy_key_the_page_asks_for_exists() {
    // The page reads strings by key through `t('…')`. A key that does not
    // exist would show the key itself to a player.
    let c = copy();
    let keys: Vec<String> = player_strings(&c).into_iter().map(|(k, _)| k).collect();
    let mut problems = Vec::new();
    let mut seen = 0;
    for entry in std::fs::read_dir(root().join("web")).unwrap() {
        let p = entry.unwrap().path();
        if p.extension().is_none_or(|e| e != "js") {
            continue;
        }
        let src = std::fs::read_to_string(&p).unwrap();
        let mut rest = src.as_str();
        while let Some(i) = rest.find("t('") {
            let after = &rest[i + 3..];
            let end = after.find('\'').unwrap();
            let key = &after[..end];
            let boundary = i == 0 || !rest.as_bytes()[i - 1].is_ascii_alphanumeric();
            if boundary && !key.contains('$') && !key.ends_with('.') {
                seen += 1;
                if !keys.iter().any(|k| k == key) {
                    problems.push(format!("{}: t('{key}')", p.file_name().unwrap().to_string_lossy()));
                }
            }
            rest = &after[end..];
        }
    }
    assert!(problems.is_empty(), "\nthe page asks for keys the copy file lacks:\n{}\n", problems.join("\n"));
    let _ = seen;
}

#[test]
fn every_string_a_player_reads_is_in_the_copy_file() {
    // The page's scripts may not carry prose of their own. A sentence in a
    // string literal — a capital letter, words, a full stop — is text the page
    // wrote itself. The gate checks the same rule against rendered text.
    let mut problems = Vec::new();
    for entry in std::fs::read_dir(root().join("web")).unwrap() {
        let p = entry.unwrap().path();
        if p.extension().is_none_or(|e| e != "js") {
            continue;
        }
        let src = strip_js_comments(&std::fs::read_to_string(&p).unwrap());
        for lit in js_string_literals(&src) {
            let words: Vec<&str> = lit.split_whitespace().collect();
            let sentence = words.len() >= 3
                && lit.chars().next().is_some_and(|c| c.is_ascii_uppercase())
                && words.iter().all(|w| w.chars().all(|c| c.is_alphabetic() || ",.'’?".contains(c)));
            if sentence {
                problems.push(format!("{}: \"{lit}\"", p.file_name().unwrap().to_string_lossy()));
            }
        }
    }
    assert!(problems.is_empty(), "\nthe page writes these sentences itself:\n{}\n", problems.join("\n"));
}

fn strip_js_comments(s: &str) -> String {
    let mut out = String::new();
    for line in s.lines() {
        let t = line.trim_start();
        if t.starts_with("//") || t.starts_with('*') || t.starts_with("/*") {
            continue;
        }
        out.push_str(line);
        out.push('\n');
    }
    out
}

fn js_string_literals(s: &str) -> Vec<String> {
    let b: Vec<char> = s.chars().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < b.len() {
        let q = b[i];
        if q == '\'' || q == '"' || q == '`' {
            let mut j = i + 1;
            let mut lit = String::new();
            while j < b.len() && b[j] != q {
                if b[j] == '\\' {
                    j += 1;
                }
                if j < b.len() {
                    lit.push(b[j]);
                }
                j += 1;
            }
            out.push(lit);
            i = j + 1;
        } else {
            i += 1;
        }
    }
    out
}

#[test]
fn the_page_prose_check_catches_a_sentence() {
    let lits = js_string_literals("const a = 'Pour into the cup.'; const b = \"bold 14px serif\";");
    assert_eq!(lits.len(), 2);
    let flagged = |lit: &str| {
        let words: Vec<&str> = lit.split_whitespace().collect();
        words.len() >= 3
            && lit.chars().next().is_some_and(|c| c.is_ascii_uppercase())
            && words.iter().all(|w| w.chars().all(|c| c.is_alphabetic() || ",.'’?".contains(c)))
    };
    assert!(flagged(&lits[0]));
    assert!(!flagged(&lits[1]));
}

#[test]
fn index_html_carries_no_text_of_its_own() {
    // Every word in the entry page is a `{{key}}` token that packaging fills
    // from the copy file, so the page cannot hold a second copy of a string.
    let html = std::fs::read_to_string(root().join("web/index.html")).unwrap();
    let c = copy();
    let keys: Vec<String> = player_strings(&c).into_iter().map(|(k, _)| k).collect();
    let mut body = html.clone();
    for (open, close) in [("<script", "</script>"), ("<style", "</style>"), ("<!--", "-->")] {
        body = remove_spans(&body, open, close);
    }
    let mut text = String::new();
    let mut attrs = Vec::new();
    let mut in_tag = false;
    let mut tag = String::new();
    for ch in body.chars() {
        match ch {
            '<' => {
                in_tag = true;
                tag.clear();
            }
            '>' if in_tag => {
                in_tag = false;
                for a in ["aria-label=\"", "title=\"", "alt=\"", "placeholder=\""] {
                    if let Some(i) = tag.find(a) {
                        let v = &tag[i + a.len()..];
                        attrs.push(v[..v.find('"').unwrap_or(v.len())].to_string());
                    }
                }
                text.push(' ');
            }
            _ if in_tag => tag.push(ch),
            _ => text.push(ch),
        }
    }
    let mut problems = Vec::new();
    for t in text.split_whitespace().chain(attrs.iter().map(|s| s.as_str())) {
        match t.strip_prefix("{{").and_then(|t| t.strip_suffix("}}")) {
            Some(k) if keys.iter().any(|x| x == k) => {}
            Some(k) => problems.push(format!("{{{{{k}}}}} is not a key in the copy file")),
            None => problems.push(format!("`{t}` is text the page wrote itself")),
        }
    }
    assert!(problems.is_empty(), "\n{}\n", problems.join("\n"));
}

fn remove_spans(s: &str, open: &str, close: &str) -> String {
    let mut out = String::new();
    let mut rest = s;
    while let Some(i) = rest.find(open) {
        out.push_str(&rest[..i]);
        rest = match rest[i..].find(close) {
            Some(j) => &rest[i + j + close.len()..],
            None => "",
        };
    }
    out.push_str(rest);
    out
}
