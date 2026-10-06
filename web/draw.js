// Drawing. Everything here is a number core sent: positions in raw fixed
// point, counts, openings. The page converts units and picks colors from the
// palette; it never integrates, predicts or detects a contact (D2).

// The view: centimeters of the line shown across the canvas. A layout
// choice of the page, not a rule of the game.
const VIEW_LEFT = -10;
const VIEW_WIDTH = 480;
// The lowest centimeter shown: a little below the belt, which is at 50.
const VIEW_BOTTOM = -5;

export class Stage {
  constructor(canvas, numbers, palette, flavors) {
    this.c = canvas;
    this.g = canvas.getContext('2d');
    this.one = 1 << numbers.frac_bits;
    this.pal = palette;
    // Flavor number -> { id, pattern }.
    this.flavors = flavors;
    this.showCodes = false;
    this.codes = [];
  }

  get scale() { return this.c.width / VIEW_WIDTH; }

  // Fit the canvas's height to the lines this frame has: from a little
  // below the lowest belt to a little above the highest handle. A mission
  // with two lines is a taller picture, not a smaller one.
  fit(frame) {
    // Wide as the page lets it be, at the screen's own pixel density, so a
    // bigger window is a bigger line (Sam, 2026-10-05: "generally
    // everything should be larger").
    const css = this.c.clientWidth;
    if (css > 0) {
      const w = Math.max(900, Math.min(2800, Math.round(css * (window.devicePixelRatio || 1))));
      if (Math.abs(this.c.width - w) > 2) this.c.width = w;
    }
    let top = 0;
    for (const L of frame.lines) for (const sp of L.spouts) top = Math.max(top, sp.tip[1] / this.one);
    const h = Math.round((top + 18 - VIEW_BOTTOM) * this.scale);
    if (h > 0 && this.c.height !== h) {
      this.c.height = h;
      if (this.onResize) this.onResize(this.c.width, h);
    }
  }
  sx(raw) { return (raw / this.one - VIEW_LEFT) * this.scale; }
  sy(raw) { return this.c.height - (raw / this.one - VIEW_BOTTOM) * this.scale; }
  len(raw) { return (raw / this.one) * this.scale; }

  fill(f) { return this.pal.flavors[this.flavors[f].id].fill; }

  // A flavor's pattern inside a circle or a box, in the pattern ink that
  // reads against its fill. Patterns are the channel that survives with no
  // color at all (PLANNING-BRIEF 0.5).
  ink(f) {
    return this.flavors[f].id === 'cola' ? this.pal.pattern_ink_on_dark : this.pal.pattern_ink;
  }

  patternInCircle(f, x, y, r) {
    const g = this.g;
    const kind = this.flavors[f].pattern;
    if (kind === 'stripes') {
      g.strokeStyle = this.ink(f);
      g.lineWidth = Math.max(1, r * 0.26);
      g.beginPath();
      g.moveTo(x - r * 0.62, y + r * 0.62);
      g.lineTo(x + r * 0.62, y - r * 0.62);
      g.stroke();
    } else if (kind === 'dots') {
      g.fillStyle = this.ink(f);
      g.beginPath();
      g.arc(x, y, Math.max(0.9, r * 0.28), 0, Math.PI * 2);
      g.fill();
    }
  }

  patternInBox(f, x, y, w, h) {
    const g = this.g;
    const kind = this.flavors[f].pattern;
    g.save();
    g.beginPath();
    g.rect(x, y, w, h);
    g.clip();
    if (kind === 'stripes') {
      g.strokeStyle = this.ink(f);
      g.lineWidth = 2;
      for (let k = -h; k < w + h; k += 7) {
        g.beginPath();
        g.moveTo(x + k, y + h);
        g.lineTo(x + k + h, y);
        g.stroke();
      }
    } else if (kind === 'dots') {
      g.fillStyle = this.ink(f);
      for (let yy = y + 3; yy < y + h; yy += 7) {
        for (let xx = x + 3 + ((yy - y) % 14 ? 3 : 0); xx < x + w; xx += 7) {
          g.beginPath();
          g.arc(xx, yy, 1.4, 0, Math.PI * 2);
          g.fill();
        }
      }
    }
    g.restore();
  }

