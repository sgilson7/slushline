# Game text

The two files a person writes before the build: `TONE.md` and `data/copy.en.json`. Both are drafted here from the attached voice guide.

## TONE.md

Every string a player reads is checked against this file: menus, the how-to pages, mission cards, the HUD, results, the lobby, errors, settings, the page's own heading and status lines, and every accessible name. The strings are in `data/copy.en.json`, and the agent implements them exactly. This file is for the two cases where a string has to be judged: a new one the build needs, and an old one somebody wants to change.

The rules come from three places. Sam's *Writing and Voice Guide* supplies the voice, and the section numbers below point back to it. `vagrancy/TONE.md` supplies the form: each rule ends in a check that can be answered yes or no about one sentence. The lecture's Procedure B supplies one rule: same label, same words, every time it appears.

**The voice, in one line: a patient instructor standing at the machine, saying what to do and why it works.**

### The ten rules

**1. The first words say what to do or what happened.** An instruction begins with its verb, and a result begins with the outcome. (Guide §1.)

> "Let go before the cup looks full."

*Check:* does the string open with "You can", "You should", "Try to", "Remember to" or "It is important to"? Then it fails.

**2. The reason comes with the instruction.** Name the connection with "because" or "so", or give the reason in the next sentence. (Guide §5, §6.)

> "Start pouring before the cup is under the spout, because the slush takes a moment to fall."

*Check:* does an instruction longer than a button label carry its reason in the same string?

**3. "You" for what the player does, and names for everything else.** In strings for two players, where "you" is two people, use the lines' names. "We" is not used. (Guide §4.)

*Check:* search the two-player strings for "you". It should not be there.

**4. One name for each thing, and the names are ours.** Slush, cup, order, share, spout, handle, belt, line, lid, blend, waste, mission. A term is explained in plain words once and is not renamed afterward. No brand name appears in any string. (Guide §7; Procedure B.)

*Check:* the glossary below lists each term's synonyms and the brand deny-list. A hit on either is a failure.

**5. A number a player reads comes from core.** Scores, shares, counts, seconds, percentages and key names are placeholders. Nobody types one into a sentence, and that includes a count spelled out from three up. (Guide §15.)

*Check:* `tests/copy.rs` fails on a digit, or on a number word from three up, in a string.

**6. Certainty matches the evidence.** "Every", "always", "never", "only", "cannot" and "nothing" state a rule of the game, and each use names the test that establishes it. Advice is one thing to try and promises no result. A real "about" or "may" stays. (Guide §2.)

*Check:* a universal with no test named in the universals table fails. Advice that contains "will", "the trick is" or "the way to" fails.

**7. A refusal names what is in the way and what to do next.** One or two sentences, with no apology, no code and no "something went wrong". (Guide §1, §11.)

*Check:* does the message name the thing that failed, and offer a next step where one exists?

**8. A label says what the control does.** A label is a verb and its object, in sentence case. A heading names its subject. (Guide §7, §10.)

> "Run the mission again"

*Check:* read the label with no screen around it. Is it clear what pressing it does?

**9. Nothing is said for effect.** No praise or hype words, no exclamation marks, no slogans, no sentence fragments, no contrast formulas, no list of three made for its rhythm, and none of the words the guide lists. A string that explains the screen to its reader, where it should tell the player what to do or what happened, is cut. (Guide §7, §11 to §14.)

*Check:* `tests/copy.rs` holds the word lists below. A reviewer reads for the rest.

**10. A flavor is named, and never pointed at by its color.** A string says "cola" or "the cola spout". It does not say "the dark one". Where a string helps a player find a flavor, it gives the pattern. No result is described with a color word.

*Check:* `tests/copy.rs` fails on a color word in any string.

### Working notes

**Mechanics.** American spelling. Sentence case. No filler. An ellipsis only in a loading line. Contractions are fine where a person would use one. (Guide §8, §21.)

**Two registers, kept apart.** A sentence is written by a person and lives in the copy file. A number is derived by core and fills a placeholder. A sentence that states a mechanic is listed in the depends table with the decision it rests on, so that a change to the decision surfaces the sentence.

**Humor.** None is manufactured. (Guide §21, §24.)

**When a rule and a good sentence disagree.** Change the rule in this file in the same commit, with the sentence as the reason.

