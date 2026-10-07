#!/usr/bin/env python3
"""The region art behind story mode's map (Sam, 2026-10-07: "take a look at
vagrancy's acrade mode and the way each enemy is a little scene on top of a
greater background of layers that meld together ... the chart of the
missions ends up taking you on a world tour").

One wide scene per row of the mission tree, each a leg of the tour, after
Vagrancy's analysis/art/regions.py and Sam_Comm_final.png: flat fills, plum
ink lines, a mustard sky, peach and sage. Every color comes from
data/palette.json's `art` section. Detail is drawn from a fixed seed, so a
scene is the same each time it is drawn. Nothing here carries a word, a
logo or a store's name: the stores are generic (TONE.md rule 4).

    python3 analysis/art/regions.py      # writes web/art/region-<n>.png
"""
import json, math, random, subprocess, sys, tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
PAL = json.loads((ROOT / "data" / "palette.json").read_text())["art"]
OUT = ROOT / "web" / "art"
W, H = 32.0, 7.0  # cm
LINE = "draw=ink, line width=0.9pt, line join=round"


def colors():
    return "\n".join(f"\\definecolor{{{k.replace('_','')}}}{{HTML}}{{{v[1:]}}}" for k, v in PAL.items() if not k.startswith("_"))


def poly(pts, fill, extra=""):
    return f"\\filldraw[fill={fill}, {LINE}{extra}] " + " -- ".join(f"({x:.2f},{y:.2f})" for x, y in pts) + " -- cycle;"


def rect(x, y, w, h, fill, extra=""):
    return f"\\filldraw[fill={fill}, {LINE}{extra}] ({x:.2f},{y:.2f}) rectangle ({x+w:.2f},{y+h:.2f});"


def circ(x, y, r, fill, lw=0.9):
    return f"\\filldraw[fill={fill}, draw=ink, line width={lw}pt] ({x:.2f},{y:.2f}) circle ({r:.2f});"


def sky(r, base="sky", light="skylight", cloud="cloud", n=5):
    out = [f"\\fill[{base}] (0,0) rectangle ({W},{H});",
           f"\\fill[{light}] (0,{H*0.55:.2f}) rectangle ({W},{H});"]
    for _ in range(n):
        cx, cy = r.uniform(1, W - 1), r.uniform(H * 0.72, H * 0.92)
        blobs = " ".join(f"({cx + dx:.2f},{cy + dy:.2f}) circle ({rad:.2f})" for dx, dy, rad in
                         [(0, 0, 0.55), (0.6, 0.15, 0.45), (-0.6, 0.1, 0.4), (1.1, -0.05, 0.3), (-1.0, -0.05, 0.3)])
        out.append(f"\\filldraw[fill={cloud}, draw=ink, line width=0.6pt] {blobs};")
    return out


def hills(r, y0, amp, fill, n=9):
    xs = [i * W / n for i in range(n + 1)]
    pts = [(x, y0 + amp * math.sin(i * 1.3 + r.random()) + r.uniform(-0.2, 0.2)) for i, x in enumerate(xs)]
    return f"\\filldraw[fill={fill}, {LINE}] plot[smooth] coordinates {{" + " ".join(f"({x:.2f},{y:.2f})" for x, y in pts) + f"}} -- ({W},0) -- (0,0) -- cycle;"


def ground(y, fill):
    return f"\\filldraw[fill={fill}, {LINE}] (0,0) rectangle ({W},{y:.2f});"


def building(x, y, w, h, wall, r, roof=None, windows="skylight"):
    out = [rect(x, y, w, h, wall)]
    if roof:
        out.append(poly([(x - 0.15, y + h), (x + w / 2, y + h + h * 0.4), (x + w + 0.15, y + h)], roof))
    cols, rows = max(1, int(w / 0.55)), max(1, int(h / 0.7))
    for i in range(cols):
        for j in range(rows):
            if r.random() < 0.85:
                out.append(rect(x + 0.18 + i * (w - 0.3) / cols, y + 0.3 + j * (h - 0.4) / rows, 0.26, 0.32, windows, ", line width=0.5pt"))
    return out


