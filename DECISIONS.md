# Decisions

Why things are the way they are, one paragraph each. The full argument for D1 to D18 is in `PLAN.md` §2; this file holds the decision, what it rests on, and anything decided since.

## Decided by Sam

- **2026-10-05, before the plan,** Sam answered PLANNING-BRIEF Part I questions 1 to 5:
  - **Q1.** The game is called Slushline.
  - **Q2.** No brand names: slush, cola, cherry cola. The deny-list lint is in `crates/content/tests/copy.rs`.
  - **Q3.** A blend counts as its parts.
  - **Q4.** "The fluid physics movement system" is the fully simulated lever: a handle driven by a servo, with a spring (D7).
  - **Q5.** The MVP ends at gate 5. The second line, two players and online play come after it, with mission 12.
- **2026-10-05, after the plan was written:** "dont stop working until the MVP is deployed on my github pages, you have free reign to create a github repo, with pages and deployments all allowed". This replaces the brief's stop after `PLAN.md` (Part B) and its no-push rule (Part G) for this run, and the house rule that a milestone waits for Sam to see the last gate. Checks that need Sam's hands are carried as notebook rows with status *the human's*.
- **Questions 6 to 12** were taken at the brief's recommendation by the agent, for Sam to overrule (PLAN.md §8).

## The build

- **The copy file was generated from Game text key for key** (128 strings; `awk '/^## data\/copy.en.json/{f=1} /^### Carried from Vagrancy/{f=0} f' "Game text.md" | grep -cE '^\| \`[a-z0-9_.]+\` \| '` prints 128). Carried from Vagrancy: `replay.*`, `results.key_hint`, `settings.keys.{title,desc,conflict,reset}`, `settings.save.*` with "on the road" changed to "through the missions", `settings.motion.label`, and `local.keyboard_limit` with "an arm" changed to "a handle". `online.*` is not carried, because online play is after the MVP (Q5).
- **Six carried strings use a universal** ("nothing" in five refusals, "cannot" in `settings.keys.desc`), which this game's `TONE.md` rule 6 checks and Vagrancy's did not. Each now names the test that establishes it in `_universals`. The strings are unchanged.
- **Gates 3 and 4 were deployed as one** (SECOND-ORDER-M2 row 1), because the simulation carried every order once the belt existed and Sam asked for the MVP without stopping.
- **Contacts are inelastic** (`slush::inelastic`, SIM_VERSION 2): a push separates units and does not throw them. Without it, slush splashed over the cups' walls.
- **The swell follows the square of a unit's age,** so it shows in the cup rather than in the air. Linear growth was hidden by the solver twice.
- **Cup walls are slick and cup floors grip:** no slope rule against a wall, so swelling slush rises up a cup's sides; the floor carries the slush with the cup.
- **Slush is drawn as outlines, then fills, then patterns,** so each unit has its outline and a cup of one flavor reads as one area in gray.
- **The belt runs at 12, 18 or 24 cm/s** (`data/line.json`), slower than first written, so a full pour fits while a cup passes a spout.
- **A replay finds its mission by its setup,** in `content`, so a replay of a mission shows that mission's result.
