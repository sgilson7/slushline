"""Drive the built page in real browsers and walk the deploy gate.

`cargo test` cannot reach any of this: whether the wasm instantiates in a
browser, whether the stamped imports resolve, whether a download produces a
file and a file input feeds it back, and whether the browser's arithmetic is
the native build's arithmetic.

A console error or a request that leaves the origin fails the run, so "nothing
is uploaded" is tested rather than asserted. The one exception is the
soundtrack (Sam, 2026-10-06): the page may ask SoundCloud's widget host for its
player, and the gate refuses those requests, so the run never depends on
SoundCloud and checks the game carries on without it. Every visible line of text must
be a string from data/copy.en.json, so "the page writes no words of its own"
is tested from what the player sees.

Ported in shape from vagrancy/testing/drive.py. `ORIGIN` points the gate at a
page that is already served, which in practice means the live one
(PLANNING-BRIEF Part G).

    python testing/drive.py [chromium|firefox|webkit ...]
    ORIGIN=https://sgilson7.github.io/slushline python testing/drive.py chromium firefox webkit
"""
import functools
import http.server
import json
import os
import re
import socketserver
import subprocess
import sys
import threading
import traceback
from pathlib import Path

from playwright.sync_api import sync_playwright

ROOT = Path(__file__).resolve().parent.parent
WEB = Path(os.environ.get("SLUSHLINE_WEB") or ROOT / "dist" / "web")
PORT = 8137
LIVE = (os.environ.get("ORIGIN") or "").rstrip("/")
ORIGIN = LIVE or f"http://127.0.0.1:{PORT}"
COPY = json.loads((ROOT / "data" / "copy.en.json").read_text())
# The soundtrack's widget (web/music.js). The page names this host and no
# other; everything SoundCloud loads after it comes from inside its frame.
SOUNDTRACK_HOSTS = ("w.soundcloud.com",)
TRACK = "tracks%2F78084887"


def soundtrack(url):
    from urllib.parse import urlparse
    return (urlparse(url).hostname or "") in SOUNDTRACK_HOSTS

CHECKS = []


def check(fn):
    CHECKS.append(fn)
    return fn


class Quiet(http.server.SimpleHTTPRequestHandler):
    def log_message(self, *a):
        pass


def serve():
    handler = functools.partial(Quiet, directory=str(WEB))
    socketserver.TCPServer.allow_reuse_address = True
    httpd = socketserver.ThreadingTCPServer(("127.0.0.1", PORT), handler)
    threading.Thread(target=httpd.serve_forever, daemon=True).start()
    return httpd


def copy_strings():
    out = []

    def walk(v, path):
        if isinstance(v, dict):
            for k, c in v.items():
                if not k.startswith("_") and k != "review":
                    walk(c, f"{path}.{k}" if path else k)
        elif isinstance(v, str):
            out.append((path, v))

    walk(COPY, "")
    return out


def copy_patterns():
    """Each copy string as a regex: placeholders match any text."""
    pats = []
    for key, s in copy_strings():
        s = s.replace("{game}", COPY["game"]["name"])
        parts = re.split(r"\{[a-z0-9_.]+\}", s)
        pats.append((key, re.compile("^" + ".+?".join(re.escape(p) for p in parts) + "$", re.S)))
    return pats


PATTERNS = copy_patterns()


def text_is_copy(text):
    return any(p.match(text) for _, p in PATTERNS)


def expected_build():
    if LIVE:
        import urllib.request
        with urllib.request.urlopen(ORIGIN + "/build.txt") as r:
            return r.read().decode().split()
    return (WEB / "build.txt").read_text().split()


