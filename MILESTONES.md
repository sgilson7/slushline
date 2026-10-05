# Milestones

One row per milestone, appended after its commit (PLANNING-BRIEF Part D). `tests` is the number from `packaging/count-tests.sh`.

| Milestone | Deliverables | Tests | Commit | State |
|---|---|---|---|---|
| M0 — Foundation | workspace, copy file, lints (boundary, copy, look, licenses), page shell, packaging, gate, Pages workflow | 25 in 9 binaries | c13690d | deploy gate 1, live and walked |
| M1 — Slush in integers | fx, rng, input, setup, world (handles, valves, slush, walls, lid), score, replay v1, frame, shim, page with the pour and replays, golden replay | 61 tests in 14 binaries | 8eb7142 | deploy gate 2 (runner never acquired; deployed with M2) |

Notebook: M0 has 5 rows (4 done, 1 open). New strings awaiting Sam: none. Universals entries added for six carried strings (DECISIONS.md).

After M1: notebook M1 has 11 rows (10 done, 1 the human's). New strings awaiting Sam: none. Reliance revisited: "many loose particles in integers" was right to be low; the first solver was slow, jittered and hid the swell, and each was caught by a test with a number in it (SECOND-ORDER-M1 rows 1 to 4).

| M2+M3 — Belt, lid, score; flavors, orders, blends | belt and lid, score and results, HUD, missions.json with eleven missions and conditions.json, blend emission, the timer pilot, save v1, settings (keys, short codes, save file), how to play, the gray check | 84 tests in 16 binaries | 6b6e5ab | deploy gates 3 and 4, merged (SECOND-ORDER-M2 row 1); its run was cancelled by M4's push and deployed with M4 |

After M2 and M3: notebook has 13 rows (11 done, 1 open, 1 the human's). New strings awaiting Sam: none. Reliance revisited: "telling flavors apart" was right to be medium: the swatches passed and the rendered page did not (SECOND-ORDER-M2 row 8).

| M4 — The path (MVP) | path with locks from the save, kc_graph.json and its sentences, the yardstick and `make ladder`, the chapter ladder test, the pilot boundary test | 87 tests in 17 binaries | 304591b, then 48fee51 (view polish) | deploy gate 5: live as build 93f5917c at 304591b, the live gate walked in Chromium, Firefox and WebKit; tagged v0.1.0-mvp at the last deployed commit |

After M4: notebook has 7 rows (4 done, 1 open, 2 the human's). New strings awaiting Sam: the `kc` and `kc_edge` sentences (30), marked `"review": "new"`. Reliance revisited: "the order of the missions" was right to be low; the ladder supports the order within chapters only (SECOND-ORDER-M4 row 2).
