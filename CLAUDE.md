# CLAUDE.md — rules for working in this repo

Kept short on purpose. This file holds the rules and the commands. Why a thing is the way it is goes in `DECISIONS.md`; what was noticed while working goes in the milestone's notebook; what the next reader needs goes in `HANDOFF.md`. If something here is out of date, that is a bug in this file.

Read `PLANNING-BRIEF.md` first, then `PLAN.md`. `PLAN.md` wins where the two disagree, and the divergence is a notebook row.

## Non-negotiable

Break one of these and the failure is silent and expensive.

- **`crates/sim` depends on `serde` and `postcard` only.** No `f32` or `f64`, no `HashMap` or `HashSet`, no `std::time`, no `rand`, no physics crate. One `Rng`, and it lives in `World`. `crates/sim/tests/boundary.rs` enforces this, reading both the manifest and the source; if it fails, fix that before anything else.
- **The world changes only through `World::step([Input; 2])`.** The shim, the pilots and the page never mutate `World`.
- **`crates/wasm` decides nothing.** It moves bytes across the boundary. An `if` in the shim is a rule that belongs in `sim`.
- **The page draws numbers core sent it.** It may interpolate between two frames it was given. It never integrates, predicts, detects a contact, or keeps its own copy of a constant.
- **Slush holds a slope and swells, and that is the design.** Do not make it level like water, or hold units at one size, to make the fluid "right". See `PLAN.md` D6 and the tests `a_pile_of_slush_keeps_its_slope` and `a_cup_keeps_rising_after_the_last_unit_lands`.
- **A spout key pulls its handle and does nothing else; the belt keys move the belt's throttle and nothing else** (Sam, 2026-10-05, replacing the brief's "no key changes the belt"). No key adds slush to a cup or moves a cup by itself. Guards: `only_spout_bits_move_a_handle`, `only_the_belt_keys_change_the_belt`.
- **No flavor is carried by hue alone.** Every flavor has a pattern and a brightness step in `data/palette.json` and `data/flavors.json`, the page never blends two flavors' colors, and `crates/content/tests/look.rs` checks both.
- **The score is the one rule in `sim::score`.** The page shows the number core sent.
- **Every unit is in one place.** A change that adds somewhere a unit can be extends the conservation test in the same commit.
- **Every string a player reads is in `data/copy.en.json` and is used exactly as written.** A string the build needs and the file lacks is written with `TONE.md` open, marked `"review": "new"`, and listed for Sam. A string that looks wrong is reported, not repaired.
- **No third-party file enters the repo or the build without a row in `LICENSES.md`.**
- **No brand name appears in anything a player can see.** `crates/content/tests/copy.rs` holds the deny-list.
- **Changing what the simulation does means bumping `SIM_VERSION`** and re-recording the golden replays, in the same commit, and saying so in the message.
- **Adding a field to `World`, the replay or the save is a compile error until the encoding carries it.** Do not fix a destructure by adding `..`.
- **Never run `git push` or `make publish` on your own judgement.** For the MVP run Sam granted push and deploy (DECISIONS.md, 2026-10-05). It is never inferred beyond that.
- **Nothing of ours runs anywhere but GitHub Pages.** No server, no signalling service, no database, no bundler, no npm step.

## Working style

- **Recon before building.** Measure against the running build, and put what you found in the commit message. A number that disagrees with the plan names which of the two is wrong.
- **Grep before you invent.** The reference repositories in `reference/` already solve most of what is not physics.
- **Every new test is broken once and watched failing before it is kept.** Restore from a copy you took yourself, never with `git checkout` on a file that has uncommitted work in it.
- **A check about a number reads the number from where it is decided.**
- **A count comes from a command.** Test totals come from `packaging/count-tests.sh`; ladder results from `make ladder`.
- **Keep the notebook while the work happens**, in `SECOND-ORDER-M<n>.md`.
- zsh does not word-split `$var`; use `${=var}` in shell helpers.
- Test names are sentences. Comments say why. Commit subjects are sentences; bodies say why, name what was rejected, and quote the test count from the script.
- At every deploy gate, and when the context reaches about 700,000 tokens, write `HANDOFF.md`.

## Commands

- `make test` — the whole suite, native, no window and no network.
- `make check` — a fast type-check.
- `make web` / `make serve` — the browser build into `dist/web/`, and a local static server.
- `make test-ui` — walk the gate in Chromium, Firefox and WebKit. Fails on a console error or a request that leaves the origin.
- `make ladder` — play the yardstick pilot on every mission and write `analysis/ladder.md`.
- `packaging/count-tests.sh` — how many tests there are.
- Live gate: `ORIGIN=https://sgilson7.github.io/slushline .venv-test/bin/python testing/drive.py chromium firefox webkit`.
