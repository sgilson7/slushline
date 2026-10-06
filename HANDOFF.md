# Handoff

Written for a reader with none of the last session's context. This one is for v0.3.0 (2026-10-06, build `ae6c5be0` at `c523ce9`), and it ends with two tasks Sam has asked for and that are not started: a pour sound and a store background. Section 7 specifies them.

## 1. What this is

Slushline is a browser game at https://sgilson7.github.io/slushline/ (repo `sgilson7/slushline`, root `~/Documents/SlurpeeGame`). A belt carries cups under slush spouts; holding a spout's key pulls a simulated handle that opens its valve. Everything that pours is integer physics in `crates/sim`: slush units that fall, hold a slope, swell, spill and settle; handles on a servo and a spring; fields that bend the stream; cups in rows and on lifts; heavy slush that sinks; a jet nozzle. A lid scores each cup by one rule (`sim::score`; layered orders by `score_layered`), and a DDR-style word and a synthesized sound mark each judged cup. There are 33 missions in a tree after Vagrancy's road, shown as a tree or as lanes (one column per chapter, Hasse lines only), with a save file, replays, settings and a waste tray. `PLANNING-BRIEF.md` is the brief, `PLAN.md` the plan; everything Sam has asked for since is quoted and decided in `DECISIONS.md`, and each round has a notebook (`SECOND-ORDER-M0.md` to `M6.md`).

## 2. Load-bearing rules, and what breaks silently when each is broken

- **Integers only in `sim`, one `Rng`, no clock.** Replays stop playing back in another browser. Guards: `crates/sim/tests/boundary.rs`; the gate's native-against-wasm checksum.
- **`SIM_VERSION` goes up with any change to what the simulation does** (it is 6), and the golden replay is re-recorded (`cargo run -q --release -p lab -- golden`). The suite fails until you do.
- **Any change to mission, cup, line, blend, flavor or judgement data, `SIM_VERSION`, the pilot or `UPPER_LINE_RISE` makes `analysis/ladder.md` stale.** Run `make ladder` (about three minutes). `the_tree_gets_no_easier_going_down_within_a_chapter` then checks that, along every requirement within a chapter, the mission below is no easier for the yardstick. If it fails, change the data (order, belt, fields), not the test.
- **Every string a player reads is in `data/copy.en.json`,** checked against `TONE.md` by `crates/content/tests/copy.rs`. The lint refuses number words from three up, digits outside `{placeholders}`, universals (every, only, never, nothing, cannot, always…) unless `_universals` names a test, color words (including "light", "dark", "blue"), glossary synonyms (the word "nozzle" is allowed only in `conditions.jet*.name`), brand names (Slurpee, 7-Eleven, Coke…), exclamation marks and filler. New strings carry `"review": "new"`. The gate also checks every visible DOM line is a copy string; key names and numbers in the DOM carry `data-value`.
- **Colors live only in `data/palette.json`;** `package-web.sh` turns its top-level keys into CSS variables (`--paper`, `--judge-top`…). Flavors carry a fill, a dim tone and a pattern (`data/flavors.json`); `crates/content/tests/look.rs` checks brightness and pattern separation, and the gate's gray check reads the rendered canvas (slush in cups, by majority pixel) and must see every flavor at least 0.08 apart.
- **Nothing leaves the origin.** The gate fails on any request off the site: no web fonts, no CDN, no remote images or audio. Everything ships in `web/` or is drawn or synthesized by the page.
- **No third-party file without a row in `LICENSES.md`,** and no audio file at all without one (`crates/content/tests/licenses.rs`, `package-web.sh`). All sound so far is synthesized in `web/sound.js`.
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

The page is `web/`: `app.js` (screens, the clock, the tree and lanes, HUD, results, settings), `draw.js` (`Stage`: canvas sized to the page, fields, belt, tray, spouts, cups, slush in three passes, nozzles, fill gauges), `sound.js` (WebAudio synth: lid thunk, judgement phrases, the groove; a 2.2 kHz low-pass and a compressor on everything), `groove.js`, `keys.js`, `files.js`. `index.html` holds only `{{copy.key}}` tokens. The gate is `testing/drive.py`.

## 4. The commands

