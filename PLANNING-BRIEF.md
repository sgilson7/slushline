# Slurpee line game: build plan for Claude Code

Oct 5, 2026 · @Sam

This is the planning brief for a browser game in which a conveyor belt carries cups under slush spouts and the player opens each spout by holding a key. It gives a Claude Code agent the game, what done means, and the procedure from the CSC 484 lecture. The agent's first job is to turn it into `PLAN.md` and stop. The brief makes five choices on your behalf, listed first in Part I, and they need your answer before the agent starts.

## How to use this brief

Save this tab as `PLANNING-BRIEF.md` at the root of a new repository. The doc exports to Markdown. Then start the agent with:

> Read PLANNING-BRIEF.md, then CLAUDE.md and TONE.md. Execute Part A to produce PLAN.md. Stop and show me PLAN.md before executing any milestone.

That prompt is the lecture's Design Insight 1 (slide 32): no code until you have read the plan. The files below are the lecture's planning artifacts (slide 28), with where each one comes from for this game.

| File | Who writes it | Where it comes from |
| --- | --- | --- |
| `PLANNING-BRIEF.md` | you | this tab |
| `TONE.md` | you | Game text, derived from your voice guide |
| `data/copy.en.json` | you | the same tab |
| `CLAUDE.md` | both | Vagrancy's, with the rules in D.0 replaced |
| `HOUSE-STYLE.md` | carried over | `vagrancy/HOUSE-STYLE.md`, unchanged |
| `PLAN.md` | the agent | Part A |
| `PROMPT-M<n>.md` | you | the six rows at the end of Part D |
| `SECOND-ORDER-M<n>.md` | the agent | Part E |
| `PLAYTEST-M<n>.md` and triage rows | you | Part F |
| `HANDOFF.md` | the agent | Part H |

## How the brief follows the lecture

The brief has one part for each of the nine stages of the GenAI-assisted prototyping procedure (slide 53). Each part carries the label of the AI-literacy step it applies. The labels are used the lecture's way: same label, same words, every time it appears.

| Stage | Who | Label | Part |
| --- | --- | --- | --- |
| brief | you write it | `C1 Frame the problem and set the rules` · `B1 Establish context` | 0 |
| plan | the agent | `B2 Require step-by-step justification` | A |
| I read the plan | you read it | `C3 Interrogate the claim` | B |
| recon | the agent | `B2 Require step-by-step justification` | C |
| build | the agent | `B2 Require step-by-step justification` | D |
| notebook | the agent | `B3 Build in self-questioning` | E |
| I play it | you play it | `C4 Verify against hand-computed cases` | F |
| deploy | you push | `C4 Verify against hand-computed cases` | G |
| handoff | the agent | `C5 Consolidate in your own words` | H |

Three more labels do work here. `A0 Calibrate your reliance` is the table in A.2. The autopsy steps `A1` to `A6` are how a divergence row is written in Part E. `B4 Plan the corrective dialogue` is the table in Part F. Part J is `C6 Question the system`, turned on this brief.

Two loops run through the stages. A plan you do not approve goes back to the agent. Anything play finds becomes a triage row that goes back into build.

This is the same shape as `vagrancy/PLANNING-BRIEF.md`, which the agent should read as the worked example. Where this brief is silent on procedure, that one applies.

## Part 0 — The brief

`C1 Frame the problem and set the rules` · `B1 Establish context`

### 0.1 What we are making

A two-dimensional game for the browser, seen from the side. A conveyor belt carries empty cups from left to right under a row of slush spouts. Each spout has a handle and one key. Holding the key pulls the handle, the valve opens with the handle, and slush falls into whatever is below.

Each cup carries an order: the flavors it should hold and the share of the cup each one gets. One mission asks for five cups of cola and lemon in equal shares. A cup passes each spout once, so the player meters every pour as the cup goes by. At the end of the belt a lid closes the cup and core scores it. The score rises as the cup gets closer to full and as its flavors get closer to their shares (0.4).

&#91;embedded content: the line · three spouts, three cups, the lid\]

The cup on the left has not reached a spout. The middle cup has its cola and is taking cherry. The cup on the right has passed all three spouts and is on its way to the lid.

Everything that pours is simulated. Slush is made of units that fall, pile up with a slope, swell for a moment after leaving the spout, and spill over a rim (0.3). The handle is a simulated lever, so a spout does not close the instant a key is released, and slush already in the air still lands. Judging that tail is the skill the first missions teach.

The missions form a single path whose orders get harder: one flavor, then two in one cup, then unequal shares, then three flavors, then blends such as cherry cola that replace a single-flavor spout (0.6). Some missions add a complication to the line. A spout that slides on a rail and a second belt are in the MVP. A rocket nozzle that fires a straight jet is designed here and built after it (0.7).

Two players can each run a line, at one keyboard or between two browsers with no server of yours (0.8).

Default keys are D, F, J and K for the four spouts of one line. With two lines at one keyboard, the upper line uses A, S, D and F and the lower line uses J, K, L and semicolon. Keys can be rebound in Settings.

### 0.2 What it is built on

The base is Vagrancy at commit `bb500e0` (2026-10-05). It already carries Floodline's fixed point, random stream, checksum and lockstep, and Gear Master 2D's packaging and browser gate, each corrected once from the code (`vagrancy/PLAN.md` §7). Clone the repositories into `reference/` (gitignored) and port from Vagrancy first. Borrow conventions and small pieces, and say in each commit what was taken from where.

| From | Take | Use as |
| --- | --- | --- |
| `vagrancy` `crates/sim` | `fx.rs`, `rng.rs`, `input.rs`, `replay.rs`, `frame.rs`; particles with the `Stick`, `Min` and `Hinge` constraints and the fixed-order solver in `world.rs`; the servo motor; the swept contact in `contact.rs`; `tests/boundary.rs`, `determinism.rs`, `replay.rs` | The start of the new `sim`. The handle is a stick on a hinge driven by the servo. Slush units are particles held apart by `Min` |
| `vagrancy` `crates/content` | `copy.rs`, `save.rs`; missions, goals and the `Tracker` in `tutorial.rs`; requirements and conditions in `road.rs`; `tests/copy.rs` | Missions (0.6), complications (0.7), the save file, the lints |
| `vagrancy` `crates/net`, `web/rtc.js`, `web/vendor/`, `web/echo.html`, `testing/online.py` | Two-seat lockstep with a measured delay; room codes and pasted codes | Two seats (0.8) |
| `vagrancy` `packaging/`, `Makefile`, `Cargo.toml` profiles, `.github/workflows/deploy.yml`, `testing/drive.py` | Packaging, the test count, the three-engine gate, the Pages workflow | The same |
| `vagrancy` `crates/pilot`, `crates/lab` | A pilot returns an `Input` and nothing else; `make ladder`; three tunings behind `?tuning=` | The yardstick that orders the missions; recon |
| `gear-master-2d` `crates/core/src/look.rs`, `crates/core/tests/look.rs`, `paintMotif` in `web/board.js` | The colorblind encoding: a motif, a brightness step and an Okabe-Ito hue, any two of which can be lost | Flavors (0.5) |
| `floodline` `crates/sim/src/water.rs` | Two tested properties of its integer water: volume is conserved, and it settles | The conservation and settling tests for slush. Its height-field method is not used |

**What "fluid" means in Vagrancy.** Vagrancy has no liquid simulation. `DECISIONS.md` records your instruction that the arms be "full fluid", meaning fully simulated: particles and sticks, moved by a motor while a key is held. This brief reads "the fluid physics movement system" as that system. It reuses it for the spout handle and extends the same integer solver to slush. If you meant something else, that is question 4 in Part I.

