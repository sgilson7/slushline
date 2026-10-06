// Sound, synthesized by the page from the events core reports (PLAN.md D18;
// Sam, 2026-10-05: "a satisfying noise ala kid pix or zoombinis" when a cup
// is judged). No audio file ships: every sound is built here from
// oscillators and a little noise, so LICENSES.md has nothing to list.
// Which phrase plays is the judgement core chose; the page only performs it.

let ctx = null;
let master = null;
let volume = 0.7;

// Browsers start audio only after a person presses something, so the page
// calls this from a click.
export function wake() {
  if (!ctx) {
    const AC = window.AudioContext || window.webkitAudioContext;
    if (!AC) return;
    ctx = new AC();
    master = ctx.createGain();
    master.gain.value = volume;
    master.connect(ctx.destination);
  }
  if (ctx.state === 'suspended') ctx.resume();
}

// Leaving the page with sound still scheduled made Firefox report
// "InvalidStateError: Navigated away from page" (CI, run 37398597927), so
// the context is closed before the page goes.
window.addEventListener('pagehide', () => {
  if (ctx) {
    const c = ctx;
    ctx = null;
    master = null;
    c.close().catch(() => {});
  }
});

export function setVolume(v) {
  volume = Math.max(0, Math.min(1, v));
  if (master) master.gain.value = volume;
}

// One note: an oscillator with a quick attack and a decay, optionally
// sliding in pitch, which is most of what makes a toy sound like a toy.
function note(t, freq, len, { type = 'square', gain = 0.18, slide = 1, vibrato = 0 } = {}) {
  const o = ctx.createOscillator();
  const g = ctx.createGain();
  o.type = type;
  o.frequency.setValueAtTime(freq, t);
  if (slide !== 1) o.frequency.exponentialRampToValueAtTime(freq * slide, t + len);
  if (vibrato) {
    const lfo = ctx.createOscillator();
    const depth = ctx.createGain();
    lfo.frequency.value = 7;
    depth.gain.value = vibrato;
    lfo.connect(depth).connect(o.frequency);
    lfo.start(t);
    lfo.stop(t + len + 0.05);
  }
  g.gain.setValueAtTime(0.0001, t);
  g.gain.exponentialRampToValueAtTime(gain, t + 0.012);
  g.gain.exponentialRampToValueAtTime(0.0001, t + len);
  o.connect(g).connect(master);
  o.start(t);
  o.stop(t + len + 0.02);
}

// The lid coming down: a short low thump of filtered noise and a falling tone.
function thunk(t) {
  const n = Math.floor(ctx.sampleRate * 0.12);
  const buf = ctx.createBuffer(1, n, ctx.sampleRate);
  const d = buf.getChannelData(0);
  for (let i = 0; i < n; i += 1) d[i] = (Math.random() * 2 - 1) * (1 - i / n) ** 3;
  const src = ctx.createBufferSource();
  src.buffer = buf;
  const f = ctx.createBiquadFilter();
  f.type = 'lowpass';
  f.frequency.value = 900;
  const g = ctx.createGain();
  g.gain.value = 0.5;
  src.connect(f).connect(g).connect(master);
  src.start(t);
  note(t, 180, 0.12, { type: 'triangle', gain: 0.3, slide: 0.5 });
}

// A sparkle: a few quick high pings, like the twinkle a stamp makes.
function sparkle(t, base) {
  [0, 0.05, 0.1, 0.16].forEach((dt, i) => note(t + dt, base * [2, 2.5, 3, 4][i], 0.09, { type: 'sine', gain: 0.08 }));
}

// Notes of C major, so every phrase is in one key.
const C5 = 523.25, D5 = 587.33, E5 = 659.25, G5 = 783.99, A5 = 880, C6 = 1046.5, E6 = 1318.5, G4 = 392, E4 = 329.63;

const PHRASES = {
  excellent(t) {
    [C5, E5, G5, C6, E6].forEach((f, i) => note(t + i * 0.07, f, 0.16, { type: i % 2 ? 'triangle' : 'square', gain: 0.14 }));
    note(t + 0.38, C6, 0.45, { type: 'triangle', gain: 0.16, vibrato: 12 });
    sparkle(t + 0.4, C6);
  },
  great(t) {
    [C5, E5, G5, C6].forEach((f, i) => note(t + i * 0.08, f, 0.16, { type: 'square', gain: 0.13 }));
    note(t + 0.34, G5, 0.3, { type: 'triangle', gain: 0.14, vibrato: 8 });
    sparkle(t + 0.34, G5);
  },
  nice(t) {
    note(t, E5, 0.12, { type: 'square', gain: 0.13 });
    note(t + 0.1, A5, 0.25, { type: 'triangle', gain: 0.15, slide: 1.06 });
  },
  ok(t) {
    note(t, D5, 0.1, { type: 'triangle', gain: 0.14 });
    note(t + 0.11, G5, 0.18, { type: 'triangle', gain: 0.12 });
  },
  // A slide down, wah-wah: the cup missed, and it is still a toy.
  miss(t) {
    note(t, G4, 0.28, { type: 'sawtooth', gain: 0.07, slide: 0.94, vibrato: 6 });
    note(t + 0.3, E4, 0.5, { type: 'sawtooth', gain: 0.07, slide: 0.8, vibrato: 9 });
  },
};

// A cup judged: the lid, then the phrase its judgement earned.
export function judged(word) {
  if (!ctx || ctx.state === 'closed' || volume <= 0) return;
  const t = ctx.currentTime + 0.01;
  thunk(t);
  (PHRASES[word] ?? PHRASES.ok)(t + 0.12);
}