def palm(x, y, s=1.0):
    out = [f"\\draw[ink, line width=2.4pt] ({x:.2f},{y:.2f}) .. controls ({x+0.3*s:.2f},{y+0.8*s:.2f}) .. ({x+0.15*s:.2f},{y+1.6*s:.2f});",
           f"\\draw[wood, line width=1.6pt] ({x:.2f},{y:.2f}) .. controls ({x+0.3*s:.2f},{y+0.8*s:.2f}) .. ({x+0.15*s:.2f},{y+1.6*s:.2f});"]
    tx, ty = x + 0.15 * s, y + 1.6 * s
    for a in (20, 60, 120, 160, 200, 340):
        ra = math.radians(a)
        ex, ey = tx + math.cos(ra) * 0.9 * s, ty + math.sin(ra) * 0.5 * s - 0.25 * s
        out.append(f"\\filldraw[fill=sagedark, draw=ink, line width=0.6pt] ({tx:.2f},{ty:.2f}) .. controls ({(tx+ex)/2:.2f},{ty+0.35*s:.2f}) .. ({ex:.2f},{ey:.2f}) .. controls ({(tx+ex)/2:.2f},{ty+0.1*s:.2f}) .. cycle;")
    return out


def pine(x, y, s=1.0, fill="sagedark", snowy=False):
    out = [rect(x - 0.06 * s, y, 0.12 * s, 0.25 * s, "wood")]
    for k in range(3):
        b = y + 0.2 * s + k * 0.35 * s
        out.append(poly([(x - (0.5 - k * 0.12) * s, b), (x + (0.5 - k * 0.12) * s, b), (x, b + 0.6 * s)], fill))
    if snowy:
        out.append(poly([(x - 0.14 * s, y + 1.2 * s), (x + 0.14 * s, y + 1.2 * s), (x, y + 1.5 * s)], "snow"))
    return out