**What is new, with no house precedent.** Hundreds of loose particles in contact at once, a material that piles up and holds a slope, and a score computed from where particles come to rest. Vagrancy's worlds hold two fighters and the pieces cut from them.

**How slush is made, and what the game borrows.** One dispenser patent describes the machine this way. Syrup, water and CO2 are chilled to about −3 °C in a chamber while a scraper takes frozen product off the walls and stirs it. Pulling a lever opens the valve, and the flow can follow how far the lever is moved. The drink expands as it is dispensed, typically by about 80%, which the trade calls overrun. The game borrows the lever, flow that follows the lever, and the swelling. It does not model the barrel. The source is listed at the end, and it describes one design.

### 0.3 Slush

Slush is the part of the game with no house code behind it. The recommendation is below, and its numbers belong to recon (Part C).

**Units.** Slush is made of units. A unit is one particle in Vagrancy's solver, with a position, a previous position, a radius and a flavor. Units have no sticks. Each tick the solver finds every pair of units that overlap and pushes the two apart with the `Min` rule, as an exact pair shift, for a fixed number of passes in a fixed order. Neighbors are found with a uniform grid held in vectors and visited by cell and then by index, so two machines visit them in the same order.

**Three rules make it slush.** Each one departs on purpose from the water a fluid tutorial produces.

1. **It holds a slope.** A unit resting on other units does not slide unless the push on it passes a threshold. A pile of slush stays a pile, where water would level.
2. **It is thick.** Each tick a unit's velocity moves part of the way toward the average of the units touching it. A stream falls as a rope and folds where it lands.
3. **It swells.** A unit leaves the spout small and grows to full size over a fixed number of ticks, so a cup keeps rising for a moment after the last unit lands. This is the game's version of overrun.

**The handle and the valve.** A handle is one stick on a hinge with a return spring, driven by Vagrancy's servo while its key is held. The valve's opening is the handle's angle as a fraction of full travel. Each tick the spout adds opening × rate to an accumulator and emits one unit for each whole number in it. The tail after release is therefore a property of the lever, with no constant of its own. A blend spout emits its flavors in a fixed repeating order that matches its shares, so no random draw is involved.

**The belt and the cups.** The belt moves cups at a speed set by the mission. A cup is three wall segments carried by the belt. Units meet walls through the swept contact, so a fast unit cannot pass through a thin wall between two ticks.

**The lid.** When a cup's center crosses the lid, core counts the units inside it by flavor, records the result, and removes those units from the world. Units that land on the belt or the floor are counted as waste and removed. The world then holds only the slush still in play, which bounds the cost of a tick.

**Every unit is in one place.** At every tick, units emitted = units loose + units in cups + units judged + units wasted (H5).

Three tunings of the slope threshold, the thickness and the swell ship behind `?tuning=0|1|2`, as Vagrancy's motor tunings did. You choose among them by playing.

Position-based methods for fluids and for piles of grains are published: Macklin and Müller, "Position Based Fluids" (2013), and Macklin and others, "Unified Particle Physics for Real-Time Applications" (2014). Both citations are from memory, so verify them before relying on them. Both use floats. The integer version here is new work.

### 0.4 The score

A cup scores from 0 to 100 by one rule, computed in `sim` in integers.

A cup has room for `capacity` units. The order gives each flavor a share of that room. A unit in the cup counts while its flavor is still under its share. Units past a flavor's share, and units of a flavor the order did not ask for, take up room and count for nothing.

```latex
\text{score} = \left\lfloor \frac{100}{C} \sum_i \min\left(n_i,\ \frac{C\,t_i}{T}\right) \right\rfloor
```

C is the capacity, n is the units of flavor i in the cup, t is that flavor's parts in the order, and T is the sum of the parts. With a capacity of 120 and an order of cola and lemon at 1 to 1, each flavor's share is 60 units:

| In the cup | Units that count | Score |
| --- | --- | --- |
| 60 cola, 60 lemon | 120 | 100 |
| 30 cola, 30 lemon | 60 | 50 |
| 120 cola | 60 | 50 |
| 70 cola, 50 lemon | 110 | 91 |
| 60 cola, 60 cherry | 60 | 50 |

The one rule covers both things you asked for. A half-full cup at the right shares scores half. A full cup at the wrong shares loses the part that is wrong and keeps the rest.

**A blend counts as its parts.** A unit from the cherry cola spout is either a cherry unit or a cola unit, so a cup's contents are always counted in single flavors and one rule scores every order (H4).

**A mission's score** is the average of its cups' scores. The mission is passed when that average reaches the mark in its data and its waste stays under the limit in its data. The result names which of three causes cost the most points: room left empty, a flavor past its share, or a flavor the order did not ask for.

**Waste** is counted and shown on the result. It does not change a cup's score. From the second mission on, each mission sets a waste limit as a share of what was poured, so holding a key down fills the cups and still fails the mission (Part I, question 7).

Rejected: averaging a fill score and a ratio score. Under that rule the cup of 120 cola scores 75, and holding one spout open becomes a reasonable way to play.

### 0.5 Flavors a player can tell apart without color

A flavor is carried on three channels, and any two of them can be lost. This is Gear Master 2D's rule for its board (`look.rs`), applied to slush.

| Channel | Where it shows | Survives |
| --- | --- | --- |
| a pattern | on every unit, spout label and order bar | no color at all |
| a brightness step | the fill of the same things | no color at all |
| an Okabe-Ito hue | the same fill | for players who see it |

The starting palette is below. Luminance is relative luminance, computed from the hex value.

| Flavor | Code | Pattern | Fill | Luminance |
| --- | --- | --- | --- | --- |
| Cola | Co | solid | `#2B2A28` | 0.02 |
| Cherry | Ch | diagonal stripes | `#D55E00` | 0.22 |
| Lemon | Le | dots | `#F0E442` | 0.74 |

Each pair differs by more than 0.08, the separation Gear Master 2D's tests require between steps. A fourth flavor fits at sky blue (`#56B4E9`, 0.41). A lime on bluish green would sit at 0.26, too close to cherry, so reach its brightness by bisection as `look.rs` does.

Five rules follow from the channels:

- **The page never blends two flavors' colors.** A mixed cup shows each unit in its own flavor's fill and pattern. A blended color would be a new hue with no pattern.
- **Every unit and swatch has an outline in the line color.** Lemon at 0.74 sits close to the paper ground at 0.81, and the outline is what separates them.
- **A flavor's short code appears wherever the flavor does:** on the spout, in the order bar and in the result.
- **The order bar on a cup is split by share.** Each segment carries its flavor's pattern and a marker for how much of that flavor the cup holds so far. The page draws it from numbers core sends.
- **Feedback is a number and a sentence.** No result is told by green against red, and no string names a flavor by its color (`TONE.md` rule 10).

The claims are checked in three places. `crates/content/tests/look.rs` holds `every_pair_of_flavors_differs_in_pattern` and `every_pair_of_flavors_differs_in_brightness`, and repeats the brightness check under simulated protanopia, deuteranopia and tritanopia. The gate renders the swatches and a mixed cup, reduces the canvas pixels to gray, and checks that each pair still differs. H7 guards the luminance function those checks rely on. The float arithmetic lives in `content` tests, never in `sim`.

Settings has one switch for this, which draws the short code on the slush in the cups as well.

### 0.6 The missions

The MVP has twelve missions in one path, and passing each opens the next. The order below is the brief's proposal. The yardstick pilot measures it in M4, and your play outranks the yardstick.