  // One frame: the line, its spouts, cups and slush.
  draw(frame, units, keyNames, lowerKeyNames = []) {
    this.fit(frame);
    this.tick = frame.tick;
    const g = this.g;
    const p = this.pal;
    g.fillStyle = p.paper;
    g.fillRect(0, 0, this.c.width, this.c.height);
    for (const line of frame.lines) this.drawFields(line);
    for (const line of frame.lines) this.drawLine(line, line.seat === 1 ? lowerKeyNames : keyNames);
    this.drawUnits(units);
    for (const line of frame.lines) this.drawBars(line);
  }

  // Fields (Sam, 2026-10-05: "gravity fields that act almost like voltage
  // fields"). Plates: a tinted box between two bars marked + and −, with
  // chevrons drifting the way it pushes now. A charge: rings that pulse out
  // if it pushes and in if it pulls, and arrows the same way. Which way is
  // carried by shape and motion, never by color alone.
  drawFields(L) {
    const g = this.g;
    const p = this.pal;
    const t = this.tick ?? 0;
    for (const f of L.fields) {
      g.save();
      if (f.kind === 'plates') {
        const x0 = this.sx(f.x0), x1 = this.sx(f.x1), y0 = this.sy(f.y1), y1 = this.sy(f.y0);
        g.globalAlpha = 0.13;
        g.fillStyle = p.field;
        g.fillRect(x0, y0, x1 - x0, y1 - y0);
        g.globalAlpha = 0.9;
        g.fillStyle = p.field_ink;
        g.fillRect(x0 - 4, y0, 4, y1 - y0);
        g.fillRect(x1, y0, 4, y1 - y0);
        const dir = Math.sign(f.ax) || 1;
        // The push runs from the + plate to the − plate.
        g.font = '700 16px system-ui, sans-serif';
        g.textAlign = 'center';
        g.fillText(dir > 0 ? '+' : '\u2212', x0 - 12, y0 + 16);
        g.fillText(dir > 0 ? '\u2212' : '+', x1 + 12, y0 + 16);
        g.strokeStyle = p.field_ink;
        g.lineWidth = 2.5;
        g.globalAlpha = 0.55;
        const span = x1 - x0;
        const shift = ((t * 0.9) % 28) * dir;
        for (let y = y0 + 14; y < y1 - 6; y += 22) {
          for (let x = x0 + 10 + ((shift % 28) + 28) % 28; x < x1 - 8; x += 28) {
            g.beginPath();
            g.moveTo(x - 5 * dir, y - 6);
            g.lineTo(x + 3 * dir, y);
            g.lineTo(x - 5 * dir, y + 6);
            g.stroke();
          }
        }
        if (f.turns) {
          // A turning field: a little ring of arrows over it.
          g.globalAlpha = 0.8;
          g.beginPath();
          g.arc(x0 + span / 2, y0 - 12, 8, 0.3, Math.PI * 1.7);
          g.stroke();
        }
      } else {
        const x = this.sx(f.x), y = this.sy(f.y), r = this.len(f.radius);
        const phase = ((t % 60) / 60);
        for (let k = 0; k < 3; k += 1) {
          const a = (phase + k / 3) % 1;
          const rr = f.pulls ? r * (1 - a) : r * a;
          g.globalAlpha = 0.45 * (1 - Math.abs(0.5 - a) * 1.2);
          g.strokeStyle = p.field;
          g.lineWidth = 3;
          g.beginPath();
          g.arc(x, y, Math.max(2, rr), 0, Math.PI * 2);
          g.stroke();
        }
        g.globalAlpha = 0.8;
        g.strokeStyle = p.field_ink;
        g.lineWidth = 2;
        for (let k = 0; k < 8; k += 1) {
          const ang = (k / 8) * Math.PI * 2;
          const a0 = r * 0.35, a1 = r * 0.6;
          const [from, to] = f.pulls ? [a1, a0] : [a0, a1];
          const fx = x + Math.cos(ang) * from, fy = y + Math.sin(ang) * from;
          const tx = x + Math.cos(ang) * to, ty = y + Math.sin(ang) * to;
          g.beginPath();
          g.moveTo(fx, fy);
          g.lineTo(tx, ty);
          g.stroke();
          const back = ang + Math.PI + (f.pulls ? 0 : 0);
          const hx = Math.cos(f.pulls ? ang + Math.PI : ang), hy = Math.sin(f.pulls ? ang + Math.PI : ang);
          g.beginPath();
          g.moveTo(tx, ty);
          g.lineTo(tx - hx * 5 - hy * 4, ty - hy * 5 + hx * 4);
          g.moveTo(tx, ty);
          g.lineTo(tx - hx * 5 + hy * 4, ty - hy * 5 - hx * 4);
          g.stroke();
          void back;
        }
        g.globalAlpha = 1;
        g.fillStyle = p.field_ink;
        g.beginPath();
        g.arc(x, y, 9, 0, Math.PI * 2);
        g.fill();
        g.fillStyle = p.paper;
        g.font = '700 14px system-ui, sans-serif';
        g.textAlign = 'center';
        g.fillText(f.pulls ? '\u2212' : '+', x, y + 5);
      }
      g.restore();
    }
  }

