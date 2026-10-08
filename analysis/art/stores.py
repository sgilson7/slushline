#!/usr/bin/env python3
"""A small scene for each mission on story mode's map (Sam, 2026-10-07:
"each enemy is a little scene on top of a greater background of layers ...
each scene is a different store in a differnet location").

After Vagrancy's analysis/art/seals.py: each mission is an island of its
row's ground (its region, regions.py) with a corner store on it and props
from its place line (copy `missions.list.<id>.place`). Every store is the
same generic chain: a flat roof, a scalloped awning of one color, a window
with the slush machine in it, and a roof sign that is a picture of a slush
cup. No store carries a word, a logo or stripes in any real chain's colors
(TONE.md rule 4). Colors come from data/palette.json `art`; the page greys
a locked scene and lights an open one.

Each store's name (copy `missions.list.<id>.store`; Sam, 2026-10-07: "the
name of the store thats written either on the banner, facade, sign, a little
stylized board like a surf board, a big golf club, a moving tube dude") is
painted on a prop in its scene, in Lilita One (analysis/art/fonts, SIL Open
Font License; LICENSES.md), exactly as the copy file has it. The names drawn
are written to web/art/store/names.json, and tests/world.rs fails when a name
in the copy file and the name in the picture differ.

    python3 analysis/art/stores.py           # writes web/art/store/<id>.png
    python3 analysis/art/stores.py s_jet     # just one
"""
import json, math, subprocess, sys, tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
PAL = json.loads((ROOT / "data" / "palette.json").read_text())["art"]
OUT = ROOT / "web" / "art" / "store"
FONTS = ROOT / "analysis" / "art" / "fonts"
COPY = json.loads((ROOT / "data" / "copy.en.json").read_text())
CUPS = json.loads((ROOT / "data" / "cups.json").read_text())
W, H = 4.0, 3.0   # cm
PX = 320          # pixels across, drawn at twice the size shown
LINE = "draw=ink, line width=0.7pt, line join=round, line cap=round"
THIN = "draw=ink, line width=0.45pt, line cap=round"
# The ground of each row's island, by depth (regions.py's order).
GROUND = ["sage", "sand", "sand", "stone", "snow", "sand", "stone", "shadow", "slate"]


def colors():
    return "\n".join(f"\\definecolor{{{k.replace('_','')}}}{{HTML}}{{{v[1:]}}}" for k, v in PAL.items() if not k.startswith("_"))


def P(x, y):
    return f"({x:.3f},{y:.3f})"


def poly(pts, fill, style=LINE):
    return f"\\filldraw[fill={fill}, {style}] " + " -- ".join(P(x, y) for x, y in pts) + " -- cycle;"


def rect(x, y, w, h, fill, style=LINE):
    return f"\\filldraw[fill={fill}, {style}] {P(x, y)} rectangle {P(x + w, y + h)};"


def circ(x, y, r, fill, style=LINE):
    return f"\\filldraw[fill={fill}, {style}] {P(x, y)} circle ({r:.3f});"


def ell(x, y, rx, ry, fill, style=LINE):
    return f"\\filldraw[fill={fill}, {style}] {P(x, y)} ellipse ({rx:.3f} and {ry:.3f});"


def line(*pts, style=THIN):
    return f"\\draw[{style}] " + " -- ".join(P(x, y) for x, y in pts) + ";"


def curve(a, c1, c2, b, style=THIN):
    return f"\\draw[{style}] {P(*a)} .. controls {P(*c1)} and {P(*c2)} .. {P(*b)};"


# --- the store --------------------------------------------------------------

def cup_sign(x, y, s):
    """The chain's mark: a slush cup with a dome lid and a straw, no words."""
    return [rect(x - 0.32 * s, y, 0.64 * s, 0.5 * s, "cream"),
            poly([(x - 0.14 * s, y + 0.08 * s), (x + 0.14 * s, y + 0.08 * s), (x + 0.18 * s, y + 0.32 * s), (x - 0.18 * s, y + 0.32 * s)], "rose", THIN),
            f"\\filldraw[fill=teal, {THIN}] {P(x - 0.19 * s, y + 0.32 * s)} arc (180:0:{0.19 * s:.3f});",
            line((x + 0.04 * s, y + 0.5 * s), (x + 0.12 * s, y + 0.62 * s), style="draw=ink, line width=0.9pt, line cap=round")]


def store(x, y, s=1.0, wall="cream", awning="teal", sign=True, floors=1, name=None):
    """A corner store standing on (x, y), its front about 1.5 s wide."""
    w, h = 1.5 * s, 0.95 * s
    o = []
    for k in range(floors):
        o.append(rect(x - w / 2, y + k * h, w, h, wall))
        if k:
            for i in range(3):
                o.append(rect(x - w / 2 + 0.15 * s + i * 0.45 * s, y + k * h + 0.3 * s, 0.3 * s, 0.35 * s, "skylight", THIN))
    top = y + floors * h
    o.append(rect(x - w / 2 - 0.05 * s, top, w + 0.1 * s, 0.1 * s, "peachdark"))
    # The window, with the slush machine in it: two bowls and their spouts.
    o.append(rect(x - w / 2 + 0.12 * s, y + 0.2 * s, 0.85 * s, 0.5 * s, "skylight"))
    for i, c in enumerate(["rose", "teal"]):
        bx = x - w / 2 + 0.3 * s + i * 0.45 * s
        o.append(rect(bx - 0.13 * s, y + 0.38 * s, 0.26 * s, 0.24 * s, c, THIN))
        o.append(rect(bx - 0.03 * s, y + 0.26 * s, 0.06 * s, 0.12 * s, "slate", THIN))
    o.append(rect(x + w / 2 - 0.42 * s, y, 0.3 * s, 0.62 * s, "wood"))
    o.append(circ(x + w / 2 - 0.18 * s, y + 0.3 * s, 0.025 * s, "gold", THIN))
    # The awning: one color, scalloped.
    ay = y + 0.72 * s
    o.append(rect(x - w / 2 - 0.05 * s, ay, w + 0.1 * s, 0.14 * s, awning))
    n = 7
    for i in range(n):
        cx = x - w / 2 - 0.05 * s + (i + 0.5) * (w + 0.1 * s) / n
        o.append(f"\\filldraw[fill={awning}, {THIN}] {P(cx - (w + 0.1 * s) / n / 2, ay)} arc (180:360:{(w + 0.1 * s) / n / 2:.3f});")
    if name:
        o += roof_board(x, top + 0.1 * s, w + 0.25 * s, name)
    elif sign:
        o.append(line((x - 0.2 * s, top + 0.1 * s), (x - 0.2 * s, top + 0.2 * s), style="draw=ink, line width=0.7pt"))
        o.append(line((x + 0.2 * s, top + 0.1 * s), (x + 0.2 * s, top + 0.2 * s), style="draw=ink, line width=0.7pt"))
        o += cup_sign(x, top + 0.2 * s, s)
    return o


# --- props: each stands on (x, y), about s centimeters tall ------------------

def gull(x, y, s=1.0):
    return [curve((x - 0.15 * s, y), (x - 0.1 * s, y + 0.08 * s), (x - 0.04 * s, y + 0.08 * s), (x, y)),
            curve((x, y), (x + 0.04 * s, y + 0.08 * s), (x + 0.1 * s, y + 0.08 * s), (x + 0.15 * s, y))]


def wind(x, y, s=1.0, n=3, back=False):
    o = []
    for k in range(n):
        yy = y + k * 0.16 * s
        d = -1 if back else 1
        o.append(curve((x, yy), (x + d * 0.3 * s, yy + 0.06 * s), (x + d * 0.5 * s, yy - 0.06 * s), (x + d * 0.75 * s, yy + 0.04 * s)))
    return o


def zigzag(x, y, s=1.0):
    pts = [(x + k * 0.18 * s, y + (0.12 * s if k % 2 else 0)) for k in range(7)]
    return [line(*pts, style="draw=ink, line width=0.6pt"), poly([pts[-1], (pts[-1][0] - 0.08 * s, pts[-1][1] + 0.08 * s), (pts[-1][0] + 0.06 * s, pts[-1][1] - 0.04 * s)], "ink", THIN)]


def sign(x, y, s=1.0, fill="gold"):
    return [line((x, y), (x, y + 0.9 * s), style="draw=ink, line width=0.9pt"), rect(x - 0.25 * s, y + 0.75 * s, 0.5 * s, 0.32 * s, fill)]