def open_page(browser, query=""):
    ctx = browser.new_context(accept_downloads=True)
    page = ctx.new_page()
    problems, offsite = [], []
    # A refused soundtrack request is reported by the browser as a load
    # failure; it is the gate's doing, not the page's. Firefox reports a
    # picture of the tour whose download a reload cut short as "corrupt or
    # truncated" (SECOND-ORDER-M10 row 7); the gate reloads often, and
    # tests/world.rs checks every such picture is whole.
    def cut_short_art(text):
        return "Image corrupt or truncated" in text and f"{ORIGIN}/art/" in text
    page.on("console", lambda m: problems.append(f"console.{m.type}: {m.text}")
            if m.type == "error" and not soundtrack(m.location.get("url") or "")
            and not any(h in m.text for h in SOUNDTRACK_HOSTS) and not cut_short_art(m.text) else None)
    page.on("pageerror", lambda e: problems.append(f"pageerror: {e}"))
    # blob: and data: URLs are the page talking to itself; anything else that
    # does not start with the origin has left it.
    page.on("request", lambda r: offsite.append(r.url)
            if not (r.url.startswith(ORIGIN) or r.url.startswith(("blob:", "data:")) or soundtrack(r.url)) else None)
    page.route(lambda url: soundtrack(url), lambda route: route.abort())
    page.goto(ORIGIN + "/" + query, wait_until="load")
    page.wait_for_function("document.body.dataset.ready === '1'", timeout=30000)
    return ctx, page, problems, offsite


VISIBLE_TEXT = """() => {
    const out = [];
    const walk = (el) => {
        if (el.nodeType !== 1) return;
        const st = getComputedStyle(el);
        if (st.display === 'none' || st.visibility === 'hidden' || el.hidden) return;
        if (['SCRIPT', 'STYLE', 'NOSCRIPT'].includes(el.tagName)) return;
        let own = '';
        for (const n of el.childNodes) if (n.nodeType === 3) own += n.textContent;
        own = own.replace(/\\s+/g, ' ').trim();
        if (own && !el.dataset.value) out.push(own);
        for (const c of el.children) walk(c);
        for (const a of ['aria-label', 'title', 'placeholder']) {
            const v = el.getAttribute(a);
            if (v) out.push(v.trim());
        }
    };
    walk(document.body);
    return out;
}"""


def visible_lines_not_in_copy(page):
    """Text that is not a copy string. Elements marked data-value hold a
    number or a key name filled into a sentence, not a sentence."""
    return [t for t in page.evaluate(VISIBLE_TEXT) if not text_is_copy(t)]


# --- the checks --------------------------------------------------------------

@check
def the_page_names_its_build(page, name):
    want = expected_build()[0]
    got = page.inner_text("#build").strip()
    if got != f"Build {want}":
        return [f"{name}: the page says {got!r}, and the build is {want}"]
    print(f"ok: {name}: the page prints its build hash, {want}")
    return []


@check
def every_visible_line_is_a_copy_string(page, name):
    bad = visible_lines_not_in_copy(page)
    if bad:
        return [f"{name}: text that is not in the copy file: {bad}"]
    print(f"ok: {name}: every visible line on the first screen is a copy string")
    return []


def native_script_checksum(ticks):
    out = subprocess.run(["cargo", "run", "-q", "--release", "-p", "lab", "--", "script-checksum", str(ticks)],
                         cwd=ROOT, capture_output=True, text=True, check=True)
    return out.stdout.strip()


SCRIPT_TICKS = 900
NATIVE = {}


@check
def the_browser_computes_the_native_checksum(page, name):
    if "sum" not in NATIVE:
        NATIVE["sum"] = native_script_checksum(SCRIPT_TICKS)
    got = page.evaluate(f"window.slushline.scriptChecksum({SCRIPT_TICKS})")
    if got != NATIVE["sum"]:
        return [f"{name}: the fixed script's checksum after {SCRIPT_TICKS} ticks is {got} here and {NATIVE['sum']} natively"]
    print(f"ok: {name}: the fixed script ends on {got}, as natively")
    return []


def lines_not_in_copy(page, where):
    bad = visible_lines_not_in_copy(page)
    return [f"text on {where} that is not in the copy file: {bad}"] if bad else []