def scene(n, r):
    if n == 0:  # your hometown: a highway at the edge of a small town
        out = sky(r)
        out.append(hills(r, 3.4, 0.35, "sage"))
        for x in range(0, 32, 4):
            out += building(x + r.uniform(0, 1.2), 2.2, r.uniform(1.6, 2.4), r.uniform(0.9, 1.5), r.choice(["peach", "cream", "stone"]), r, roof=r.choice(["peachdark", "wood", None]))
        out.append(ground(2.2, "olive"))
        out.append(rect(0, 0.6, W, 1.0, "stone"))
        for k in range(16):
            out.append(rect(k * 2.0 + 0.4, 1.06, 0.9, 0.08, "cream", ", line width=0.4pt"))
        for x in (3.0, 11.5, 20.0, 28.5):
            out.append(f"\\draw[ink, line width=1.1pt] ({x:.2f},1.6) -- ({x:.2f},3.6) -- ({x+0.5:.2f},3.6);")
            out.append(circ(x + 0.6, 3.55, 0.12, "cloud", 0.6))
    elif n == 1:  # the boardwalk: the sea, a pier, a wheel
        out = sky(r)
        out.append(f"\\filldraw[fill=teal, {LINE}] (0,1.6) rectangle ({W},3.4);")
        for k in range(50):
            x, y = r.uniform(0.2, W), r.uniform(1.8, 3.2)
            out.append(f"\\draw[tealdark, line width=0.6pt] ({x:.2f},{y:.2f}) .. controls ++(0.2,0.12) and ++(-0.2,0.12) .. ++(0.6,0);")
        cx, cy, rr = 22.0, 4.2, 1.6
        out.append(f"\\draw[ink, line width=1.2pt] ({cx:.2f},{cy:.2f}) circle ({rr:.2f});")
        for a in range(0, 360, 30):
            ra = math.radians(a)
            out.append(f"\\draw[ink, line width=0.6pt] ({cx:.2f},{cy:.2f}) -- ({cx+math.cos(ra)*rr:.2f},{cy+math.sin(ra)*rr:.2f});")
            out.append(rect(cx + math.cos(ra) * rr - 0.15, cy + math.sin(ra) * rr - 0.3, 0.3, 0.25, r.choice(["peach", "gold", "teal"]), ", line width=0.5pt"))
        out.append(f"\\draw[ink, line width=1.4pt] ({cx-1.0:.2f},1.6) -- ({cx:.2f},{cy:.2f}) -- ({cx+1.0:.2f},1.6);")
        out.append(rect(0, 1.3, W, 0.35, "wood"))
        for k in range(40):
            out.append(f"\\draw[ink, line width=0.4pt] ({k*0.8:.2f},1.3) -- ({k*0.8:.2f},1.65);")
        out.append(ground(1.3, "sand"))
        for k in range(8):
            x = r.uniform(1, 30)
            out.append(f"\\draw[ink, line width=0.7pt] ({x:.2f},5.6) .. controls ++(0.15,0.15) .. ++(0.3,0) .. controls ++(0.15,0.15) .. ++(0.3,0);")
    elif n == 2:  # the desert highway: mesas, cacti, poles
        out = sky(r, n=3)
        for x0, w, h in [(1, 5, 2.4), (9, 3.5, 1.8), (17, 6, 2.8), (26, 4, 2.0)]:
            out.append(poly([(x0, 2.6), (x0 + 0.6, 2.6 + h), (x0 + w - 0.6, 2.6 + h), (x0 + w, 2.6)], "peachdark"))
            out.append(f"\\draw[ink, line width=0.5pt] ({x0+0.3:.2f},{2.6+h*0.5:.2f}) -- ({x0+w-0.3:.2f},{2.6+h*0.5:.2f});")
        out.append(ground(2.6, "sand"))
        out.append(rect(0, 0.6, W, 0.9, "stone"))
        for k in range(16):
            out.append(rect(k * 2.0 + 0.4, 1.0, 0.9, 0.08, "gold", ", line width=0.4pt"))
        for x in [2.5, 7.0, 13.5, 19.5, 24.0, 30.0]:
            s = r.uniform(0.7, 1.1)
            out.append(f"\\filldraw[fill=sage, {LINE}] ({x-0.12*s:.2f},1.7) rectangle ({x+0.12*s:.2f},{1.7+1.0*s:.2f});")
            out.append(f"\\filldraw[fill=sage, {LINE}] ({x+0.12*s:.2f},{1.9+0.3*s:.2f}) -- ++({0.3*s:.2f},0) -- ++(0,{0.4*s:.2f}) -- ++({-0.12*s:.2f},0) -- ++(0,{-0.25*s:.2f}) -- ({x+0.12*s:.2f},{2.05+0.3*s:.2f}) -- cycle;")
        for x in range(1, 32, 5):
            out.append(f"\\draw[ink, line width=1pt] ({x:.2f},1.5) -- ({x:.2f},3.9) ({x-0.3:.2f},3.7) -- ({x+0.3:.2f},3.7);")
        out.append(f"\\draw[ink, line width=0.5pt] plot[smooth] coordinates {{" + " ".join(f"({x+0.0:.2f},{3.7-0.25*math.sin((x%5)/5*math.pi):.2f})" for x in [i * 0.5 + 1 for i in range(61)]) + "};")
    elif n == 3:  # the big city: towers and water tanks
        out = sky(r, n=3)
        x = 0.0
        while x < W:
            w = r.uniform(1.2, 2.4); h = r.uniform(2.0, 4.8)
            out += building(x, 1.6, w, h, r.choice(["peach", "rose", "stone", "sage"]), r)
            if r.random() < 0.3:
                out.append(rect(x + w * 0.3, 1.6 + h, 0.5, 0.45, "wood"))
                out.append(f"\\draw[ink, line width=0.6pt] ({x+w*0.32:.2f},{1.6+h:.2f}) -- ++(-0.1,-0.0) ({x+w*0.3+0.5:.2f},{1.6+h+0.45:.2f}) -- ++(-0.25,0.2) -- ++(-0.25,-0.2);")
            x += w + r.uniform(0.0, 0.3)
        out.append(ground(1.6, "stone"))
        for k in range(32):
            out.append(f"\\draw[ink, line width=0.4pt] ({k:.2f},0) -- ({k+0.2:.2f},1.6);")
    elif n == 4:  # the mountains: peaks, snow, pines
        out = sky(r)
        for x0, w, h in [(-1, 9, 4.6), (6, 8, 3.6), (12, 11, 5.4), (21, 9, 4.2), (27, 7, 3.8)]:
            out.append(poly([(x0, 2.2), (x0 + w / 2, 2.2 + h), (x0 + w, 2.2)], "slate"))
            out.append(poly([(x0 + w / 2 - w * 0.12, 2.2 + h * 0.76), (x0 + w / 2, 2.2 + h), (x0 + w / 2 + w * 0.12, 2.2 + h * 0.76), (x0 + w / 2 + 0.2, 2.2 + h * 0.7), (x0 + w / 2 - 0.2, 2.2 + h * 0.72)], "snow"))
        out.append(ground(2.2, "snow"))
        for k in range(22):
            out += pine(r.uniform(0.4, W - 0.4), r.uniform(0.6, 1.8), r.uniform(0.6, 1.0), snowy=True)
    elif n == 5:  # the islands: lagoon, palms
        out = sky(r)
        out.append(f"\\filldraw[fill=teal, {LINE}] (0,0.9) rectangle ({W},3.4);")
        for x0, w in [(2, 5), (13, 7), (25, 5)]:
            out.append(f"\\filldraw[fill=sagedark, {LINE}] ({x0:.2f},3.4) .. controls ({x0+w*0.3:.2f},{4.6:.2f}) and ({x0+w*0.7:.2f},{4.6:.2f}) .. ({x0+w:.2f},3.4) -- cycle;")
        for k in range(40):
            x, y = r.uniform(0.2, W), r.uniform(1.1, 3.2)
            out.append(f"\\draw[tealdark, line width=0.6pt] ({x:.2f},{y:.2f}) .. controls ++(0.2,0.12) and ++(-0.2,0.12) .. ++(0.6,0);")
        out.append(f"\\filldraw[fill=sand, {LINE}] (0,0) -- (0,1.2) " + " ".join(f"-- ({x:.2f},{1.2+0.25*math.sin(x):.2f})" for x in range(1, 33)) + f" -- ({W},0) -- cycle;")
        for x in [1.5, 6.0, 10.5, 17.0, 23.0, 29.5]:
            out += palm(x, 1.0, r.uniform(0.8, 1.2))
    elif n == 6:  # the old town: gabled rows, a canal, a clock tower
        out = sky(r)
        x = 0.0
        while x < W:
            w = r.uniform(1.0, 1.6); h = r.uniform(1.6, 2.6)
            out += building(x, 2.0, w, h, r.choice(["peach", "cream", "rose", "gold"]), r, roof=r.choice(["peachdark", "slate"]))
            x += w
        out.append(rect(14.8, 2.0, 1.6, 4.2, "stone"))
        out.append(poly([(14.6, 6.2), (15.6, 7.0), (16.6, 6.2)], "slate"))
        out.append(circ(15.6, 5.4, 0.5, "cream"))
        out.append(f"\\draw[ink, line width=0.8pt] (15.6,5.4) -- (15.6,5.75) (15.6,5.4) -- (15.85,5.3);")
        out.append(f"\\filldraw[fill=teal, {LINE}] (0,0.9) rectangle ({W},2.0);")
        for k in range(30):
            x0, y = r.uniform(0.2, W), r.uniform(1.1, 1.8)
            out.append(f"\\draw[tealdark, line width=0.6pt] ({x0:.2f},{y:.2f}) .. controls ++(0.2,0.1) and ++(-0.2,0.1) .. ++(0.5,0);")
        out.append(ground(0.9, "stone"))
        for k in range(64):
            out.append(f"\\draw[ink, line width=0.35pt] ({k*0.5:.2f},0.45) -- ({k*0.5+0.25:.2f},0.45);")
    elif n == 7:  # the night market: night sky, signs, lanterns
        out = [f"\\fill[night] (0,0) rectangle ({W},{H});", f"\\fill[nightlight] (0,{H*0.5:.2f}) rectangle ({W},{H});"]
        for k in range(40):
            out.append(f"\\fill[cloud] ({r.uniform(0,W):.2f},{r.uniform(4.5,6.9):.2f}) circle (0.04);")
        out.append(circ(27.0, 5.9, 0.55, "cloud"))
        x = 0.0
        while x < W:
            w = r.uniform(1.4, 2.4); h = r.uniform(2.0, 4.2)
            out += building(x, 1.6, w, h, r.choice(["shadow", "slate", "nightlight"]), r, windows="gold")
            if r.random() < 0.6:
                c = r.choice(["rose", "teal", "gold", "peach"])
                out.append(rect(x + w * 0.25, 1.6 + h * 0.45, 0.35, h * 0.45, c, ", line width=0.7pt"))
            x += w + r.uniform(0.0, 0.3)
        out.append(ground(1.6, "shadow"))
        out.append(f"\\draw[ink, line width=0.6pt] plot[smooth] coordinates {{" + " ".join(f"({x:.2f},{3.0-0.25*math.sin(x/2.0*math.pi):.2f})" for x in [i * 0.5 for i in range(65)]) + "};")
        for k in range(16):
            x = k * 2.0 + 1.0
            y = 3.0 - 0.25 * math.sin(x / 2.0 * math.pi) - 0.35
            out.append(f"\\filldraw[fill={'rose' if k % 2 else 'gold'}, draw=ink, line width=0.6pt] ({x:.2f},{y:.2f}) ellipse (0.2 and 0.28);")
    else:  # the end of the world: a storm at sea, a lighthouse
        out = [f"\\fill[slate] (0,0) rectangle ({W},{H});", f"\\fill[nightlight] (0,{H*0.55:.2f}) rectangle ({W},{H});"]
        for _ in range(8):
            cx, cy = r.uniform(0, W), r.uniform(H * 0.7, H * 0.95)
            blobs = " ".join(f"({cx + dx:.2f},{cy + dy:.2f}) circle ({rad:.2f})" for dx, dy, rad in
                             [(0, 0, 0.75), (0.8, 0.15, 0.6), (-0.8, 0.1, 0.55), (1.5, -0.05, 0.4), (-1.4, -0.05, 0.4)])
            out.append(f"\\filldraw[fill=shadow, draw=ink, line width=0.6pt] {blobs};")
        for k in range(70):
            x, y = r.uniform(0, W), r.uniform(1.5, 5.6)
            out.append(f"\\draw[cloud, line width=0.5pt] ({x:.2f},{y:.2f}) -- ++(-0.15,-0.45);")
        out.append(f"\\draw[gold, line width=1.4pt] (9.0,6.3) -- (8.5,5.4) -- (9.0,5.3) -- (8.4,4.3);")
        out.append(f"\\filldraw[fill=tealdark, {LINE}] (0,0) -- (0,2.4) " + " ".join(f"-- ({x*0.8:.2f},{2.4+0.35*math.sin(x*1.3):.2f})" for x in range(1, 41)) + f" -- ({W},0) -- cycle;")
        for k in range(40):
            x = r.uniform(0.2, W)
            out.append(f"\\draw[cloud, line width=0.7pt] ({x:.2f},{r.uniform(0.6,2.2):.2f}) .. controls ++(0.2,0.25) and ++(-0.1,0.2) .. ++(0.5,0.05);")
        out.append(poly([(20.5, 2.4), (24.5, 2.4), (23.8, 3.2), (21.2, 3.2)], "stone"))
        out.append(poly([(22.0, 3.2), (23.0, 3.2), (22.8, 6.0), (22.2, 6.0)], "cream"))
        for k in range(3):
            out.append(poly([(22.0 + 0.05 * k * 0 + 0.07 * k, 3.6 + k * 0.8), (23.0 - 0.07 * k, 3.6 + k * 0.8), (22.97 - 0.07 * k, 3.95 + k * 0.8), (22.03 + 0.07 * k, 3.95 + k * 0.8)], "rose"))
        out.append(rect(22.15, 6.0, 0.7, 0.45, "gold"))
        out.append(poly([(22.05, 6.45), (22.95, 6.45), (22.5, 6.85)], "rose"))
        out.append(f"\\fill[gold, opacity=0.35] (22.5,6.2) -- (14,6.9) -- (14,5.5) -- cycle;")
    return out