def lamp(x, y, s=1.0):
    return [line((x, y), (x, y + 1.0 * s), (x + 0.2 * s, y + 1.0 * s), style="draw=ink, line width=0.9pt"), circ(x + 0.22 * s, y + 0.96 * s, 0.06 * s, "cloud", THIN)]


def turbine(x, y, s=1.0):
    hy = y + 1.4 * s
    o = [poly([(x - 0.05 * s, y), (x + 0.05 * s, y), (x + 0.025 * s, hy), (x - 0.025 * s, hy)], "cloud", THIN)]
    for a in (90, 210, 330):
        ra = math.radians(a)
        o.append(poly([(x, hy), (x + math.cos(ra + 0.15) * 0.15 * s, hy + math.sin(ra + 0.15) * 0.15 * s), (x + math.cos(ra) * 0.6 * s, hy + math.sin(ra) * 0.6 * s)], "cloud", THIN))
    return o + [circ(x, hy, 0.05 * s, "slate", THIN)]


def water(x, y, w, h=0.25, fill="teal"):
    o = [rect(x, y, w, h, fill)]
    for k in range(int(w / 0.35)):
        xx = x + 0.1 + k * 0.35
        o.append(curve((xx, y + h * 0.5), (xx + 0.06, y + h * 0.6), (xx + 0.12, y + h * 0.6), (xx + 0.18, y + h * 0.5), style="draw=tealdark, line width=0.4pt"))
    return o


def pier(x, y, w, s=1.0):
    o = water(x - 0.1, y - 0.25 * s, w + 0.2, 0.3 * s)
    o.append(rect(x, y, w, 0.08 * s, "wood"))
    for k in range(int(w / 0.3) + 1):
        o.append(line((x + k * 0.3, y), (x + k * 0.3, y - 0.22 * s), style="draw=ink, line width=0.8pt"))
    return o