| # | Mission | Spouts, in belt order | Order | What it teaches |
| --- | --- | --- | --- | --- |
| 1 | First pour | cola | 3 cups of cola | Hold a key to pull a handle, and stop at full |
| 2 | The tail | cola | 5 smaller cups of cola | Let go early, and start before the cup arrives |
| 3 | Two spouts | cola, lemon | 4 cups, cola and lemon by turns | Read the order on the cup |
| 4 | Half and half | cola, lemon | 5 cups of cola and lemon, 1 to 1 | Two flavors in one cup |
| 5 | Two to one | cola, lemon | 5 cups of cola and lemon, 2 to 1 | Unequal shares |
| 6 | A new spout | cola, cherry, lemon | 6 cups, single flavors and 1 to 1 pairs | Three handles, and letting a cup pass a spout |
| 7 | Cola, cherry and lemon | cola, cherry, lemon | 5 cups of cola, cherry and lemon, 2 to 1 to 1 | Three shares with one pass each |
| 8 | Cherry cola | cola, cherry cola, lemon | 5 cups of cola and cherry, 1 to 1 | A blend pours its parts |
| 9 | More cola than cherry | cola, cherry cola, lemon | 5 cups of cola and cherry, 3 to 1 | A blend topped up with a single flavor |
| 10 | A spout on a rail | cola, lemon on a rail | 5 cups of cola and lemon, 1 to 1 | Timing a pour from a moving spout |
| 11 | Lemon cherry cola | cherry cola, lemon on a rail | 5 cups of cola, cherry and lemon, 1 to 1 to 2 | A blend and a rail together |
| 12 | Two lines | two lines, each with cola and lemon | 4 cups a line, single flavors and 1 to 1 | Two belts at once |

Mission 4 is your example. Mission 8 is where cherry cola replaces cherry. From there cherry reaches a cup only with cola attached, and mission 9 turns that into arithmetic: 3 to 1 is half a cup from the blend and half from the cola spout (H4). Missions 10 to 12 carry the two complications in the MVP (0.7).

`data/missions.json` keeps the shape of Vagrancy's `data/tutorial.json` (`id`, `chapter`, `teaches`, `builds_on`, `requires`) and adds the line, the orders, the conditions and the pass mark:

```json
{"id": "m_more_cola", "chapter": "blends",
 "teaches": ["K10"], "builds_on": ["K09_K10", "K05_K10"],
 "requires": [{"pass": "m_cherry_cola"}],
 "line": {"belt": "steady", "cup": "regular", "spouts": ["cola", "cherry_cola", "lemon"]},
 "orders": [{"count": 5, "parts": {"cola": 3, "cherry": 1}}],
 "conditions": [], "pass": {"average": 60, "waste_pct": 20}}
```

`requires` is a list of requirement objects, as in Vagrancy's `data/road.json`. The path can then become a tree, with requirements such as passing a mission with no waste, without a change of format. In the MVP a lint holds it to a chain.

The last column becomes the knowledge components in `data/kc_graph.json`, with their sentences in the copy file as Vagrancy's are. The content lints, each a test in `crates/content/tests/missions.rs`:

- `every_order_can_reach_full_marks_from_its_line` (H6)
- `every_share_is_a_whole_number_of_units`
- `every_component_is_taught_after_the_ones_it_builds_on`
- `the_path_is_a_chain`
- `every_condition_a_mission_names_exists`
- `every_mission_can_be_passed`, by a pilot that returns an `Input` and nothing else
- `the_path_gets_no_easier`, which reads `analysis/ladder.md`

### 0.7 Complications

A complication is a condition on a mission, after the conditions on Vagrancy's road (`deep_ink`, `light`). It is data. `data/conditions.json` gives each condition an id and the change it makes to the mission's `Setup`, and `content` applies that change before the world is built. `sim` does not know which mission it is running. A replay carries its `Setup`, so a replay of a mission with a condition plays back with no extra work.

The MVP ships two conditions and the plan carries a third:

| Condition | Lands in | What changes | What it asks of the player |
| --- | --- | --- | --- |
| A spout on a rail | M4, in the MVP | The spout slides back and forth over the belt at a constant speed, turning at each end (H8). A unit falls straight down from where the spout was when the unit left | Time the pour to the spout as well as to the cup |
| A second line | M5, in the MVP | A second belt runs below the first, with its own spouts, cups and keys. It is the two-seat world of 0.8 with one keyboard routed to both seats | Run one line with each hand |
| A rocket nozzle | M6, after the MVP | The spout fires a straight jet that gravity does not bend. Core traces the jet with the swept contact to the first thing it meets and delivers units there, with no fall time and almost no tail. The jet pushes what it hits, so it knocks slush out of a cup that is already mostly full | Use it on empty cups and keep it off full ones |

Each condition uses a seam that is part of `Setup` from the first milestone that touches spouts, so adding the condition later does not reshape the world:

| Seam in `Setup` | What it allows |
| --- | --- |
| `Spout.rail` | A spout that moves on a fixed schedule |
| `lines: [Option<Line>; 2]` | A second belt with its own spouts and cups |
| `Spout.nozzle`, either `Fall` or `Jet` | A spout that fires a jet in place of falling slush |
| `Belt.schedule` | A belt that changes speed or pauses |
| two reserved bits in `Input` | A spout the player moves, or a nozzle the player aims |

More conditions fit these seams with no new code: a faster belt, smaller cups, a belt that pauses, a jet mounted sideways across both lines.

Four rules keep conditions cheap:

- A condition has a name and one sentence in the copy file, shown on the mission's card before the mission starts.
- `every_condition_is_used_by_a_mission_or_marked_held`. The rocket nozzle is marked held until M6.
- A condition adds no branch to the page. The page draws what `frame` returns.
- A new seam bumps `SIM_VERSION`. A new condition on an existing seam is a data change.

### 0.8 Two seats

The world has two seats, and a seat is a line. `World::step([Input; 2])` is the only way the world changes, as in Vagrancy. The three ways to play are the same world with different hands on the keys:

| Mode | Seat 0 | Seat 1 | Result |
| --- | --- | --- | --- |
| A mission | the player | empty | the mission's score |
| A mission with a second line | the player's left hand | the player's right hand | one score over both lines |
| Two players | one player | the other player | each line is scored, and the higher average wins |

Two players get the same orders on the same belt, so the comparison is fair, and neither line can reach the other's slush. They play at one keyboard or online.

Online play is Vagrancy's `net::Session`, ported as it stands. The host relays, nobody simulates ahead and nobody rolls back. Inputs take effect after a delay of 2 to 8 ticks, measured from the round trip and shown to both players. A checksum mismatch stops both sides on the same tick and names it. Trystero over public relays introduces the two browsers, and a pasted code replaces the relays when they fail. Port the code with its tests (`crates/net/tests/lockstep.rs`), `web/rtc.js`, `web/echo.html` and `testing/online.py`. The host's `Welcome` carries the `Setup`, which here holds the line and its orders.

Two things differ from Vagrancy and are measured in recon:

- The checksum hashes the whole world every tick, and this world holds many more particles.
- Input delay shifts the moment a valve opens, in a game about timing. Both players get the same delay and the HUD shows it.

Lockstep and replays both depend on two machines computing the same world. For that reason `sim` is integers only from M1, although online play arrives in M5.

Two players sharing one line, with the spouts split between them, is left for after the MVP (Part I, question 6).

### 0.9 The words

Every string a player reads is written before the build, against `TONE.md`, and lives in `data/copy.en.json`. Both files are drafted in Game text from your voice guide. The agent implements those strings exactly and does not rewrite, shorten or extend them. Two cases need handling, and both follow Vagrancy:

- **A string the build needs and the file lacks.** Write it with `TONE.md` open, add it under the same key scheme with `"review": "new"` beside it, and list it in the milestone's final message.
- **A string that states something a decision has changed.** `_depends` in the copy file maps such strings to the decisions in A.3. List the string under open questions and leave it as written.

