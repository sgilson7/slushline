#!/usr/bin/env python3
"""The home-screen icon (Sam, 2026-10-10: Slushline on the iPad), in the
store art's style: the chain's sign, a slush cup with a dome lid and a
straw, on the mustard sky, with plum ink lines. Colors from
data/palette.json's `art` section, as the rest of the art.

    python3 analysis/art/icon.py     # writes web/art/icon-180.png, -192, -512
"""
import json, subprocess, sys, tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
PAL = json.loads((ROOT / "data" / "palette.json").read_text())["art"]
OUT = ROOT / "web" / "art"


def colors():
    return "\n".join(f"\\definecolor{{{k.replace('_','')}}}{{HTML}}{{{v[1:]}}}" for k, v in PAL.items() if not k.startswith("_"))


BODY = r"""
\fill[sky] (0,0) rectangle (10,10);
\fill[skylight] (0,5.6) rectangle (10,10);
\foreach \x/\y/\r in {1.6/8.6/0.55, 2.3/8.75/0.45, 8.2/8.3/0.5, 8.8/8.45/0.4} { \filldraw[fill=cloud, draw=ink, line width=1.2pt] (\x,\y) circle (\r); }
\filldraw[fill=sage, draw=ink, line width=2pt] (0,0) rectangle (10,1.6);
% The cup: tapered, its slush showing under a dome.
\filldraw[fill=cream, draw=ink, line width=3pt, line join=round] (3.1,1.6) -- (6.9,1.6) -- (7.7,6.4) -- (2.3,6.4) -- cycle;
\filldraw[fill=rose, draw=ink, line width=2pt, line join=round] (3.25,2.3) -- (6.75,2.3) -- (7.3,5.6) -- (2.7,5.6) -- cycle;
\filldraw[fill=teal, draw=ink, line width=3pt] (2.1,6.4) arc (180:0:2.9) -- cycle;
\filldraw[fill=cream, draw=ink, line width=2pt] (1.9,6.25) rectangle (8.1,6.65);
\draw[ink, line width=7pt, line cap=round] (5.5,8.6) -- (6.4,9.6);
\draw[gold, line width=4pt, line cap=round] (5.5,8.6) -- (6.4,9.6);
\foreach \x/\y in {4.0/4.1, 5.3/3.4, 4.7/4.9, 6.0/4.5} { \filldraw[fill=peachdark, draw=ink, line width=1pt] (\x,\y) circle (0.22); }
"""


def render():
    tex = f"""\\documentclass[tikz,border=0pt]{{standalone}}
\\usepackage{{tikz}}
{colors()}
\\begin{{document}}
\\begin{{tikzpicture}}
\\clip (0,0) rectangle (10,10);
{BODY}
\\end{{tikzpicture}}
\\end{{document}}
"""
    OUT.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory() as d:
        Path(d, "f.tex").write_text(tex)
        rr = subprocess.run(["pdflatex", "-interaction=nonstopmode", "-halt-on-error", "f.tex"], cwd=d, capture_output=True, text=True)
        if rr.returncode:
            sys.exit("pdflatex failed:\n" + "\n".join(l for l in rr.stdout.splitlines() if l.startswith("!") or l.startswith("l.")))
        # Opaque: iOS fills a transparent corner with black.
        for px in (180, 192, 512):
            subprocess.run(["pdftocairo", "-png", "-singlefile", "-scale-to-x", str(px), "-scale-to-y", str(px), "-x", "0", "-y", "0", "-W", str(px), "-H", str(px), "f.pdf", str(OUT / f"icon-{px}")], cwd=d, check=True)


if __name__ == "__main__":
    render()
    print("drew the icon")