  // The waste tray (Sam, 2026-10-05): under the belt, a grate, and under the
  // grate the slush that missed, piled in the bins core kept it in, in the
  // order it landed, each unit in its flavor's fill and pattern.
  drawTray(L) {
    const g = this.g;
    const p = this.pal;
    const one = this.one;
    const top = L.belt_y - 30 * one;      // the grate
    const floor = L.belt_y - 52 * one;    // the tray's bottom
    const x0 = this.sx(0), x1 = this.sx(L.end_x);
    const yTop = this.sy(top), yFloor = this.sy(floor);
    g.fillStyle = p.machine;
    g.fillRect(x0, yTop, x1 - x0, yFloor - yTop);
    const r = 1.3;
    const perRow = Math.floor(L.tray_bin / one / (2 * r));
    const rowH = 2 * r * 0.88;
    // Lay the waste out as mounds: units are taken a turn from each bin at a
    // time, roughly the order they fell, and one that would sit a whole row
    // above a neighbor rolls down to it. How many fell where is core's; this
    // only arranges them so a pile looks like a pile.
    const n = L.tray.length;
    const filled = new Array(n).fill(0);
    const rows = Math.floor((top - floor) / one / rowH);
    const at = [];
    const longest = Math.max(0, ...L.tray.map((b) => b.length));
    for (let k = 0; k < longest; k += 1) {
      for (let b0 = 0; b0 < n; b0 += 1) {
        const f = L.tray[b0][k];
        if (f === undefined) continue;
        let b = b0;
        for (let step = 0; step < n; step += 1) {
          const l = b > 0 ? filled[b - 1] : Infinity;
          const r2 = b < n - 1 ? filled[b + 1] : Infinity;
          const low = l <= r2 ? b - 1 : b + 1;
          if (Math.min(l, r2) + perRow <= filled[b]) b = low; else break;
        }
        const slot = filled[b];
        filled[b] += 1;
        const row = Math.floor(slot / perRow);
        if (row >= rows) continue;
        const col = slot % perRow;
        const x = b * L.tray_bin / one + r + col * 2 * r + (row % 2 ? r * 0.5 : 0);
        const y = floor / one + r + row * rowH;
        at.push([this.sx(x * one), this.sy(y * one), this.len(r * one), f]);
      }
    }
    g.fillStyle = p.line;
    for (const [x, y, rr] of at) { g.beginPath(); g.arc(x, y, rr + 0.8, 0, Math.PI * 2); g.fill(); }
    for (const [x, y, rr, f] of at) { g.fillStyle = this.fill(f); g.beginPath(); g.arc(x, y, rr, 0, Math.PI * 2); g.fill(); }
    for (const [x, y, rr, f] of at) this.patternInCircle(f, x, y, rr);
    g.strokeStyle = p.line;
    g.lineWidth = 1.5;
    g.strokeRect(x0, yTop, x1 - x0, yFloor - yTop);
    // The grate: a bar with slots, the way the drip grate on a machine looks.
    g.fillStyle = p.belt_mark;
    g.fillRect(x0, yTop - 4, x1 - x0, 5);
    g.fillStyle = p.paper;
    for (let x = x0 + 4; x < x1 - 6; x += 10) g.fillRect(x, yTop - 3, 6, 3);
  }

