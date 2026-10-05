# Slushline: the plan

Written 2026-10-05 by the agent, from `PLANNING-BRIEF.md` (Part A), after reading the seven repositories in `reference/` at the brief's commits (`vagrancy` `bb500e0`, `floodline` `e18601b`, `gear-master` `1deaa69`, `gear-master-2d` `23a135e`, `pdf-redactor` `6a7a065`, `perturbation-workbench` `3cc83a6`, `sgilson7.github.io` `31fdd17`; each checked with `git -C reference/<repo> log -1 --format=%h`).

**How this plan was approved.** The brief asks the agent to stop after this file. Sam answered Part I questions 1 to 5 before the plan was written, then said: "dont stop working until the MVP is deployed on my github pages, you have free reign to create a github repo, with pages and deployments all allowed". That instruction replaces Part B's stop, Part G's no-push rule for this run, and `CLAUDE.md`'s "do not start a milestone before Sam has seen the previous gate". Every check that needs Sam's hands is carried as a notebook row with status *the human's*, as Vagrancy did after its approval (`vagrancy/DECISIONS.md:202`). His answers are in §8 and `DECISIONS.md`.

**The MVP is gates 1 to 5** (Q5): missions 1 to 11, the rail, a save file, replays. The second line, two players and online play move to after the MVP, with mission 12. The seams that carry them (`lines: [Option<Line>; 2]`, `World::step([Input; 2])`) stay in `Setup` from M1, so adding them later does not reshape the world.

## 1. Reliance, calibrated

`A0 Calibrate your reliance`. The agent's column is how much it trusts its own first answer, and why. Revisited at the end of each milestone in `MILESTONES.md`.

| Part of the game | How common | Filled in from | Decides | Agent's trust in its first answer |
| --- | --- | --- | --- | --- |
| Workspace, shim, packaging, CI, the gate | Common; Vagrancy's code exists | none | The page prints its build hash | High. A port, and the toolchain matches Vagrancy's lockfile pin (`wasm-bindgen 0.2.127` installed and pinned) |
| Two-seat lockstep | A port | Rollback | 10,000 ticks, equal checksums | Not built in the MVP (Q5). The seam is checked by `two_worlds_fed_the_same_inputs_agree_for_ten_thousand_ticks` |
| Many loose particles in integers | Rare | `f32`, kernels, `HashMap` grid | H1, H5 | Low. Nothing in the house. Every rule gets a test with a number before it is trusted |
| **Slush holds a slope** | Perturbed | Water that levels | `a_pile_of_slush_keeps_its_slope` | Low. The first draft will level; the test is written first and watched failing against a frictionless solver |
| **Slush swells** | Perturbed | Fixed-size particles | `a_cup_keeps_rising_after_the_last_unit_lands` | Medium. The rule is simple; the risk is that the solver hides it |
| A unit meeting a thin wall | Rare; the house has paid for it | End-of-tick test | `no_unit_passes_through_a_cup_wall` | Medium. Walls are axis-aligned, so the sweep can be exact (D9) |
| A cup that moves while slush falls | Looks common | A standing cup | H2 | Medium. H2 is a test on the real integrator, not on a formula |
| **One rule for fill and shares** | Perturbed | Averaging two scores | H3 | High once H3 exists; the rule is one line |
| **A blend counts as its parts** | Perturbed | A blend as its own flavor | H4, H6 | High. There is no blend flavor in `sim` at all: a blend spout emits single-flavor units |
| Telling flavors apart | Looks common | Hues; green and red; hex as brightness | H7 | Medium. H7 pins the luminance function; the gray check runs on rendered pixels |
| The order of the missions | Rare | Intuition | The ladder | Low. `make ladder` measures it; Sam's play outranks it |
| Strings | Common, wrong register | Marketing voice | The `TONE.md` lints | High for the given strings, which are copied exactly. Low for any new one, which is marked `"review": "new"` |
| Whether it is fun | Not judgeable by the agent | | Sam plays | None |

