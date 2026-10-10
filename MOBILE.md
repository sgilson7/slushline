# Mobile: Slushline on the iPad, and the foundation for the other tools

Sam, 2026-10-10: "create a version of slushline optimized for play on the ipad … i want this work on slushline to lay the foundation for getting these builder based tools … to be able to be deployed to mobile, starting with slushline for ipad".

This file says what Slushline does on a touch screen, which parts carry to any Builder tool unchanged, and what the Builder template would take on to make a tool mobile from its first commit. Nothing in `~/Documents/Builder/builder-setting` was changed: it is public and tied to a manuscript under review, so its template is Sam's to change. The last section is the proposal.

## What was there before

A survey of Builder (`builder-setting`) and the four repositories newer than Slushline (LongTimeComin, CirqueGame, LargeFlatEscape, AC6Browser) found no mobile work anywhere: no touch events, no fullscreen, no manifest, no safe areas, and no device emulation in any browser check. Two pieces point the right way. CirqueGame's `web/input.js` uses Pointer Events with pointer capture and `pointercancel`, with `touch-action: none` on its lab canvas. Every newer repository has a near-identical `web/input.js` (`makeInput(canvas, controls, actions)`), which turns devices into actions and actions into the core's input bits. That shape is what a touch layer extends.

## The four layers

Each is independent and none holds a game rule, so each can be copied into another tool.

### 1. Devices become actions; only actions become input (`web/touch.js`)

The rule from `keys.js` and the newer repos' `input.js`, extended to fingers: a device produces *actions* (`spout_2`, `belt_faster`), and the actions are turned into the core's input bits in one place. The core never learns which device was used, so replays, tests and the core's rules are untouched by touch.

- **Pointer Events, not touch events.** One code path covers a finger, a pen and a mouse. A pressed pointer is captured, so a finger that slides off its target still releases where it lifts. `pointercancel` and `lostpointercapture` release as well, and a capture the browser refuses is caught, because an error thrown in an input listener is a console error.
- **The page answers one question:** what action is under this point. In Slushline that is `Stage.spoutAt`, using the same geometry the stage draws with and never smaller than a 44-pixel fingertip (Apple's minimum target). `touch.js` knows nothing about spouts.
- **Two modes**, a saved option (`options.touch_mode`, core's save): hold, where an action is on while the finger is down, and tap, where a press latches the action until the next press. The latch is the input device's, like a sticky key. The core still sees only which bits are held each tick. Buttons that must never latch (the belt's throttle) use `holdButton`.
- **Every device together:** `keys.bits(...) | touch.bits(...)`. A keyboard on an iPad still works.

### 2. Filling the screen (`web/fullscreen.js`, the manifest)

- **The Fullscreen API with Safari's prefix.** iPadOS Safari supports it for the whole page; a phone's Safari and Playwright's mobile WebKit do not. Where it is missing, the page shows no button. On a touch screen it says how to fill the screen another way: "To play fullscreen, choose Share, then Add to Home Screen."
- **The home screen is the dependable route on iOS.** `manifest.webmanifest` (`display: fullscreen`, landscape) is written by `package-web.sh` from the copy file and the palette, so its name and colors have one home. The page carries `apple-mobile-web-app-capable`, an opaque `apple-touch-icon` (iOS fills a transparent corner with black) and `viewport-fit=cover`. Started that way, the page pads itself by the safe-area insets.
- **No service worker.** It would make the game work offline, but the Builder check fails on a worker, and an install that caches a build needs its own decision about updates.

### 3. A touch baseline in CSS (`web/styles.css`, "touch screens")

- `touch-action: manipulation` on the page, so there is no double-tap zoom and no tap delay. `touch-action: none` on the play area, so pressing it never scrolls.
- No selection, callout or tap flash over the game.
- Under `@media (pointer: coarse)`: targets at least 44 pixels high, keyboard hints hidden ("Press Enter", "Hold A"), and on-screen controls shown in place of keys.
- A run on a touch screen hides the page around the game and limits the canvas to the screen's height, so the controls stay on screen on a short display (an iPad mini, Split View, Safari's toolbars).

### 4. Checking it as an iPad (`testing/drive.py`, `ipad`)

A pass in WebKit with Playwright's "iPad Pro 11 landscape" device: touch, a mobile browser, an iPad's screen. Menus are driven by `page.tap`. A held finger is a dispatched `PointerEvent` with `pointerType: 'touch'`, which is what a held touch becomes in the browser. A tap is Playwright's own touchscreen. It checks:

- the menus by tap;
- hold mode (a held finger opens a spout and lifting closes it) and tap mode (a tap latches it open and the next tap closes it);
- the belt buttons;
- the whole line on screen during a run;
- the fullscreen button where the API exists, and the home-screen line where it does not;
- the manifest and its icons;
- `touch-action` on the page and on the line;
- no console error, and no request off the site.

Each check was broken once and seen failing (SECOND-ORDER-M13).

**What emulation cannot show:** real multitouch timing, Safari's actual fullscreen on an iPad, the home-screen install, and how the game feels under a hand. Those are rows for Sam with a real iPad.

## Proposal for the Builder template (Sam's decision)

If adopted, a new Builder tool would be mobile-ready from its first commit. Suggested as reference defaults, so a project can opt out in its "Changes to the defaults" table:

| # | Proposed default | Where in the template |
| --- | --- | --- |
| D10 | Input goes devices → actions → core bits, with Pointer Events for touch, pen and mouse. The page supplies only "which action is at this point". | `web/input.js` gains `area()` and `holdButton()` from `touch.js`; `data/controls.json` names the on-screen controls |
| D11 | A touch baseline in CSS: `touch-action`, no callout or selection over the play area, 44-pixel targets under `pointer: coarse`. | `web/style.css` |
| D12 | A web app manifest and apple-touch-icon, written from `data/` by the build. A fullscreen helper that shows a button only where the API exists. | `scripts/build.sh`, `web/fullscreen.js`, `web/index.html` |
| D13 | The browser check runs a fourth pass as an iPad (WebKit, touch), and `interactions.py` takes a `device` argument so a project can drive its tool by touch. | `tests/browser/check.py`, `interactions.py` |

Two template rules bear on this. **D3** allows no numbers other than 0 and 1 in page JavaScript: `touch.js` and `fullscreen.js` have none, but the 44-pixel fingertip in Slushline's `draw.js` would move to `data/` in a template tool. **D4**, the CSP, needs `manifest-src 'self'` added to the policy, or the manifest is refused.

**Tools that need more than the template's layers:**
- **The three pointer-lock games** (LongTimeComin, LargeFlatEscape, AC6Browser) aim with a locked mouse. On a touch screen that becomes a virtual stick or a drag-to-aim area: a project decision per game, not a template default.
- **CirqueGame** is closest: its drag code is already Pointer Events.
- **The DOM-only tools** (menus, lists, forms) mostly need only D11 and D12.