Strings for the lobby, replays, the save file and key bindings are carried from Vagrancy's copy file, which you have already reviewed, with the noun changes listed in that tab.

Your instruction of 2026-10-04 in Vagrancy applies here from the start: a string tells the player what to do or what happened, and a string that explains the screen to its reader is cut. The draft has no content note, no screen introductions and no asides.

### 0.10 Done

The MVP is finished when all of these are true in the deployed browser build:

1. Holding a spout's key pulls its handle, and the valve opens with the handle. No key puts slush in a cup, moves a cup or changes the belt.
2. Slush is simulated in integers. It falls, holds a slope, swells after leaving the spout and spills over a rim. The slope and the swell are tests with numbers in them.
3. A cup is scored at the lid by the rule in 0.4, and the result names what cost the most points.
4. Every flavor differs from every other in pattern and in brightness, and no string names a flavor by its color. Both are lints, and the gate checks the rendered page in gray.
5. Twelve missions form one path, each opening the next. Every order can reach full marks from its line, and the path's order is measured against one yardstick. Progress is kept in a save file the player can download and load.
6. One mission replaces a single-flavor spout with a blend, and one asks for shares that need the blend and a single flavor together.
7. The path includes a spout on a rail and a second line.
8. Two players can each run a line, at one keyboard and between two browsers on two networks with no server of yours.
9. **Any run can be downloaded as a replay and played back to the same final checksum in Chromium, Firefox and WebKit.** This is the hard gate. A build that cannot round-trip a replay is not shippable, whatever else works.
10. It is static files on GitHub Pages in its own repository, and outside online play no request leaves the origin.

### 0.11 What the agent may not decide on its own

Each of these is yours. The agent raises it as a question and continues with independent work.

- The game's name, the flavors' names, and anything on the brand deny-list.
- Any change to a string in `data/copy.en.json`.
- Any change to the scoring rule in 0.4.
- A flavor told apart by hue alone, for any reason.
- A dependency in `crates/sim` beyond `serde` and `postcard`, a float in `sim`, a server, a bundler, or an npm step.
- `git push`, `make publish`, or anything else that deploys (Part G).
- Whether a pour feels right. The agent measures and you play.
- Every item in Part I.

## Part A — Produce the plan (the agent's first job)

`B2 Require step-by-step justification`

The agent's first deliverable is `PLAN.md`. It writes no game code until you approve `PLAN.md`. Every decision in it carries its reason, and every claim about a source carries a file and a line, a commit, or the command that produced it.

### A.1 Read the sources

Clone the seven repositories into `reference/`: the six named in 0.2 and `sgilson7.github.io`. Read `HOUSE-STYLE.md` whole. Then read Vagrancy's `PLAN.md` §7, `DECISIONS.md` and `HANDOFF.md` §6, which record what the last brief got wrong and what each mistake cost. Read `TONE.md` and every string in `data/copy.en.json`, and note each `_depends` entry. The lecture's Procedures A, B and C are the label table at the end of this brief.

### A.2 Calibrate before deciding

`A0 Calibrate your reliance`

The lecture's claim is that reliability tracks how common a problem is, and that a rare problem gets filled in from the common one it resembles (slide 15). Several parts of this game are rare, and four are common problems perturbed on purpose, shown in bold. For each row, `PLAN.md` states how much the agent expects to trust its first answer, and why. Revisit the table at the end of each milestone.

| Part of the game | How common | The textbook it will be filled in from | The small case that decides |
| --- | --- | --- | --- |
| Workspace, shim, packaging, CI, the gate | Common, and Vagrancy's code exists | (none expected) | The page prints its build hash |
| Two-seat lockstep | A port of Vagrancy's | Rollback; an authoritative host | Two worlds, 10,000 ticks, equal checksums |
| Many loose particles in integers | Rare | Float fluid code: `f32`, smoothing kernels, a `HashMap` grid | H1, H5 |
| **Slush holds a slope** | Perturbed | Water that levels | `a_pile_of_slush_keeps_its_slope` |
| **Slush swells after it leaves the spout** | Perturbed | Particles of one fixed size | `a_cup_keeps_rising_after_the_last_unit_lands` |
| A unit meeting a thin cup wall | Rare, and the house has paid for it once | Testing where things are at the end of a tick | `no_unit_passes_through_a_cup_wall` |
| A cup that moves while slush falls | Looks common | Dropping into a cup that stands still | H2 |
| **One rule for fill and shares** | Perturbed | Averaging a fill score and a ratio score | H3 |
| **A blend counts as its parts** | Perturbed | A blend filed as a flavor of its own | H4, H6 |
| Telling flavors apart | Looks common | Distinct hues; green for good and red for bad; brightness read off the hex value | H7 |
| The order of the missions | Rare | Difficulty asserted by intuition | The ladder table |
| Strings | Common, in the wrong register | Game marketing voice; brand names | The `TONE.md` lints |
| Whether it is fun | Not something the agent can judge |  | You play |

### A.3 Make the design decisions the brief leaves open

`PLAN.md` states a decision and a one-paragraph rationale for each row. The agent may depart from a recommendation and says so in the decision's first sentence. A row with a question number waits on your answer in Part I.

| # | Decision | Recommendation |
| --- | --- | --- |
| D1 | Crates | Vagrancy's six: `sim`, `content`, `pilot`, `net`, `wasm`, `lab`. Start each from Vagrancy's and remove what is about fighting. |
| D2 | Drawing | Canvas 2D from a vanilla ES module. The page may interpolate between two frames core sent. It may not integrate, predict or detect a contact. |
| D3 | Numbers | Vagrancy's `fx.rs`: `i32` with 12 fractional bits, one unit a centimeter, products through `i64`, overflow checks on in release. Recon M1.0 checks that 12 bits is enough for particles a centimeter or two across. |
| D4 | Time | Sixty ticks a second, fixed. The page owns the clock and `sim` has none. |
| D5 | Slush | 0.3: particles, `Min` contacts found through a vector grid, a fixed number of passes in a fixed order. |
| D6 | What makes it slush | 0.3: the slope threshold, the thickness and the swell, in three tunings for you to choose among. |
| D7 | Handle and valve | 0.3: one stick, a hinge, a spring and the servo. If recon M2.0 finds the lever adds nothing a player can feel, a plain ramp on the opening is the fallback, raised as a worklist row. (Q4) |
| D8 | Input | Vagrancy's `Input(u16)`. Bits 0 to 3 pull the handles of spouts one to four. Bits 4 and 5 are reserved for moving or aiming a spout. Bit 6 is ready. Every other bit must be zero, and a replay that sets one is refused. |
| D9 | Belt, cups and the lid | 0.3. A cup's capacity is a number of units per cup size in `data/cups.json`, measured in M1.0 and written in one place. |
| D10 | Blends | A blend is a spout whose units are single flavors, emitted in a fixed repeating order. (Q3) |
| D11 | Score | 0.4, in `sim::score`, in integers. |
| D12 | Missions | 0.6: Vagrancy's mission shape, held to a chain in the MVP. |
| D13 | Conditions | 0.7: changes to `Setup`, applied in `content`. |
| D14 | Pilots | A pilot returns an `Input` and nothing else. Three kinds: idle; a timer that holds each key for the ticks the order needs; and the yardstick, which is the timer with a seeded error in its timing. No learned policy. |
| D15 | Two seats | 0.8: two players on two lines, over Vagrancy's lockstep. (Q5, Q6) |
| D16 | Files | Replay v1 and save v1 in Vagrancy's formats, each with `format` and `version`, each refused with a sentence when it cannot be read. The save holds each mission's best score, the key bindings and the options. |
| D17 | Look | Flat shapes on a paper ground, and 0.5 for flavors. `data/palette.json` is the only place a color is written. |
| D18 | Sound | After the MVP, synthesized by the page from the events core reports. No audio file ships without a row in `LICENSES.md`. |