  drawLine(L, keyNames) {
    this.drawTray(L);
    const g = this.g;
    const p = this.pal;
    // The belt, with marks that move with it.
    const top = this.sy(L.belt_y);
    const left = this.sx(0);
    const right = this.sx(L.end_x);
    g.fillStyle = p.belt;
    g.fillRect(left, top, right - left, this.len(6 * this.one));
    g.strokeStyle = p.belt_mark;
    g.lineWidth = 2;
    const step = 12 * this.one;
    const shift = L.travel % step;
    for (let x = shift; x < L.end_x; x += step) {
      g.beginPath();
      g.moveTo(this.sx(x), top + 3);
      g.lineTo(this.sx(x), top + this.len(6 * this.one) - 3);
      g.stroke();
    }
    // The lid's press, over the belt where cups are judged.
    const lidX = this.sx(L.lid_x);
    const rimY = this.sy(L.rim);
    const u = this.scale;
    g.fillStyle = p.machine;
    g.fillRect(lidX - this.len(L.inner_half) - 3 * u, rimY - 14 * u, this.len(L.inner_half) * 2 + 6 * u, 5 * u);
    g.fillStyle = p.lid;
    g.fillRect(lidX - 1.2 * u, rimY - 9 * u, 2.4 * u, 5 * u);
    // Spouts: the machine above, the nozzle, and the handle as core placed
    // it, sized in centimeters so they grow with the canvas.
    L.spouts.forEach((s, i) => {
      const x = this.sx(s.x);
      const y = this.sy(s.y);
      g.fillStyle = p.machine;
      g.fillRect(x - 9 * u, y - 15 * u, 18 * u, 13 * u);
      g.strokeStyle = p.line;
      g.lineWidth = Math.max(1.5, 0.7 * u);
      g.strokeRect(x - 9 * u, y - 15 * u, 18 * u, 13 * u);
      // The nozzle, filled with the flavors it pours, in their cycle.
      const pours = s.pours;
      const nw = 9 * u;
      const w = nw / pours.length;
      pours.forEach((f, k) => {
        g.fillStyle = this.fill(f);
        g.fillRect(x - nw / 2 + k * w, y - 2 * u, w, 5 * u);
        this.patternInBox(f, x - nw / 2 + k * w, y - 2 * u, w, 5 * u);
      });
      g.lineWidth = Math.max(1.5, 0.8 * u);
      g.strokeRect(x - nw / 2, y - 2 * u, nw, 5 * u);
      // The short codes and the key, on the machine.
      g.fillStyle = p.line;
      g.font = `700 ${Math.round(5.2 * u)}px system-ui, sans-serif`;
      g.textAlign = 'center';
      g.fillText(pours.map((f) => this.codes[f]).filter((c, k, a) => a.indexOf(c) === k).join('+'), x, y - 9.4 * u);
      g.font = `800 ${Math.round(5.6 * u)}px system-ui, sans-serif`;
      g.fillText(keyNames[i] ?? '', x, y - 3.6 * u);
      // The handle: a stick from its pivot, opening toward the belt's end.
      g.strokeStyle = p.line;
      g.lineWidth = 2.2 * u;
      g.lineCap = 'round';
      g.beginPath();
      g.moveTo(this.sx(s.pivot[0]), this.sy(s.pivot[1]));
      g.lineTo(this.sx(s.tip[0]), this.sy(s.tip[1]));
      g.stroke();
      g.fillStyle = p.focus;
      g.beginPath();
      g.arc(this.sx(s.tip[0]), this.sy(s.tip[1]), 2.8 * u, 0, Math.PI * 2);
      g.fill();
    });
    // Cups.
    for (const c of L.cups) {
      const cx = this.sx(c.x);
      if (cx < -80 || cx > this.c.width + 80) continue;
      const half = this.len(L.inner_half);
      const wall = this.len(L.wall_half) * 2;
      const floorY = this.sy(L.floor);
      g.fillStyle = p.cup;
      g.fillRect(cx - half, rimY, half * 2, floorY - rimY);
      g.fillStyle = p.line;
      g.fillRect(cx - half - wall, rimY, wall, floorY - rimY + wall);
      g.fillRect(cx + half, rimY, wall, floorY - rimY + wall);
      g.fillRect(cx - half - wall, floorY, half * 2 + wall * 2, wall);
      if (c.judged) {
        g.fillStyle = p.lid;
        g.fillRect(cx - half - wall - 2, rimY - 6, half * 2 + wall * 2 + 4, 6);
        // The score the lid gave it, as core sent it.
        if (c.score !== null && c.score !== undefined) {
          g.fillStyle = p.line;
          g.font = '700 18px system-ui, sans-serif';
          g.textAlign = 'center';
          g.fillText(String(c.score), cx, rimY - 14);
        }
      }
    }
  }