## 2. Decisions D1 to D18

**D1 Crates.** `sim`, `content`, `pilot`, `wasm`, `lab`, started from Vagrancy's and stripped of fighting. `net` is not created in the MVP: with two seats out (Q5), a crate with no caller would be code nobody runs, and Vagrancy's `net` is ported whole when two seats come back. *Rejected:* porting `net` now "for later", which would put an untested crate in the test count.

**D2 Drawing.** Canvas 2D from vanilla ES modules (`web/app.js`, `draw.js`, `keys.js`, `files.js`). The page draws the frame core sent and may interpolate between the last two; it never integrates, predicts or tests a contact. The order bar's markers, the HUD numbers and the result sentences' values all come in the frame or the result payload. *Rejected:* WebGL, which buys nothing for a few hundred circles.

**D3 Numbers.** Vagrancy's `fx.rs` as it stands: `i32` with 12 fractional bits, one unit a centimeter, every product through `i64`, rounding toward zero, `fx::narrow` panics, overflow checks on in release. Recon M1.0 checks 12 bits against units 1 to 2 cm across: a radius of 1.5 cm is 6,144 raw, and the smallest correction that matters (a tenth of a millimeter) is 41 raw, so 12 bits leave room (by hand; confirmed in M1 by the slope and settle tests).

**D4 Time.** 60 ticks a second, fixed. The page owns the clock (`requestAnimationFrame` with an accumulator); `sim` has none.

**D5 Slush.** Each unit is a Verlet particle (`p`, `q`, radius, flavor, age). Contacts between units are `Min` pair shifts with `len = r_a + r_b`, equal masses, so each unit moves half. Neighbors come from a uniform grid of cell size 2 × the full radius, held as a `Vec` of cell starts built by counting sort over unit index each pass, visited cell by cell and then by index, so every machine visits pairs in one order. A fixed number of passes per tick (`SOLVER_PASSES`, measured in M1.0). *Rejected:* a smoothed-particle method with kernels, which needs a `sqrt` per pair and floats in every tutorial it would be filled in from.

**D6 What makes it slush.** Three rules, each a number in `sim::balance::Tuning`, three tunings behind `?tuning=0|1|2`:
- *Slope:* friction in the contact projection. After a pair is pushed apart, the tangential part of their relative travel this tick is cancelled entirely if it is under `static_k` × the overlap, and reduced by `kinetic_k` × the overlap otherwise. A unit resting on others therefore stays put until the push along the surface passes the threshold. This is position-based friction after Macklin and others (2014), in integers; the citation is still from memory and is marked so in `LICENSES.md`'s neighbour, `DECISIONS.md`.
- *Thickness:* after integration, each unit's velocity moves `thick_k` of the way toward the mean velocity of the units touching it at the end of the last tick.
- *Swell:* a unit's radius grows linearly from `r_birth` to `r_full` over `swell_ticks`.
*Rejected:* a cohesion force (makes clumps fly), and a single global viscosity (levels like honey).

**D7 Handle and valve.** As 0.3, answered by Q4. The handle is one stick from an anchored pivot to a tip particle. Holding the key runs Vagrancy's servo (`world.rs:421-458`: accelerate up to `handle_accel` until the handle turns at `handle_speed`); a return spring always pulls the tip toward closed with a strength from the tuning. Two stops hold the handle between closed and full travel. The valve's opening is the handle's travel as a fraction of full (from the cross product of the closed direction and the handle, divided by its value at full travel; no trigonometry at run time). Each tick the spout adds `opening × rate` to an accumulator and emits one unit per whole number. **Correction:** Vagrancy's `Hinge` is a bend limit between three points (`world.rs:68-69`, `:828-835`), not a pivot; the pivot here is an anchored particle (`m = 0`) and a `Stick`, and the stops are a new projection.