### Word lists for the lint

| Rule | List |
| --- | --- |
| 1 | Openers: You can, You should, Try to, Remember to, It is important to, I think |
| 4 | Brands, as a draft for Sam: Slurpee, ICEE, Slush Puppie, Coke, Coca-Cola, Pepsi, 7-Eleven |
| 6 | Universals: every, always, never, only, cannot, nothing, impossible, guarantees, instantly |
| 6 | Promises in advice: will, the trick is, the way to |
| 9 | From the guide: delve, tapestry, crucially, robust, leverage, underscore, landscape, realm, deep dive, key takeaway, not only |
| 9 | Praise and hype: perfect, awesome, amazing, epic, ultimate, insane, legendary, delicious, great job, nice work |
| 9 | Filler: basically, essentially, just, simply, really, actually |
| 9 | Contrast formulas: not just, more than just, isn't just, rather than |
| 10 | Color words: red, orange, yellow, green, blue, purple, pink, brown, black, white, gray, dark, light |

## data/copy.en.json

These are the strings, by key, written to the ten rules above. They are in tables so that you can edit a cell or comment on it. In M0 the agent writes `data/copy.en.json` from these tables, key for key, and changes no wording. A value in braces is a placeholder that the page fills from core or from data, and the last table says where each one comes from.

### The page and the menu

| Key | String |
| --- | --- |
| `game.name` | Slushline |
| `game.loading` | Loading {game}… |
| `game.loading_error` | The game could not load. Reload the page and try again. |
| `game.fallback` | This game requires JavaScript and WebAssembly. |
| `game.canvas_name` | {game} game |
| `game.back_link` | Tools and games |
| `game.build` | Build {hash} |
| `menu.missions.label` | Play the missions |
| `menu.missions.desc` | Fill each mission's orders to open the next one. |
| `menu.local.label` | Play at one keyboard |
| `menu.local.desc` | Two players share this keyboard, and each one runs a line. |
| `menu.online.label` | Play a friend online |
| `menu.online.desc` | One of you hosts a match and the other joins it. |
| `menu.replay.label` | Load a replay |
| `menu.replay.desc` | Open a replay file and watch that run again. |
| `menu.how.label` | Read how to play |
| `menu.settings.label` | Open settings |
| `menu.back.label` | Go back to the menu |

The `game.*` strings other than the name, and seven of the menu strings, are Vagrancy's.

### How to play

| Key | String |
| --- | --- |
| `how.spout.title` | Spouts and handles |
| `how.spout.body` | Hold {key.spout\_1} to pull the handle on the first spout. The spout opens further the longer you hold the key, until it is fully open. Let go and a spring swings the handle shut. |
| `how.lead.title` | The belt keeps moving |
| `how.lead.body` | Start pouring before the cup is under the spout, because the slush takes a moment to fall and the belt carries the cup along while it does. |
| `how.tail.title` | The tail |
| `how.tail.body` | Let go before the cup looks full. The handle takes a moment to swing shut, and the slush already in the air still has to land. |
| `how.swell.title` | Slush swells |
| `how.swell.body` | Slush grows for about {swell\_s} seconds after it leaves the spout, so a cup keeps rising after you let go. Stop short of the rim and let the slush fill the rest. |
| `how.order.title` | Orders |
| `how.order.body` | Each cup carries its order as a bar above it. The bar shows which flavors the cup should hold and how much of the cup each one gets, which is that flavor's share. Each flavor has its own pattern and short name, so the bar can be read without color. |
| `how.score.title` | How a cup is scored |
| `how.score.body` | A cup is scored out of {max\_score} when it reaches the lid. Slush counts toward the score while its flavor is still under its share. Slush past that share, and slush of a flavor the order did not ask for, takes up room and adds nothing. |
| `how.pass.title` | Passing a mission |
| `how.pass.body` | A mission's score is the average of its cups. To pass, reach the mission's mark and keep your waste under its limit. Waste is slush that lands outside a cup. |
| `how.blend.title` | Blends |
| `how.blend.body` | A blend spout pours more than one flavor, in shares that do not change. The label on the spout lists them. When an order's shares differ from the blend's, pour part of the cup from the blend and the rest from a single-flavor spout. |
| `how.files.title` | Replays and save files |
| `how.files.body` | Download a run as a replay from its result screen. Loading that file plays the same run again, input for input. Settings has the save file, which holds your progress through the missions. |

