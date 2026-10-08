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

/// CRC-32 as PNG uses it (ISO 3309), bit by bit: a test, not a hot path.
fn crc32(bytes: &[u8]) -> u32 {
    let mut c = 0xFFFF_FFFFu32;
    for &b in bytes {
        c ^= b as u32;
        for _ in 0..8 {
            c = if c & 1 != 0 { 0xEDB8_8320 ^ (c >> 1) } else { c >> 1 };
        }
    }
    !c
}

fn pngs(dir: &Path, out: &mut Vec<PathBuf>) {
    for e in std::fs::read_dir(dir).unwrap() {
        let p = e.unwrap().path();
        if p.is_dir() {
            pngs(&p, out);
        } else if p.extension().is_some_and(|x| x == "png") {
            out.push(p);
        }
    }
}

#[test]
fn every_picture_of_the_tour_is_a_whole_png() {
    // Firefox reports an image whose download was cut short by a navigation
    // as "corrupt or truncated", and the gate ignores that message for these
    // pictures (testing/drive.py), so this is where a picture that really is
    // cut short or damaged is caught: each chunk's checksum holds and the
    // file ends at its end marker.
    let mut files = Vec::new();
    pngs(&root().join("web/art"), &mut files);
    assert!(files.len() >= 40, "only {} pictures under web/art", files.len());
    let mut bad = Vec::new();
    for f in &files {
        let b = std::fs::read(f).unwrap();
        let name = f.strip_prefix(root()).unwrap().display().to_string();
        if b.len() < 8 || b[..8] != [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A] {
            bad.push(format!("{name}: not a PNG"));
            continue;
        }
        let mut i = 8;
        let mut ended = false;
        while i + 12 <= b.len() {
            let n = u32::from_be_bytes(b[i..i + 4].try_into().unwrap()) as usize;
            if i + 12 + n > b.len() {
                break;
            }
            let crc = u32::from_be_bytes(b[i + 8 + n..i + 12 + n].try_into().unwrap());
            if crc32(&b[i + 4..i + 8 + n]) != crc {
                bad.push(format!("{name}: a damaged {} chunk", String::from_utf8_lossy(&b[i + 4..i + 8])));
            }
            let kind = &b[i + 4..i + 8];
            i += 12 + n;
            if kind == b"IEND" {
                ended = true;
                break;
            }
        }
        if !ended || i != b.len() {
            bad.push(format!("{name}: cut short or with bytes after its end"));
        }
    }
    assert!(bad.is_empty(), "\n{}\n", bad.join("\n"));
}

#[test]
fn each_store_picture_carries_the_store_name_the_copy_file_has() {
    // Sam, 2026-10-07: each store's name is written on its scene. The name
    // is a string a player reads, so it lives in the copy file; the art
    // script records the name it painted in each picture (names.json), and
    // a name changed in the copy file without redrawing fails here.
    let copy = content::copy::copy();
    let drawn: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(root().join("web/art/store/names.json")).expect("web/art/store/names.json: run analysis/art/stores.py")).unwrap();
    let mut stale = Vec::new();
    for m in content::missions::missions() {
        let want = copy["missions"]["list"][&m.id]["store"].as_str().unwrap_or_else(|| panic!("{} has no store name (missions.list.{}.store)", m.id, m.id));
        if drawn[&m.id].as_str() != Some(want) {
            stale.push(format!("{}: the copy file says {want:?}, the picture says {:?}", m.id, drawn[&m.id]));
        }
    }
    assert!(stale.is_empty(), "\nredraw with analysis/art/stores.py:\n{}\n", stale.join("\n"));
}