@check
def a_mission_plays_to_a_result_and_round_trips_through_a_replay_file(page, name):
    page.evaluate("localStorage.clear()")
    page.reload(wait_until="load")
    page.wait_for_function("document.body.dataset.ready === '1'")
    page.click("#menu-missions")
    problems = lines_not_in_copy(page, "the path")
    problems += lines_not_in_copy(page, "the tree")
    page.click("#mission-m_first_pour")
    page.click("#open-mission")
    problems += lines_not_in_copy(page, "the mission card")
    page.click("#start-mission")
    page.wait_for_function("window.slushline.tick() > 5")
    # A player's hands for a moment, then the timer pilot to the end.
    # Holding a spout key opens its valve, and the pour's sound follows the
    # opening core sent (Sam, 2026-10-06: "a gentle pshht sound"): a voice
    # sounds while the valve is open and stops once it has closed.
    page.keyboard.down("a")
    page.wait_for_timeout(300)
    pouring = page.evaluate("window.slushline.pourVoices()")
    page.keyboard.up("a")
    try:
        page.wait_for_function("window.slushline.frame().lines[0].spouts[0].opening === 0", timeout=5000)
        page.wait_for_timeout(100)
        after = page.evaluate("window.slushline.pourVoices()")
    except Exception:
        after = "the valve did not close"
    if pouring != 1 or after != 0:
        problems.append(f"holding A left {pouring} pour voices sounding, and letting go {after}")
    # The belt's throttle, on the other hand: held, it speeds up, and it keeps
    # the speed it is left at (Sam, 2026-10-05).
    page.keyboard.down("ArrowRight")
    page.wait_for_timeout(500)
    page.keyboard.up("ArrowRight")
    page.wait_for_timeout(100)
    faster = page.evaluate("window.slushline.frame().lines[0].belt_pct")
    page.keyboard.down("ArrowLeft")
    page.wait_for_timeout(1500)
    page.keyboard.up("ArrowLeft")
    slower = page.evaluate("window.slushline.frame().lines[0].belt_pct")
    if not (faster > 100 and slower < 100):
        problems.append(f"the belt keys left the belt at {faster} and then {slower} percent")
    problems += lines_not_in_copy(page, "the mission")
    # The first cup's lid: its sound and its word (Sam, 2026-10-05). Step
    # until a lid closes, then check the word is the copy string for the
    # judgement core chose.
    seen = None
    for _ in range(250):
        page.evaluate("window.slushline.autoplay(40)")
        seen = page.evaluate("window.slushline.judgement()")
        if seen:
            break
    if not seen:
        problems.append("no judgement word appeared when a lid closed")
    elif seen[1] != COPY["judge"][seen[0]]:
        problems.append(f"the judgement word reads {seen[1]!r}, not judge.{seen[0]}")
    # The groove for three top cups in a row: its animation shows and plays
    # with no console error. Core's rule for when is a cargo test.
    page.evaluate("window.slushline.groove()")
    page.wait_for_timeout(400)
    if not page.evaluate("window.slushline.grooveShowing()"):
        problems.append("the groove's animation did not show")
    page.evaluate("window.slushline.autoplay(10000)")
    page.wait_for_selector(".verdict", timeout=20000)
    problems += lines_not_in_copy(page, "the result")
    verdict = page.inner_text(".verdict")
    final = page.evaluate("window.slushline.checksum && document.body.dataset.result")
    with page.expect_download() as dl:
        page.click("#download-replay")
    path = dl.value.path()
    size = len(Path(path).read_bytes())
    page.reload(wait_until="load")
    page.wait_for_function("document.body.dataset.ready === '1'")
    with page.expect_file_chooser() as fc:
        page.click("#menu-replay")
    fc.value.set_files(path)
    page.wait_for_function("window.slushline.tick() > 2")
    page.evaluate("window.slushline.skipReplay(100000)")
    page.wait_for_function("window.slushline.replayDone() === true", timeout=60000)
    got, want = page.evaluate("[window.slushline.checksum(), window.slushline.recordedChecksum()]")
    if got != want or not got:
        problems.append(f"the replay ended on {got}, and recorded {want}")
    page.wait_for_selector(".verdict", timeout=20000)
    if page.inner_text(".verdict") != verdict:
        problems.append(f"the replay's result reads {page.inner_text('.verdict')!r}, the run's read {verdict!r}")
    if problems:
        return [f"{name}: {p}" for p in problems]
    print(f"ok: {name}: holding A sounded {pouring} pour voice and letting go {after}; the first lid showed {seen[1] if seen else None!r}; the belt ran at {faster} % after the faster key and {slower} % after the slower; the first mission played to {verdict!r} ({final}); its replay ({size} bytes) loaded after a reload, ends on {got}, and shows the same result")
    page.click("#to-missions")
    page.click("#back-to-menu")
    return []