### Flavors, spouts and the HUD

| Key | String |
| --- | --- |
| `flavors.cola.name` | Cola |
| `flavors.cola.name_mid` | cola |
| `flavors.cola.short` | Co |
| `flavors.cherry.name` | Cherry |
| `flavors.cherry.name_mid` | cherry |
| `flavors.cherry.short` | Ch |
| `flavors.lemon.name` | Lemon |
| `flavors.lemon.name_mid` | lemon |
| `flavors.lemon.short` | Le |
| `blends.cherry_cola.name` | Cherry cola |
| `blends.cherry_cola.name_mid` | cherry cola |
| `patterns.solid` | solid |
| `patterns.stripes` | diagonal stripes |
| `patterns.dots` | dots |
| `spout.single` | {flavor} ({short}), {pattern} |
| `spout.blend` | {blend}: {recipe} |
| `spout.key` | Hold {key} |
| `recipe.one` | {flavor\_mid} |
| `recipe.two_equal` | equal shares of {a} and {b} |
| `recipe.two` | {a} and {b}, {a\_parts} to {b\_parts} |
| `recipe.three` | {a}, {b} and {c}, {a\_parts} to {b\_parts} to {c\_parts} |
| `hud.order` | Order: {recipe} |
| `hud.cup` | Cup {n} of {count} |
| `hud.score` | {score} of {max\_score} |
| `hud.waste` | Waste: {waste\_pct} percent |
| `hud.keys` | Your keys, in spout order: {keys\_upper}. |
| `hud.delay` | Input delay: {delay\_ms} ms |

### Missions

A mission's card shows its name, its order line, its pass line and one thing to try. The order line and the pass line are filled from `data/missions.json`.

| Key | String |
| --- | --- |
| `missions.order.same` | Fill {count} cups with {recipe}. |
| `missions.order.mixed` | Fill {count} cups. Each cup carries its own order. |
| `missions.pass` | To pass, average {pass} of {max\_score} and keep waste under {waste\_limit\_pct} percent. |
| `missions.pass_no_limit` | To pass, average {pass} of {max\_score}. |
| `missions.locked` | Pass {mission} to open this mission. |
| `missions.best` | Your best: {avg} of {max\_score}. |
| `missions.start.label` | Start the mission |
| `missions.list.m_first_pour.name` | First pour |
| `missions.list.m_first_pour.try` | Hold {key.spout\_1} while the cup is under the spout. Let go as the slush nears the rim, because slush that goes over it is wasted. |
| `missions.list.m_tail.name` | The tail |
| `missions.list.m_tail.try` | Let go earlier than looks right. The handle takes a moment to swing shut, and these cups are small enough for that last slush to overfill them. |
| `missions.list.m_two_spouts.name` | Two spouts |
| `missions.list.m_two_spouts.try` | Read each cup's order before the cup reaches the first spout, so you know which handle to pull. |
| `missions.list.m_half.name` | Half and half |
| `missions.list.m_half.try` | Fill the cup halfway at the first spout and finish it at the second. A cup that is full when it reaches the second spout has no room left, and what you pour there is wasted. |
| `missions.list.m_two_one.name` | Two to one |
| `missions.list.m_two_one.try` | Stop each pour where its share ends on the order bar. The first share is the larger one here, so the first pour is the longer one. |
| `missions.list.m_third.name` | A new spout |
| `missions.list.m_third.try` | Let a cup pass a spout its order does not name, because a flavor the order did not ask for takes up room. {key.spout\_3} pulls the new handle. |
| `missions.list.m_three.name` | Cola, cherry and lemon |
| `missions.list.m_three.try` | Stop each pour where its share ends on the order bar. Each spout gets one pass at the cup, and the belt does not go back. |
| `missions.list.m_cherry_cola.name` | Cherry cola |
| `missions.list.m_cherry_cola.try` | Read the label on the second spout. It now pours cherry and cola together, in the shares the label lists. |
| `missions.list.m_more_cola.name` | More cola than cherry |
| `missions.list.m_more_cola.try` | Work out how much of the cup the blend should fill, then finish with cola. Cherry now reaches a cup only through the blend, and it brings cola with it. |
| `missions.list.m_rail.name` | A spout on a rail |
| `missions.list.m_rail.try` | Pull the handle as the sliding spout comes over the cup, and let go before it slides past. Slush falls straight down from wherever the spout is. |
| `missions.list.m_lemon_cherry_cola.name` | Lemon cherry cola |
| `missions.list.m_lemon_cherry_cola.try` | Fill half the cup from the blend, which covers the cherry and the cola together. The sliding spout pours the rest. |
| `missions.list.m_two_lines.name` | Two lines |
| `missions.list.m_two_lines.try` | Run the upper line with {keys\_upper} and the lower line with {keys\_lower}. Watch whichever cup is closest to a spout, since that one needs you next. |
| `conditions.rail.name` | A spout on a rail |
| `conditions.rail.desc` | One spout slides back and forth above the belt. |
| `conditions.second_line.name` | A second line |
| `conditions.second_line.desc` | A second belt runs below the first, with its own spouts and its own keys. |
| `conditions.jet.name` | A rocket nozzle |
| `conditions.jet.desc` | One spout fires a straight jet with no fall time, and the jet pushes what it hits. |

