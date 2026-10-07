//! Story mode's world tour (Sam, 2026-10-07): each mission is a store in a
//! different place, and each row of the tree is a leg of the tour, drawn as a
//! band of region art (analysis/art/regions.py, stores.py).
//!
//! The page builds each picture's path from a mission's id and its row, so a
//! mission or a row without its picture shows a broken image; this test
//! fails first.

use std::path::{Path, PathBuf};

fn root() -> PathBuf {
    Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../..")).canonicalize().unwrap()
}

#[test]
fn every_mission_has_a_store_and_every_row_of_the_tree_has_a_region() {
    let copy = content::copy::copy();
    let depths = content::missions::depths(&content::missions::missions());
    let mut problems = Vec::new();
    for (id, &row) in &depths {
        if !copy["missions"]["list"][id]["place"].is_string() {
            problems.push(format!("{id} has no place line (missions.list.{id}.place)"));
        }
        if !root().join(format!("web/art/store/{id}.png")).is_file() {
            problems.push(format!("{id} has no store picture (run analysis/art/stores.py)"));
        }
        if !copy["world"]["region"][row.to_string()]["name"].is_string() {
            problems.push(format!("row {row} has no region name (world.region.{row}.name)"));
        }
        if !root().join(format!("web/art/region-{row}.png")).is_file() {
            problems.push(format!("row {row} has no region band (run analysis/art/regions.py)"));
        }
    }
    problems.sort();
    problems.dedup();
    assert!(problems.is_empty(), "\n{}\n", problems.join("\n"));
}