@check
def the_save_keeps_a_pass_and_round_trips_through_a_file(page, name):
    page.click("#menu-missions")
    opened = page.locator("#mission-m_tail:not(.locked)").count()
    if not opened:
        return [f"{name}: passing the first mission did not open the second"]
    page.click("#back-to-menu")
    page.click("#menu-settings")
    with page.expect_download() as dl:
        page.click("#download-save")
    path = dl.value.path()
    saved = json.loads(Path(path).read_text())
    page.evaluate("localStorage.clear()")
    page.reload(wait_until="load")
    page.wait_for_function("document.body.dataset.ready === '1'")
    page.click("#menu-missions")
    if page.locator("#mission-m_tail:not(.locked)").count():
        return [f"{name}: a cleared browser still opened the second mission"]
    page.click("#back-to-menu")
    page.click("#menu-settings")
    with page.expect_file_chooser() as fc:
        page.click("#load-save")
    fc.value.set_files(path)
    page.wait_for_selector("#save-note:not([hidden])")
    note = page.inner_text("#save-note")
    if note != COPY["settings"]["save"]["loaded"]:
        return [f"{name}: loading the save says {note!r}"]
    problems = lines_not_in_copy(page, "settings")
    page.click("#back-to-menu")
    page.click("#menu-missions")
    if not page.locator("#mission-m_tail:not(.locked)").count():
        problems.append("the loaded save did not open the second mission")
    page.click("#back-to-menu")
    if problems:
        return [f"{name}: {p}" for p in problems]
    print(f"ok: {name}: a pass opened the next mission; the save file ({saved['passed']}) brought it back after the browser was cleared")
    return []


@check
def two_actions_cannot_share_a_key_in_settings(page, name):
    page.click("#menu-settings")
    page.click("#bind-spout_1")
    page.keyboard.press("s")
    page.wait_for_selector("#bind-note:not([hidden])")
    note = page.inner_text("#bind-note")
    want = COPY["settings"]["keys"]["conflict"].replace("{key}", "S").replace("{action}", COPY["settings"]["keys"]["actions"]["spout"].replace("{n}", "2"))
    problems = [] if note == want else [f"the conflict reads {note!r}, not {want!r}"]
    page.click("#bind-spout_1")
    page.keyboard.press("q")
    page.wait_for_function("document.querySelector('#bind-spout_1').textContent === 'Q'")
    page.click("#reset-keys")
    page.wait_for_function("document.querySelector('#bind-spout_1').textContent === 'A'")
    problems += lines_not_in_copy(page, "settings")
    page.click("#back-to-menu")
    page.click("#menu-how")
    problems += lines_not_in_copy(page, "how to play")
    page.click("#back-to-menu")
    if problems:
        return [f"{name}: {p}" for p in problems]
    print(f"ok: {name}: a taken key is refused with {note!r}; a free one binds; reset restores A")
    return []


