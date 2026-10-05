// Drawing. Everything here is a number core sent: positions in raw fixed
// point, counts, openings. The page converts units and picks colors from the
// palette; it never integrates, predicts or detects a contact (D2).

// The view: centimeters of the line shown across the canvas. A layout
// choice of the page, not a rule of the game.
const VIEW_LEFT = -10;
const VIEW_WIDTH = 480;

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
  sx(raw) { return (raw / this.one - VIEW_LEFT) * this.scale; }
  sy(raw) { return this.c.height - (raw / this.one) * this.scale; }
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
  draw(frame, units, keyNames) {
    const g = this.g;
    const p = this.pal;
    g.fillStyle = p.paper;
    g.fillRect(0, 0, this.c.width, this.c.height);
    for (const line of frame.lines) this.drawLine(line, keyNames);
    this.drawUnits(units);
    for (const line of frame.lines) this.drawBars(line);
  }

  drawLine(L, keyNames) {
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
    g.fillStyle = p.machine;
    g.fillRect(lidX - this.len(L.inner_half) - 8, rimY - 34, this.len(L.inner_half) * 2 + 16, 14);
    g.fillStyle = p.lid;
    g.fillRect(lidX - 3, rimY - 22, 6, 12);
    // Spouts: the machine above, the nozzle, and the handle as core placed it.
    L.spouts.forEach((s, i) => {
      const x = this.sx(s.x);
      const y = this.sy(s.y);
      g.fillStyle = p.machine;
      g.fillRect(x - 18, y - 40, 36, 34);
      g.strokeStyle = p.line;
      g.lineWidth = 1.5;
      g.strokeRect(x - 18, y - 40, 36, 34);
      // The nozzle, filled with the flavors it pours, in their cycle.
      const pours = s.pours;
      const w = 12 / pours.length;
      pours.forEach((f, k) => {
        g.fillStyle = this.fill(f);
        g.fillRect(x - 6 + k * w, y - 6, w, 8);
        this.patternInBox(f, x - 6 + k * w, y - 6, w, 8);
      });
      g.strokeRect(x - 6, y - 6, 12, 8);
      // The short codes and the key, on the machine.
      g.fillStyle = p.line;
      g.font = '600 12px system-ui, sans-serif';
      g.textAlign = 'center';
      g.fillText(pours.map((f) => this.codes[f]).filter((c, k, a) => a.indexOf(c) === k).join('+'), x, y - 24);
      g.font = '700 13px system-ui, sans-serif';
      g.fillText(keyNames[i] ?? '', x, y - 10);
      // The handle: a stick from its pivot, opening toward the belt's end.
      g.strokeStyle = p.line;
      g.lineWidth = 4;
      g.lineCap = 'round';
      g.beginPath();
      g.moveTo(this.sx(s.pivot[0]), this.sy(s.pivot[1]));
      g.lineTo(this.sx(s.tip[0]), this.sy(s.tip[1]));
      g.stroke();
      g.fillStyle = p.focus;
      g.beginPath();
      g.arc(this.sx(s.tip[0]), this.sy(s.tip[1]), 5, 0, Math.PI * 2);
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
      at.push([this.sx(u[k + 1]), this.sy(u[k + 2]), this.len(u[k + 3]), u[k + 4] & 255]);
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

  // The order bar over each cup: split by share, each segment in its
  // flavor's fill and pattern, with a marker for how much of that flavor the
  // cup holds so far. Drawn from the shares and counts core sent.
  drawBars(L) {
    const g = this.g;
    const p = this.pal;
    const rimY = this.sy(L.rim);
    for (const c of L.cups) {
      if (c.judged) continue;
      const cx = this.sx(c.x);
      if (cx < -80 || cx > this.c.width + 80) continue;
      const half = this.len(L.inner_half) + 2;
      const y = rimY - 22;
      const h = 12;
      let x = cx - half;
      const total = c.shares.reduce((a, s) => a + s[1], 0) || 1;
      for (const [f, share] of c.shares) {
        const w = (share / total) * half * 2;
        g.fillStyle = this.fill(f);
        g.fillRect(x, y, w, h);
        this.patternInBox(f, x, y, w, h);
        g.strokeStyle = p.line;
        g.lineWidth = 1;
        g.strokeRect(x, y, w, h);
        // How much of this flavor's share is in the cup: a tick mark.
        const have = Math.min(c.counts[f] ?? 0, share * 2);
        const mx = x + Math.min(1, have / Math.max(share, 1)) * w;
        g.fillStyle = p.line;
        g.fillRect(mx - 1.5, y - 5, 3, h + 10);
        g.fillStyle = p.line;
        g.font = '600 10px system-ui, sans-serif';
        g.textAlign = 'left';
        g.fillText(this.codes[f], x + 2, y - 3);
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
