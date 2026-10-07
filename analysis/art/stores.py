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

    python3 analysis/art/stores.py           # writes web/art/store/<id>.png
    python3 analysis/art/stores.py s_jet     # just one
"""
import json, math, subprocess, sys, tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
PAL = json.loads((ROOT / "data" / "palette.json").read_text())["art"]
OUT = ROOT / "web" / "art" / "store"
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


def store(x, y, s=1.0, wall="cream", awning="teal", sign=True, floors=1):
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
    if sign:
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


# --- the scenes -------------------------------------------------------------
# Each is (things behind the store, the store's options, things in front).
# Coordinates are in the 4 by 3 cm frame; the island's top is near y = 0.62.

G = 0.62  # the ground the store stands on

SCENES = {
    # Your hometown
    "m_first_pour": (sign(0.5, G, 1.0, "gold") + lamp(3.4, G, 0.9), dict(awning="teal"), []),
    # The boardwalk
    "m_push": (turbine(3.5, G + 0.1, 0.8), dict(awning="sage", x=1.8), pier(0.15, 0.55, 0.9) + wind(0.25, 1.2, 0.8) + gull(0.9, 2.2)),
    "m_tail": ([], dict(awning="peach"), pier(0.2, 0.55, 1.0) + gull(0.6, 1.6) + gull(3.3, 1.9) + gull(3.6, 1.5)),
    "m_two_spouts": (big_wheel(3.3, G, 1.0), dict(awning="gold", x=1.6), []),
    "s_zigzag": (cactus(0.4, G) + cactus(3.6, G, 0.8), dict(awning="teal", x=1.9), dust_devil(3.1, G, 1.0) + zigzag(0.15, 1.5, 1.0) + zigzag(2.8, 2.15, 0.9)),
    # The desert highway
    "m_half": (canopy(2.55, G, 1.2, 1.0), dict(awning="gold", x=1.4), pump(2.9, G) + pump(3.45, G) + [line((2.0, 0.25), (2.0, 2.3), style="draw=ink, line width=0.6pt, dash pattern=on 3pt off 2pt")]),
    "m_third": (truck(2.6, G - 0.05, 1.0), dict(awning="rose", x=1.5), cactus(0.3, G)),
    "m_charge": (pylon(0.55, G, 1.0) + pylon(3.45, G, 1.0) + wires(0.55, 3.45, G + 1.35, 0.25), dict(awning="gold"), []),
    "s_field_maze": (tower(0.5, G, 0.8, 2.1, "peach") + tower(3.5, G, 0.8, 1.9, "slate"), dict(awning="peach"), wind(0.2, 1.9, 0.6) + wind(3.8, 1.5, 0.6, back=True)),
    "s_heavy": (mine_entry(3.2, G, 1.0), dict(awning="olive", x=1.5), mine_cart(3.1, G - 0.15, 0.8)),
    "s_jet": (rocket(3.4, G, 1.2), dict(awning="teal", x=1.6), []),
    # The big city
    "m_two_one": (tower(0.55, G, 0.9, 2.0, "rose") + tower(3.45, G, 0.9, 1.7, "sage"), dict(awning="peach"), []),
    "m_cherry_cola": (tower(3.45, G, 0.9, 1.9, "stone"), dict(awning="rose", x=1.6, wall="peach"), diner_sign(3.4, G, 0.9)),
    "m_push_back": (tower(0.5, G, 0.8, 2.2, "slate") + tower(3.5, G, 0.8, 2.3, "stone"), dict(awning="gold"), wind(3.5, 1.2, 0.8, back=True)),
    "s_flip_pair": (peak(2.0, G + 0.2, 3.6, 1.8) + pine(0.35, G), dict(awning="teal", x=1.5), vane(3.3, G, 1.0)),
    "s_two_rows": (stands(0.2, G + 0.15, 3.6, 1.0) + floodlight(0.25, G, 1.0) + floodlight(3.75, G, 1.0), dict(awning="peach", s=0.85), []),
    "s_sink": (ship(3.1, 0.55, 0.9) + crane(0.4, G, 1.0), dict(awning="teal", x=1.7), anchor(0.6, G, 0.9)),
    "s_jet_pair": (firehouse(3.2, G, 1.0), dict(awning="gold", x=1.5), hydrant(0.35, G, 1.0, spray=True)),
    # The mountains
    "m_three": (peak(2.0, G + 0.3, 3.4, 2.1) + pine(0.4, G) + pine(3.6, G), dict(awning="rose"), skis(3.05, G, 0.9)),
    "m_more_cola": (pine(3.6, G) + pine(3.3, G + 0.05, 0.8), dict(awning="peach", x=1.4, wall="wood"), ice(3.2, 0.5, 0.65, 0.18) + skater(3.15, 0.45, 0.8)),
    "m_attract": (peak(2.0, G, 3.8, 1.5) + star(0.4, 2.6) + star(3.5, 2.75) + star(2.8, 2.55, 0.04), dict(awning="teal", x=1.5, sign=False), dome(3.1, G, 1.0)),
    "m_rail": (cable_car(0.2, 2.7, 3.9, 1.8, 0.62) + peak(3.4, G, 1.6, 1.4), dict(awning="gold", x=1.5), []),
    "m_two_lines": (pine(0.35, G) + pine(3.65, G), dict(awning="rose", floors=2, s=0.78), []),
    "s_bobbing": (lift_pole(3.4, G, 1.9) + [line((0.0, 2.55), (3.4, 2.5), (4.0, 2.6), style="draw=ink, line width=0.7pt")] + chair(1.0, 2.53) + chair(2.6, 2.5), dict(awning="peach", x=1.7), pine(3.75, G, 0.8)),
    "s_layers": (strata(2.7, G, 1.3, 1.6), dict(awning="olive", x=1.4), []),
    "s_jet_rows": (pine(0.4, G) + pine(3.6, G + 0.05, 0.9), dict(awning="teal", x=1.7), snow_cannon(2.7, G, 0.9)),
    # The islands
    "m_lemon_cherry_cola": (palm(0.4, G, 1.0, 0.25), dict(skip=True), water(0.15, 0.45, 3.7, 0.18) + hut(2.2, G, 1.0)),
    "m_alternating": (palm(0.5, G, 1.1, 0.5) + palm(3.4, G, 1.0, -0.4), dict(awning="sage"), wind(0.1, 1.6, 0.6) + wind(3.9, 1.9, 0.6, back=True)),
    "s_rows_bob": (water(0.1, 0.35, 3.8, 0.35), dict(awning="teal", y=0.82, s=0.85), boat(2.0, 0.55, 1.0) + [curve((0.2, 0.4), (0.6, 0.6), (1.0, 0.25), (1.4, 0.4), style="draw=tealdark, line width=0.6pt")]),
    # The old town
    "m_field_blend": (clock_tower(3.3, G, 1.0), dict(awning="rose", x=1.5, wall="gold"), []),
    "m_two_lines_blend": ([], dict(awning="sage", y=G + 0.55, s=0.8), bridge(0.6, 0.45, 2.8)),
    # The night market
    "m_two_speeds": (lanterns(0.1, 3.9, 2.55) + moon(3.5, 2.75), dict(skip=True), stall(2.0, G, 1.0)),
    # The end of the world
    "m_storm": (storm_cloud(0.8, 2.6) + storm_cloud(3.3, 2.75) + rain(0.3, 1.2, 3.4, 1.2, 14) + bolt(1.6, 2.5), dict(awning="teal", x=1.4, s=0.85), lighthouse(3.2, G, 1.0)),
}


def levels():
    """Each mission's depth in the tree, as core decides it (`lab levels`)."""
    out = subprocess.run(["cargo", "run", "-q", "--release", "-p", "lab", "--", "levels"], cwd=ROOT, capture_output=True, text=True, check=True).stdout
    return json.loads(out)