MISSION_IDS = [m["id"] for m in json.loads((ROOT / "data" / "missions.json").read_text())["missions"]]
ALL_PASSED = {"format": "slushline.save", "version": 2, "best": {i: 90 for i in MISSION_IDS}, "passed": MISSION_IDS,
              "clean": {i: 5 for i in MISSION_IDS},
              "keys": {"spout_1": "KeyA", "spout_2": "KeyS", "spout_3": "KeyD", "spout_4": "KeyF", "belt_slower": "ArrowLeft",
                       "belt_faster": "ArrowRight", "lower_1": "KeyJ", "lower_2": "KeyK", "lower_3": "KeyL", "lower_4": "Semicolon"},
              "options": {"short_codes": False, "sound_volume": 70}}


@check
def the_map_draws_one_route_into_each_store_and_the_lanes_their_hasse_lines(page, name):
    # Sam, 2026-10-07: clean up the map after Vagrancy's chart, which draws
    # one route into each fight and folds its other views away. With every
    # mission passed, the map draws a route into each mission but the first,
    # and the lanes (Sam, 2026-10-05: "a hasse diagram with seperate lanes")
    # draw the Hasse diagram's lines, from the fold at the foot of the screen.
    page.evaluate(f"localStorage.setItem('slushline.save', {json.dumps(json.dumps(ALL_PASSED))}); localStorage.removeItem('slushline.view')")
    page.reload(wait_until="load")
    page.wait_for_function("document.body.dataset.ready === '1'")
    page.click("#menu-missions")
    page.wait_for_function("document.querySelector('#tree.rows') && Number(document.querySelector('#tree').dataset.lines) > 0")
    stores = page.locator("#tree .node").count()
    routes = int(page.get_attribute("#tree", "data-lines"))
    folded = not page.evaluate("document.getElementById('view-fold').open")
    page.click("#view-fold summary")
    page.click("#view-chart")
    page.wait_for_function("document.querySelector('#tree.chart') && Number(document.querySelector('#tree').dataset.lines) > 0")
    chart = int(page.get_attribute("#tree", "data-lines"))
    lanes = page.locator("#tree .lane-name").count()
    problems = lines_not_in_copy(page, "the lanes")
    page.click("#view-tree")
    page.click("#back-to-menu")
    if routes != stores - 1 or not folded:
        problems.append(f"the map drew {routes} routes for {stores} stores, and the other views were {'folded' if folded else 'open'}")
    if not (chart > routes) or lanes < 5:
        problems.append(f"the lanes drew {chart} lines in {lanes} lanes, against the map's {routes}")
    if problems:
        return [f"{name}: {p}" for p in problems]
    print(f"ok: {name}: the map draws {routes} routes for {stores} stores, its other views folded; the lanes draw {chart}, in {lanes} lanes")
    return []


@check
def a_mission_with_two_lines_plays_with_the_lower_line_on_its_own_keys(page, name):
    # Sam, 2026-10-05: levels with more than one belt. The lower line's
    # spouts answer J, K, L and ;, the upper line's A, S, D and F, and the
    # mission plays to one result over both lines.
    page.evaluate(f"localStorage.setItem('slushline.save', {json.dumps(json.dumps(ALL_PASSED))})")
    page.reload(wait_until="load")
    page.wait_for_function("document.body.dataset.ready === '1'")
    page.click("#menu-missions")
    page.click("#mission-m_two_lines")
    page.click("#open-mission")
    problems = lines_not_in_copy(page, "a two-line card")
    page.click("#start-mission")
    page.wait_for_function("window.slushline.tick() > 2")
    page.keyboard.down("j")
    page.wait_for_timeout(400)
    lower, upper = page.evaluate("[window.slushline.frame().lines[1].spouts[0].opening, window.slushline.frame().lines[0].spouts[0].opening]")
    page.keyboard.up("j")
    if not (lower > 0 and upper == 0):
        problems.append(f"J opened the lower line's first spout to {lower} and the upper one's to {upper}")
    problems += lines_not_in_copy(page, "a two-line mission")
    for _ in range(300):
        page.evaluate("window.slushline.autoplay(60)")
        if page.locator(".verdict").count():
            break
    page.wait_for_selector(".verdict", timeout=20000)
    verdict = page.inner_text(".verdict")
    problems += lines_not_in_copy(page, "a two-line result")
    page.click("#to-missions")
    page.click("#back-to-menu")
    if problems:
        return [f"{name}: {p}" for p in problems]
    print(f"ok: {name}: J pulled only the lower line's handle; the mission over two lines played to {verdict!r}")
    return []


