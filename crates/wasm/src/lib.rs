//! The shim. It moves bytes across the boundary and decides nothing: an `if`
//! here is a rule that belongs in `sim`, where the test suite can reach it.

use wasm_bindgen::prelude::*;

/// The copy file, shipped inside the module so the strings and the build that
/// uses them cannot drift apart.
#[wasm_bindgen]
pub fn copy_json() -> String {
    content::copy::COPY_JSON.to_string()
}

#[wasm_bindgen]
pub fn sim_version() -> u32 {
    sim::SIM_VERSION
}
