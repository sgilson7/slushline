//! `data/*.json` into the things core needs, the copy file, and the look.
//!
//! This crate may use `serde_json` and floats in its tests, and `sim` may
//! not, so everything that reads a human-written file lives here.

pub mod copy;
pub mod look;
