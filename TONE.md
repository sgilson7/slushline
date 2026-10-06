# TONE.md

Every string a player reads is checked against this file: menus, the how-to pages, mission cards, the HUD, results, the lobby, errors, settings, the page's own heading and status lines, and every accessible name. The strings are in `data/copy.en.json`, and the agent implements them exactly. This file is for the two cases where a string has to be judged: a new one the build needs, and an old one somebody wants to change.

The rules come from three places. Sam's *Writing and Voice Guide* supplies the voice, and the section numbers below point back to it. `vagrancy/TONE.md` supplies the form: each rule ends in a check that can be answered yes or no about one sentence. The lecture's Procedure B supplies one rule: same label, same words, every time it appears.

**The voice, in one line: a patient instructor standing at the machine, saying what to do and why it works.**

## The ten rules

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

*Exception:* the judgement words under `judge.*` (EXCELLENT, GREAT, NICE, OK, MISS) are said for effect, at Sam's request on 2026-10-05: "a dance dance revolution type NICE or EXCELLENT … whenever a cup gets evaluated". They are the only strings this rule allows that, and each sits beside the score and the sentence that say what happened.

**10. A flavor is named, and never pointed at by its color.** A string says "cola" or "the cola spout". It does not say "the dark one". Where a string helps a player find a flavor, it gives the pattern. No result is described with a color word.

*Check:* `tests/copy.rs` fails on a color word in any string.

## Working notes

**Mechanics.** American spelling. Sentence case. No filler. An ellipsis only in a loading line. Contractions are fine where a person would use one. (Guide §8, §21.)

**Two registers, kept apart.** A sentence is written by a person and lives in the copy file. A number is derived by core and fills a placeholder. A sentence that states a mechanic is listed in the depends table with the decision it rests on, so that a change to the decision surfaces the sentence.

**Humor.** None is manufactured. (Guide §21, §24.)

**When a rule and a good sentence disagree.** Change the rule in this file in the same commit, with the sentence as the reason.

## Word lists for the lint

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
