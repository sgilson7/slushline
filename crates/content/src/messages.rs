//! Which sentence the player reads for an outcome, as a copy key and the
//! values for its placeholders. `content` chooses; the page fills.

use serde_json::{json, Value};
use sim::replay::ReplayError;

/// The sentence that refuses a replay file.
pub fn replay_error(e: ReplayError) -> Value {
    match e {
        ReplayError::Format => json!({ "key": "replay.error.format", "values": {} }),
        ReplayError::Sim { theirs, ours } => json!({ "key": "replay.error.sim", "values": { "theirs": theirs, "ours": ours } }),
        ReplayError::Damaged => json!({ "key": "replay.error.damaged", "values": {} }),
    }
}