`conditions.jet.*` is held until M6 and is the one place the word nozzle appears.

### Results

| Key | String |
| --- | --- |
| `results.cup.line` | Cup {n}: {score} of {max\_score}. |
| `results.cup.full` | The cup was full and each flavor matched its share. |
| `results.cup.short` | The cup was {fill\_pct} percent full. |
| `results.cup.over` | The cup held more {flavor\_mid} than its share. |
| `results.cup.wrong` | The cup held {flavor\_mid}, which its order did not ask for. |
| `results.mission.pass` | You passed. Your cups averaged {avg} of {max\_score}, and the mark is {pass}. |
| `results.mission.fail_score` | You did not pass. Your cups averaged {avg} of {max\_score}, and the mark is {pass}. |
| `results.mission.fail_waste` | You did not pass. {waste\_pct} percent of your slush landed outside a cup, and the limit is {waste\_limit\_pct} percent. |
| `results.mission.cause` | Most of the lost points came from {cause}. |
| `results.cause.empty` | room left empty |
| `results.cause.over` | flavors poured past their share |
| `results.cause.wrong` | flavors the order did not ask for |
| `results.versus.win` | {winner} won, {win\_avg} to {lose\_avg}. |
| `results.versus.tie` | The two lines tied at {avg}. |
| `results.next.label` | Go to the next mission |
| `results.again.label` | Run the mission again |
| `results.rematch.label` | Play another match |
| `results.to_missions.label` | Go back to the missions |
| `results.replay.label` | Download replay |

### Two players and settings

| Key | String |
| --- | --- |
| `lines.upper.name` | Upper line |
| `lines.lower.name` | Lower line |
| `local.intro` | Two players share this keyboard. {upper\_name} uses {keys\_upper}, and {lower\_name} uses {keys\_lower}. Both lines get the same orders. |
| `local.start.label` | Start the match |
| `settings.look.title` | Flavor patterns |
| `settings.look.short.label` | Show short names on the slush |
| `settings.look.short.desc` | Draws each flavor's short name on its slush in the cups, as it already appears on the spouts and the order bars. |
| `settings.keys.actions.spout` | Pull the handle on spout {n} |
| `settings.keys.solo.heading` | Keys for missions and online matches |

### Carried from Vagrancy

These strings are already in `vagrancy/data/copy.en.json` and are copied across.

| Keys | Change |
| --- | --- |
| `online.*` | None |
| `replay.*` | None |
| `results.key_hint` | None |
| `settings.keys.title`, `.desc`, `.conflict`, `.reset.label` | None |
| `settings.save.*` | In `download.desc`, "on the road" becomes "through the missions" |
| `settings.motion.label` | None. Its `desc` names effects this game may not have, so write it when the effects exist and mark it `"review": "new"` |
| `local.keyboard_limit` | "an arm" becomes "a handle" |

Not carried: the music strings, the practice yard, the road, the opponents, the weapons and the knowledge-component sentences. The knowledge-component sentences for this game are new strings for M4, marked `"review": "new"`.