### A.4 Hand-computed cases

`C4 Verify against hand-computed cases`

Small cases computed by hand decide who is right. Each becomes a named test in the milestone shown, and `PLAN.md` restates each one with the agent's own arithmetic beside it. If the agent's arithmetic disagrees with the brief's, that is a notebook row and a question for you. It is never a silent correction in either direction.

**H1 — The order of the integrator (M1).** In a test world with no drag and the speed cap out of reach, a unit starts with downward speed 1 and gravity adds 4 a tick. Update speed first, then position. It falls 5, then 9, then 13, and after n ticks it has fallen n(2n + 3): **14** at n = 2 and **230** at n = 10. Updating position first gives 6 at n = 2, and the familiar n(n + 1)/2 gives 3. This is the lecture's example (slide 18) and Vagrancy's H1. It is ported with the integrator, and H2 is built on it.

**H2 — The cup moves while the slush falls (M2).** A unit leaves a spout 230 above a cup's rim with downward speed 1 and gravity 4. By H1 it reaches the rim on tick 10. The belt moves 3 a tick, so the cup has traveled 30. Take a cup with an inner half-width of 20 that was centered under the spout when the unit left. On tick 10 its upstream wall is 10 past the spout. **The unit lands 10 outside the cup.** Released when the cup's center was still 30 short of the spout, the same unit lands in the center. A test that drops into a standing cup reports a hit both times.

**H3 — The score (M2).** The five rows of the table in 0.4, with a capacity of 120: **100, 50, 50, 91, 50**. The fourth is ⌊100 × 110 / 120⌋. Averaging a fill score and a ratio score gives 75 for the cup of 120 cola.

**H4 — A blend counts as its parts (M3).** Capacity 120, and an order of cola and cherry at 3 to 1, so the shares are 90 and 30. Pour 60 units from a cherry cola spout, which is 30 cherry and 30 cola, and 60 from the cola spout. The cup holds 90 cola and 30 cherry. **The score is 100.** An implementation that files cherry cola as its own flavor counts 60 cola and nothing else, and scores 50. The first six units out of the cherry cola spout are cherry, cola, cherry, cola, cherry, cola.

**H5 — Every unit is in one place (M1).** In a test world a fully open spout emits 2 units a tick. Held open for 10 ticks it emits 20. **At every later tick, loose + in cups + judged + wasted = 20**, including the tick a cup crosses the lid and the tick a unit rolls off a rim. The familiar errors are a unit on a rim counted twice and a judged unit dropped from the total.

**H6 — An order the line cannot make (M3).** A line has a cola spout and a cherry cola spout, and the capacity is 120. An order of cherry alone can hold at most 60 cherry, from 120 units of the blend, so its best score is **50** and the lint rejects the mission. An order of cola and cherry at 1 to 3 wants 90 cherry and tops out at **75**, and is rejected. At 3 to 1 it reaches 100 (H4) and is accepted.

**H7 — Brightness is not the hex value (M3).** The relative luminance of `#808080` is **0.216**. One channel is 128/255 = 0.502, and ((0.502 + 0.055) / 1.055)^2.4 = 0.216. Reading the hex value as light gives 0.50. Black is 0 and white is 1.

**H8 — A spout on a rail (M4).** A rail has an amplitude of 30 and a period of 120 ticks, and the spout starts at the center moving downstream at 1 a tick. Its offset is 0 at tick 0, +30 at tick 30, **+15 at tick 45**, 0 at tick 60, −30 at tick 90 and 0 at tick 120. A sine wave gives 21 at tick 45.

Vagrancy's integer helpers and their pinned values come across with `fx.rs` and keep their tests.

### A.5 Write PLAN.md in this shape

```
1. Reliance, calibrated (the A.2 table, with the agent's column filled in)
2. Decisions D1 to D18 (decision, one paragraph of why, and what was rejected)
3. Repo layout and conventions (D.0)
4. Milestones M0 to M6 (Part D), each with:
     - Goal (one sentence)
     - Recon (what is measured first, and the guess it is checked against)
     - Deliverables (files and features, checkable)
     - Acceptance (test names as sentences, and what a person verifies)
     - Hand-computed cases that land here
     - Deployable? (what the deployed page lets a visitor do)
     - Risks specific to this milestone
5. The replay, save and wire formats, each as an example
6. Data formats: flavors, blends, cups, missions, conditions, palette, controls
7. Corrections to the brief, from the code
8. Open questions for Sam (Part I, plus anything the agent could not decide)
```

Section 7 is required. This brief was written from reading the repositories and not from running them (Part J), so some of what it says about them is wrong. Vagrancy's `PLAN.md` §7 is the model: each claim quoted, given a verdict, and cited by line.

Then stop and present it.

## Part B — I read the plan

`C3 Interrogate the claim`

You read `PLAN.md` and approve it or send it back. The agent makes that reading possible:

- A reference the agent cannot find is a question for you. It does not guess.
- Every claim about a source names where it was read. Every number names the command that produced it, or is marked as a guess.
- When you send the plan back, the agent revises the plan. It does not start building the parts you did not question.

After approval, `PLAN.md` wins where it and this brief disagree, and each divergence is a row in the notebook.

## Part C — Recon

`B2 Require step-by-step justification`

Every milestone begins by measuring. Recon is done against the running build, before anything is authored on top of a guess, and what it found goes in the commit message. A number that disagrees with the plan names which of the two is wrong.

The brief's guesses are below so that they can be checked. None has been measured.

| Recon | What is measured | The brief's guess |
| --- | --- | --- |
| M1.0 | The cost of one tick, natively and in wasm, with 500, 1,000 and 2,000 loose units | 2,000 units fit in 2 ms in wasm. If they do not, units get larger and cups hold fewer |
| M1.0 | How many units fill the regular cup to its rim, at three unit sizes | About 120 at the size that looks right. This sets `capacity` |
| M1.0 | Whether units poured from one point stack in a single column | They do, so the nozzle spreads units across its width in a fixed order |
| M1.0 | For three tunings: the slope of a pile of 60 units after 600 ticks, and how long a filled cup takes to come to rest | One tuning holds a slope a player can see and comes to rest within two seconds |
| M2.0 | The tail: units that land after the key is released, for three spring strengths | Between a tenth and a fifth of a cup |
| M2.0 | Units that leave a full cup as the belt carries it, at three belt speeds | None at the speeds the missions use |
| M2.0 | The cost of encoding and hashing the world each tick at 2,000 units | Under a fifth of a millisecond natively |
| M3.0 | Brightness and pattern separation of the palette, plain and under three simulated color deficiencies | Cola, cherry and lemon pass as given |
| M4.0 | The yardstick's average score on each mission, over 200 seeded runs | It does not rise from one mission to the next |
| M5.0 | The round trip between two home networks, and how a pour feels at the resulting delay. **This needs you and a friend.** | 30 to 80 ms, which makes the delay 3 to 6 ticks |

## Part D — Build

`B2 Require step-by-step justification` · `C2 Elicit one step with its rule named`

One milestone at a time, in this order. Each commit names the decision or the plan section it implements. A milestone is complete when `make test` passes, `make web` builds, `make test-ui` walks the gate in three engines, the deploy gate is live, and you have seen it.

&#91;embedded content: roadmap · six milestones, six deploy gates, then M6\]

Each gate puts its milestone's work on the live page before the next milestone starts. The single-player path is complete at gate 5, and the MVP is tagged at gate 6.

