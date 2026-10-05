"""Drive the built page in real browsers and walk the deploy gate.

`cargo test` cannot reach any of this: whether the wasm instantiates in a
browser, whether the stamped imports resolve, whether a download produces a
file and a file input feeds it back, and whether the browser's arithmetic is
the native build's arithmetic.

A console error or a request that leaves the origin fails the run, so "nothing
is uploaded" is tested rather than asserted. Every visible line of text must
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
    page.on("console", lambda m: problems.append(f"console.{m.type}: {m.text}")
            if m.type == "error" else None)
    page.on("pageerror", lambda e: problems.append(f"pageerror: {e}"))
    # blob: and data: URLs are the page talking to itself; anything else that
    # does not start with the origin has left it.
    page.on("request", lambda r: offsite.append(r.url)
            if not (r.url.startswith(ORIGIN) or r.url.startswith(("blob:", "data:"))) else None)
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
    page.click("#mission-m_first_pour")
    problems += lines_not_in_copy(page, "the mission card")
    page.click("#start-mission")
    page.wait_for_function("window.slushline.tick() > 5")
    # A player's hands for a moment, then the timer pilot to the end.
    page.keyboard.down("a")
    page.wait_for_timeout(300)
    page.keyboard.up("a")
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
    print(f"ok: {name}: the belt ran at {faster} % after the faster key and {slower} % after the slower; the first mission played to {verdict!r} ({final}); its replay ({size} bytes) loaded after a reload, ends on {got}, and shows the same result")
    page.click("#to-missions")
    page.click("#back-to-menu")
    return []


@check
def the_save_keeps_a_pass_and_round_trips_through_a_file(page, name):
    page.click("#menu-missions")
    opened = page.locator("#mission-m_tail").count()
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
    if page.locator("#mission-m_tail").count():
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
    if not page.locator("#mission-m_tail").count():
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
    page.evaluate("""localStorage.setItem('slushline.save', JSON.stringify({format:'slushline.save',version:1,best:{},
        passed:['m_first_pour','m_tail','m_two_spouts','m_half','m_two_one','m_third'],
        keys:{spout_1:'KeyA',spout_2:'KeyS',spout_3:'KeyD',spout_4:'KeyF',belt_slower:'ArrowLeft',belt_faster:'ArrowRight'},options:{short_codes:false}}))""")
    page.reload(wait_until="load")
    page.wait_for_function("document.body.dataset.ready === '1'")
    page.click("#menu-missions")
    page.click("#mission-m_three")
    page.click("#start-mission")
    page.wait_for_function("window.slushline.tick() > 2")
    page.evaluate("window.slushline.autoplay(1150)")
    page.wait_for_timeout(100)
    lums = page.evaluate("""() => {
        const c = document.getElementById('stage');
        const g = c.getContext('2d');
        const by = {};
        const us = window.slushline.unitPixels();
        us.forEach((u, i) => {
            if (u.r < 3) return;
            const x = Math.round(u.x + u.r * 0.45), y = Math.round(u.y + u.r * 0.45);
            // Units are drawn in order; a point a later unit (or its
            // outline) covers shows that unit, not this one.
            // Fills are drawn after every outline, so only a later unit's
            // fill can cover this point.
            for (let j = i + 1; j < us.length; j += 1) {
                const v = us[j];
                if (v.f !== u.f && (v.x - x) ** 2 + (v.y - y) ** 2 < (v.r + 0.5) ** 2) return;
            }
            const d = g.getImageData(x, y, 1, 1).data;
            (by[u.f] ??= []).push([d[0], d[1], d[2]]);
        });
        return by;
    }""")
    page.click("#leave-run")
    page.click("#back-to-menu")
    gray = {}
    for f, px in lums.items():
        ls = sorted(srgb_lum(p) for p in px)
        gray[int(f)] = ls[len(ls) // 2]
    problems = []
    if len(gray) < 3:
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