- `make test` (or `cargo test -q --workspace --no-fail-fast`: plain `cargo test` stops at the first failing binary), `make web`, `make serve`, `make test-ui`, `make ladder`, `make count`.
- Live gate: `ORIGIN=https://sgilson7.github.io/slushline .venv-test/bin/python testing/drive.py chromium firefox webkit`.
- To see a screen: build, then drive it with Playwright (the gate's hooks: `window.slushline.autoplay(ticks)`, `.frame()`, `.groove()`, `.judgement()`, `.unitPixels()`); a save with everything open can be put in `localStorage['slushline.save']`.

## 5. What will bite within the hour

- zsh does not word-split `$var`; use `${=var}`.
- Every new test is broken once and watched failing (CLAUDE.md). Several breaks in this project did not break what they meant to; read the red line.
- Web Audio: create the context on a user gesture (`sound.wake()` runs on clicks and any keydown), catch `resume()` and `close()` rejections (CI's headless Firefox has no sound device and logs them as console errors, which fail the gate), and close the context on `pagehide`.
- WebKit and Firefox anti-alias differently; a pixel check must not depend on one pixel.
- The upper line of a two-line mission rises 215 cm, and the canvas grows to fit; anything drawn below a belt must stay above the next line's handles.

## 6. Mistakes that cost time, by system

- **Solver:** a tick ten times over budget; piles that never rested; slush that splashed out of cups; the swell hidden three times; heavy slush that would not sink.
- **Line:** a belt too fast for a full pour; charges that reached into cups; two lines that overlapped.
- **Pilots:** waste blamed on overfilling when it was the window's margin.
- **Drawing:** swatches that passed the brightness test and a canvas that failed it; a gray check that sampled patterns and nozzles.
- **Process:** a Python splice that duplicated half of `world.rs`; my own new strings failing the tone lint (number words, "only", "every", "light", "judge").

## 7. The next actions: two tasks from Sam, not started

Sam, 2026-10-06: "now there needs to be slurpee sound when you open a nozzle, something like a gentle pshht sound, look for videos of the noise slurpee machines make when they run. also add a blurry background that looks like the inside of a 7/11 from the perspective of the worker, that is the main background image for the game canvas window."

Record both in `DECISIONS.md` with that quote, keep a notebook (`SECOND-ORDER-M7.md`), and deploy as in section 2.

### 7a. The pour sound

- **What it is:** while a spout's valve is open, a soft, airy "pshht" of a frozen-drink machine dispensing: a hiss of breathy noise with a little low gurgle and occasional soft crackle, starting with a short burst as the valve opens and fading as it closes. Gentle, not harsh. Sam's earlier complaints about sound: "too tinny … hurts the ears after a while" and "desyncs easily". Keep everything under the existing low-pass, keep it quiet next to the lid sounds, and start it the moment the handle opens.
- **Research first.** Search for descriptions and recordings of slush and frozen-beverage dispensers (the hiss of the valve, the thick pour, the freezing barrel's hum) to decide the sound's shape: its band, its attack, whether it pulses. Note what you found and its sources in the notebook. **Do not download, sample or ship any recording** (LICENSES rule; and nothing off the origin). The sound is synthesized in `web/sound.js` from noise and filters, like `pour()` in the groove, which is a fair start.
- **Driven by core, as everything else is.** The page must not guess when a valve is open. Each frame already carries every spout's `opening` (`frame.lines[i].spouts[j].opening`, raw fixed point, 4096 is fully open). Drive a continuous voice per spout from it: gain following the opening (smoothed over about 30 ms with `setTargetAtTime` to avoid clicks), a short "pshht" burst when an opening goes from 0 to above 0, a soft tail when it returns to 0. Stop every voice when a run ends, on pause and on `pagehide`. Replays should sound the same, since the frame carries the opening.
- **Volume:** honor the existing Settings slider (`options.sound_volume`).
- **Checks:** the gate must still pass with no console error in all three engines. Add a gate check that holding a spout key creates a pour voice and letting go stops it (a hook such as `window.slushline.pourVoices()` returning how many are sounding). A test cannot hear; Sam judges the sound and it goes in the notebook as a row for him.

### 7b. The store background

- **What it is:** the canvas's background, behind the belt and cups, a blurry view of a convenience store's inside as seen by the worker behind the counter: aisles of shelves with colorful packs, cooler doors glowing along a wall, fluorescent ceiling lights, a window and the door at the front, the counter edge nearest. Blurry, so it reads as depth and atmosphere and never competes with the slush.
- **No brand.** Sam called it "a 7/11", but the game uses no brand names or marks (`TONE.md` rule 4; brand deny-list in `crates/content/tests/copy.rs`). Draw a generic store: no logos, no stripes in a brand's colors, no store name, no readable text at all.
- **Made here, shipped here.** Either paint it procedurally in a new module (for example `web/backdrop.js`: rectangles, gradients and soft shapes) or generate an image file you author yourself and commit it under `web/` with a `LICENSES.md` row saying it was made for this project. No stock photo, no remote image. Colors come from `data/palette.json`: add the backdrop's colors there as new keys.
- **Blur and stay out of the way.** Paint the scene once into an offscreen canvas at a low resolution and scale it up, which blurs it; or draw it into a separate element behind a transparent stage canvas and use CSS `filter: blur()`. Do not rely on `CanvasRenderingContext2D.filter`, which WebKit may not support. Knock its contrast and saturation down, or lay a translucent paper wash over it, so that slush, cups, the fill gauges, the field chevrons and the judgement word stay as legible as now.
- **Where the canvas paints `p.paper` today:** `Stage.draw` fills the whole canvas with `pal.paper` first. Replace that with the backdrop, scaled to the canvas's current size (it changes with `Stage.fit` and with two-line missions; repaint or rescale on resize).
- **Checks:** the gray check samples slush in cups by majority pixel and should be unaffected (cups are opaque), but run the gate and confirm all four flavors still read 0.023, 0.222, 0.744 and 0.405. Look at screenshots of a one-line mission, a two-line mission and a field mission, and judge legibility yourself before deploying.

### Then

Write `HANDOFF.md` again, deploy, walk the live gate, and tell Sam what to listen and look for. His other open items are listed as "the human's" rows in `SECOND-ORDER-M2` to `M6`.