### D.0 Repo layout and conventions

The layout is Vagrancy's. The root documents, `Cargo.toml` and the `Makefile` keep their names and jobs. What differs is inside the crates and `data/`:

```
slushline/
  crates/sim/       the line. serde + postcard. no floats. no clock.
                    fx, rng, input, world (solver, handle, valve),
                    slush (grid, slope, thickness, swell),
                    line (belt, cups, lid), score, replay, frame
  crates/content/   data/*.json -> Setup; copy, look, missions,
                    conditions, save
  crates/pilot/     idle, timer, yardstick:
                    (&World, seat, &mut state) -> Input
  crates/net/       Session, wire, Loopback (from Vagrancy)
  crates/wasm/      the shim. decides nothing.
  crates/lab/       recon, ladder, golden replays. not shipped.
  web/              index.html, app.js, draw.js, keys.js, files.js,
                    rtc.js, config.js, echo.html, styles.css, vendor/
  data/             copy.en.json, flavors.json, blends.json, cups.json,
                    missions.json, conditions.json, kc_graph.json,
                    palette.json, controls.json
  testing/          drive.py, online.py, replays/
  packaging/        package-web.sh, count-tests.sh
```

`CLAUDE.md` starts as Vagrancy's. Its working style and its commands stay. In its non-negotiable list, the rules about `sim`'s dependencies, `World::step`, the shim, the page, the copy file, `SIM_VERSION`, exhaustive destructuring, pushing, GitHub Pages and deploy gates stay as written. Five rules are about fighting and are replaced:

| Vagrancy's rule | This game's rule |
| --- | --- |
| A swing adds momentum from nowhere | **Slush holds a slope and swells, and that is the design.** Do not make it level like water, or hold units at one size, to make the fluid "right". See D6 and the two tests named in A.2. |
| Only arm bits drive a joint motor | **A key pulls a handle and does nothing else.** No key adds slush to a cup, moves a cup or changes the belt. |
| Nothing drawn for ink, a cut or a fighter is red | **No flavor is carried by hue alone.** Every flavor has a pattern and a brightness step in `data/palette.json`, the page never blends two flavors' colors, and `crates/content/tests/look.rs` checks both. |
| No audio file without a `LICENSES.md` row | **No third-party file enters the repo or the build without a row in `LICENSES.md`.** |
| Nothing from the show | **No brand name appears in anything a player can see.** `crates/content/tests/copy.rs` holds the deny-list (Part I, question 2). |

Two rules are added:

- **The score is the one rule in `sim::score`.** The page shows the number core sent.
- **Every unit is in one place.** A change that adds somewhere a unit can be extends the conservation test in the same commit.

### M0 — Foundation, and the voice in the repo

**Goal:** an empty page deploys and names its build, and every lint that guards the words and the look exists and has been seen failing.

- **Deliverables:** Vagrancy's workspace, profiles, `Makefile`, packaging, Pages workflow and gate, with the crates compiling empty; a page that shows the entry strings and the build hash; `boundary.rs`; `copy.rs` with the `TONE.md` lints and the brand deny-list; `look.rs`; `count-tests.sh`; `LICENSES.md`; `MILESTONES.md`.
- **Acceptance:** `sim_depends_on_serde_and_postcard_and_nothing_else`; `sim_has_no_float_no_hashmap_and_no_clock`; `every_string_a_player_reads_is_in_the_copy_file`; `no_string_breaks_the_tone_file`; `no_string_names_a_brand`; the two look tests; H7. The gate passes in three engines with no console error and no request that leaves the origin. Each lint was broken once and watched failing.
- **Deploy gate 1:** a visitor sees the game's name, a loading line and the build hash.

### M1 — Slush in integers, and a pour you can play back

**Goal:** one spout pours into one standing cup, and the same inputs give the same world everywhere.

- **Recon:** M1.0.
- **Deliverables:** `fx`, `Rng`, `Setup`, `World`, and `World::step` as the only door; units, the grid, `Min` contacts, the slope threshold, the thickness and the swell; cup walls through the swept contact; the handle, its spring and the servo; the valve and its accumulator; the checksum; `SIM_VERSION`; replay v1 with download and load; the shim; a page that draws what `frame` returns; three tunings behind `?tuning=`.
- **Acceptance:** H1 and H5; `two_worlds_fed_the_same_inputs_agree_for_ten_thousand_ticks`; `a_pile_of_slush_keeps_its_slope`; `a_cup_keeps_rising_after_the_last_unit_lands`; `a_filled_cup_comes_to_rest`; `no_unit_passes_through_a_cup_wall` over seeded runs; `only_spout_bits_move_a_handle`; Vagrancy's replay tests and a golden replay. In the gate, the native checksum of a fixed input script equals the wasm checksum in all three engines.
- **Deploy gate 2:** pour into a cup, download the replay, reload, load it, and watch the same pour.

### M2 — The belt, the lid and a score

**Goal:** missions 1 and 2 can be played to a result.

- **Recon:** M2.0. You choose the slush tuning and the spring after playing the candidates.
- **Deliverables:** the belt and the cups it carries; the lid; waste; `sim::score`; cup and mission results with their strings; the HUD; an event list out of `step` (unit landed, spill, cup judged) for the page to draw and later to sound.
- **Acceptance:** H2 and H3; `a_cup_is_judged_once_on_the_tick_it_crosses_the_lid`; `judged_units_leave_the_world`; `the_result_names_what_cost_the_most_points`. In the gate, a scripted mission runs to a result in three engines and its replay round-trips.
- **Deploy gate 3:** the first build worth playing.

### M3 — Flavors, orders and blends

**Goal:** every order in 0.6 that needs no condition can be poured, and read without color.

- **Recon:** M3.0.
- **Deliverables:** several spouts on a line; `flavors.json`, `blends.json` and `palette.json`; orders on cups and the order bar; blend emission; `content::look` and the pattern drawing; the feasibility lint; key bindings and the short-code switch in Settings.
- **Acceptance:** H4 and H6; `a_blend_spout_emits_its_flavors_in_its_fixed_order`; `every_order_can_reach_full_marks_from_its_line`; the look tests against the shipped palette. In the gate, the gray check on rendered swatches and on a mixed cup.
- **Deploy gate 4:** missions 3 to 9 can be played from a plain list.

### M4 — The path

**Goal:** a player can go from the first mission to the eleventh, and a save file keeps their place.

- **Recon:** M4.0.
- **Deliverables:** `missions.json`, `kc_graph.json` and `conditions.json`; the mission screen and its cards; the rail, with missions 10 and 11; the three pilots; `make ladder`, which writes `analysis/ladder.md`; save v1 with download, load and the labeled convenience copy.
- **Acceptance:** H8; the mission lints listed in 0.6; `a_pilot_returns_an_input_and_nothing_else`; the save round-trip tests.
- **Deploy gate 5:** the single-player path, through the rail.

The order of the path is a measurement, and the copy does not depend on it. If the ladder disagrees with `data/missions.json`, reorder the data. The order your play reports outranks the ladder's.

### M5 — Two seats (MVP complete)

**Goal:** one player can run two lines, and two players can each run one, at one keyboard and across two networks.

- **Part one, no browser.** The second line in `World`; mission 12; two players at one keyboard; `net::Session` on `Loopback` with latency and jitter injected.
- **Part two, the transport.** `web/rtc.js`; the Trystero bundle vendored and pinned by name and sha256; `web/echo.html`; the room code path and the pasted-code path; the lobby and its strings.
- **Recon:** M5.0, with you and a friend.
- **Acceptance:** Vagrancy's lockstep tests, with a seat read as a line; `the_two_lines_never_share_a_unit`; echo checks for both paths; a two-tab match in the gate's online mode; the referee script over a three-minute match. You play one match across two networks. The checklist in 0.10 is walked by hand and every line is a yes.
- **Deploy gate 6:** tag `v0.1.0-mvp`.