**D8 Input.** `Input(u16)`. Bits 0 to 3 pull spouts one to four. Bits 4 and 5 are reserved for moving or aiming a spout. Bit 6 is ready. Every other bit must be zero; a replay that sets one, or sets a reserved bit, is refused.

**D9 Belt, cups and the lid.** The belt carries cups at the mission's speed. A cup is three thin walls (two vertical, one floor) of half-thickness `WALL_HALF`. Because every wall is axis-aligned, the swept contact is exact rather than substepped: at the start of each tick the side of each wall a unit is on is decided from where its path, in the wall's own frame, crossed the wall's line (above the rim it went over; at or below it, it did not go through), and every pass holds the unit on that side. The rounded rim is a circle at the wall's top. This keeps the guarantee of `vagrancy/crates/sim/src/contact.rs` (no tunnelling at any speed) without its substep count, and nothing from `contact.rs` is needed. Capacity per cup size lives in `data/cups.json`, measured in M1.0.

**D10 Blends.** As the brief and Q3: a blend spout emits single-flavor units in a fixed repeating order built from its parts. With parts listed cherry 1, cola 1, the order is cherry, cola, cherry, cola (H4). A blend has no flavor id in `sim`.

**D11 Score.** `sim::score`, integers, the one rule of 0.4: `⌊100 · Σ min(nᵢ, C·tᵢ/T) / C⌋`. The cause named on a result is the largest of three losses: room left empty (`C − Σnᵢ`, floored at 0), units past their share, units the order did not ask for.

**D12 Missions.** `data/missions.json` in Vagrancy's tutorial shape (`id`, `chapter`, `teaches`, `builds_on`, `requires`), plus `line`, `orders`, `conditions`, `pass`. Eleven in the MVP (Q5), held to a chain by a lint.

**D13 Conditions.** `data/conditions.json` maps an id to a change to `Setup`, applied by `content`. MVP: `rail`. Carried, marked held: `second_line` (after the MVP, Q5) and `jet` (M6).

**D14 Pilots.** `(&World, seat, &mut State) -> Input`. *Idle* returns nothing. *Timer* reads the world as a player sees it (cups, orders, the counts the frame shows) and holds a spout's key while the cup coming under it still wants that flavor, letting go early by the tail it measured on the first pour. *Yardstick* is the timer with a seeded error in when it lets go. No learned policy.

**D15 Two seats.** Out of the MVP (Q5). `World::step([Input; 2])` and `Setup.lines[1]` are kept so the second line is a data change plus the code that steps it.