def big_wheel(x, y, s=1.0):
    r = 0.55 * s
    cy = y + r + 0.2 * s
    o = [line((x - 0.3 * s, y), (x, cy), (x + 0.3 * s, y), style="draw=ink, line width=0.9pt"), f"\\draw[ink, line width=0.9pt] {P(x, cy)} circle ({r:.3f});"]
    for a in range(0, 360, 45):
        ra = math.radians(a)
        o.append(line((x, cy), (x + math.cos(ra) * r, cy + math.sin(ra) * r)))
        o.append(rect(x + math.cos(ra) * r - 0.06 * s, cy + math.sin(ra) * r - 0.1 * s, 0.12 * s, 0.1 * s, ["peach", "gold", "teal"][a // 45 % 3], THIN))
    return o


def pump(x, y, s=1.0):
    return [rect(x - 0.15 * s, y, 0.3 * s, 0.6 * s, "rose"), rect(x - 0.1 * s, y + 0.35 * s, 0.2 * s, 0.15 * s, "skylight", THIN),
            curve((x + 0.15 * s, y + 0.45 * s), (x + 0.3 * s, y + 0.45 * s), (x + 0.3 * s, y + 0.15 * s), (x + 0.22 * s, y + 0.1 * s))]


def canopy(x, y, w, h, s=1.0):
    return [line((x, y), (x, y + h), style="draw=ink, line width=0.9pt"), line((x + w, y), (x + w, y + h), style="draw=ink, line width=0.9pt"),
            rect(x - 0.1 * s, y + h, w + 0.2 * s, 0.14 * s, "cream")]


def truck(x, y, s=1.0, flip=False):
    d = -1 if flip else 1
    o = [rect(x - (0.0 if d > 0 else 0.9 * s), y + 0.1 * s, 0.9 * s, 0.45 * s, "cream"),
         poly([(x + d * 0.9 * s, y + 0.1 * s), (x + d * 1.25 * s, y + 0.1 * s), (x + d * 1.25 * s, y + 0.35 * s), (x + d * 1.1 * s, y + 0.45 * s), (x + d * 0.9 * s, y + 0.45 * s)], "peach")]
    for wx in (0.2, 0.7, 1.08):
        o.append(circ(x + d * wx * s, y + 0.1 * s, 0.09 * s, "slate", THIN))
    return o


def pylon(x, y, s=1.0):
    t = y + 1.5 * s
    return [line((x - 0.25 * s, y), (x - 0.05 * s, t), (x + 0.05 * s, t), (x + 0.25 * s, y), style="draw=ink, line width=0.8pt"),
            line((x - 0.2 * s, y + 0.3 * s), (x + 0.15 * s, y + 0.75 * s)), line((x + 0.2 * s, y + 0.3 * s), (x - 0.15 * s, y + 0.75 * s)),
            line((x - 0.12 * s, y + 0.9 * s), (x + 0.1 * s, y + 1.2 * s)), line((x + 0.12 * s, y + 0.9 * s), (x - 0.1 * s, y + 1.2 * s)),
            line((x - 0.4 * s, t - 0.15 * s), (x + 0.4 * s, t - 0.15 * s), style="draw=ink, line width=0.8pt")]


def wires(x0, x1, y, sag=0.15):
    return [curve((x0, y), ((2 * x0 + x1) / 3, y - sag), ((x0 + 2 * x1) / 3, y - sag), (x1, y)),
            curve((x0, y - 0.08), ((2 * x0 + x1) / 3, y - 0.08 - sag), ((x0 + 2 * x1) / 3, y - 0.08 - sag), (x1, y - 0.08))]


def spire(x, y, w, h, fill="peachdark"):
    return [poly([(x - w / 2, y), (x - w * 0.3, y + h), (x + w * 0.3, y + h), (x + w / 2, y)], fill), line((x - w * 0.4, y + h * 0.5), (x + w * 0.4, y + h * 0.5))]


def cactus(x, y, s=1.0):
    return [rect(x - 0.07 * s, y, 0.14 * s, 0.6 * s, "sage"), rect(x + 0.07 * s, y + 0.25 * s, 0.15 * s, 0.08 * s, "sage", THIN), rect(x + 0.14 * s, y + 0.25 * s, 0.08 * s, 0.22 * s, "sage", THIN)]


def mine_cart(x, y, s=1.0):
    o = [line((x - 0.5 * s, y), (x + 0.5 * s, y), style="draw=ink, line width=0.8pt")]
    for k in range(5):
        o.append(line((x - 0.45 * s + k * 0.22 * s, y - 0.03 * s), (x - 0.45 * s + k * 0.22 * s, y + 0.03 * s)))
    o.append(poly([(x - 0.3 * s, y + 0.38 * s), (x + 0.3 * s, y + 0.38 * s), (x + 0.22 * s, y + 0.08 * s), (x - 0.22 * s, y + 0.08 * s)], "wood"))
    o += [circ(x - 0.15 * s, y + 0.07 * s, 0.06 * s, "slate", THIN), circ(x + 0.15 * s, y + 0.07 * s, 0.06 * s, "slate", THIN)]
    for k in range(4):
        o.append(circ(x - 0.15 * s + k * 0.1 * s, y + 0.42 * s, 0.06 * s, "shadow", THIN))
    return o


def mine_entry(x, y, s=1.0):
    return [poly([(x - 0.55 * s, y), (x - 0.4 * s, y + 0.8 * s), (x + 0.4 * s, y + 0.8 * s), (x + 0.55 * s, y)], "peachdark"),
            f"\\filldraw[fill=shadow, {LINE}] {P(x - 0.25 * s, y)} -- {P(x - 0.25 * s, y + 0.35 * s)} arc (180:0:{0.25 * s:.3f}) -- {P(x + 0.25 * s, y)} -- cycle;",
            line((x - 0.3 * s, y), (x - 0.3 * s, y + 0.62 * s), (x + 0.3 * s, y + 0.62 * s), (x + 0.3 * s, y), style="draw=wood, line width=1.2pt")]


def rocket(x, y, s=1.0):
    return [line((x - 0.35 * s, y), (x - 0.35 * s, y + 1.6 * s), style="draw=ink, line width=0.9pt"),
            line((x - 0.35 * s, y + 0.5 * s), (x - 0.12 * s, y + 0.5 * s)), line((x - 0.35 * s, y + 1.1 * s), (x - 0.12 * s, y + 1.1 * s)),
            rect(x - 0.12 * s, y + 0.2 * s, 0.24 * s, 1.2 * s, "cloud"),
            poly([(x - 0.12 * s, y + 1.4 * s), (x, y + 1.75 * s), (x + 0.12 * s, y + 1.4 * s)], "rose"),
            poly([(x - 0.12 * s, y + 0.2 * s), (x - 0.25 * s, y), (x - 0.12 * s, y + 0.45 * s)], "rose"),
            poly([(x + 0.12 * s, y + 0.2 * s), (x + 0.25 * s, y), (x + 0.12 * s, y + 0.45 * s)], "rose"),
            circ(x, y + 1.05 * s, 0.06 * s, "teal", THIN)]


def tower(x, y, w, h, fill="stone", win="skylight"):
    o = [rect(x - w / 2, y, w, h, fill)]
    cols, rows = max(1, int(w / 0.22)), max(1, int(h / 0.28))
    for i in range(cols):
        for j in range(rows):
            o.append(rect(x - w / 2 + 0.07 + i * (w - 0.1) / cols, y + 0.12 + j * (h - 0.15) / rows, 0.1, 0.13, win, THIN))
    return o


def diner_sign(x, y, s=1.0):
    return [line((x, y), (x, y + 1.0 * s), style="draw=ink, line width=0.9pt"), circ(x, y + 1.15 * s, 0.25 * s, "rose"),
            circ(x, y + 1.15 * s, 0.16 * s, "cream", THIN), rect(x - 0.06 * s, y + 1.4 * s, 0.12 * s, 0.1 * s, "gold", THIN)]


def revolving(x, y, s=1.0):
    return [rect(x - 0.3 * s, y, 0.6 * s, 0.6 * s, "teal"), line((x, y), (x, y + 0.6 * s)), line((x - 0.22 * s, y + 0.1 * s), (x + 0.22 * s, y + 0.5 * s)),
            f"\\draw[{THIN}] {P(x - 0.3 * s, y + 0.62 * s)} arc (180:0:{0.3 * s:.3f});"]


def stands(x, y, w, s=1.0):
    o = []
    for k in range(4):
        o.append(rect(x, y + k * 0.22 * s, w - k * 0.12 * s, 0.22 * s, "stone", THIN))
        for i in range(int((w - k * 0.12 * s) / 0.14)):
            o.append(circ(x + 0.07 + i * 0.14, y + k * 0.22 * s + 0.29 * s, 0.05 * s, ["peach", "teal", "gold", "rose"][(i + k) % 4], THIN))
    return o


def floodlight(x, y, s=1.0):
    return [line((x, y), (x, y + 1.6 * s), style="draw=ink, line width=0.9pt"), rect(x - 0.18 * s, y + 1.6 * s, 0.36 * s, 0.2 * s, "cloud")]


def ship(x, y, s=1.0):
    o = [poly([(x - 0.8 * s, y + 0.3 * s), (x + 0.8 * s, y + 0.3 * s), (x + 0.65 * s, y), (x - 0.7 * s, y)], "slate")]
    for k, c in enumerate(["rose", "teal", "gold", "peach", "teal"]):
        o.append(rect(x - 0.6 * s + k * 0.24 * s, y + 0.3 * s, 0.22 * s, 0.18 * s, c, THIN))
    o.append(rect(x + 0.45 * s, y + 0.3 * s, 0.2 * s, 0.38 * s, "cream", THIN))
    return o


def crane(x, y, s=1.0):
    return [line((x, y), (x, y + 1.6 * s), style="draw=ink, line width=1.1pt"), line((x - 0.3 * s, y + 1.6 * s), (x + 0.8 * s, y + 1.6 * s), style="draw=ink, line width=1.1pt"),
            line((x + 0.7 * s, y + 1.6 * s), (x + 0.7 * s, y + 1.1 * s)), rect(x + 0.6 * s, y + 0.95 * s, 0.2 * s, 0.15 * s, "rose", THIN)]


def anchor(x, y, s=1.0):
    return [line((x, y), (x, y + 0.4 * s), style="draw=ink, line width=1.1pt"), circ(x, y + 0.45 * s, 0.05 * s, "none", "draw=ink, line width=0.9pt"),
            f"\\draw[draw=ink, line width=1.1pt] {P(x - 0.18 * s, y + 0.12 * s)} arc (180:360:{0.18 * s:.3f});", line((x - 0.12 * s, y + 0.3 * s), (x + 0.12 * s, y + 0.3 * s))]


def firehouse(x, y, s=1.0):
    return [rect(x - 0.55 * s, y, 1.1 * s, 0.9 * s, "rose"), poly([(x - 0.6 * s, y + 0.9 * s), (x, y + 1.15 * s), (x + 0.6 * s, y + 0.9 * s)], "peachdark"),
            f"\\filldraw[fill=shadow, {LINE}] {P(x - 0.3 * s, y)} -- {P(x - 0.3 * s, y + 0.45 * s)} arc (180:0:{0.3 * s:.3f}) -- {P(x + 0.3 * s, y)} -- cycle;",
            circ(x, y + 0.98 * s, 0.07 * s, "gold", THIN)]


def hydrant(x, y, s=1.0, spray=False):
    o = [rect(x - 0.08 * s, y, 0.16 * s, 0.3 * s, "rose"), f"\\filldraw[fill=rose, {THIN}] {P(x - 0.08 * s, y + 0.3 * s)} arc (180:0:{0.08 * s:.3f});",
         rect(x - 0.14 * s, y + 0.15 * s, 0.28 * s, 0.06 * s, "rose", THIN)]
    if spray:
        for k in range(3):
            o.append(curve((x + 0.14 * s, y + 0.18 * s), (x + 0.4 * s, y + (0.4 + k * 0.1) * s), (x + 0.6 * s, y + (0.35 + k * 0.1) * s), (x + 0.75 * s, y + (0.1 + k * 0.05) * s), style="draw=teal, line width=0.9pt"))
    return o


def pine(x, y, s=1.0, snowy=True):
    o = [rect(x - 0.04 * s, y, 0.08 * s, 0.15 * s, "wood", THIN)]
    for k in range(3):
        b = y + 0.12 * s + k * 0.22 * s
        o.append(poly([(x - (0.3 - k * 0.07) * s, b), (x + (0.3 - k * 0.07) * s, b), (x, b + 0.38 * s)], "sagedark", THIN))
    if snowy:
        o.append(poly([(x - 0.09 * s, y + 0.75 * s), (x + 0.09 * s, y + 0.75 * s), (x, y + 0.92 * s)], "snow", THIN))
    return o


def skis(x, y, s=1.0):
    return [line((x, y), (x + 0.1 * s, y + 0.8 * s), style="draw=rose, line width=1.6pt"), line((x + 0.12 * s, y), (x + 0.22 * s, y + 0.8 * s), style="draw=teal, line width=1.6pt"),
            line((x + 0.4 * s, y), (x + 0.36 * s, y + 0.7 * s), style="draw=ink, line width=0.7pt"), line((x + 0.5 * s, y), (x + 0.46 * s, y + 0.7 * s), style="draw=ink, line width=0.7pt")]


def peak(x, y, w, h):
    return [poly([(x - w / 2, y), (x, y + h), (x + w / 2, y)], "slate"), poly([(x - w * 0.14, y + h * 0.72), (x, y + h), (x + w * 0.14, y + h * 0.72), (x, y + h * 0.66)], "snow", THIN)]


def ice(x, y, rx, ry):
    return [ell(x, y, rx, ry, "cloud"), line((x - rx * 0.5, y), (x - rx * 0.2, y + ry * 0.3), (x + rx * 0.1, y - ry * 0.1)), line((x + rx * 0.3, y + ry * 0.4), (x + rx * 0.5, y))]


def skater(x, y, s=1.0):
    return [circ(x, y + 0.55 * s, 0.07 * s, "peach", THIN), line((x, y + 0.48 * s), (x, y + 0.22 * s), (x - 0.12 * s, y + 0.02 * s), style="draw=ink, line width=0.9pt"),
            line((x, y + 0.22 * s), (x + 0.15 * s, y + 0.1 * s)), line((x - 0.15 * s, y + 0.4 * s), (x + 0.15 * s, y + 0.38 * s)), rect(x - 0.08 * s, y + 0.3 * s, 0.16 * s, 0.18 * s, "rose", THIN)]


def dome(x, y, s=1.0):
    return [rect(x - 0.45 * s, y, 0.9 * s, 0.45 * s, "cream"), f"\\filldraw[fill=snow, {LINE}] {P(x - 0.45 * s, y + 0.45 * s)} arc (180:0:{0.45 * s:.3f}) -- cycle;",
            poly([(x - 0.06 * s, y + 0.55 * s), (x + 0.06 * s, y + 0.55 * s), (x + 0.12 * s, y + 0.88 * s), (x - 0.0 * s, y + 0.9 * s)], "shadow", THIN)]


def star(x, y, r=0.05):
    pts = []
    for k in range(10):
        a = math.pi / 2 + k * math.pi / 5
        rr = r if k % 2 == 0 else r * 0.45
        pts.append((x + math.cos(a) * rr, y + math.sin(a) * rr))
    return [poly(pts, "gold", THIN)]


def cable_car(x0, y0, x1, y1, t=0.5, s=1.0):
    cx, cy = x0 + (x1 - x0) * t, y0 + (y1 - y0) * t
    return [line((x0, y0), (x1, y1), style="draw=ink, line width=0.7pt"), line((cx, cy), (cx, cy - 0.2 * s)),
            rect(cx - 0.22 * s, cy - 0.5 * s, 0.44 * s, 0.32 * s, "rose"), rect(cx - 0.15 * s, cy - 0.4 * s, 0.3 * s, 0.14 * s, "skylight", THIN)]


def lift_pole(x, y, h, s=1.0):
    return [line((x, y), (x, y + h), style="draw=ink, line width=1.1pt"), line((x - 0.15 * s, y + h), (x + 0.15 * s, y + h), style="draw=ink, line width=1.1pt")]


def chair(x, y, s=1.0):
    return [line((x, y), (x, y - 0.3 * s)), line((x - 0.15 * s, y - 0.3 * s), (x - 0.15 * s, y - 0.45 * s), (x + 0.15 * s, y - 0.45 * s), style="draw=ink, line width=0.9pt"),
            rect(x - 0.15 * s, y - 0.5 * s, 0.3 * s, 0.05 * s, "peach", THIN)]


def strata(x, y, w, h):
    o = []
    cs = ["peachdark", "sand", "stone", "peach", "slate", "sand"]
    for k in range(6):
        y0 = y + k * h / 6
        o.append(poly([(x + k * 0.06, y0), (x + w - k * 0.06, y0), (x + w - (k + 1) * 0.06, y0 + h / 6), (x + (k + 1) * 0.06, y0 + h / 6)], cs[k], THIN))
    return o


def snow_cannon(x, y, s=1.0, spray=True):
    o = [line((x, y), (x, y + 0.5 * s), style="draw=ink, line width=1pt"),
         poly([(x - 0.05 * s, y + 0.45 * s), (x + 0.35 * s, y + 0.75 * s), (x + 0.45 * s, y + 0.65 * s), (x + 0.05 * s, y + 0.35 * s)], "cloud")]
    if spray:
        for k in range(6):
            o.append(circ(x + 0.55 * s + k * 0.1 * s, y + 0.8 * s + (k % 2) * 0.08 * s, 0.04 * s, "snow", THIN))
    return o


def palm(x, y, s=1.0, lean=0.3):
    tx, ty = x + lean * s, y + 1.2 * s
    o = [curve((x, y), (x + lean * 0.5 * s, y + 0.5 * s), (x + lean * s, y + 0.9 * s), (tx, ty), style="draw=wood, line width=2.0pt")]
    for a in (15, 60, 120, 165, 210, 330):
        ra = math.radians(a)
        ex, ey = tx + math.cos(ra) * 0.6 * s, ty + math.sin(ra) * 0.3 * s - 0.18 * s
        o.append(f"\\filldraw[fill=sagedark, {THIN}] {P(tx, ty)} .. controls {P((tx + ex) / 2, ty + 0.25 * s)} .. {P(ex, ey)} .. controls {P((tx + ex) / 2, ty + 0.05 * s)} .. cycle;")
    return o


def hut(x, y, s=1.0):
    return [line((x - 0.5 * s, y), (x - 0.5 * s, y + 0.3 * s), style="draw=wood, line width=1.4pt"), line((x + 0.5 * s, y), (x + 0.5 * s, y + 0.3 * s), style="draw=wood, line width=1.4pt"),
            rect(x - 0.6 * s, y + 0.3 * s, 1.2 * s, 0.6 * s, "sand"), poly([(x - 0.8 * s, y + 0.9 * s), (x, y + 1.35 * s), (x + 0.8 * s, y + 0.9 * s)], "olive"),
            rect(x - 0.4 * s, y + 0.45 * s, 0.5 * s, 0.3 * s, "skylight", THIN),
            rect(x - 0.35 * s, y + 0.5 * s, 0.15 * s, 0.15 * s, "rose", THIN), rect(x - 0.15 * s, y + 0.5 * s, 0.15 * s, 0.15 * s, "teal", THIN),
            cup_sign(x, y + 1.0 * s, 0.6 * s)][:-1] + cup_sign(x + 0.35 * s, y + 0.95 * s, 0.55 * s)


def boat(x, y, s=1.0):
    return [poly([(x - 1.1 * s, y + 0.3 * s), (x + 1.1 * s, y + 0.3 * s), (x + 0.9 * s, y), (x - 0.95 * s, y)], "wood"),
            line((x - 1.0 * s, y + 0.15 * s), (x + 1.0 * s, y + 0.15 * s))]


def clock_tower(x, y, s=1.0):
    return [rect(x - 0.25 * s, y, 0.5 * s, 1.6 * s, "stone"), poly([(x - 0.3 * s, y + 1.6 * s), (x, y + 2.0 * s), (x + 0.3 * s, y + 1.6 * s)], "slate"),
            circ(x, y + 1.3 * s, 0.17 * s, "cream"), line((x, y + 1.3 * s), (x, y + 1.42 * s)), line((x, y + 1.3 * s), (x + 0.09 * s, y + 1.26 * s))]


def bridge(x, y, w, s=1.0):
    o = water(x - 0.1, y - 0.05, w + 0.2, 0.3)
    o.append(f"\\filldraw[fill=stone, {LINE}] {P(x, y)} -- {P(x, y + 0.55 * s)} -- {P(x + w, y + 0.55 * s)} -- {P(x + w, y)} -- {P(x + w * 0.75, y)} arc (0:180:{w * 0.25:.3f}) -- cycle;")
    for k in range(int(w / 0.25)):
        o.append(line((x + 0.1 + k * 0.25, y + 0.55 * s), (x + 0.1 + k * 0.25, y + 0.75 * s)))
    o.append(line((x, y + 0.75 * s), (x + w, y + 0.75 * s)))
    return o


def lanterns(x0, x1, y, n=6):
    o = [curve((x0, y), ((2 * x0 + x1) / 3, y - 0.2), ((x0 + 2 * x1) / 3, y - 0.2), (x1, y))]
    for k in range(n):
        t = (k + 0.5) / n
        lx = x0 + (x1 - x0) * t
        ly = y - 0.2 * 4 * t * (1 - t) * 0.75 - 0.12
        o.append(ell(lx, ly, 0.08, 0.11, "rose" if k % 2 else "gold", THIN))
    return o


def stall(x, y, s=1.0):
    o = [line((x - 0.7 * s, y), (x - 0.7 * s, y + 1.0 * s), style="draw=ink, line width=1pt"), line((x + 0.7 * s, y), (x + 0.7 * s, y + 1.0 * s), style="draw=ink, line width=1pt"),
         rect(x - 0.75 * s, y, 1.5 * s, 0.45 * s, "wood")]
    n = 6
    for i in range(n):
        o.append(poly([(x - 0.8 * s + i * 1.6 * s / n, y + 1.0 * s), (x - 0.8 * s + (i + 1) * 1.6 * s / n, y + 1.0 * s), (x - 0.8 * s + (i + 0.5) * 1.6 * s / n, y + 1.25 * s)], "rose" if i % 2 else "gold", THIN))
    o.append(rect(x - 0.8 * s, y + 0.95 * s, 1.6 * s, 0.08 * s, "rose", THIN))
    for i, c in enumerate(["rose", "teal", "gold"]):
        o.append(rect(x - 0.55 * s + i * 0.4 * s, y + 0.45 * s, 0.26 * s, 0.25 * s, c, THIN))
    return o + cup_sign(x, y + 1.25 * s, 0.7 * s)


def lighthouse(x, y, s=1.0):
    o = [poly([(x - 0.3 * s, y), (x + 0.3 * s, y), (x + 0.2 * s, y + 1.6 * s), (x - 0.2 * s, y + 1.6 * s)], "cream")]
    for k in range(3):
        b = y + 0.25 * s + k * 0.45 * s
        o.append(poly([(x - 0.28 * s + 0.02 * k, b), (x + 0.28 * s - 0.02 * k, b), (x + 0.27 * s - 0.025 * k, b + 0.18 * s), (x - 0.27 * s + 0.025 * k, b + 0.18 * s)], "rose", THIN))
    o += [rect(x - 0.17 * s, y + 1.6 * s, 0.34 * s, 0.25 * s, "gold"), poly([(x - 0.24 * s, y + 1.85 * s), (x, y + 2.1 * s), (x + 0.24 * s, y + 1.85 * s)], "rose"),
          f"\\fill[gold, opacity=0.35] {P(x, y + 1.72 * s)} -- {P(x - 1.7 * s, y + 2.2 * s)} -- {P(x - 1.7 * s, y + 1.4 * s)} -- cycle;"]
    return o


def storm_cloud(x, y, s=1.0):
    return [f"\\filldraw[fill=shadow, draw=ink, line width=0.5pt] " + " ".join(f"{P(x + dx * s, y + dy * s)} circle ({r * s:.3f})" for dx, dy, r in [(0, 0, 0.25), (0.28, 0.06, 0.2), (-0.28, 0.04, 0.18), (0.5, -0.02, 0.14)]) + ";"]


def rain(x, y, w, h, n=10):
    return [line((x + k * w / n, y + h - (k % 3) * 0.1), (x + k * w / n - 0.08, y + h - 0.3 - (k % 3) * 0.1), style="draw=cloud, line width=0.5pt") for k in range(n)]


def bolt(x, y, s=1.0):
    return [line((x, y), (x - 0.12 * s, y - 0.25 * s), (x, y - 0.27 * s), (x - 0.14 * s, y - 0.55 * s), style="draw=gold, line width=1.2pt")]


def dust_devil(x, y, s=1.0):
    o = []
    for k in range(5):
        w = (0.08 + k * 0.07) * s
        o.append(f"\\draw[draw=peachdark, line width=0.8pt] {P(x + k * 0.03 * s, y + k * 0.22 * s)} ellipse ({w:.3f} and {0.05 * s:.3f});")
    return o


def vane(x, y, s=1.0):
    t = y + 1.3 * s
    return [line((x, y), (x, t), style="draw=ink, line width=1pt"), line((x - 0.3 * s, t - 0.25 * s), (x + 0.3 * s, t - 0.25 * s)),
            line((x, t - 0.4 * s), (x, t - 0.1 * s)),
            poly([(x - 0.35 * s, t + 0.02 * s), (x + 0.2 * s, t + 0.02 * s), (x + 0.2 * s, t + 0.12 * s), (x + 0.38 * s, t + 0.07 * s), (x + 0.2 * s, t + 0.02 * s - 0.0), (x - 0.35 * s, t + 0.02 * s)], "gold", THIN),
            poly([(x - 0.35 * s, t + 0.07 * s), (x - 0.45 * s, t + 0.0 * s), (x - 0.45 * s, t + 0.14 * s)], "rose", THIN),
            f"\\draw[{THIN}, ->] {P(x + 0.4 * s, t + 0.25 * s)} arc (20:160:{0.42 * s:.3f});"]


def moon(x, y, r=0.18):
    return [circ(x, y, r, "cloud")]


# --- names: props that carry the store's name -------------------------------

def tex(t):
    return t.replace("\\", "").replace("&", "\\&").replace("%", "\\%").replace("#", "\\#").replace("$", "\\$").replace("_", "\\_")


def text(x, y, t, size=9, maxw=1.6, color="ink", rot=0):
    """The name, centered on (x, y), at `size` points, shrunk to `maxw` cm if it is wider."""
    return (f"\\node[inner sep=0, text={color}, rotate={rot}] at {P(x, y)} "
            f"{{\\resizebox{{\\ifdim\\width>{maxw:.2f}cm {maxw:.2f}cm\\else\\width\\fi}}{{!}}{{\\fontsize{{{size}}}{{{size}}}\\signfont {tex(t)}}}}};")


def board(x, y, w, h, t, fill="cream", rot=0, size=9, color="ink", posts=None):
    """A sign board centered on (x, y); `posts` is the ground height to stand it on two posts."""
    o = []
    if posts is not None:
        for dx in (-w * 0.32, w * 0.32):
            o.append(line((x + dx, posts), (x + dx, y - h / 2), style="draw=ink, line width=1pt"))
    o.append(f"\\begin{{scope}}[shift={{{P(x, y)}}}, rotate={rot}, transform shape]")
    o.append(rect(-w / 2, -h / 2, w, h, fill))
    o.append(text(0, 0, t, size, w - 0.12, color))
    o.append("\\end{scope}")
    return o


def roof_board(x, top, w, t, fill="cream"):
    """The store's front: a long board on the roof with the chain's cup at one end."""
    o = [line((x - w * 0.3, top), (x - w * 0.3, top + 0.12), style="draw=ink, line width=0.7pt"), line((x + w * 0.3, top), (x + w * 0.3, top + 0.12), style="draw=ink, line width=0.7pt"),
         rect(x - w / 2, top + 0.12, w, 0.36, fill)]
    o += cup_sign(x - w / 2 + 0.2, top + 0.15, 0.42)[1:]
    o.append(text(x + 0.12, top + 0.3, t, 9, w - 0.5))
    return o


def banner(x0, x1, y, t, fill="rose", ground=None, color="cream"):
    """A cloth banner between two poles, its ends notched."""
    o = []
    if ground is not None:
        o += [line((x0, ground), (x0, y + 0.22), style="draw=ink, line width=1pt"), line((x1, ground), (x1, y + 0.22), style="draw=ink, line width=1pt")]
    o.append(poly([(x0, y + 0.2), (x1, y + 0.2), (x1 - 0.08, y), (x1, y - 0.2), (x0, y - 0.2), (x0 + 0.08, y)], fill))
    o.append(text((x0 + x1) / 2, y, t, 9, x1 - x0 - 0.3, color))
    return o


def flag(x, y, t, fill="teal"):
    """A banner flying from a pole in the wind."""
    o = [line((x, y - 1.3), (x, y + 0.25), style="draw=ink, line width=1pt")]
    o.append(f"\\filldraw[fill={fill}, {LINE}] {P(x, y + 0.22)} .. controls {P(x + 0.5, y + 0.34)} and {P(x + 0.9, y + 0.1)} .. {P(x + 1.45, y + 0.24)} -- {P(x + 1.35, y + 0.0)} -- {P(x + 1.45, y - 0.22)} .. controls {P(x + 0.9, y - 0.36)} and {P(x + 0.5, y - 0.1)} .. {P(x, y - 0.2)} -- cycle;")
    o.append(text(x + 0.7, y + 0.0, t, 9, 1.15, "cream", rot=2))
    return o


def surfboard(x, y, t, rot=70, fill="teal", stripe="gold"):
    o = [f"\\begin{{scope}}[shift={{{P(x, y)}}}, rotate={rot}, transform shape]", ell(0, 0, 0.82, 0.17, fill),
         line((-0.75, 0), (-0.6, 0), style=f"draw={stripe}, line width=1.4pt"), text(0.05, 0, t, 8, 1.2, "cream"), "\\end{scope}"]
    return o


def golf_club(x, y, t):
    """A giant golf club stuck in the ground, the name up its shaft, and a ball on a tee."""
    o = [f"\\begin{{scope}}[shift={{{P(x, y)}}}, rotate=-8, transform shape]",
         rect(-0.11, 0.0, 0.22, 1.85, "stone"), rect(-0.13, 1.55, 0.26, 0.45, "shadow"),
         poly([(-0.11, 0.0), (0.11, 0.0), (0.5, -0.12), (0.5, 0.12), (0.11, 0.16)], "slate"),
         text(0, 0.8, t, 8, 1.4, "ink", rot=90), "\\end{scope}"]
    o += [line((x + 0.55, y), (x + 0.55, y + 0.1), style="draw=ink, line width=1pt"), circ(x + 0.55, y + 0.17, 0.07, "snow", THIN)]
    return o


def tube_man(x, y, t, fill="rose"):
    """An inflatable tube dude, dancing, the name down his body."""
    o = [rect(x - 0.22, y, 0.44, 0.14, "slate"),
         f"\\filldraw[fill={fill}, {LINE}] {P(x - 0.16, y + 0.14)} .. controls {P(x - 0.3, y + 0.7)} and {P(x - 0.05, y + 1.1)} .. {P(x - 0.2, y + 1.6)} -- {P(x + 0.14, y + 1.66)} .. controls {P(x + 0.28, y + 1.1)} and {P(x + 0.02, y + 0.7)} .. {P(x + 0.16, y + 0.14)} -- cycle;",
         f"\\filldraw[fill={fill}, {THIN}] {P(x - 0.18, y + 1.3)} .. controls {P(x - 0.5, y + 1.5)} and {P(x - 0.55, y + 1.8)} .. {P(x - 0.75, y + 2.0)} -- {P(x - 0.65, y + 2.08)} .. controls {P(x - 0.45, y + 1.85)} and {P(x - 0.4, y + 1.6)} .. {P(x - 0.12, y + 1.45)} -- cycle;",
         f"\\filldraw[fill={fill}, {THIN}] {P(x + 0.12, y + 1.38)} .. controls {P(x + 0.45, y + 1.4)} and {P(x + 0.55, y + 1.7)} .. {P(x + 0.8, y + 1.75)} -- {P(x + 0.78, y + 1.86)} .. controls {P(x + 0.5, y + 1.82)} and {P(x + 0.4, y + 1.55)} .. {P(x + 0.1, y + 1.52)} -- cycle;",
         circ(x - 0.03, y + 1.72, 0.17, fill), circ(x - 0.09, y + 1.75, 0.03, "ink", THIN), circ(x + 0.04, y + 1.75, 0.03, "ink", THIN),
         f"\\draw[{THIN}] {P(x - 0.09, y + 1.66)} arc (200:340:0.07);",
         text(x - 0.02, y + 0.8, t, 7, 1.15, "cream", rot=85)]
    for k in range(3):
        o.append(curve((x + 0.35, y + 0.4 + k * 0.4), (x + 0.45, y + 0.45 + k * 0.4), (x + 0.45, y + 0.55 + k * 0.4), (x + 0.38, y + 0.6 + k * 0.4)))
    return o


def hung_board(x, top, w, t, fill="cream"):
    """A board hanging on two chains from (x, top)."""
    return [line((x - w * 0.3, top), (x - w * 0.3, top - 0.25)), line((x + w * 0.3, top), (x + w * 0.3, top - 0.25)),
            rect(x - w / 2, top - 0.6, w, 0.35, fill), text(x, top - 0.43, t, 8.5, w - 0.12)]


def giant_cup(x, base, kind, s, slush="rose", band=None, label=None, lid=False):
    """A giant cup of a shape the game has, drawn from data/cups.json's
    profile, so the art keeps the physics' shape: `s` cm of drawing for each
    cm of cup. Slush fills its lower part; `band` wraps it with a label."""
    c = CUPS[kind]
    prof, h = c["profile"], c["inner_height"]
    pts = [(x + prof[k] * s, base + h * s * k / 4) for k in range(5)]
    left = [(x - px + x, py) for px, py in pts]
    outline = pts + left[::-1]
    o = [poly(outline, "cloud", "draw=ink, line width=1pt, line join=round")]
    # The slush: the same profile, up to three fifths of the height.
    fill_h = h * 0.6
    sl = []
    for k in range(5):
        hh = h * k / 4
        if hh <= fill_h:
            sl.append((x + prof[k] * s, base + hh * s))
    # Where the slush's top cuts the wall.
    kk = min(3, int(fill_h * 4 / h))
    t = (fill_h - h * kk / 4) / (h / 4)
    sl.append((x + (prof[kk] + (prof[kk + 1] - prof[kk]) * t) * s, base + fill_h * s))
    o.append(poly(sl + [(2 * x - px, py) for px, py in sl[::-1]], slush, THIN))
    if lid:
        mw = prof[4] * s
        o += [rect(x - mw - 0.04, base + h * s, 2 * mw + 0.08, 0.12, "gold"), line((x - mw - 0.04, base + h * s + 0.06), (x + mw + 0.04, base + h * s + 0.06))]
    if band is not None:
        hh = h * band
        k = min(3, int(band * 4))
        tt = (hh - h * k / 4) / (h / 4)
        bw = (prof[k] + (prof[k + 1] - prof[k]) * tt) * s
        o.append(rect(x - bw, base + hh * s - 0.17, 2 * bw, 0.34, "cream"))
        if label:
            o.append(text(x, base + hh * s, label, 7.5, 2 * bw - 0.1))
    return o


def traffic_cone(x, y, s=1.0):
    return [poly([(x - 0.16 * s, y + 0.04 * s), (x + 0.16 * s, y + 0.04 * s), (x + 0.04 * s, y + 0.55 * s), (x - 0.04 * s, y + 0.55 * s)], "peachdark", THIN),
            poly([(x - 0.11 * s, y + 0.2 * s), (x + 0.11 * s, y + 0.2 * s), (x + 0.08 * s, y + 0.32 * s), (x - 0.08 * s, y + 0.32 * s)], "cream", THIN),
            rect(x - 0.22 * s, y, 0.44 * s, 0.06 * s, "peachdark", THIN)]


def barrier(x, y, w, t):
    o = [line((x - w / 2 + 0.1, y), (x - w / 2 + 0.1, y + 0.5), style="draw=ink, line width=1pt"), line((x + w / 2 - 0.1, y), (x + w / 2 - 0.1, y + 0.5), style="draw=ink, line width=1pt"),
         rect(x - w / 2, y + 0.45, w, 0.36, "cream")]
    o.append(text(x, y + 0.63, t, 8.5, w - 0.14))
    for k in range(int(w / 0.3)):
        o.append(poly([(x - w / 2 + k * 0.3, y + 0.45), (x - w / 2 + k * 0.3 + 0.12, y + 0.45), (x - w / 2 + k * 0.3 + 0.2, y + 0.53), (x - w / 2 + k * 0.3 + 0.08, y + 0.53)], "peachdark", "draw=none"))
    return o


def dish(x, y, s=1.0):
    """A radio telescope: a bowl on a stand, tipped to the sky."""
    return [line((x - 0.25 * s, y), (x, y + 0.7 * s), (x + 0.25 * s, y), style="draw=ink, line width=1pt"),
            f"\\begin{{scope}}[shift={{{P(x, y + 0.85 * s)}}}, rotate=25, transform shape]",
            f"\\filldraw[fill=cloud, {LINE}] {P(-0.7 * s, 0.1 * s)} .. controls {P(-0.5 * s, -0.45 * s)} and {P(0.5 * s, -0.45 * s)} .. {P(0.7 * s, 0.1 * s)} -- cycle;",
            line((0, -0.25 * s), (0, 0.45 * s)), circ(0, 0.48 * s, 0.05 * s, "gold", THIN), "\\end{scope}"]


def fish(x, y, s=1.0, fill="gold"):
    return [ell(x, y, 0.12 * s, 0.07 * s, fill, THIN), poly([(x + 0.1 * s, y), (x + 0.2 * s, y + 0.07 * s), (x + 0.2 * s, y - 0.07 * s)], fill, THIN), circ(x - 0.06 * s, y + 0.015 * s, 0.012 * s, "ink", THIN)]


def bottle(x, y, s=1.0, fill="teal", rot=0):
    return [f"\\begin{{scope}}[shift={{{P(x, y)}}}, rotate={rot}, transform shape]", rect(-0.08 * s, 0, 0.16 * s, 0.32 * s, fill, THIN), rect(-0.035 * s, 0.32 * s, 0.07 * s, 0.12 * s, fill, THIN),
            rect(-0.05 * s, 0.1 * s, 0.1 * s, 0.1 * s, "cream", THIN), "\\end{scope}"]


def hourglass_frame(x, base, s):
    c = CUPS["hourglass"]
    top = base + c["inner_height"] * s
    w = max(c["profile"]) * s + 0.12
    return [rect(x - w, base - 0.1, 2 * w, 0.1, "wood"), rect(x - w, top, 2 * w, 0.1, "wood"),
            line((x - w + 0.04, base), (x - w + 0.04, top), style="draw=wood, line width=2pt"), line((x + w - 0.04, base), (x + w - 0.04, top), style="draw=wood, line width=2pt")]


# --- the scenes -------------------------------------------------------------
# Each is (things behind the store, the store's options, things in front).
# Coordinates are in the 4 by 3 cm frame; the island's top is near y = 0.62.

G = 0.62  # the ground the store stands on

SCENES = {
    # Your hometown
    "m_first_pour": (sign(0.5, G, 1.0, "gold") + lamp(3.4, G, 0.9), dict(awning="teal"), [], "store"),
    # The boardwalk
    "m_push": (turbine(3.6, G + 0.1, 0.75), dict(awning="sage", x=2.1), pier(0.15, 0.55, 0.9) + wind(0.25, 0.95, 0.6), lambda t: flag(0.35, 2.2, t, "teal")),
    "m_tail": ([], dict(awning="peach", x=2.2), pier(0.2, 0.55, 1.0) + gull(1.4, 2.6) + gull(3.4, 2.4), lambda t: surfboard(0.75, 1.35, t, 75, "teal")),
    "m_two_spouts": (big_wheel(3.35, G, 0.95), dict(awning="gold", x=2.0), [], lambda t: golf_club(0.55, G - 0.05, t)),
    "s_zigzag": (cactus(3.75, G, 0.8), dict(awning="teal", x=2.25), dust_devil(3.3, G, 0.9) + zigzag(2.6, 2.3, 0.8), lambda t: board(0.75, 1.55, 1.25, 0.42, t, "sand", rot=-6, posts=G)),
    "m_half": (canopy(2.65, G, 1.15, 1.0), dict(awning="gold", x=1.45), pump(3.0, G) + pump(3.5, G) + [line((2.0, 0.25), (2.0, 2.0), style="draw=ink, line width=0.6pt, dash pattern=on 3pt off 2pt")], lambda t: board(3.22, 2.2, 1.35, 0.4, t, "rose", color="cream")),
    "m_third": (truck(2.75, G - 0.05, 0.9), dict(awning="rose", x=1.85), cactus(3.75, G, 0.7), lambda t: tube_man(0.5, G - 0.05, t, "rose")),
    "m_charge": (pylon(0.45, G, 0.95) + pylon(3.55, G, 0.95) + wires(0.45, 3.55, G + 1.27, 0.25), dict(awning="gold"), [], "store"),
    "s_heavy": (mine_entry(3.2, G, 1.0), dict(awning="olive", x=1.45), mine_cart(3.1, G - 0.15, 0.8), lambda t: board(3.2, G + 1.05, 1.25, 0.36, t, "wood", color="cream")),
    "s_jet": (rocket(3.4, G, 1.25), dict(awning="teal", x=1.6), [], lambda t: text(3.4, G + 0.73, t, 7, 0.78, "ink", rot=90)),
    # The big city
    "m_two_one": (tower(0.45, G, 0.8, 2.0, "rose") + tower(3.55, G, 0.8, 1.7, "sage"), dict(awning="peach"), [], "store"),
    "m_cherry_cola": (tower(3.5, G, 0.8, 1.9, "stone"), dict(awning="rose", x=1.55, wall="peach"), [], lambda t: board(3.25, 1.95, 1.3, 0.42, t, "rose", color="cream", posts=G)),
    "m_push_back": (tower(0.5, G, 0.8, 2.2, "slate") + tower(3.5, G, 0.8, 2.3, "stone"), dict(awning="gold"), wind(3.45, 1.2, 0.7, back=True), lambda t: [rect(3.28, 1.35, 0.44, 1.35, "gold"), text(3.5, 2.02, t, 8, 1.2, "ink", rot=90)]),
    "s_field_maze": (tower(0.45, G, 0.75, 2.1, "peach") + tower(3.55, G, 0.75, 1.9, "slate"), dict(awning="peach"), wind(0.2, 2.3, 0.5) + wind(3.85, 2.0, 0.5, back=True),
                     lambda t: [line((2.0, G + 1.4), (2.0, 2.85), style="draw=ink, line width=1pt")] + board(2.0, 2.62, 1.3, 0.32, t, "sage", rot=-4, color="cream")),
    "s_two_rows": (stands(0.2, G + 0.15, 3.6, 1.0) + floodlight(0.25, G, 1.0) + floodlight(3.75, G, 1.0), dict(awning="peach", s=0.85), [], lambda t: banner(0.7, 3.3, 2.6, t, "teal")),
    "s_sink": (ship(3.15, 0.55, 0.85) + crane(0.3, G, 1.2), dict(awning="teal", x=2.15), anchor(3.75, G + 0.05, 0.6), lambda t: [line((1.1, 2.54), (1.1, 2.34))] + hung_board(1.1, 2.34, 1.15, t, "gold")),
    "s_jet_pair": (firehouse(3.25, G, 0.95), dict(awning="gold", x=1.55), hydrant(0.35, G, 1.0, spray=True), lambda t: banner(2.4, 4.0, 2.3, t, "rose")),
    # The mountains
    "m_three": (peak(2.0, G + 0.3, 3.4, 2.1) + pine(0.35, G) + pine(3.7, G), dict(awning="rose", x=1.8), skis(3.05, G, 0.9), lambda t: board(3.2, 1.9, 1.15, 0.36, t, "teal", rot=4, color="cream", posts=G)),
    "m_more_cola": (pine(3.65, G) + pine(3.35, G + 0.05, 0.8), dict(awning="peach", x=1.4, wall="wood"), ice(3.2, 0.5, 0.65, 0.18) + skater(3.15, 0.45, 0.8), lambda t: board(1.4, 2.2, 1.3, 0.38, t, "cloud", posts=None) + [line((0.85, 1.7), (0.85, 2.01)), line((1.95, 1.7), (1.95, 2.01))]),
    "m_attract": (peak(2.0, G, 3.8, 1.5) + star(0.4, 2.7) + star(3.5, 2.8) + star(2.8, 2.6, 0.04), dict(awning="teal", x=1.5, sign=False), dome(3.1, G, 1.0), lambda t: board(1.5, 1.95, 1.3, 0.38, t, "night", color="gold") + [line((1.0, 1.58), (1.0, 1.76)), line((2.0, 1.58), (2.0, 1.76))]),
    "m_rail": (peak(3.4, G, 1.6, 1.4), dict(awning="gold", x=1.5), [line((0.0, 2.85), (4.0, 2.05), style="draw=ink, line width=0.7pt"), line((2.6, 2.33), (2.6, 2.1)), rect(2.0, 1.45, 1.2, 0.65, "rose"), rect(2.1, 1.8, 1.0, 0.2, "skylight", THIN)], lambda t: text(2.6, 1.62, t, 8, 1.05, "cream")),
    "m_two_lines": (pine(0.35, G) + pine(3.65, G), dict(awning="rose", floors=2, s=0.78), [], "store"),
    "s_bobbing": (lift_pole(3.4, G, 1.9) + [line((0.0, 2.55), (3.4, 2.5), (4.0, 2.6), style="draw=ink, line width=0.7pt")], dict(awning="peach", x=1.75), pine(3.75, G, 0.8),
                  lambda t: [line((1.3, 2.53), (1.3, 2.2))] + [rect(0.7, 1.82, 1.2, 0.38, "peach")] + [text(1.3, 2.01, t, 8, 1.1, "ink")] + [line((0.75, 1.82), (0.75, 1.68), (1.85, 1.68), (1.85, 1.82), style="draw=ink, line width=0.9pt")]),
    "s_layers": (strata(2.75, G, 1.2, 1.6), dict(awning="olive", x=1.45), [], lambda t: board(3.35, G + 1.85, 1.35, 0.36, t, "cream")),
    "s_jet_rows": (pine(0.35, G) + pine(3.65, G + 0.05, 0.9), dict(awning="teal", x=1.75), [], lambda t: [line((2.9, G), (2.9, G + 0.5), style="draw=ink, line width=1pt"),
                   f"\\begin{{scope}}[shift={{{P(2.9, G + 0.6)}}}, rotate=30, transform shape]", rect(-0.1, -0.16, 1.15, 0.32, "cloud"), text(0.47, 0, t, 7, 1.0), "\\end{scope}"] + [circ(3.95, 1.55 + k * 0.1, 0.04, "snow", THIN) for k in range(3)]),
    "s_flip_pair": (peak(2.0, G + 0.2, 3.6, 1.8) + pine(0.35, G), dict(awning="teal", x=1.5), vane(3.3, G, 1.0), lambda t: board(3.3, G + 0.6, 1.0, 0.32, t, "cream", posts=None)),
    # The islands
    "m_lemon_cherry_cola": (palm(0.4, G, 1.0, 0.25), dict(skip=True), water(0.15, 0.45, 3.7, 0.18) + hut(2.2, G, 1.0), lambda t: board(2.2, 2.12, 1.3, 0.34, t, "sand")),
    "m_alternating": (palm(3.4, G, 1.0, -0.4), dict(awning="sage", x=2.1), wind(3.85, 2.0, 0.6, back=True), lambda t: surfboard(0.6, 1.4, t, 72, "rose", "teal")),
    "s_rows_bob": (water(0.1, 0.35, 3.8, 0.35), dict(awning="teal", y=0.92, s=0.85), [poly([(0.9, 0.95), (3.1, 0.95), (2.9, 0.62), (1.05, 0.62)], "wood")], lambda t: text(2.0, 0.78, t, 8, 1.7, "cream")),
    # The old town
    "m_field_blend": (clock_tower(3.3, G, 1.0), dict(awning="rose", x=1.5, wall="gold"), [], lambda t: [line((2.6, 2.05), (3.05, 2.05))] + [poly([(2.62, 2.05), (3.05, 2.05), (3.05, 1.2), (2.83, 1.32), (2.62, 1.2)], "teal")] + [text(2.835, 1.67, t, 7, 0.8, "cream", rot=90)]),
    "m_two_lines_blend": ([], dict(awning="sage", y=G + 0.55, s=0.8), bridge(0.6, 0.45, 2.8), lambda t: text(2.0, 0.83, t, 7.5, 1.5, "ink")),
    # The night market
    "m_two_speeds": (lanterns(0.1, 3.9, 2.55) + moon(3.5, 2.75), dict(skip=True), stall(2.0, G, 1.0), lambda t: banner(1.2, 2.8, 2.3, t, "rose")),
    # Cup shapes (Sam, 2026-10-08): a mission on each row of the tour, each
    # scene with a giant cup of its shape, drawn from data/cups.json.
    "s_tall": (giant_cup(3.3, G, "tall", 0.035, "teal") + gull(3.7, 2.8), dict(awning="teal", x=1.75), pier(0.1, 0.55, 0.8), lambda t: banner(0.55, 2.95, 2.6, t, "rose", ground=None) + [line((0.55, 2.4), (0.55, 2.82), style="draw=ink, line width=0.9pt"), line((2.95, 2.4), (2.95, 2.82), style="draw=ink, line width=0.9pt")]),
    "s_bowl": (palm(3.6, G, 1.0, -0.3) + water(0.1, 0.42, 3.8, 0.16), dict(awning="gold", x=1.6), giant_cup(3.05, G, "bowl", 0.022, "cream"), lambda t: board(0.65, 1.7, 1.15, 0.34, t, "sand", rot=-4, posts=G)),
    "s_cone": (spire(0.35, G - 0.1, 0.6, 1.4) + giant_cup(3.3, G + 0.05, "cone", 0.021, "rose") + cactus(3.85, G, 0.6), dict(awning="peach", x=1.75), traffic_cone(0.75, G) + traffic_cone(1.05, G - 0.05, 0.8) + traffic_cone(2.62, G - 0.05, 0.75), lambda t: barrier(0.95, G + 0.95, 1.25, t)),
    "s_jar": (tower(0.4, G, 0.7, 2.0, "sage") + tower(2.2, G + 0.9, 0.9, 1.4, "rose"), dict(awning="rose", x=1.55, wall="wood"), giant_cup(3.2, G, "jar", 0.03, "rose", band=0.4, lid=True), lambda t: giant_cup(3.2, G, "jar", 0.03, "rose", band=0.42, lid=True, label=t)[-2:]),
    "s_hourglass": (peak(2.0, G + 0.3, 3.6, 1.9) + pine(0.35, G) + hourglass_frame(3.2, G + 0.1, 0.03) + giant_cup(3.2, G + 0.1, "hourglass", 0.03, "sand"), dict(awning="sage", x=1.75), [], lambda t: board(1.75, 2.35, 1.3, 0.34, t, "sand") + [line((1.3, 2.18), (1.3, 1.98)), line((2.2, 2.18), (2.2, 1.98))]),
    "s_fishbowl": ([], dict(awning="teal", x=1.6, wall="gold"), giant_cup(3.2, G, "fishbowl", 0.022, "teal") + fish(3.15, G + 0.45, 1.0) + fish(3.35, G + 0.65, 0.8, "rose"), "store"),
    "s_glass_bowl": (lanterns(0.1, 3.9, 2.6), dict(skip=True), stall(1.6, G, 0.9) + giant_cup(3.3, G, "tall", 0.022, "teal") + giant_cup(3.75, G, "bowl", 0.011, "gold"), lambda t: banner(0.85, 2.35, 2.25, t, "teal")),
    "s_jar_hourglass": (storm_cloud(0.8, 2.6) + storm_cloud(3.3, 2.75) + rain(0.3, 1.3, 3.4, 1.1, 12), dict(awning="teal", x=1.5, s=0.9), bottle(2.85, G - 0.05, 1.0, "teal", -70) + bottle(3.35, G, 1.1, "sage", 15) + bottle(3.65, G - 0.05, 0.9, "rose", 80) + giant_cup(0.45, G, "jar", 0.018, "rose", lid=True),
                        lambda t: board(3.25, 1.45, 1.1, 0.32, t, "wood", rot=-5, color="cream")),
    # The end of the world
    "m_storm": (storm_cloud(0.8, 2.6) + storm_cloud(3.3, 2.75) + rain(0.3, 1.2, 3.4, 1.2, 14) + bolt(1.6, 2.5), dict(awning="teal", x=1.35, s=0.85), lighthouse(3.2, G, 1.0), lambda t: board(3.2, 0.95, 1.0, 0.28, t, "wood", rot=-4, color="cream")),
}


def levels():
    """Each mission's depth in the tree, as core decides it (`lab levels`)."""
    out = subprocess.run(["cargo", "run", "-q", "--release", "-p", "lab", "--", "levels"], cwd=ROOT, capture_output=True, text=True, check=True).stdout
    return json.loads(out)


def store_name(i):
    return COPY["missions"]["list"][i]["store"]


def scene(i, level):
    behind, opts, front, namer = SCENES[i]
    ground = GROUND[min(level, len(GROUND) - 1)]
    o = [f"\\filldraw[fill=shadow, draw=none, opacity=0.35] {P(2.06, 0.52)} ellipse (1.88 and 0.42);",
         f"\\filldraw[fill={ground}, {LINE}] {P(2.0, 0.62)} ellipse (1.85 and 0.42);"]
    o += behind
    opts = dict(opts)
    if namer == "store":
        opts["name"] = store_name(i)
    if not opts.pop("skip", False):
        x, y, s = opts.pop("x", 2.0), opts.pop("y", G - 0.05), opts.pop("s", 1.0)
        o += store(x, y, s, **opts)
    o += front
    if namer != "store":
        named = namer(store_name(i))
        # A prop that is a single line of TikZ comes back as a string; adding
        # a string to a list adds its characters, and TikZ drops them quietly.
        o += [named] if isinstance(named, str) else named
    return o


def render(i, level):
    tex = f"""\\documentclass[tikz,border=0pt]{{standalone}}
\\usepackage{{fontspec}}
\\newfontfamily\\signfont{{LilitaOne-Regular.ttf}}[Path={FONTS}/]
\\usepackage{{tikz}}
{colors()}
\\begin{{document}}
\\begin{{tikzpicture}}
\\useasboundingbox (0,0) rectangle ({W},{H});
{chr(10).join(scene(i, level))}
\\end{{tikzpicture}}
\\end{{document}}
"""
    OUT.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory() as d:
        Path(d, "f.tex").write_text(tex)
        rr = subprocess.run(["xelatex", "-interaction=nonstopmode", "-halt-on-error", "f.tex"], cwd=d, capture_output=True, text=True)
        if rr.returncode:
            sys.exit(f"xelatex failed for {i}:\n" + "\n".join(l for l in rr.stdout.splitlines() if l.startswith("!") or l.startswith("l.")))
        subprocess.run(["pdftocairo", "-png", "-transp", "-singlefile", "-scale-to-x", str(PX), "-scale-to-y", "-1", "f.pdf", str(OUT / i)], cwd=d, check=True)


if __name__ == "__main__":
    lv = levels()
    missing = sorted(set(lv) - set(SCENES))
    if missing:
        sys.exit(f"no scene for {missing}")
    manifest = OUT / "names.json"
    drawn = json.loads(manifest.read_text()) if manifest.exists() else {}
    for i in (sys.argv[1:] or sorted(SCENES)):
        render(i, lv[i])
        drawn[i] = store_name(i)
        print("drew", i)
    # Which name each picture carries, for tests/world.rs.
    manifest.write_text(json.dumps(dict(sorted(drawn.items())), indent=1, ensure_ascii=False) + "\n")