### M6 — After the MVP

- The rocket nozzle: `Nozzle::Jet`, its condition, and two missions that use it.
- Sound effects synthesized from the event list.
- The path as a tree, in Vagrancy's road format, with requirements beyond a pass: no waste, or every cup above a mark.
- A fourth and a fifth flavor, each with its brightness reached by bisection.
- Two players on one line.
- A pass over every string against `TONE.md`, and an accessibility pass: focus order, the canvas name, the reduce-motion setting, contrast on the ground color.

Four things are left out on purpose, so the next plan does not cost them again: rollback, a learned pilot, gamepad and touch input, and a model of the freezing barrel.

### The status table

After every milestone's commit, append Vagrancy's status table to `MILESTONES.md` and print it last in the milestone's final message. It has one row per milestone with its deliverables, tests, commit and state, then the notebook's counts and the new strings awaiting you. `tests` is the number from `packaging/count-tests.sh` and never a number read off `cargo test`'s output (slide 36).

### The prompt for each milestone

You start each milestone with a `PROMPT-M<n>.md`. Its six rows are the lecture's Design Insight 2 (slide 33):

| Row | Label | What it says |
| --- | --- | --- |
| Reading order | `B1` | Read `HANDOFF.md`, then the rules in `CLAUDE.md`, then this milestone's section of `PLAN.md` |
| The ask, verbatim | `B1` | What this milestone adds, in your own words |
| Recon before content | `B2` | The Part C rows for this milestone, measured before anything is built on them |
| Findings in the commit | `B2` | What recon found goes in the commit message. A number that disagrees says which side is wrong |
| Keep the notebook | `B3` | `SECOND-ORDER-M<n>.md`, with a row written the moment something is noticed |
| Deploy points | `C1` | Which gate ends the milestone. You deploy, and the gate then walks the live page |

## Part E — The notebook

`B3 Build in self-questioning` · `C3 Interrogate the claim`

The notebook is `SECOND-ORDER-M<n>.md`, one per milestone, written the moment something is noticed. Each row is one assumption: a bold first sentence that states what was seen, then the measurement, the file, and what it changes. A row has a kind and a status.

- **divergence**: the build and the plan disagree. Say which is wrong.
- **finding**: something true about the game that nobody had written down. It is closed by `DECISIONS.md` or by a test.
- **worklist**: a change that reaches past this milestone, or a decision that is yours. It is brought to you and not taken.

Status is open, done, or the human's.

A divergence row is written by walking the lecture's Procedure A, with the labels in the row. `A1`: quote the plan's sentence or the test's assertion exactly. `A2`: quote the line of code that holds the rule the build used. `A3`: name the first line, or the first tick, where the two part ways. For a replay that drifts, compare checksums tick by tick. `A4`: run the nearest hand-computed case. `A5`: classify the failure. `A6`: connect it to A.2 and name the common problem this one was filled in from.

The three failure modes, as they appear in this game:

- **Axiom reversion.** The build swaps in the textbook rule the game was perturbed from. The code matches a fluid tutorial and not the plan: slush that levels, units of one size, an `f32`, a score that averages, a blend filed as a flavor, a hue with no pattern.
- **Invalid syntactic shortcut.** The form is right and the claim is false, and the check cannot fail: a pour test whose spout never opens, a conservation test over an empty world, a look test that compares a color with itself, a `..` in a destructure.
- **Spatial / counting miscount.** The parts do not sum to the whole: a unit through a wall between two ticks, a unit on a rim counted twice, a cup judged a tick late, shares that do not sum to the capacity, a rail off by a tick at its turn.

The house has already paid for each trap below.

