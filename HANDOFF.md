# Handoff

Written for a reader with none of the last session's context. This one is for v0.4.0 (2026-10-06, build `3bf187c1`, live gate walked in three engines): a pour sound and a blurred store behind the line (`SECOND-ORDER-M7`), then flower-pot cups (SIM_VERSION 7), the fix for a lag the store caused, and drawing between ticks (`SECOND-ORDER-M8`). Nothing Sam has asked for is unstarted; section 7 lists what waits on him.

## 1. What this is

Slushline is a browser game at https://sgilson7.github.io/slushline/ (repo `sgilson7/slushline`, root `~/Documents/SlurpeeGame`). A belt carries cups under slush spouts; holding a spout's key pulls a simulated handle that opens its valve. Everything that pours is integer physics in `crates/sim`: slush units that fall, hold a slope, swell, spill and settle; handles on a servo and a spring; fields that bend the stream; cups in rows and on lifts; heavy slush that sinks; a jet nozzle. A lid scores each cup by one rule (`sim::score`; layered orders by `score_layered`), and a DDR-style word and a synthesized sound mark each judged cup. An open valve hisses, and behind the line is a blurred, generic shop seen from behind its counter. There are 33 missions in a tree after Vagrancy's road, shown as a tree or as lanes (one column per chapter, Hasse lines only), with a save file, replays, settings and a waste tray. `PLANNING-BRIEF.md` is the brief, `PLAN.md` the plan; everything Sam has asked for since is quoted and decided in `DECISIONS.md`, and each round has a notebook (`SECOND-ORDER-M0.md` to `M8.md`).

## 2. Load-bearing rules, and what breaks silently when each is broken

- **Integers only in `sim`, one `Rng`, no clock.** Replays stop playing back in another browser. Guards: `crates/sim/tests/boundary.rs`; the gate's native-against-wasm checksum.
- **`SIM_VERSION` goes up with any change to what the simulation does** (it is 7), and the golden replay is re-recorded (`cargo run -q --release -p lab -- golden`). The suite fails until you do.
- **Any change to mission, cup, line, blend, flavor or judgement data, `SIM_VERSION`, the pilot or `UPPER_LINE_RISE` makes `analysis/ladder.md` stale.** Run `make ladder` (about three minutes). `the_tree_gets_no_easier_going_down_within_a_chapter` then checks that, along every requirement within a chapter, the mission below is no easier for the yardstick. If it fails, change the data, not the test. The cup's size moves the yardstick most; a belt's speed, a bob and an uneven order barely move it (M8 row 4).
- **Every string a player reads is in `data/copy.en.json`,** checked against `TONE.md` by `crates/content/tests/copy.rs`. The lint refuses number words from three up, digits outside `{placeholders}`, universals (every, only, never, nothing, cannot, always…) unless `_universals` names a test, color words (including "light", "dark", "blue"), glossary synonyms (the word "nozzle" is allowed only in `conditions.jet*.name`), brand names (Slurpee, 7-Eleven, Coke…), exclamation marks and filler. New strings carry `"review": "new"`. The gate also checks every visible DOM line is a copy string; key names and numbers in the DOM carry `data-value`.
- **Colors live only in `data/palette.json`;** `package-web.sh` turns its top-level keys into CSS variables (`--paper`, `--judge-top`…). Flavors carry a fill, a dim tone and a pattern (`data/flavors.json`); `crates/content/tests/look.rs` checks brightness and pattern separation, and the gate's gray check reads the rendered canvas (slush in cups, by majority pixel) and must see every flavor at least 0.08 apart.
- **Nothing leaves the origin.** The gate fails on any request off the site: no web fonts, no CDN, no remote images or audio. Everything ships in `web/` or is drawn or synthesized by the page.
- **No third-party file without a row in `LICENSES.md`,** and no audio file at all without one (`crates/content/tests/licenses.rs`, `package-web.sh`). All sound so far is synthesized in `web/sound.js`, and the store behind the line is painted by `web/backdrop.js`; no image or audio file ships.
- **The page decides nothing about the game.** It draws numbers core sends (`frame`, `hud`, `path_json`), may lay them out, and performs sounds for events core reports.
- **Sam has granted push and deploy rights for this run.** Every deploy: `make test`, `make web`, `make test-ui`, commit, push, wait for the Actions run (re-run a job with `gh run rerun <id> --failed` when GitHub reports "not acquired by Runner"), then walk the live gate (section 4).

## 3. The shape of the code

