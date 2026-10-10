# SECOND-ORDER-M13

The notebook for Slushline on the iPad (Sam, 2026-10-10; DECISIONS.md quotes the request; MOBILE.md is the design).

| # | Kind | Status | Row |
|---|---|---|---|
| 1 | finding | done | **No repository in the family had any mobile work.** Builder (`builder-setting`) and the four newer repos have no touch, fullscreen, manifest or device emulation. CirqueGame's `web/input.js` (Pointer Events, capture, `pointercancel`) and the shared `input.js` shape (devices to actions to bits) are the starting points. The newer repos follow Builder's D3 (no numbers in page JavaScript beyond 0 and 1) and D4 (a CSP). `touch.js` and `fullscreen.js` keep both, so they port. |
| 2 | divergence | done | **Playwright's mobile WebKit has no Fullscreen API**, as a phone's Safari has none; iPadOS Safari 16.4 and later has it. The button shows only where the API exists. Elsewhere on a touch screen a line points to Add to Home Screen, and the gate checks whichever case the browser presents. A real iPad's button is a row for Sam (row 9). |
| 3 | finding | done | **The copy lint caught three things in my first draft:** the glossary's synonyms for spout ("nozzle", "tap") in the settings strings, the brand word in a comment quoting Sam in `touch.js`, and the game's name spelled in comments in `index.html` and `styles.css`. All are fixed; the lint holds for comments under `web/`. |
| 4 | finding | done | **A `cargo test` sat for over half an hour.** Two other sessions were building their own games, and cargo's package cache lock is shared across every project on the machine. Run alone, each test passed or failed in seconds. |
| 5 | finding | done | **A cut-off screenshot was my screenshot script, not the game.** Playwright scrolled the page while tapping. On the emulated 11-inch iPad the line fits either way. On a short screen (1194 by 560) the run's height limit is what keeps it in view: without it the line ran from 72 to 596 and the gate failed, "the line runs from 72 to 596 on a screen 560 tall". |
| 6 | finding | done | **Tests, each broken once:** `a_save_from_before_touch_loads_holding_and_keeps_a_tap_choice` (without the default, an old save was "Damaged"); the iPad gate's hold check (no spout under a finger: "opened the nozzle to 0"), tap check (tap mode ignored), touch-action check ("touch-action is 'auto'") and line-on-screen check (row 5). |
| 7 | worklist | open | **The mission cards' tries name keys** ("Hold A for a moment…"), from `{key.*}` placeholders. On a touch screen the machines still carry their letters, so they read, but a touch-specific try for each mission is copy for Sam to decide on. |
| 8 | worklist | open | **The SoundCloud panel is hidden during a run on a touch screen**, so the line fits; it is on the menu, where Safari's one press on the player starts it. |
| 9 | worklist | the human's | **Sam plays on a real iPad:** the fullscreen button (iPadOS 16.4 or later), Add to Home Screen and the icon, several fingers on several spouts, hold and tap modes, the belt buttons, and the menus by touch. |
| 10 | worklist | the human's | **Sam decides on the Builder template proposal** (MOBILE.md, D10 to D13). |

New strings, marked `"review": "new"`: `screen.fullscreen.enter`, `.leave`, `.home`, and `settings.touch.title`, `.desc`, `.hold.label`, `.tap.label`.