def scene(i, level):
    behind, opts, front = SCENES[i]
    ground = GROUND[min(level, len(GROUND) - 1)]
    o = [f"\\filldraw[fill=shadow, draw=none, opacity=0.35] {P(2.06, 0.52)} ellipse (1.88 and 0.42);",
         f"\\filldraw[fill={ground}, {LINE}] {P(2.0, 0.62)} ellipse (1.85 and 0.42);"]
    o += behind
    opts = dict(opts)
    if not opts.pop("skip", False):
        x, y, s = opts.pop("x", 2.0), opts.pop("y", G - 0.05), opts.pop("s", 1.0)
        o += store(x, y, s, **opts)
    return o + front


def render(i, level):
    tex = f"""\\documentclass[tikz,border=0pt]{{standalone}}
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
        rr = subprocess.run(["pdflatex", "-interaction=nonstopmode", "-halt-on-error", "f.tex"], cwd=d, capture_output=True, text=True)
        if rr.returncode:
            sys.exit(f"pdflatex failed for {i}:\n" + "\n".join(l for l in rr.stdout.splitlines() if l.startswith("!") or l.startswith("l.")))
        subprocess.run(["pdftocairo", "-png", "-transp", "-singlefile", "-scale-to-x", str(PX), "-scale-to-y", "-1", "f.pdf", str(OUT / i)], cwd=d, check=True)


if __name__ == "__main__":
    lv = levels()
    missing = sorted(set(lv) - set(SCENES))
    if missing:
        sys.exit(f"no scene for {missing}")
    for i in (sys.argv[1:] or sorted(SCENES)):
        render(i, lv[i])
        print("drew", i)