### Glossary

| Term | Means | Not |
| --- | --- | --- |
| slush | what a spout pours | slushie, slurry, ice, drink, any brand name |
| cup | what the belt carries and the lid scores | glass, drink |
| order | the flavors a cup should hold and their shares | recipe, ticket, request |
| share | how much of a cup an order gives one flavor | ratio, portion, fraction |
| spout | where slush comes out | tap, dispenser, nozzle |
| handle | what a key pulls to open a spout | lever, crank |
| belt | what moves the cups | conveyor |
| line | one belt with its spouts | lane, station |
| lid | where a cup is scored | cap, judge |
| blend | a spout that pours more than one flavor | mix, combo |
| waste | slush that lands outside a cup | spill, mess |
| mark | the average a mission asks for | threshold, target |
| mission | one set of orders on one line | level, stage |
| run | one play of a mission or a match, as a replay holds it | recording, attempt |

Match, replay, save file, room code and input delay keep Vagrancy's entries.

### Depends

A string here states a mechanic. If the plan decides differently, the agent lists the string under open questions and leaves it as written.

| String | Rests on |
| --- | --- |
| `how.spout.body`, `how.tail.body` | D7, D8, and Part I question 4 |
| `how.swell.body` | D6 |
| `how.score.body` | D11 |
| `how.pass.body`, `missions.pass`, `results.mission.fail_waste` | D11, and Part I question 7 |
| `how.blend.body`, `spout.blend` | D10, and Part I question 3 |
| `missions.list.m_two_one.try`, `m_cherry_cola.try`, `m_more_cola.try`, `m_lemon_cherry_cola.try` | That mission's line and orders in `data/missions.json` |
| `missions.list.m_rail.try`, `conditions.rail.desc` | D13, H8 |
| `local.intro`, `results.versus.*` | D15, and Part I question 6 |
| `game.name`, `flavors.*`, `blends.*` | Part I questions 1 and 2 |

### Universals

| String | Word | The test that establishes it |
| --- | --- | --- |
| `how.score.body` | nothing | `slush_past_its_share_scores_nothing` |
| `missions.list.m_more_cola.try` | only | `every_order_can_reach_full_marks_from_its_line`, with a check that the mission's line has no cherry spout |

Two more strings state a rule without a listed word, and each has a test: `how.blend.body` (`a_blend_spout_emits_its_flavors_in_its_fixed_order`) and `missions.list.m_three.try` (`the_belt_moves_one_way`).

### Placeholders

| Placeholder | Filled from |
| --- | --- |
| `{game}` | `game.name` |
| `{hash}` | the build hash stamped by `packaging/package-web.sh` |
| `{key}`, `{key.spout_1}` to `{key.spout_4}` | the current binding for that handle, from `data/controls.json` or the player's save |
| `{keys_upper}`, `{keys_lower}` | the bound keys of that line, joined in spout order |
| `{flavor}`, `{flavor_mid}`, `{short}` | `flavors.<id>.name`, `.name_mid` and `.short` |
| `{blend}` | `blends.<id>.name` |
| `{pattern}` | `patterns.<id>`, for the pattern `data/palette.json` gives that flavor |
| `{recipe}`, `{a}`, `{b}`, `{c}`, `{a_parts}`, `{b_parts}`, `{c_parts}` | built by core from an order or a blend with the `recipe.*` strings |
| `{n}`, `{count}`, `{mission}` | the cup's or spout's number, the cups in the order, and `missions.list.<id>.name` |
| `{score}`, `{max_score}`, `{avg}`, `{win_avg}`, `{lose_avg}` | `sim::score` |
| `{pass}`, `{waste_limit_pct}` | that mission's `pass` in `data/missions.json` |
| `{fill_pct}`, `{waste_pct}`, `{cause}` | computed by core for the result; `{cause}` is `results.cause.<id>` |
| `{swell_s}` | the swell time in `sim::balance`, converted to seconds by core |
| `{upper_name}`, `{lower_name}`, `{winner}` | `lines.upper.name` or `lines.lower.name` |
| `{delay_ms}`, `{tick}`, `{code}`, `{theirs}`, `{ours}`, `{file_name}`, `{error}`, `{action}` | as in Vagrancy's copy file |
