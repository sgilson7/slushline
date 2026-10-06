//! How a flavor reads, before anything draws it (PLANNING-BRIEF 0.5).
//!
//! Gear Master 2D's rule for its board (`gear-master-2d/crates/core/src/look.rs:17-25`),
//! applied to slush: a flavor is carried on three channels, and any two of
//! them can be lost.
//!
//! | channel | survives |
//! |---|---|
//! | a pattern on every unit, spout label and order bar | no color at all |
//! | a brightness step in the fill | no color at all |
//! | an Okabe-Ito hue in the same fill | for players who see it |
//!
//! Brightness here is **relative luminance**, computed from the hex value
//! through the sRGB curve (H7), never the hex value read as a lightness.
//! The arithmetic is floating point and lives in `content`, never in `sim`.

use serde::Deserialize;

/// The least luminance two flavors may differ by: Gear Master 2D's
/// `ROLE_SEPARATION` (`look.rs:131`). There it separates consecutive role
/// steps; the brief applies it to every pair of flavors, which is stricter.
pub const SEPARATION: f64 = 0.08;

pub const FLAVORS_JSON: &str = include_str!("../../../data/flavors.json");
pub const PALETTE_JSON: &str = include_str!("../../../data/palette.json");

#[derive(Deserialize, Debug, Clone)]
pub struct FlavorDef {
    pub id: String,
    pub pattern: String,
    #[serde(default = "one")]
    pub mass: u8,
    #[serde(default = "hundred")]
    pub gravity_pct: u16,
}

fn one() -> u8 {
    1
}
fn hundred() -> u16 {
    100
}

#[derive(Deserialize)]
struct FlavorsFile {
    flavors: Vec<FlavorDef>,
}

/// The flavors, in the order that numbers them in `sim`.
pub fn flavors() -> Vec<FlavorDef> {
    serde_json::from_str::<FlavorsFile>(FLAVORS_JSON).expect("data/flavors.json is valid").flavors
}

pub fn palette() -> serde_json::Value {
    serde_json::from_str(PALETTE_JSON).expect("data/palette.json is valid JSON")
}

/// A flavor's fill, from the palette.
pub fn fill(id: &str) -> String {
    palette()["flavors"][id]["fill"].as_str().unwrap_or_else(|| panic!("palette.json has no fill for {id}")).to_string()
}

/// `#RRGGBB` as three bytes.
pub fn rgb(hex: &str) -> [u8; 3] {
    let h = hex.trim_start_matches('#');
    assert_eq!(h.len(), 6, "{hex} is not #RRGGBB");
    let b = |i: usize| u8::from_str_radix(&h[i..i + 2], 16).unwrap_or_else(|_| panic!("{hex} is not hex"));
    [b(0), b(2), b(4)]
}

/// One sRGB channel, 0 to 255, as linear light.
pub fn linear(c: u8) -> f64 {
    let c = c as f64 / 255.0;
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

/// Relative luminance (WCAG 2): 0 for black, 1 for white. H7: `#808080` is
/// 0.216, not 0.50.
pub fn luminance(rgb: [u8; 3]) -> f64 {
    let [r, g, b] = rgb.map(linear);
    0.2126 * r + 0.7152 * g + 0.0722 * b
}

/// The luminance of a color as a player with a color deficiency sees it.
///
/// Machado, Oliveira and Fernandes (2009), severity 1.0, applied to linear
/// RGB. The matrices are quoted from memory and not yet checked against the
/// paper; `SECOND-ORDER-M0.md` row 3 carries that.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Deficiency {
    Protanopia,
    Deuteranopia,
    Tritanopia,
}

impl Deficiency {
    pub const ALL: [Deficiency; 3] = [Deficiency::Protanopia, Deficiency::Deuteranopia, Deficiency::Tritanopia];

    fn matrix(self) -> [[f64; 3]; 3] {
        match self {
            Deficiency::Protanopia => [
                [0.152286, 1.052583, -0.204868],
                [0.114503, 0.786281, 0.099216],
                [-0.003882, -0.048116, 1.051998],
            ],
            Deficiency::Deuteranopia => [
                [0.367322, 0.860646, -0.227968],
                [0.280085, 0.672501, 0.047413],
                [-0.011820, 0.042940, 0.968881],
            ],
            Deficiency::Tritanopia => [
                [1.255528, -0.076749, -0.178779],
                [-0.078411, 0.930809, 0.147602],
                [0.004733, 0.691367, 0.303900],
            ],
        }
    }

    /// Relative luminance after simulation, with each channel clamped to 0..1.
    pub fn luminance(self, rgb: [u8; 3]) -> f64 {
        let lin = rgb.map(linear);
        let m = self.matrix();
        let ch = |row: [f64; 3]| (row[0] * lin[0] + row[1] * lin[1] + row[2] * lin[2]).clamp(0.0, 1.0);
        0.2126 * ch(m[0]) + 0.7152 * ch(m[1]) + 0.0722 * ch(m[2])
    }
}