**D16 Files.** Replay v1 (`slushline.replay`): `format`, `version`, `sim_version`, `setup`, `inputs`, `ticks`, `checksum`, `checkpoints` (Vagrancy's, `replay.rs:23-32`). Save v1 (`slushline.save`, JSON): `format`, `version`, best score per mission, key bindings, options. Each refused with a sentence.

**D17 Look.** Flat shapes on paper. `data/palette.json` is the only place a color is written; the CSS gets variables from it at packaging; flavors carry a pattern, a brightness and an Okabe-Ito hue (0.5).

**D18 Sound.** After the MVP.

## 3. Repo layout and conventions

As D.0, without `crates/net` (D1) and `web/rtc.js`/`echo.html` (Q5). `CLAUDE.md` is Vagrancy's with the five rules swapped and two added, as D.0 says. The repository root is `~/Documents/SlurpeeGame` (the folder Sam started the session in); the GitHub repository and the URL are `slushline` (Q1).

## 4. Milestones

Each ends with `make test`, `make web`, `make test-ui` green, a push, the workflow green, and the gate walked against the live page. Sam's play is a row, not a wait.

### M0 — Foundation
- **Goal:** an empty page deploys and names its build, and every lint that guards the words and the look exists and has been seen failing.
- **Recon:** none; the toolchain was checked above.
- **Deliverables:** workspace, profiles, `Makefile`, `packaging/`, `deploy.yml`, `testing/drive.py`; crates compiling empty; `data/copy.en.json` from Game text, key for key; `data/palette.json`, `flavors.json`; `boundary.rs`, `copy.rs`, `look.rs`, `licenses.rs`; `LICENSES.md`, `MILESTONES.md`, `DECISIONS.md`.
- **Acceptance:** `sim_depends_on_serde_and_postcard_and_nothing_else`; `sim_has_no_float_no_hashmap_and_no_clock`; `every_string_a_player_reads_is_in_the_copy_file`; `no_string_breaks_the_tone_file`; `no_string_names_a_brand`; `every_pair_of_flavors_differs_in_pattern`; `every_pair_of_flavors_differs_in_brightness`; `h7_brightness_is_not_the_hex_value`. Each broken once.
- **Deployable:** a visitor sees the name, a loading line, and the build hash.
- **Risk:** Pages must be switched to Actions on a new repository before the first deploy.

### M1 — Slush in integers, and a pour you can play back
- **Goal:** one spout pours into one standing cup, and the same inputs give the same world everywhere.
- **Recon M1.0:** tick cost at 500/1,000/2,000 units native and wasm; units to fill the regular cup at three radii; whether a point source stacks in a column; slope and settle time for three tunings. Guesses as Part C.
- **Deliverables:** as Part D M1.
- **Acceptance:** H1, H5; `two_worlds_fed_the_same_inputs_agree_for_ten_thousand_ticks`; `a_pile_of_slush_keeps_its_slope`; `a_cup_keeps_rising_after_the_last_unit_lands`; `a_filled_cup_comes_to_rest`; `no_unit_passes_through_a_cup_wall`; `only_spout_bits_move_a_handle`; replay tests and a golden replay; native checksum equals wasm checksum in three engines.
- **Deployable:** pour into a cup, download the replay, reload, load it, watch the same pour.
- **Risk:** the pile levels (axiom reversion); the solver is too slow in wasm.

### M2 — The belt, the lid and a score
- **Goal:** missions 1 and 2 can be played to a result.
- **Recon M2.0:** the tail at three spring strengths; spill from a full moving cup at three belt speeds; encode-and-hash cost at 2,000 units.
- **Acceptance:** H2, H3; `a_cup_is_judged_once_on_the_tick_it_crosses_the_lid`; `judged_units_leave_the_world`; `the_result_names_what_cost_the_most_points`; `slush_past_its_share_scores_nothing`. The gate runs a scripted mission to a result and round-trips its replay.
- **Deployable:** the first build worth playing. The tuning and spring are the agent's pick from the recon numbers, carried for Sam to overrule by playing.

### M3 — Flavors, orders and blends
- **Goal:** every order that needs no condition can be poured, and read without color.
- **Recon M3.0:** brightness and pattern separation, plain and under three simulated deficiencies.
- **Acceptance:** H4, H6; `a_blend_spout_emits_its_flavors_in_its_fixed_order`; `every_order_can_reach_full_marks_from_its_line`; the look tests against the shipped palette; the gate's gray check on swatches and a mixed cup.
- **Deployable:** missions 3 to 9 from a list.

### M4 — The path (MVP complete)
- **Goal:** a player can go from the first mission to the eleventh, and a save file keeps their place.
- **Recon M4.0:** the yardstick's average on each mission over seeded runs.
- **Acceptance:** H8; the mission lints of 0.6 (less `the_path_gets_no_easier` if the ladder disagrees, in which case the data is reordered); `a_pilot_returns_an_input_and_nothing_else`; save round-trip tests; `no_mission_after_the_first_is_passed_with_every_key_held`.
- **Deployable:** the single-player path through the rail. Tagged `v0.1.0-mvp`.

### After the MVP
The second line and mission 12, two players at one keyboard, `net` and online play (the old M5); then the brief's M6.

## 5. Formats, by example

**Replay v1** (postcard):
```
Replay { format: "slushline.replay", version: 1, sim_version: 1,
         setup: Setup { seed, tuning, lines: [Some(Line {..}), None], .. },
         inputs: [[1, 0], [1, 0], [0, 0], …], ticks: 1840,
         checksum: 0x…, checkpoints: [one per 60 ticks] }
```
**Save v1** (JSON, so a player can read it):
```json
{"format": "slushline.save", "version": 1,
 "best": {"m_first_pour": 84, "m_tail": 71},
 "keys": {"spout_1": "KeyD", "spout_2": "KeyF", "spout_3": "KeyJ", "spout_4": "KeyK"},
 "options": {"short_codes": false}}
```

## 6. Data formats

- `flavors.json`: `{"cola": {"code": "Co", "pattern": "solid"}, …}`; colors are in `palette.json` under `flavors.<id>`.
- `blends.json`: `{"cherry_cola": {"parts": [["cherry", 1], ["cola", 1]]}}`; the emission order follows the list.
- `cups.json`: `{"regular": {"inner_width": …, "inner_height": …, "capacity": 120}, "small": {…, "capacity": 60}}`.
- `missions.json`: as 0.6.
- `conditions.json`: `{"rail": {"setup": {"spout": 1, "rail": {"amplitude": 30, "period": 120}}}, "second_line": {"held": true}, "jet": {"held": true}}`.
- `palette.json`: Vagrancy's shape plus `flavors`.
- `controls.json`: `{"solo": {"spout_1": "KeyD", "spout_2": "KeyF", "spout_3": "KeyJ", "spout_4": "KeyK"}}`.

## 7. Corrections to the brief, from the code

Each claim quoted, given a verdict, cited by line in `reference/` at the commits above.

1. **"particles with the `Stick`, `Min` and `Hinge` constraints … The handle is a stick on a hinge driven by the servo" (0.2).** Partly. `Con::Hinge { a, j, c, sign }` keeps a three-point joint from bending the wrong way (`vagrancy/crates/sim/src/world.rs:68-69`, applied at `:828-835`); it is not a pivot. A pivot is an anchored particle, `m = 0` (`:35-36`, `shift_pair` at `:873-885`). The handle is built from an anchored pivot and a `Stick`, with stops as a new projection (D7).
2. **"the servo motor" (0.2).** Confirmed: `drive_motors` accelerates a limb by `motor_accel` until it turns at `motor_speed` (`world.rs:421-458`). There is no spring in Vagrancy; the return spring is new.
3. **"the swept contact in `contact.rs`" (0.2, 0.3).** Confirmed for segments against capsules with `S = ceil(travel / radius)` substeps (`contact.rs:46-64`). Cup walls are axis-aligned, so the plan uses an exact crossing test instead, and a circle for the rim (D9).
4. **"`tests/boundary.rs`, `determinism.rs`, `replay.rs`" (0.2).** Confirmed at `vagrancy/crates/sim/tests/`. `boundary.rs` already scans the source with comments stripped (`boundary.rs:93-107`), so the correction Vagrancy made to Floodline's (its `PLAN.md` §7 item 1) carries over.
5. **"missions, goals and the `Tracker` in `tutorial.rs`; requirements and conditions in `road.rs`" (0.2).** Confirmed (`vagrancy/crates/content/src/tutorial.rs`, `road.rs`). The goals and `Tracker` are about fighting and are not ported; the requirement shape (`{"pass": id}`) is.
6. **"`gear-master-2d` `crates/core/src/look.rs` … any two of which can be lost" (0.2, 0.5).** Confirmed (`look.rs:17-25`). Its separation constant is `ROLE_SEPARATION = 0.08` (`look.rs:131`) and applies to consecutive role steps; the brief applies it to every pair of flavors, which is stricter, and the plan keeps the brief's version.
7. **"Lemon at 0.74 sits close to the paper ground at 0.81" (0.5).** Confirmed: Vagrancy's paper `#EFE8D8` has relative luminance 0.81 (computed with the H7 formula). The flavor luminances 0.02, 0.22, 0.74 and sky blue 0.41 are confirmed to two places (the command is in `SECOND-ORDER-M0.md`).
8. **"Default keys are D, F, J and K" and "With two lines at one keyboard, the upper line uses A, S, D and F" (0.1).** Not a contradiction in the MVP (one line), but they collide when two lines arrive: D and F are in both. Carried as a question for after the MVP.
9. **"`results.cup.short`: The cup was {fill_pct} percent full."** Fine, but `{fill_pct}` can pass 100 when slush is heaped over the rim; the lid scrapes units above the rim into waste (D9), so fill is at most the units under the rim over capacity, which can still pass 100 by a few percent on a well-packed cup. Core caps the sentence's choice: a cup at or over capacity never gets `results.cup.short`.
10. **"`every_order_can_reach_full_marks_from_its_line` (H6)" and mission 11's line, "cherry cola, lemon on a rail", for "cola, cherry and lemon, 1 to 1 to 2" (0.6).** Confirmed feasible: half the cup from the blend gives 30 cherry and 30 cola, lemon fills 60, which are exactly the shares 30, 30, 60 at a capacity of 120.
11. **Mission 6's order, "6 cups, single flavors and 1 to 1 pairs".** Underspecified; the plan writes it as cola, cherry, lemon, cola and cherry, cherry and lemon, cola and lemon.
12. **The brief's Part J lists `gear-master` as one of "six named in 0.2".** 0.2 names five (`vagrancy`, `gear-master-2d`, `floodline`, and the shapes inside them); A.1 counts six plus the site. All seven were cloned.
13. **`index.html` and `game.about_link`.** Vagrancy's copy has `game.about_link` (`vagrancy/data/copy.en.json`); this game's copy does not, which matches Q9's "no About link until the project page exists".

**Found while building** (each has its notebook row)

14. **"2,000 units fit in 2 ms in wasm" (Part C).** No: 6.6 ms natively at 2,000. A mission's live units are bounded by the lid, and a three-spout mission costs 0.10 to 0.16 ms a tick in wasm (SECOND-ORDER-M1 row 1, M4 row 5).
15. **"Held open for 10 ticks it emits 20 … = 20" (H5).** True for a ramp, not for the lever Sam chose: the tail adds units after the key is let go (SECOND-ORDER-M1 row 6).
16. **Vagrancy's release profile, `opt-level = "z"`,** did not inline fixed-point arithmetic; this game builds with 3 (SECOND-ORDER-M1 row 2).
17. **`the_path_gets_no_easier` (0.6).** The ladder supports it within chapters only (SECOND-ORDER-M4 row 2).
18. **`builds_on` as component ids** in the first data; the brief's example uses edges (SECOND-ORDER-M4 row 3).

## 8. Open questions for Sam

Answered before the plan (2026-10-05):
1. **Name:** Slushline.
2. **Brand names:** no. Slush, cola, cherry cola; the deny-list lint is in M0.
3. **A blend counts as its parts:** yes.
4. **"The fluid physics movement system":** the fully simulated lever (D7).
5. **Two seats in the MVP:** no. The MVP ends at gate 5.
11. **Does the agent push:** yes, for this run, through deployment of the MVP.

Taken at the agent's recommendation, for Sam to overrule:
6. **Compete or cooperate:** each player a line, compared (after the MVP).
7. **Waste:** gates the pass and does not change a cup's score.
8. **Marks:** an average of 60 to pass, waste under 20 percent, from mission 2. Eleven missions. Measured by the ladder and adjusted in data.
9. **Vagrancy's answers carry over:** public relays later; no About link; `TONE.md` public.
10. **The knowledge-component kit:** not run in the MVP; the components are in its format.
12. **Cherry is vermillion.** No red ban.

New:
13. **The default keys collide with the two-line keys** (§7 item 8). Recommendation: when two lines arrive, the upper line takes A, S, D, F and the lower J, K, L, semicolon, and a single line keeps D, F, J, K.
14. **Which tuning ships.** The agent picks from the recon numbers; Sam's play decides.
