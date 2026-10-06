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
  // A browser with no sound device (CI's headless Firefox) leaves the
  // context suspended, and the resume is refused when the page goes; that
  // refusal is expected and is not an error of the game's.
  if (ctx.state === 'suspended') ctx.resume().catch(() => {});
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

// --- the groove: three of the top word in a row (Sam, 2026-10-05) --------
//
// "the most asmr and satisfying and is groovy": two bars of a soft lo-fi
// groove at 96 beats a minute, in C like everything else. A round sine bass
// walks under electric-piano chords (a sine with a gentle bell partial and a
// slow tremolo), finger snaps on two and four, and over it the sound of slush
// itself: filtered noise swelling and crunching like a cup being poured.
// It ends on a bubble pop and a high chime that rings out.

function snap(t) {
  const n = Math.floor(ctx.sampleRate * 0.05);
  const buf = ctx.createBuffer(1, n, ctx.sampleRate);
  const d = buf.getChannelData(0);
  for (let i = 0; i < n; i += 1) d[i] = (Math.random() * 2 - 1) * Math.exp(-i / (n / 8));
  const src = ctx.createBufferSource();
  src.buffer = buf;
  const f = ctx.createBiquadFilter();
  f.type = 'bandpass';
  f.frequency.value = 2400;
  f.Q.value = 1.2;
  const g = ctx.createGain();
  g.gain.value = 0.22;
  src.connect(f).connect(g).connect(master);
  src.start(t);
}

function keys(t, freqs, len) {
  for (const fr of freqs) {
    for (const [mult, gain] of [[1, 0.05], [2.001, 0.012], [3.998, 0.004]]) {
      const o = ctx.createOscillator();
      const g = ctx.createGain();
      const trem = ctx.createOscillator();
      const tremDepth = ctx.createGain();
      o.type = 'sine';
      o.frequency.value = fr * mult;
      trem.frequency.value = 4.5;
      tremDepth.gain.value = gain * 0.35;
      trem.connect(tremDepth).connect(g.gain);
      g.gain.setValueAtTime(0.0001, t);
      g.gain.exponentialRampToValueAtTime(gain, t + 0.02);
      g.gain.exponentialRampToValueAtTime(0.0001, t + len);
      o.connect(g).connect(master);
      o.start(t);
      trem.start(t);
      o.stop(t + len + 0.05);
      trem.stop(t + len + 0.05);
    }
  }
}

function bass(t, freq, len) {
  note(t, freq, len, { type: 'sine', gain: 0.26 });
  note(t, freq * 2, len * 0.6, { type: 'triangle', gain: 0.03 });
}

// Slush: noise through a resonant band that sweeps, with a little grain,
// swelling in and out. The satisfying part.
function pour(t, len) {
  const n = Math.floor(ctx.sampleRate * len);
  const buf = ctx.createBuffer(1, n, ctx.sampleRate);
  const d = buf.getChannelData(0);
  let crunch = 0;
  for (let i = 0; i < n; i += 1) {
    if (Math.random() < 0.004) crunch = 1;
    crunch *= 0.996;
    d[i] = (Math.random() * 2 - 1) * (0.35 + crunch * 0.9);
  }
  const src = ctx.createBufferSource();
  src.buffer = buf;
  const f = ctx.createBiquadFilter();
  f.type = 'bandpass';
  f.Q.value = 3;
  f.frequency.setValueAtTime(500, t);
  f.frequency.exponentialRampToValueAtTime(1800, t + len * 0.5);
  f.frequency.exponentialRampToValueAtTime(700, t + len);
  const g = ctx.createGain();
  g.gain.setValueAtTime(0.0001, t);
  g.gain.exponentialRampToValueAtTime(0.16, t + len * 0.35);
  g.gain.exponentialRampToValueAtTime(0.0001, t + len);
  src.connect(f).connect(g).connect(master);
  src.start(t);
}

function bubble(t) {
  note(t, 420, 0.12, { type: 'sine', gain: 0.22, slide: 2.6 });
}

// Two bars at 96 a minute. Returns the length in seconds, so the animation
// can keep time with it.
export const GROOVE_BPM = 96;
export function groove() {
  const beat = 60 / GROOVE_BPM;
  if (!ctx || ctx.state === 'closed' || volume <= 0) return 8 * beat;
  const t = ctx.currentTime + 0.05;
  const C3 = 130.81, A2 = 110, F2 = 87.31, G2 = 98;
  // Cmaj9, Am7, Fmaj7, G6/9: a turn round that resolves home.
  const chords = [
    [C3 * 2, 329.63, 392, 493.88, 587.33],
    [220, 261.63, 329.63, 392],
    [174.61, 220, 261.63, 329.63],
    [196, 246.94, 293.66, 329.63, 440],
  ];
  const roots = [C3, A2, F2, G2];
  for (let bar = 0; bar < 2; bar += 1) {
    for (let k = 0; k < 2; k += 1) {
      const i = bar * 2 + k;
      const at = t + i * 2 * beat;
      keys(at, chords[i], 2 * beat * 1.1);
      bass(at, roots[i], beat * 0.9);
      bass(at + beat * 1.5, roots[i] * 1.5, beat * 0.4);
      snap(at + beat);
    }
  }
  pour(t + beat * 0.5, beat * 6);
  bubble(t + 8 * beat - 0.12);
  note(t + 8 * beat, 1046.5, 1.4, { type: 'sine', gain: 0.12, vibrato: 4 });
  note(t + 8 * beat, 1567.98, 1.2, { type: 'sine', gain: 0.05 });
  return 8 * beat + 1.2;
}