| The house's version | This game's version | The guard |
| --- | --- | --- |
| A ball through rock (Gear Master 2D `SECOND-ORDER-M22` row 1); a blade through a limb (Vagrancy H2) | A unit through a cup wall | Swept contact; `no_unit_passes_through_a_cup_wall` |
| A check that compares zero with zero | A score test on a cup nothing was poured into | Break every new test and watch it fail |
| A goal met by standing still (Vagrancy's first tutorial goals) | A mission passed by holding every key down | The waste limit in 0.4; `no_mission_after_the_first_is_passed_with_every_key_held` |
| A second copy of a constant | The page's own capacity, tick rate or pass mark | The page reads the payload |
| 1,137 tests that were 1,037 (slide 36) | Any count in a document | `count-tests.sh`; `make ladder` |
| Overflow that means two things in two builds | A fixed-point product in a crowded cup | Overflow checks on in release; `fx::narrow` |
| A field the save forgot | A field the replay or the checksum forgot | Hash the whole encoding; destructure exhaustively |
| `git checkout` on a file with uncommitted work | The same | Copy the file before breaking it |
| A deployed fix a returning player cannot receive | The same | Stamp everything the browser caches |

## Part F — I play it

`C4 Verify against hand-computed cases`

At each deploy gate from the third on, you play the deployed build against what the plan claimed. Play is the hand-computed case for everything the agent cannot check: whether a pour feels like a pour, whether the tail can be learned, whether a mission teaches what its card says.

A report is written from the screen, in `PLAYTEST-M<n>.md`. Wherever possible it comes with a replay file saved from the result screen into `testing/replays/`. The simulation is deterministic, so a replay is the whole of a bug report.

For each reported moment the agent first reproduces it by running the replay headlessly in `crates/lab`, and writes down the tick. Then it writes a triage row with a severity (blocks, wrong but survivable, cosmetic), a cost (content, a number, a function, or a design decision that is yours) and a disposition (fixed, carried, or declined, with the reason). Blockers are fixed before the next milestone starts.

`B4 Plan the corrective dialogue`

The corrections are written in advance. When the agent reads one, it answers the question asked before it changes any code.

| When you see | You say |
| --- | --- |
| A float, a clock or a new dependency in `sim` | "`crates/sim` has `<what>` at `<file:line>`. Remove it, make `boundary.rs` catch it, and write the row: which textbook did this come from?" |
| A test that has never been seen failing | "Show me this test failing. Break the rule it guards, run it, paste the red line, and restore the file from the copy you took." |
| A number with no command behind it | "Which command produced this number? If none did, it is a guess. Mark it as one or measure it." |
| A string that differs from the copy file | "This string is not the one in `data/copy.en.json`. Restore it, and put your wording in the notebook as a worklist row." |
| Slush that levels like water | "A pile of 60 units is flat after 600 ticks. D6 says it holds a slope. Restate the rule as given, show me the line where the build parts from it, and run `a_pile_of_slush_keeps_its_slope`." |
| A score that does not follow the rule | "This cup scored `<n>`. Compute it by hand from the counts with the rule in 0.4, and show me both numbers." |
| Two flavors that look alike | "Show me this cup in gray. Which two swatches fail the brightness check, and what does `look.rs` report for them?" |
| A fix for a bug the report had no replay for | "What tick does this happen on, in which replay? If there is no replay, make one first." |

## Part G — Deploy

`C4 Verify against hand-computed cases`

- **The agent never pushes on its own judgement.** It does not run `git push` or `make publish`, even when the work is green. The exception is an explicit ask from you, and it is never inferred. For Vagrancy you gave the agent push rights for the run when you approved the plan. If you want this agent to deploy, say so at the same point (Part I, question 11).
- A push runs the workflow: test, build, the gate in three engines, then deploy to Pages.
- **A deploy is not finished until the gate has walked the live page:** `ORIGIN=https://sgilson7.github.io/<repo> testing/drive.py chromium firefox webkit`, every check, exit 0, with the build hash on the page matching the commit that was pushed.
- A locally built page and the deployed one cannot join each other online, because the room name carries the build hash. That is the guard working.
- The game's entry on the site's Tools and games page is yours to add. `sgilson7.github.io` holds the built site and not its source.

## Part H — Handoff

`C5 Consolidate in your own words`

`HANDOFF.md` is rewritten at every deploy gate, and whenever the session's context reaches about 700,000 tokens, whichever comes first (slide 37). It is written for a reader with none of the session's context, in the agent's own words, in Vagrancy's seven sections:

1. What this is, in five sentences.
2. The rules that are load-bearing, and what breaks silently when each is broken.
3. The shape of the code.
4. The commands.
5. What will bite within the hour.
6. The mistakes that cost time, each filed under the system it happened to.
7. The single next action.

History goes in `DECISIONS.md` and the notebooks. `CLAUDE.md` stays short enough to read in a minute.

## Part I — Open questions for Sam

The agent puts these in `PLAN.md` section 8 with its recommendation beside each.

### Before the agent starts

The brief made a choice on each of these so that it could be written. Vagrancy's brief had your answers to its four questions before it was finished. This one does not have them yet.

1. **What is the game called?** The brief uses Slushline as a placeholder, after Floodline. The name is held once, as `game.name`, and the repository and the URL follow it.
2. **May the game use the names Slurpee, Coke and Cherry Coke?** They are other companies' brand names, and the build is a public page linked from your research site. The brief uses slush, cola and cherry cola, and M0 adds a deny-list lint for the brand names, as Vagrancy's did for the show. This is the cautious choice and not a legal judgment. Each flavor's name is written once in the copy file, so changing it is a one-line edit.
3. **Does a blend count as its parts?** The brief says yes (0.4, H4): cherry cola is cherry and cola, and an order is always written in single flavors. The other reading is that cherry cola is a flavor of its own that an order asks for by name. Under that reading missions 9 and 11 need a new design.
4. **What did you mean by "the fluid physics movement system"?** The brief reads it as Vagrancy's fully simulated arm and reuses it for the handle (0.2, D7). If you meant only that a key opens a spout, D7 becomes a plain ramp and M1 gets shorter.
5. **Is two-seat play inside the MVP?** The brief says yes, as the last milestone, because you named multiplayer as part of the base. If the MVP should be tagged at gate 5 instead, the second line moves out with it and the rail is the one complication in the MVP.

### Can wait

6. **Do two players compete or cooperate?** The brief gives each player a line and compares scores. Two players sharing one line is in M6. (Wanted before M5.)
7. **Should waste cost points, or only gate the pass?** The brief gates the pass with a limit per mission and leaves cup scores alone. (Wanted before M2.)
8. **How many missions, and what marks?** The brief has twelve, and guesses a pass at an average of 60 with waste under 20%. (Wanted before M4.)
9. **Do your Vagrancy answers carry over?** There, public relays introduce online players and pasted codes are the alternative, there is no About link until the project page exists, and `TONE.md` is public. The new `TONE.md` holds rules derived from the voice guide you attached and none of the guide's own text. (Wanted before M0 for `TONE.md`, before M5 for relays.)
10. **Should your knowledge-component kit order the path?** Vagrancy's tutorial order came from running it. The brief orders the missions by hand and keeps the components in the kit's format, so the kit can be run after M4.
11. **Does the agent push?** Your request says build and deploy. The lecture and Vagrancy's `CLAUDE.md` say the agent does not push on its own judgement. The brief keeps the rule, and you can grant push rights for the run when you approve the plan.
12. **May cherry be red?** Vagrancy forbids red because ink stands in for blood. Nothing here does, so the brief drops that lint and uses vermillion for cherry.

## Part J — Question the system

`C6 Question the system`

The lecture ends by asking who built the system, on whose work, and what you would want to know before leaning on it (slide 52). The same questions apply to this brief.

- **Who wrote it.** Claude, in a chat session on 2026-10-05, from your description of the game, the lecture deck, the voice guide, and seven repositories cloned that day: `vagrancy` at `bb500e0`, `floodline` at `e18601b`, `gear-master` at `1deaa69`, `gear-master-2d` at `23a135e`, `pdf-redactor` at `6a7a065`, `perturbation-workbench` at `3cc83a6` and `sgilson7.github.io` at `31fdd17`.
- **What it did not do.** It compiled nothing, ran no test and played no game. A fetch of the live Vagrancy page returned only its loading line, so everything said here about Vagrancy comes from its repository. It has not built a particle fluid in integers, and neither has anything in the house.
- **Where it is strongest.** The procedure, the document shapes and the reuse table, which were read from Vagrancy's own brief, plan, decisions and handoff. The scores in H3, H4 and H6 and the luminance values were computed.
- **Where it is guessing.** Every number in Part C. Whether a slope threshold and a velocity average are enough to make particles read as slush. Whether 2,000 units run at sixty ticks a second in wasm. Whether the lever gives a tail a player can learn. Whether a game about timing is playable under lockstep delay. The order of the twelve missions, the pass mark and the waste limit.
- **What rests on one source.** The account of how slush is made comes from one patent's description of one machine. The two papers named in 0.3 and the palette's hex values are from memory.
- **What to check before leaning on it.** Answer the first five questions in Part I. Recompute H2, H3, H4 and H6 by hand. Read section 7 of `PLAN.md`, where the agent corrects this brief from the code, before the rest of it. Play the three tunings before accepting any number behind D6.
- **Did the confident tone mislead?** The recommendations are written as plain statements because the house style and your voice guide both ask for that. A plain statement here is a proposal, and A.3 says the agent may depart from it.

## Sources

- *Applications of AI Literacy to GenAI-Assisted Game Development*, CSC 484/584 guest lecture, September 2026. Slide numbers are the deck's own order, 1 to 55.
- *Sam Gilson — Writing and Voice Guide*, as attached.
- [sgilson7/vagrancy](https://github.com/sgilson7/vagrancy), [floodline](https://github.com/sgilson7/floodline), [gear-master](https://github.com/sgilson7/gear-master), [gear-master-2d](https://github.com/sgilson7/gear-master-2d), [pdf-redactor](https://github.com/sgilson7/pdf-redactor), [perturbation-workbench](https://github.com/sgilson7/perturbation-workbench) and [sgilson7.github.io](https://github.com/sgilson7/sgilson7.github.io), at the commits in Part J.
- [US patent 8,485,393, "Beverage dispenser"](https://image-ppubs.uspto.gov/dirsearch-public/print/downloadPdf/8485393), for the scraper, the temperature, the lever whose travel sets the flow, and overrun.
- Not opened, cited from memory: Macklin and Müller, "Position Based Fluids" (2013); Macklin, Müller, Chentanez and Kim, "Unified Particle Physics for Real-Time Applications" (2014); the Okabe-Ito palette's hex values.

## The labels

Same label, same words, every time it appears.

| Label | What the step is for |
| --- | --- |
| `A0` | Calibrate your reliance |
| `A1` | Restate the claim as given |
| `A2` | Extract the rule the chatbot actually used |
| `A3` | Locate the first divergence |
| `A4` | Test a small case |
| `A5` | Classify the failure mode |
| `A6` | Explain why the failure was likely |
| `B1` | Establish context |
| `B2` | Require step-by-step justification |
| `B3` | Build in self-questioning |
| `B4` | Plan the corrective dialogue |
| `C1` | Frame the problem and set the rules |
| `C2` | Elicit one step with its rule named |
| `C3` | Interrogate the claim |
| `C4` | Verify against hand-computed cases |
| `C5` | Consolidate in your own words |
| `C6` | Question the system |
