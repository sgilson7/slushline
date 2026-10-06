// Drawing between two ticks. Core steps sixty times a second and the screen
// refreshes at its own rate (60, 75, 120, 144 Hz…), so a frame drawn at the
// newest tick advances one tick, then none, then two, unevenly. That reads as
// a stutter wherever things move fast, which is the falling slush the moment
// a spout opens (Sam, 2026-10-06: "whenever the spout opens there is a little
// bit of framerate lag or something"). The page may interpolate between two
// frames it was given (CLAUDE.md), so it draws at the right fraction between
// the last two ticks core sent. It predicts nothing past the newest tick.

const lerp = (a, b, t) => a + (b - a) * t;

// Each unit where it is `t` of the way from its place in `prev` to its place
// in `cur`, matched by its id and line. A unit new this tick is drawn where
// core put it.
export function blendUnits(prev, cur, t) {
  if (!prev || t >= 1) return cur;
  const at = new Map();
  for (let k = 0; k < prev.length; k += 5) at.set(prev[k] * 2 + (prev[k + 4] >> 8), k);
  const out = new Float64Array(cur.length);
  for (let k = 0; k < cur.length; k += 5) {
    const j = at.get(cur[k] * 2 + (cur[k + 4] >> 8));
    out[k] = cur[k];
    out[k + 4] = cur[k + 4];
    if (j === undefined) {
      out[k + 1] = cur[k + 1];
      out[k + 2] = cur[k + 2];
      out[k + 3] = cur[k + 3];
    } else {
      out[k + 1] = lerp(prev[j + 1], cur[k + 1], t);
      out[k + 2] = lerp(prev[j + 2], cur[k + 2], t);
      out[k + 3] = lerp(prev[j + 3], cur[k + 3], t);
    }
  }
  return out;
}

const pair = (a, b, t) => [lerp(a[0], b[0], t), lerp(a[1], b[1], t)];

// The frame `t` of the way from `prev` to `cur`: the belt, the cups (matched
// by their order number), and the spouts and handles. Counts, scores and
// everything else are `cur`'s, as core sent them.
export function blendFrame(prev, cur, t) {
  if (!prev || t >= 1 || prev.tick !== cur.tick - 1) return cur;
  return {
    ...cur,
    tick: lerp(prev.tick, cur.tick, t),
    lines: cur.lines.map((L) => {
      const P = prev.lines.find((x) => x.seat === L.seat);
      if (!P) return L;
      const cups = new Map(P.cups.map((c) => [c.order, c]));
      return {
        ...L,
        travel: lerp(P.travel, L.travel, t),
        spouts: L.spouts.map((s, i) => {
          const p = P.spouts[i];
          if (!p) return s;
          return { ...s, x: lerp(p.x, s.x, t), y: lerp(p.y, s.y, t), pivot: pair(p.pivot, s.pivot, t), tip: pair(p.tip, s.tip, t) };
        }),
        cups: L.cups.map((c) => {
          const p = cups.get(c.order);
          if (!p) return c;
          return { ...c, x: lerp(p.x, c.x, t), base: lerp(p.base, c.base, t), floor: lerp(p.floor, c.floor, t), rim: lerp(p.rim, c.rim, t) };
        }),
      };
    }),
  };
}
