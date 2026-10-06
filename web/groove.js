// The groove's animation (Sam, 2026-10-05: three of the highest tier in a
// row gets a sound "and it has an animation"). Slush in every flavor's fill
// and pattern spirals over the middle of the line in time with the groove, rings swell
// from the middle on each beat, and on the last chime it all bursts outward. Drawn on
// its own canvas over the line with the line's own pattern helpers, so a
// flavor still reads without color. Decoration only: it reads nothing from
// the world but where the lid is.
import { Stage } from './draw.js';

let raf = null;

export function play(overlay, numbers, palette, flavors, lidX, rimY, seconds, bpm) {
  stop(overlay);
  const reduce = window.matchMedia?.('(prefers-reduced-motion: reduce)').matches;
  const s = new Stage(overlay, numbers, palette, flavors);
  const g = overlay.getContext('2d');
  overlay.hidden = false;
  const beat = 60 / bpm;
  const n = reduce ? 0 : 96;
  const bits = Array.from({ length: n }, (_, i) => ({
    f: i % flavors.length,
    a: (i / n) * Math.PI * 2 * 3,
    r: 40 + (i * 37) % 260,
    size: 6 + (i * 13) % 8,
    spin: 0.6 + ((i * 7) % 10) / 10,
  }));
  const start = performance.now();
  const frame = (now) => {
    const t = (now - start) / 1000;
    if (t > seconds) { stop(overlay); return; }
    g.clearRect(0, 0, overlay.width, overlay.height);
    const pulse = 1 + 0.12 * Math.max(0, Math.cos(((t % beat) / beat) * Math.PI));
    const end = Math.max(0, (t - (seconds - 1.3)) / 1.3);
    const fade = Math.min(1, t / 0.3) * (1 - end * end);
    // Rings from the lid, one a beat.
    for (let k = 0; k < 3; k += 1) {
      const age = ((t + k * beat / 3) % beat) / beat;
      g.globalAlpha = (1 - age) * 0.5 * fade;
      g.strokeStyle = palette.judge_glow;
      g.lineWidth = 6 * (1 - age) + 1;
      g.beginPath();
      g.arc(lidX, rimY - 40, 20 + age * 420, 0, Math.PI * 2);
      g.stroke();
    }
    if (reduce) {
      g.globalAlpha = 0.35 * fade;
      g.fillStyle = palette.judge_top;
      g.beginPath();
      g.arc(lidX, rimY, 160, 0, Math.PI * 2);
      g.fill();
    }
    // The slush, spiralling in time, then bursting.
    for (const b of bits) {
      const ang = b.a + t * b.spin * Math.PI * (bpm / 60);
      const rad = (b.r * pulse) * (1 + end * 3.5);
      const x = lidX + Math.cos(ang) * rad;
      const y = rimY - 40 + Math.sin(ang) * rad * 0.55;
      g.globalAlpha = fade;
      g.fillStyle = palette.line;
      g.beginPath();
      g.arc(x, y, b.size + 1, 0, Math.PI * 2);
      g.fill();
      g.fillStyle = s.fill(b.f);
      g.beginPath();
      g.arc(x, y, b.size, 0, Math.PI * 2);
      g.fill();
      s.patternInCircle(b.f, x, y, b.size);
    }
    g.globalAlpha = 1;
    raf = requestAnimationFrame(frame);
  };
  raf = requestAnimationFrame(frame);
}

export function stop(overlay) {
  if (raf) cancelAnimationFrame(raf);
  raf = null;
  if (overlay) {
    overlay.getContext('2d').clearRect(0, 0, overlay.width, overlay.height);
    overlay.hidden = true;
  }
}