def render(n):
    r = random.Random(7000 + n)
    body = "\n".join(scene(n, r))
    tex = f"""\\documentclass[tikz,border=0pt]{{standalone}}
\\usepackage{{tikz}}
{colors()}
\\begin{{document}}
\\begin{{tikzpicture}}
\\clip (0,0) rectangle ({W},{H});
{body}
\\end{{tikzpicture}}
\\end{{document}}
"""
    OUT.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory() as d:
        Path(d, "f.tex").write_text(tex)
        rr = subprocess.run(["pdflatex", "-interaction=nonstopmode", "-halt-on-error", "f.tex"], cwd=d, capture_output=True, text=True)
        if rr.returncode:
            sys.exit(f"pdflatex failed for region {n}:\n" + "\n".join(l for l in rr.stdout.splitlines() if l.startswith("!") or l.startswith("l.")))
        subprocess.run(["pdftocairo", "-png", "-singlefile", "-scale-to-x", "1800", "-scale-to-y", "-1", "f.pdf", str(OUT / f"region-{n}")], cwd=d, check=True)


if __name__ == "__main__":
    regions = json.loads((ROOT / "data" / "copy.en.json").read_text())["world"]["region"]
    for k in (sys.argv[1:] or sorted((k for k in regions if k.isdigit()), key=int)):
        render(int(k))
        print("drew region", k)
