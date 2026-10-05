# SECOND-ORDER-M0

The notebook for M0 (PLANNING-BRIEF Part E). One row per assumption, written when it was noticed.

| # | Kind | Status | Row |
|---|---|---|---|
| 1 | finding | done | **The carried Vagrancy strings trip this game's universals rule.** Vagrancy's lint had no "nothing" or "cannot" in its list (`reference/vagrancy/crates/content/tests/copy.rs:43-46`); this `TONE.md` rule 6 does. Six strings were flagged. Closed by naming a test for each in `_universals` (DECISIONS.md), without changing a string. `every_universal_names_a_test_that_exists` keeps the eight not-yet-written tests on a pending list, each leaving it in the milestone that writes it. |
| 2 | finding | done | **zsh did not word-split the test filter in the break-and-watch helper**, so the first round of breaks ran nothing and printed nothing (`error: invalid character ' ' in package name`). Vagrancy's HANDOFF §5 warned of this. Rerun with `${=3}`; all eight lints went red when broken and green when restored. Added to `CLAUDE.md`. |
| 3 | worklist | open | **The color-deficiency matrices in `content::look` are quoted from memory** (Machado, Oliveira and Fernandes 2009, severity 1.0). `simulating_a_deficiency_changes_hue_and_keeps_gray_as_gray` shows they are not the identity and keep gray as gray, which is not the same as being right. Check against the paper. |
| 4 | finding | done | **The brief's luminance column is right to two places.** `python3` with the H7 formula: cola 0.023, cherry 0.222, lemon 0.744, sky blue 0.405, bluish green 0.257, `#808080` 0.216. `the_brief_s_luminance_column_matches_the_palette` pins the first three. |
| 5 | divergence | done | **The brief's default keys and its two-line keys share D and F** (0.1). `A1`: "Default keys are D, F, J and K … the upper line uses A, S, D and F". No conflict with one line, which is all the MVP has (Q5). Carried as PLAN.md §8 Q13. |

Breaks watched failing (each restored from a copy in `$TMPDIR/breaks`): a float in `sim` (`boundary.rs:104`); a third dependency (`:27`); lemon at cherry's brightness (`look.rs:35`, and `:48` under deficiency); lemon with stripes (`:27`); "Cherry Coke" (`copy.rs:316`); an exclamation mark (`:187`); text written into `index.html` (`:500`); luminance read off the hex value (`look.rs:69`).
