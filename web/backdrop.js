// The store behind the line (Sam, 2026-10-06: "a blurry background that
// looks like the inside of a 7/11 from the perspective of the worker"). A
// generic shop, seen from behind its counter: lights in the ceiling, the
// front windows and door on the left, coolers glowing along the right wall,
// aisles of packs between, the floor, and the counter nearest. No brand, no
// sign and no text (TONE.md rule 4): it is shapes in the palette's colors.
//
// It is scenery, not the game. It is painted small, blurred in script (the
// canvas `filter` is not in every engine), scaled up, and washed in paper so
// the slush, cups and gauges stay as legible as on plain paper.

// How much paper lies over the store: enough that it reads as depth and
// never as something to look at.
const WASH = 0.5;
// The width the scene is painted at before it is scaled up.
const SMALL = 360;

// A fixed little generator, so the packs sit in the same places every time
// the canvas is resized. Scenery only: nothing here touches the game.
function seeded(seed) {
  let a = seed >>> 0;
  return () => {
    a = (a + 0x6D2B79F5) >>> 0;
    let t = a;
    t = Math.imul(t ^ (t >>> 15), t | 1);
    t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

function paint(g, W, H, b) {
  const rnd = seeded(7);
  const packs = [b.pack_1, b.pack_2, b.pack_3, b.pack_4, b.pack_5, b.pack_6];
  const vx = W * 0.52, vy = H * 0.42;

  g.fillStyle = b.wall;
  g.fillRect(0, 0, W, H);

  // The ceiling and its strip lights, rows of them running away to the back.
  const ceil = H * 0.22;
  g.fillStyle = b.ceiling;
  g.fillRect(0, 0, W, ceil);
  g.fillStyle = b.light;
  for (let k = 0; k < 6; k += 1) {
    const s = 1 / (1 + 0.75 * k);
    const y = vy - H * 0.52 * s;
    if (y > ceil) continue;
    for (const c of [-1, 0, 1]) {
      const w = W * 0.13 * s;
      g.fillRect(vx + c * W * 0.4 * s - w / 2, y, w, Math.max(1, H * 0.03 * s));
    }
  }

  // The front: tall windows, and the door among them, bright with the day.
  const top = H * 0.19, low = H * 0.62;
  g.fillStyle = b.window;
  g.fillRect(W * 0.02, top, W * 0.28, low - top);
  g.fillStyle = b.door;
  g.fillRect(W * 0.12, top + H * 0.04, W * 0.08, low - top - H * 0.04);
  g.fillStyle = b.window_frame;
  for (const x of [0.02, 0.12, 0.2, 0.3]) g.fillRect(W * x - 1, top, 2, low - top);
  g.fillRect(W * 0.02, top, W * 0.28, 2);
  g.fillRect(W * 0.02, H * 0.4, W * 0.1, 1.5);
  g.fillRect(W * 0.2, H * 0.4, W * 0.1, 1.5);
  g.fillRect(W * 0.185, H * 0.36, 1.5, H * 0.1);

  // The coolers along the right wall: lit doors with shelves of drinks.
  const doors = 5;
  const cx0 = W * 0.66, cw = (W * 0.98 - cx0) / doors;
  g.fillStyle = b.cooler;
  g.fillRect(cx0, H * 0.17, W * 0.98 - cx0, low - H * 0.17);
  for (let d = 0; d < doors; d += 1) {
    const x = cx0 + d * cw + 2, w = cw - 4, y = H * 0.2, h = low - H * 0.2 - 3;
    g.fillStyle = b.cooler_glow;
    g.fillRect(x, y, w, h);
    for (let r = 0; r < 5; r += 1) {
      const sy = y + (r + 0.8) * (h / 5.2);
      for (let k = 0; k < 4; k += 1) {
        g.fillStyle = packs[Math.floor(rnd() * packs.length)];
        g.fillRect(x + 1 + k * (w - 2) / 4, sy - h * 0.09, (w - 2) / 4 - 1, h * 0.09);
      }
      g.fillStyle = b.shelf_edge;
      g.fillRect(x, sy, w, 1);
    }
  }

  // The floor, from the back of the shop to the counter.
  const floor = H * 0.6;
  const grad = g.createLinearGradient(0, floor, 0, H * 0.86);
  grad.addColorStop(0, b.floor_far);
  grad.addColorStop(1, b.floor);
  g.fillStyle = grad;
  g.fillRect(0, floor, W, H * 0.86 - floor);

  // The aisles between: shelf units of packs, the far ones smaller.
  for (let k = 3; k >= 0; k -= 1) {
    const s = 1 / (1 + 0.45 * k);
    const w = W * 0.1 * s, h = H * 0.3 * s;
    const base = vy + (H * 0.66 - vy) * s;
    for (const side of [-1, 1]) {
      const x = vx + side * W * 0.11 * s - w / 2;
      g.fillStyle = b.shelf;
      g.fillRect(x, base - h, w, h);
      for (let r = 0; r < 4; r += 1) {
        const sy = base - h + (r + 1) * h / 4.4;
        for (let p = 0; p < 5; p += 1) {
          g.fillStyle = packs[Math.floor(rnd() * packs.length)];
          g.fillRect(x + 1 + p * (w - 2) / 5, sy - h * 0.14, (w - 2) / 5 - 0.5, h * 0.14);
        }
        g.fillStyle = b.shelf_edge;
        g.fillRect(x, sy, w, Math.max(1, h * 0.02));
      }
    }
  }

  // The counter, nearest, where the machine stands.
  g.fillStyle = b.counter;
  g.fillRect(0, H * 0.86, W, H * 0.14);
  g.fillStyle = b.counter_edge;
  g.fillRect(0, H * 0.86, W, Math.max(1.5, H * 0.012));
}

// A box blur, run a few times, which is close to a gaussian. Done on the
// small picture, so it is cheap and the same in every engine.
function blur(g, W, H, r, passes = 3) {
  const img = g.getImageData(0, 0, W, H);
  const d = img.data;
  const tmp = new Float32Array(d.length);
  const line = (n, at) => {
    for (let c = 0; c < 4; c += 1) {
      let sum = 0;
      const idx = (i) => at(Math.max(0, Math.min(n - 1, i))) + c;
      for (let i = -r; i <= r; i += 1) sum += d[idx(i)];
      for (let i = 0; i < n; i += 1) {
        tmp[at(i) + c] = sum / (2 * r + 1);
        sum += d[idx(i + r + 1)] - d[idx(i - r)];
      }
    }
    for (let i = 0; i < n; i += 1) for (let c = 0; c < 4; c += 1) d[at(i) + c] = tmp[at(i) + c];
  };
  for (let p = 0; p < passes; p += 1) {
    for (let y = 0; y < H; y += 1) line(W, (x) => (y * W + x) * 4);
    for (let x = 0; x < W; x += 1) line(H, (y) => (y * W + x) * 4);
  }
  g.putImageData(img, 0, 0);
}

// The backdrop at a canvas's size, made once per size and kept.
export class Backdrop {
  constructor(palette) {
    this.pal = palette;
    this.key = '';
    this.big = null;
  }

  at(w, h) {
    const key = `${w}x${h}`;
    if (key === this.key) return this.big;
    const W = SMALL, H = Math.max(40, Math.round(SMALL * h / w));
    const small = document.createElement('canvas');
    small.width = W;
    small.height = H;
    const sg = small.getContext('2d');
    paint(sg, W, H, this.pal.backdrop);
    blur(sg, W, H, 2);
    const big = document.createElement('canvas');
    big.width = w;
    big.height = h;
    const bg = big.getContext('2d');
    bg.imageSmoothingEnabled = true;
    if ('imageSmoothingQuality' in bg) bg.imageSmoothingQuality = 'high';
    bg.drawImage(small, 0, 0, w, h);
    bg.globalAlpha = WASH;
    bg.fillStyle = this.pal.paper;
    bg.fillRect(0, 0, w, h);
    this.key = key;
    this.big = big;
    return big;
  }
}