def srgb_lum(rgb):
    def lin(c):
        c = c / 255
        return c / 12.92 if c <= 0.04045 else ((c + 0.055) / 1.055) ** 2.4
    r, g, b = (lin(x) for x in rgb)
    return 0.2126 * r + 0.7152 * g + 0.0722 * b


@check
def every_flavor_differs_in_gray_on_the_rendered_page(page, name):
    # PLANNING-BRIEF 0.5: render a mixed cup, reduce the canvas to gray, and
    # check that each pair of flavors still differs. Each unit is sampled away
    # from its pattern mark.
    lums = {}
    # m_three renders cola, cherry and lemon; s_sink adds raspberry.
    for mission, ticks in (("m_three", 1150), ("s_sink", 1400)):
        page.evaluate(f"localStorage.setItem('slushline.save', {json.dumps(json.dumps(ALL_PASSED))})")
        page.reload(wait_until="load")
        page.wait_for_function("document.body.dataset.ready === '1'")
        page.click("#menu-missions")
        page.click(f"#mission-{mission}")
        page.click("#open-mission")
        page.click("#start-mission")
        page.wait_for_function("window.slushline.tick() > 2")
        page.evaluate(f"window.slushline.autoplay({ticks})")
        page.wait_for_timeout(100)
        got = page.evaluate("""() => {
            const c = document.getElementById('stage');
            const g = c.getContext('2d');
            const by = {};
            const us = window.slushline.unitPixels();
            us.forEach((u, i) => {
                if (u.r < 3 || !u.cup) return;
                const x = Math.round(u.x + u.r * 0.45), y = Math.round(u.y + u.r * 0.45);
                // Units are drawn in order; a point a later unit (or its
                // outline) covers shows that unit, not this one.
                // Fills are drawn after every outline, so only a later unit's
                // fill can cover this point.
                for (let j = i + 1; j < us.length; j += 1) {
                    const v = us[j];
                    if (v.f !== u.f && (v.x - x) ** 2 + (v.y - y) ** 2 < (v.r + 0.5) ** 2) return;
                }
                // The fill is the color most of the unit shows: take a small
                // grid inside it and keep the most common pixel, so a
                // pattern's mark, however a browser strokes it, is outvoted.
                const votes = new Map();
                const d = g.getImageData(Math.round(u.x - u.r * 0.7), Math.round(u.y - u.r * 0.7), Math.max(1, Math.round(u.r * 1.4)), Math.max(1, Math.round(u.r * 1.4))).data;
                for (let k = 0; k < d.length; k += 4) {
                    const key = `${d[k]},${d[k + 1]},${d[k + 2]}`;
                    votes.set(key, (votes.get(key) ?? 0) + 1);
                }
                const best = [...votes.entries()].sort((a, b) => b[1] - a[1])[0][0].split(',').map(Number);
                (by[u.f] ??= []).push(best);
            });
            return by;
        }""")
        page.click("#leave-run")
        page.click("#back-to-menu")
        for f, px in got.items():
            lums.setdefault(f, []).extend(px)
    gray = {}
    for f, px in lums.items():
        ls = sorted(srgb_lum(p) for p in px)
        gray[int(f)] = ls[len(ls) // 2]
    problems = []
    if len(gray) < 4:
        problems.append(f"only {len(gray)} flavors were on the canvas")
    fs = sorted(gray)
    for i, a in enumerate(fs):
        for b in fs[i + 1:]:
            if abs(gray[a] - gray[b]) <= 0.08:
                problems.append(f"flavors {a} and {b} are {gray[a]:.3f} and {gray[b]:.3f} in gray")
    if problems:
        return [f"{name}: {p}" for p in problems]
    print(f"ok: {name}: in gray the rendered flavors read " + ", ".join(f"{gray[f]:.3f}" for f in fs))
    return []


@check
def the_page_draws_between_two_ticks_and_never_past_the_newest(page, name):
    # Sam, 2026-10-06: "whenever the spout opens there is a little bit of
    # framerate lag". The screen draws at its own rate between the last two
    # ticks core sent (web/blend.js). Halfway is halfway, a unit new this tick
    # is where core put it, and two frames that are not one tick apart are
    # not blended.
    got = page.evaluate("""async () => {
        const b = await import('./blend.js');
        // id, x, y, r, flavor + 256 * line
        const prev = [7, 0, 1000, 10, 1];
        const cur = [7, 200, 600, 30, 1, 8, 50, 50, 5, 2];
        const u = Array.from(b.blendUnits(prev, cur, 0.5));
        const f0 = { tick: 9, lines: [{ seat: 0, travel: 100, spouts: [{ x: 0, y: 0, pivot: [0, 0], tip: [0, 10] }], cups: [{ order: 3, x: 40, base: 0, floor: 0, rim: 0 }] }] };
        const f1 = { tick: 10, lines: [{ seat: 0, travel: 140, spouts: [{ x: 0, y: 0, pivot: [0, 0], tip: [10, 0] }], cups: [{ order: 3, x: 80, base: 0, floor: 0, rim: 0 }, { order: 4, x: -60, base: 0, floor: 0, rim: 0 }] }] };
        const f = b.blendFrame(f0, f1, 0.25);
        const far = b.blendFrame({ ...f0, tick: 7 }, f1, 0.25);
        return { u, travel: f.lines[0].travel, cups: f.lines[0].cups.map((c) => c.x), tip: f.lines[0].spouts[0].tip, far: far === f1 };
    }""")
    want = {"u": [7, 100, 800, 20, 1, 8, 50, 50, 5, 2], "travel": 110, "cups": [50, -60], "tip": [2.5, 7.5], "far": True}
    if got != want:
        return [f"{name}: blending gave {got}, not {want}"]
    print(f"ok: {name}: halfway is halfway, a new unit stays where core put it, and frames two ticks apart are not blended")
    return []


@check
def the_soundtrack_names_its_track_and_the_game_plays_without_it(page, name):
    # Sam, 2026-10-06: "have it play the following song from soundcloud as
    # you play". The player's frame names the track; with SoundCloud refused
    # (as the gate refuses it) the widget never loads, a run still starts and
    # plays, and the panel's words are copy strings.
    page.reload(wait_until="load")
    page.wait_for_function("document.body.dataset.ready === '1'")
    problems = []
    src = page.get_attribute("#music-player", "src") or ""
    if TRACK not in src or not soundtrack(src):
        problems.append(f"the player's frame is {src!r}")
    page.click("#menu-missions")
    page.click("#mission-m_first_pour")
    page.click("#open-mission")
    page.click("#start-mission")
    page.wait_for_function("window.slushline.tick() > 30")
    state = page.evaluate("window.slushline.music()")
    if state["loaded"] or not state["wanted"]:
        problems.append(f"with SoundCloud refused the music reads {state}")
    page.click("#music-fold")
    if page.is_visible("#music-player"):
        problems.append("the fold button left the player showing")
    problems += lines_not_in_copy(page, "the soundtrack panel, folded")
    page.click("#music-fold")
    page.click("#leave-run")
    page.click("#back-to-menu")
    if problems:
        return [f"{name}: {p}" for p in problems]
    print(f"ok: {name}: the player names the track; with SoundCloud refused a run still plays, and the panel folds")
    return []


@check
def a_missed_cup_plays_sam_s_recording(page, name):
    # Sam, 2026-10-07: his own "Oh no" for a missed cup, and "Darn" for its
    # flair (web/voice). A test cannot hear; it checks both files are served
    # and decode, and that MISS plays the first and DARN the second.
    page.reload(wait_until="load")
    page.wait_for_function("document.body.dataset.ready === '1'")
    page.click("#menu-missions")
    page.click("#mission-m_first_pour")
    page.click("#open-mission")
    page.click("#start-mission")
    page.wait_for_function("window.slushline.tick() > 2")
    problems = []
    for f in ("voice/oh-no.wav", "voice/darn.wav"):
        head = page.evaluate(f"fetch('{f}').then(r => r.ok ? r.arrayBuffer() : null).then(b => b && String.fromCharCode(...new Uint8Array(b, 0, 4)))")
        if head != "RIFF":
            problems.append(f"{f} is not served as a WAV file ({head!r})")
    try:
        page.wait_for_function("window.slushline.voiceClips().loaded.length === 2", timeout=10000)
    except Exception:
        problems.append(f"the clips did not decode: {page.evaluate('window.slushline.voiceClips()')}")
    played = []
    for word in ("miss", "darn"):
        page.evaluate(f"window.slushline.performJudgement('{word}')")
        played.append((page.evaluate("window.slushline.voiceClips().last"), page.inner_text("#judge")))
    want = [("voice/oh-no.wav", COPY["judge"]["miss"]), ("voice/darn.wav", COPY["judge"]["darn"])]
    if not problems and played != want:
        problems.append(f"MISS and DARN played and showed {played}, not {want}")
    page.click("#leave-run")
    page.click("#back-to-menu")
    if problems:
        return [f"{name}: {p}" for p in problems]
    print(f"ok: {name}: both clips are served and decode; MISS played oh-no and DARN played darn")
    return []


@check
def a_file_that_is_not_a_replay_is_refused_with_a_sentence(page, name):
    junk = ROOT / "testing" / "replays" / "not-a-replay.txt"
    junk.write_text("this is not a replay")
    try:
        with page.expect_file_chooser() as fc:
            page.click("#menu-replay")
        fc.value.set_files(str(junk))
        page.wait_for_selector("#notice:not([hidden])")
        got = page.inner_text("#notice").strip()
    finally:
        junk.unlink()
    want = COPY["replay"]["error"]["format"].replace("{game}", COPY["game"]["name"])
    if got != want:
        return [f"{name}: the refusal reads {got!r}"]
    print(f"ok: {name}: a file that is not a replay is refused: {got!r}")
    return []


def run(engine_names):
    httpd = None if LIVE else serve()
    failures = []
    with sync_playwright() as pw:
        for name in engine_names:
            browser = getattr(pw, name).launch()
            ctx, page, problems, offsite = open_page(browser)
            for fn in CHECKS:
                try:
                    failures += fn(page, name)
                except Exception:
                    failures.append(f"{name}: {fn.__name__} raised\n{traceback.format_exc()}")
            if problems:
                failures.append(f"{name}: console errors: {problems}")
            else:
                print(f"ok: {name}: no console error")
            if offsite:
                failures.append(f"{name}: requests left the origin: {offsite}")
            else:
                print(f"ok: {name}: no request left the origin")
            ctx.close()
            browser.close()
    if httpd:
        httpd.shutdown()
    if failures:
        print("\nFAILED:\n" + "\n".join(failures))
        sys.exit(1)
    print("\ngate: every check passed")


if __name__ == "__main__":
    run(sys.argv[1:] or ["chromium", "firefox", "webkit"])
