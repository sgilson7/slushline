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
- **2026-10-05, after the MVP deployed:** "for mission 2, 20% is too tight. also the fill meter for each cup should be underneath the cup. you should also have controls on the other hand to speed up or slow down the conveyer belt". The agent decided:
  - Mission 2's waste limit is 35 %. The other missions keep 20 %.
  - The order bar (the fill meter) is drawn under each cup, below the belt.
  - The belt has a throttle: holding one key speeds it up, the other slows it, and it keeps the speed it is left at, from a quarter of the mission's speed to double (`sim::balance::BELT_MIN`, `BELT_MAX`, `BELT_STEP`; one second held moves it by half its speed). Input bits 7 and 8, keeping bits 4 and 5 reserved for moving a spout. SIM_VERSION 3, so earlier replays are refused with their sentence.
  - The spouts move to A, S, D, F (one hand) and the belt is on ← and → (the other), all rebindable. A save from before the belt keys loads, keeping its bindings and taking the belt keys' defaults.
  - New strings, marked review: `settings.keys.actions.belt_slower`, `.belt_faster`, `hud.belt`, `hud.belt_keys`, `how.belt.*`.
  - This replaces the brief's rule that no key changes the belt; `CLAUDE.md` says so.
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
- **`builds_on` lists edges of `data/kc_graph.json`,** as the brief's example does; the components' sentences are new strings for Sam.
- **The ladder test checks the path within chapters.** The yardstick's averages rise within every chapter and fall where a chapter opens; the whole-path claim waits on Sam's play (SECOND-ORDER-M4 row 2).
