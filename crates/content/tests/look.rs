//! Every flavor can be told from every other without color (PLANNING-BRIEF 0.5).
//!
//! These are what stop the three channels decaying back into decoration: each
//! fails the moment a flavor is picked by hue alone. Shape after
//! `gear-master-2d/crates/core/tests/look.rs`.

use content::look::{fill, flavors, luminance, rgb, Deficiency, SEPARATION};

fn pairs() -> Vec<(String, String)> {
    let f = flavors();
    let mut out = Vec::new();
    for (i, a) in f.iter().enumerate() {
        for b in &f[i + 1..] {
            out.push((a.id.clone(), b.id.clone()));
        }
    }
    assert!(out.len() >= 3, "only {} pairs of flavors to check", out.len());
    out
}

#[test]
fn every_pair_of_flavors_differs_in_pattern() {
    let f = flavors();
    for (a, b) in pairs() {
        let pa = &f.iter().find(|x| x.id == a).unwrap().pattern;
        let pb = &f.iter().find(|x| x.id == b).unwrap().pattern;
        assert_ne!(pa, pb, "{a} and {b} share the pattern {pa}");
    }
}

#[test]
fn every_pair_of_flavors_differs_in_brightness() {
    for (a, b) in pairs() {
        let (la, lb) = (luminance(rgb(&fill(&a))), luminance(rgb(&fill(&b))));
        assert!(
            (la - lb).abs() > SEPARATION,
            "{a} ({la:.3}) and {b} ({lb:.3}) are only {:.3} apart in brightness; they must differ by more than {SEPARATION}",
            (la - lb).abs()
        );
    }
}

#[test]
fn every_pair_of_flavors_differs_in_brightness_under_each_color_deficiency() {
    for d in Deficiency::ALL {
        for (a, b) in pairs() {
            let (la, lb) = (d.luminance(rgb(&fill(&a))), d.luminance(rgb(&fill(&b))));
            assert!((la - lb).abs() > SEPARATION, "{d:?}: {a} ({la:.3}) and {b} ({lb:.3}) are too close");
        }
    }
}

#[test]
fn every_pattern_is_one_the_page_draws_and_the_copy_names() {
    let copy = content::copy::copy();
    for f in flavors() {
        assert!(["solid", "stripes", "dots"].contains(&f.pattern.as_str()), "{} has the pattern {}, which draw.js does not draw", f.id, f.pattern);
        assert!(copy["patterns"][&f.pattern].is_string(), "the copy file has no patterns.{}", f.pattern);
        assert!(copy["flavors"][&f.id]["short"].is_string(), "the copy file has no short code for {}", f.id);
    }
}

/// H7 — brightness is not the hex value. One channel of `#808080` is
/// 128/255 = 0.502, and ((0.502 + 0.055) / 1.055)^2.4 = 0.216. Read as light,
/// the hex value would say 0.50.
#[test]
fn h7_brightness_is_not_the_hex_value() {
    let l = luminance(rgb("#808080"));
    assert!((l - 0.216).abs() < 0.0005, "#808080 has luminance {l:.4}, and H7 says 0.216");
    assert_eq!(luminance(rgb("#000000")), 0.0);
    assert!((luminance(rgb("#FFFFFF")) - 1.0).abs() < 1e-9);
}

#[test]
fn the_brief_s_luminance_column_matches_the_palette() {
    // PLANNING-BRIEF 0.5's table, to two places, so a palette edit that
    // changes a brightness step is seen here before it is seen on the page.
    for (id, want) in [("cola", 0.02), ("cherry", 0.22), ("lemon", 0.74)] {
        let got = luminance(rgb(&fill(id)));
        assert!((got - want).abs() < 0.006, "{id}: {got:.3}, brief says {want}");
    }
}

#[test]
fn simulating_a_deficiency_changes_hue_and_keeps_gray_as_gray() {
    // A check that compared a color with itself would pass whatever the
    // matrices were; this one would fail on an identity matrix.
    let red = rgb("#D55E00");
    assert!(Deficiency::ALL.iter().any(|d| (d.luminance(red) - luminance(red)).abs() > 0.01));
    for d in Deficiency::ALL {
        let g = d.luminance(rgb("#808080"));
        assert!((g - 0.216).abs() < 0.01, "{d:?} moves a gray to {g:.3}");
    }
}

#[test]
fn each_flavor_s_dim_tone_is_grayer_than_its_fill_and_differs_from_it() {
    // The gauge's unfilled part (Sam, 2026-10-05): a grayed-down tone of the
    // flavor that fills up into the flavor itself. It has to read as "not
    // yet" beside the fill, so it is less saturated and not the same color.
    let pal = content::look::palette();
    let sat = |c: [u8; 3]| {
        let (mx, mn) = (*c.iter().max().unwrap() as f64, *c.iter().min().unwrap() as f64);
        if mx == 0.0 { 0.0 } else { (mx - mn) / mx }
    };
    for f in flavors() {
        let full = rgb(&fill(&f.id));
        let dim = rgb(pal["flavors"][&f.id]["dim"].as_str().unwrap_or_else(|| panic!("{} has no dim tone", f.id)));
        assert_ne!(full, dim, "{}", f.id);
        assert!(sat(dim) < sat(full) || sat(full) < 0.1, "{}: the dim tone is no grayer than the fill", f.id);
        assert!((luminance(full) - luminance(dim)).abs() > 0.05, "{}: the dim tone and the fill are too close in brightness to tell apart in gray", f.id);
    }
}