  // Slush in three passes: every unit's outline in the line color, a little
  // larger than the unit; then every fill; then every pattern. Each unit has
  // its outline, and where units touch the fills cover the inner edges, so
  // the outline shows round the mass and a cup of lemon reads as one bright
  // area. Outlined and dotted one by one at 4 to 5 pixels, lemon read darker
  // than cherry in gray (SECOND-ORDER-M3).
  drawUnits(u) {
    const g = this.g;
    const at = [];
    for (let k = 0; k < u.length; k += 5) {
      at.push([this.sx(u[k + 1]), this.sy(u[k + 2]), this.len(u[k + 3]) * 1.12, u[k + 4] & 255]);
    }
    g.fillStyle = this.pal.line;
    for (const [x, y, r] of at) {
      g.beginPath();
      g.arc(x, y, r + 1, 0, Math.PI * 2);
      g.fill();
    }
    for (const [x, y, r, f] of at) {
      g.fillStyle = this.fill(f);
      g.beginPath();
      g.arc(x, y, r, 0, Math.PI * 2);
      g.fill();
    }
    for (const [x, y, r, f] of at) this.patternInCircle(f, x, y, r);
  }

  // The order bar under each cup, below the belt (Sam, 2026-10-05): split by share, each segment in its
  // flavor's fill and pattern, with a marker for how much of that flavor the
  // cup holds so far. Drawn from the shares and counts core sent.
  drawBars(L) {
    const g = this.g;
    const p = this.pal;
    const u = this.scale;
    for (const c of L.cups) {
      if (c.judged) continue;
      const cx = this.sx(c.x);
      if (cx < -80 || cx > this.c.width + 80) continue;
      // The fill gauge (Sam, 2026-10-05: "the fill bar below the cup being
      // quite large and quite visible"): wider than the cup, a segment for
      // each share, each filling from its left in its flavor's fill and
      // pattern as that flavor reaches its share, with its short code on it.
      const half = this.len(L.inner_half) + this.len(L.wall_half) * 2 + 3 * u;
      const y = this.sy(L.belt_y) + this.len(6 * this.one) + 3 * u;
      const h = 15 * u;
      let x = cx - half;
      const total = c.shares.reduce((a, s) => a + s[1], 0) || 1;
      g.fillStyle = p.paper;
      g.fillRect(x - 0.6 * u, y - 0.6 * u, half * 2 + 1.2 * u, h + 1.2 * u);
      for (const [f, share] of c.shares) {
        const w = (share / total) * half * 2;
        const have = c.counts[f] ?? 0;
        const part = Math.min(1, have / Math.max(share, 1));
        g.fillStyle = p.cup;
        g.fillRect(x, y, w, h);
        if (part > 0) {
          g.fillStyle = this.fill(f);
          g.fillRect(x, y, w * part, h);
          this.patternInBox(f, x, y, w * part, h);
        }
        // Past its share: a heavy notch at the segment's end.
        if (have > share) {
          g.fillStyle = p.line;
          g.fillRect(x + w - 2.2 * u, y - 2 * u, 2.2 * u, h + 4 * u);
        }
        g.strokeStyle = p.line;
        g.lineWidth = Math.max(2, 0.9 * u);
        g.strokeRect(x, y, w, h);
        // The short code, centered, in dark letters edged with paper so it
        // reads on any fill without hiding how full the segment is.
        g.font = `800 ${Math.round(Math.min(7.5 * u, w * 0.42))}px system-ui, sans-serif`;
        g.textAlign = 'center';
        g.lineJoin = 'round';
        g.strokeStyle = p.paper;
        g.lineWidth = 1.1 * u;
        g.strokeText(this.codes[f], x + w / 2, y + h / 2 + 2.6 * u);
        g.fillStyle = p.line;
        g.fillText(this.codes[f], x + w / 2, y + h / 2 + 2.6 * u);
        x += w;
      }
      // Settings' switch: each flavor's short code on its slush in the cup,
      // at the middle of that flavor's units, which core worked out.
      if (this.showCodes) {
        g.font = '700 12px system-ui, sans-serif';
        g.textAlign = 'center';
        for (const [f, x, y] of c.centers) {
          const px = this.sx(x);
          const py = this.sy(y);
          g.fillStyle = p.cup;
          g.fillRect(px - 10, py - 9, 20, 13);
          g.fillStyle = p.line;
          g.fillText(this.codes[f], px, py + 1);
        }
      }
    }
  }
}
