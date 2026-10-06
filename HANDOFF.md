# Handoff

Written for a reader with none of this session's context, and rewritten at every deploy gate. This one is for the build after the MVP (2026-10-05): two lines, fields, and a tree of 21 missions.

## 1. What this is

Slushline is a browser game at https://sgilson7.github.io/slushline/: a belt carries cups under slush spouts, and holding a spout's key pulls its handle, which opens its valve. Everything that pours is integer physics in `crates/sim`: units of slush that fall, hold a slope, thicken, swell after they land and spill over rims, and a handle driven by a servo against a return spring, so a spout does not close the instant its key is let go. At the end of the belt a lid scores each cup by one rule (`sim::score`): a unit counts while its flavor is under its share. Twenty-one missions form a tree after Vagrancy's road (`data/missions.json`): each opens when its requirements (pass, mark, clean) are met. Some missions run two lines at once, and some carry force fields that bend falling slush; progress is a save file the player keeps, and any run downloads as a replay that plays back to the same checksum in Chromium, Firefox and WebKit. `PLANNING-BRIEF.md` is the brief, `PLAN.md` the plan; Sam's answers and the decisions since are in `DECISIONS.md`.

## 2. Load-bearing rules, and what breaks silently when each is broken

- **Integers only in `sim`, one `Rng`, no clock.** Replays stop playing back in another browser. Guards: `crates/sim/tests/boundary.rs`; the gate's native-against-wasm checksum.
- **Contacts are inelastic** (`slush::inelastic`). Without it slush splashes over cup walls and every mission's waste triples (SECOND-ORDER-M2 row 2).
- **The swell grows with the square of a unit's age, and cup walls are slick.** Either change hides the swell again (`a_cup_keeps_rising_after_the_last_unit_lands`).
- **`SIM_VERSION` goes up with any change to what the simulation does,** with the golden replay re-recorded (`cargo run -p lab -- golden`). Otherwise old replays are refused or, worse, play differently.
- **A data change to missions, cups, the line, blends or flavors makes `analysis/ladder.md` stale,** and `the_path_gets_no_easier_within_a_chapter` fails until `make ladder` (about two minutes) is run.
- **Strings come only from `data/copy.en.json`.** Guards: `crates/content/tests/copy.rs`; every gate screen checks its visible text. Key names on screen carry `data-value` so the gate reads them as values.
- **Colors come only from `data/palette.json`, and slush is drawn outlines, fills, patterns** (`web/draw.js`). Drawn unit by unit, lemon reads as cherry in gray; the gate's gray check catches it.

## 3. The shape of the code

| crate | holds |
|---|---|
| `sim` | `fx` (12-bit fixed point), `setup` (lines, spouts, rails, orders, fields), `world` (handles, valves, belt, cup walls, waste, lid), `slush` (grid, contacts, slope rule, thickness, swell, inelastic, rest), `score`, `replay`, `frame` |
| `content` | `data/*.json` into setups (`setup`), `missions` (cards, recipes, results, the HUD, the ladder's fingerprint), `save` (v1, rebinding, which missions are open), `look` (luminance, color deficiency), `copy` |
| `pilot` | idle, timer (plans each spout's pour from the order), yardstick (timer with seeded error) |
| `wasm` | the shim: `Game`, the path, the save |
| `lab` | `recon-m1`, `recon-m2`, `ladder`, `play`, `splash`, `golden`, `script-checksum`, `bench` |

The page is `web/`: `app.js` (screens, the clock, settings), `draw.js`, `keys.js`, `files.js`. The gate is `testing/drive.py`.

## 4. The commands

- `make test`, `make web`, `make serve`, `make test-ui`, `make ladder`, `make count`.
- The live gate: `ORIGIN=https://sgilson7.github.io/slushline .venv-test/bin/python testing/drive.py chromium firefox webkit`.
- To feel the tunings: `make serve`, then `?tuning=0|1|2`.

## 5. What will bite within the hour

- `make ladder` takes about two minutes on twelve cores, and any edit to the mission data needs it.
- zsh does not word-split `$var`; use `${=var}` in shell helpers.
- `cargo test` stops at the first failing binary; use `--no-fail-fast` to see every failure.
- GitHub's hosted runners have been slow to pick up jobs here; one deploy was never run ("The job was not acquired by Runner of type hosted"). Re-run it with `gh run rerun <id>`.
- The gate's gray check samples unit pixels; it depends on the three-pass drawing in `draw.js`.

## 6. Mistakes that cost time, by system

- **Solver:** a tick ten times over budget (an `isqrt` started from `n`, and `opt-level = "z"`); piles that never came to rest; slush that splashed out of cups; the swell hidden twice.
- **Line:** a belt too fast for a full pour.
- **Pilots:** waste blamed on overfilling when it was the window's margin.
- **Drawing:** swatches that passed the brightness test and a canvas that failed it.
- **Process:** break-and-watch edits that did not break what they meant to (SECOND-ORDER-M1 row 9); a Python splice that duplicated half of `world.rs`.

## 7. The single next action

Sam plays the deployed tree, the fields and the two-line missions, and files `PLAYTEST-M5.md`, with replays from the result screens in `testing/replays/`: the pace, the tail, whether the swell is visible, the three tunings, and the order of the path (SECOND-ORDER-M4 row 2).