| crate | holds |
|---|---|
| `sim` | `fx`, `setup` (lines, spouts, rails, nozzles, fields, rows, bob, orders, flavor physics), `world` (handles, valves and the jet, belt and throttle, per-cup walls, waste tray, lid), `slush` (grid, mass-weighted contacts, slope rule, thickness, quadratic swell, inelastic, rest), `score`, `replay`, `frame` |
| `content` | `setup` (data into lines), `missions` (the tree, requirements, conditions, cards, recipes, outcomes, HUD, judgement and streak, Hasse lines, depths, ladder fingerprint), `save` (v2), `look`, `copy`, `messages` |
| `pilot` | idle, timer (plans each spout's pour, steps a particle through fields to find the landing), yardstick |
| `wasm` | the shim: `Game` (step, frame, hud, outcome, take_judged, autoplay), the path, chapters, the save |
| `lab` | `recon-m1`, `recon-m2`, `ladder`, `play`, `fields`, `splash`, `golden`, `script-checksum`, `bench` |

The page is `web/`: `app.js` (screens, the clock, the tree and lanes, HUD, results, settings), `draw.js` (`Stage`: canvas sized to the page, fields, belt, tray, spouts, cups, slush in three passes, nozzles, fill gauges), `sound.js` (WebAudio synth: lid thunk, judgement phrases, the groove, and a pour voice per spout that follows each frame's `opening`; a 2.2 kHz low-pass and a compressor on everything), `backdrop.js` (the store: painted small, box-blurred, scaled up and washed in paper, once per canvas size; colors in `palette.json` `backdrop.*`), `groove.js`, `keys.js`, `files.js`, `blend.js` (draws between the last two ticks: every screen refresh paints at the share of a tick the clock has run; the canvas is sized from the newest real frame). `index.html` holds only `{{copy.key}}` tokens. The gate is `testing/drive.py`.

## 4. The commands

- `make test` (or `cargo test -q --workspace --no-fail-fast`: plain `cargo test` stops at the first failing binary), `make web`, `make serve`, `make test-ui`, `make ladder`, `make count`.
- Live gate: `ORIGIN=https://sgilson7.github.io/slushline .venv-test/bin/python testing/drive.py chromium firefox webkit`.
- To see a screen: build, then drive it with Playwright (the gate's hooks: `window.slushline.autoplay(ticks)`, `.frame()`, `.groove()`, `.judgement()`, `.unitPixels()`, `.pourVoices()`); a save with everything open can be put in `localStorage['slushline.save']`.

## 5. What will bite within the hour

- zsh does not word-split `$var`; use `${=var}`.
- Every new test is broken once and watched failing (CLAUDE.md). Several breaks in this project did not break what they meant to; read the red line.
- Web Audio: create the context on a user gesture (`sound.wake()` runs on clicks and any keydown), catch `resume()` and `close()` rejections (CI's headless Firefox has no sound device and logs them as console errors, which fail the gate), and close the context on `pagehide`.
- WebKit and Firefox anti-alias differently; a pixel check must not depend on one pixel.
- The canvas's height must not follow anything that moves each frame: a resize repaints the store (M8 row 1).
- The upper line of a two-line mission rises 215 cm, and the canvas grows to fit; anything drawn below a belt must stay above the next line's handles.

## 6. Mistakes that cost time, by system

- **Solver:** a tick ten times over budget; piles that never rested; slush that splashed out of cups; the swell hidden three times; heavy slush that would not sink.
- **Line:** a belt too fast for a full pour; charges that reached into cups; two lines that overlapped.
- **Pilots:** waste blamed on overfilling when it was the window's margin.
- **Drawing:** swatches that passed the brightness test and a canvas that failed it; a gray check that sampled patterns and nozzles.
- **Process:** a Python splice that duplicated half of `world.rs`; my own new strings failing the tone lint (number words, "only", "every", "light", "judge").

## 7. What waits on Sam

The pour sound and the store are built and deployed (DECISIONS.md, 2026-10-06; notebook `SECOND-ORDER-M7.md`). What a test cannot judge:

- **The pour** (`web/sound.js`, `pourFrame`; level `POUR_GAIN`): a psst as a valve opens, then a breathy hiss over a low wobbling body, fading as it closes. Whether it reads as a slush machine and is gentle enough. The gate checks only that holding a key starts a voice and letting go stops it.
- **The store** (`web/backdrop.js`; `WASH`, `SMALL`; colors `backdrop.*` in `palette.json`). On two-line missions it is stretched tall (M7 row 6).
- **WebKit crashed once on CI** after the first push of this round and passed on a re-run (M7 row 9). If it happens again, suspect the pour voices first.
- **Whether the stutter on opening a spout is gone** (M8 row 10). It was never reproduced here; if it is still there, ask which browser and screen.
- **The flower-pot cups** (M8 row 7): whether they read as slush cups, and whether Layers, now on a small cup, is too hard.
- His other open items are the "the human's" rows in `SECOND-ORDER-M2` to `M8`.
