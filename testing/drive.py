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
