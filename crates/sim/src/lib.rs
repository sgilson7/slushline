//! The line.
//!
//! Everything that pours — the handles, the valves, the slush, the belt and
//! the cups — is integer arithmetic in this crate, and the world changes only
//! through `World::step([Input; 2])`. No floats, no clock, no `HashMap`, one
//! random stream, and two dependencies (`serde`, `postcard`).
//! `tests/boundary.rs` holds all of that, because the way these rules get
//! broken is not a decision to break them.
#![forbid(unsafe_code)]
#![deny(clippy::float_arithmetic)]

/// Bumped whenever what the simulation does changes, together with the
/// golden replays, in the same commit (CLAUDE.md). A replay from another
/// version is refused with a sentence.
pub const SIM_VERSION: u32 = 1;
