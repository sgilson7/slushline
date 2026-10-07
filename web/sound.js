// Sound, synthesized by the page from the events core reports (PLAN.md D18;
// Sam, 2026-10-05: "a satisfying noise ala kid pix or zoombinis" when a cup
// is judged). No audio file ships: every sound is built here from
// oscillators and a little noise, so LICENSES.md has nothing to list.
// Which phrase plays is the judgement core chose; the page only performs it.

let ctx = null;
let master = null;
let squeeze = null;
let voiceBus = null;
let volume = 0.7;

// Browsers start audio only after a person presses something, so the page
// calls this from a click.
export function wake() {
  if (!ctx) {
    const AC = window.AudioContext || window.webkitAudioContext;
    if (!AC) return;
    // The lowest latency the browser offers, so a lid is heard when it is
    // seen (Sam, 2026-10-05: "the sound of the cups finishing also desyncs
    // easily").
    ctx = new AC({ latencyHint: 'interactive' });
    // Everything passes a gentle low-pass and a soft compressor: no sound
    // reaches the ear above about 2 kHz at full strength, and a run of
    // lids never stacks into a loud one ("slightly too tinny all around, it
    // hurts the ears after a while").
    const soften = ctx.createBiquadFilter();
    soften.type = 'lowpass';
    soften.frequency.value = 2200;
    soften.Q.value = 0.5;
    squeeze = ctx.createDynamicsCompressor();
    squeeze.threshold.value = -20;
    squeeze.knee.value = 12;
    squeeze.ratio.value = 3;
    squeeze.attack.value = 0.01;
    squeeze.release.value = 0.2;
    master = ctx.createGain();
    master.gain.value = volume * 0.6;
    master.connect(soften).connect(squeeze).connect(ctx.destination);
    // Sam's recorded voice skips the low-pass, which would muffle a real
    // voice, and shares the compressor with everything else.
    voiceBus = ctx.createGain();
    voiceBus.gain.value = volume * 0.5;
    voiceBus.connect(squeeze);
    loadClips();
  }
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
    squeeze = null;
    voiceBus = null;
    voices.clear();
    c.close().catch(() => {});
  }
});

export function setVolume(v) {
  volume = Math.max(0, Math.min(1, v));
  if (master) master.gain.value = volume * 0.6;
  if (voiceBus) voiceBus.gain.value = volume * 0.5;
}

// One note: an oscillator with a quick attack and a decay, optionally
// sliding in pitch, which is most of what makes a toy sound like a toy.
function note(t, freq, len, { type = 'sine', gain = 0.18, slide = 1, vibrato = 0 } = {}) {
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

// A soft mallet: a sine with a quiet second partial, struck and let ring.
// Warmer than the square and triangle waves the first sounds used.
function mallet(t, freq, len, gain = 0.2) {
  note(t, freq, len, { type: 'sine', gain });
  note(t, freq * 2, len * 0.5, { type: 'sine', gain: gain * 0.18 });
}

// The lid coming down: a soft low thump of filtered noise and a falling tone.
function thunk(t) {
  const n = Math.floor(ctx.sampleRate * 0.12);
  const buf = ctx.createBuffer(1, n, ctx.sampleRate);
  const d = buf.getChannelData(0);
  for (let i = 0; i < n; i += 1) d[i] = (Math.random() * 2 - 1) * (1 - i / n) ** 3;
  const src = ctx.createBufferSource();
  src.buffer = buf;
  const f = ctx.createBiquadFilter();
  f.type = 'lowpass';
  f.frequency.value = 500;
  const g = ctx.createGain();
  g.gain.value = 0.35;
  src.connect(f).connect(g).connect(master);
  src.start(t);
  note(t, 140, 0.14, { type: 'sine', gain: 0.3, slide: 0.55 });
}

// A sparkle: a few soft pings, no higher than the C two octaves above
// middle C.
function sparkle(t, base) {
  [0, 0.06, 0.12].forEach((dt, i) => note(t + dt, base * [1, 1.25, 1.5][i], 0.14, { type: 'sine', gain: 0.05 }));
}

// Notes of C major, an octave lower than the first sounds, so every phrase
// sits in one warm range.
const C4 = 261.63, D4 = 293.66, E4 = 329.63, G4 = 392, A4 = 440, C5 = 523.25, E5 = 659.25, G3 = 196, E3 = 164.81;

const PHRASES = {
  excellent(t) {
    [C4, E4, G4, C5, E5].forEach((f, i) => mallet(t + i * 0.075, f, 0.32, 0.17));
    note(t + 0.4, C5, 0.6, { type: 'sine', gain: 0.12, vibrato: 5 });
    sparkle(t + 0.42, C5 * 2);
  },
  great(t) {
    [C4, E4, G4, C5].forEach((f, i) => mallet(t + i * 0.085, f, 0.3, 0.16));
    note(t + 0.36, G4, 0.45, { type: 'sine', gain: 0.11, vibrato: 4 });
  },
  nice(t) {
    mallet(t, E4, 0.25, 0.17);
    mallet(t + 0.11, A4, 0.4, 0.17);
  },
  ok(t) {
    mallet(t, D4, 0.22, 0.15);
    mallet(t + 0.12, G4, 0.3, 0.13);
  },
  // A soft slide down, wah-wah: the cup missed, and it is still a toy.
  miss(t) {
    note(t, G3, 0.3, { type: 'sine', gain: 0.14, slide: 0.94, vibrato: 4 });
    note(t, G3 * 2, 0.3, { type: 'sine', gain: 0.03, slide: 0.94 });
    note(t + 0.32, E3, 0.55, { type: 'sine', gain: 0.14, slide: 0.8, vibrato: 6 });
  },
};

// --- the flair: deep house, after the soundtrack (Sam, 2026-10-06) -------
//
// The flair words (GROOVY, FUNKY, SMOOTH, CHILL) get "a noise that is more
// inline with the sunnybeatz feel the heat song", which SoundCloud files as
// deep house: a round four-on-the-floor kick, an open hat on the off-beat,
// and a warm minor-ninth chord stab through a low-pass that opens and
// closes. At 122 beats a minute, in A minor.

const HOUSE_BEAT = 60 / 122;

function kick(t, gain = 0.4) {
  const o = ctx.createOscillator();
  const g = ctx.createGain();
  o.type = 'sine';
  o.frequency.setValueAtTime(120, t);
  o.frequency.exponentialRampToValueAtTime(45, t + 0.12);
  g.gain.setValueAtTime(0.0001, t);
  g.gain.exponentialRampToValueAtTime(gain, t + 0.005);
  g.gain.exponentialRampToValueAtTime(0.0001, t + 0.28);
  o.connect(g).connect(master);
  o.start(t);
  o.stop(t + 0.3);
}

// The open hat, soft: under the low-pass it is a breath on the off-beat.
function hat(t, gain = 0.07) {
  const n = Math.floor(ctx.sampleRate * 0.14);
  const buf = ctx.createBuffer(1, n, ctx.sampleRate);
  const d = buf.getChannelData(0);
  for (let i = 0; i < n; i += 1) d[i] = (Math.random() * 2 - 1) * Math.exp(-i / (n / 4));
  const src = ctx.createBufferSource();
  src.buffer = buf;
  const f = ctx.createBiquadFilter();
  f.type = 'highpass';
  f.frequency.value = 1400;
  const g = ctx.createGain();
  g.gain.value = gain;
  src.connect(f).connect(g).connect(master);
  src.start(t);
}

// A chord stab: detuned pairs through a resonant low-pass that sweeps open
// and shut, which is most of the deep house sound. Triangles, not the saws a
// house record would use: no square or sawtooth wave plays here, since Sam
// found the first sounds too tinny (DECISIONS.md, 2026-10-05).
function stab(t, freqs, len, { gain = 0.05, open = 1600 } = {}) {
  const f = ctx.createBiquadFilter();
  f.type = 'lowpass';
  f.Q.value = 4;
  f.frequency.setValueAtTime(300, t);
  f.frequency.exponentialRampToValueAtTime(open, t + len * 0.25);
  f.frequency.exponentialRampToValueAtTime(350, t + len);
  const g = ctx.createGain();
  g.gain.setValueAtTime(0.0001, t);
  g.gain.exponentialRampToValueAtTime(gain, t + 0.01);
  g.gain.exponentialRampToValueAtTime(0.0001, t + len);
  f.connect(g).connect(master);
  for (const fr of freqs) {
    for (const detune of [-7, 7]) {
      const o = ctx.createOscillator();
      o.type = 'triangle';
      o.frequency.value = fr;
      o.detune.value = detune;
      o.connect(f);
      o.start(t);
      o.stop(t + len + 0.05);
    }
  }
}

// Am9, Dm9 and Fmaj7, voiced low and close.
const AM9 = [220, 261.63, 329.63, 392, 493.88];
const DM9 = [146.83, 174.61, 220, 261.63, 329.63];
const FMAJ7 = [174.61, 220, 261.63, 329.63];

Object.assign(PHRASES, {
  // A bar of the groove: kick on each beat, hats between, two stabs, and a
  // last chord that rings with the filter wide.
  groovy(t) {
    const b = HOUSE_BEAT;
    for (let i = 0; i < 4; i += 1) {
      kick(t + i * b, 0.32);
      hat(t + i * b + b / 2);
    }
    stab(t + b * 0.5, AM9, b * 0.9, { gain: 0.045 });
    stab(t + b * 1.5, AM9, b * 0.9, { gain: 0.045 });
    stab(t + b * 2.5, DM9, b * 0.9, { gain: 0.045 });
    stab(t + b * 3, AM9, b * 2.2, { gain: 0.05, open: 2000 });
    bass(t, 110, b * 0.8);
    bass(t + b * 2, 73.42, b * 0.8);
  },
  funky(t) {
    const b = HOUSE_BEAT;
    for (let i = 0; i < 2; i += 1) {
      kick(t + i * b, 0.3);
      hat(t + i * b + b / 2);
    }
    stab(t + b * 0.5, DM9, b * 0.8, { gain: 0.045 });
    stab(t + b * 1.5, AM9, b * 1.4, { gain: 0.045 });
    bass(t, 110, b * 0.7);
  },
  smooth(t) {
    const b = HOUSE_BEAT;
    kick(t, 0.28);
    hat(t + b / 2);
    stab(t + b / 2, FMAJ7, b * 1.4, { gain: 0.045, open: 1300 });
  },
  chill(t) {
    const b = HOUSE_BEAT;
    hat(t + b / 4, 0.05);
    stab(t, DM9, b * 1.6, { gain: 0.04, open: 900 });
  },
});

// --- a voice for a missed cup (Sam, 2026-10-06 and 2026-10-07) -----------
//
// "a man sadly saying OH NO, and the variation is he says DARN like a
// cowboy", in Sam's own voice: two clips he recorded for the game, trimmed
// to the word (web/voice, LICENSES.md). They are fetched from this site when
// sound wakes; until they arrive, the old wah-wah plays instead.

const CLIPS = { miss: 'voice/oh-no.wav', darn: 'voice/darn.wav' };
const clips = {};       // word -> decoded clip
let lastClip = null;    // the file that played last, for the gate

function loadClips() {
  const c = ctx;
  for (const [word, url] of Object.entries(CLIPS)) {
    if (clips[word]) continue;
    fetch(url)
      .then((r) => (r.ok ? r.arrayBuffer() : Promise.reject(new Error(url))))
      .then((bytes) => c.decodeAudioData(bytes))
      .then((buf) => { clips[word] = buf; })
      .catch(() => { /* the wah-wah plays instead */ });
  }
}

function playClip(word, t) {
  const buf = clips[word];
  if (!buf || !voiceBus) return false;
  const src = ctx.createBufferSource();
  src.buffer = buf;
  src.connect(voiceBus);
  src.start(t);
  lastClip = CLIPS[word];
  return true;
}

// For the gate, which cannot hear: which clips have loaded, and which
// played last.
export function clipState() {
  return { loaded: Object.keys(clips).sort(), last: lastClip };
}

// A twang under the cowboy: a plucked low string that bends down.
function twang(t) {
  note(t, 98, 0.5, { type: 'triangle', gain: 0.12, slide: 0.85 });
  note(t, 196, 0.35, { type: 'sine', gain: 0.04, slide: 0.85 });
}

// A cup judged: the lid, then the phrase its judgement earned, which core
// chose (the word, or its flair). A missed cup is Sam saying so.
export function judged(word) {
  if (!ctx || ctx.state === 'closed' || volume <= 0) return;
  // Now, not a moment ahead: the lid is closing on screen this frame.
  const t = ctx.currentTime;
  thunk(t);
  if (CLIPS[word]) {
    if (word === 'darn') twang(t + 0.08);
    if (!playClip(word, t + 0.06)) PHRASES.miss(t + 0.08);
    return;
  }
  (PHRASES[word] ?? PHRASES.ok)(t + 0.08);
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
  f.frequency.value = 1500;
  f.Q.value = 1;
  const g = ctx.createGain();
  g.gain.value = 0.14;
  src.connect(f).connect(g).connect(master);
  src.start(t);
}

function keys(t, freqs, len) {
  for (const fr of freqs) {
    for (const [mult, gain] of [[1, 0.05], [2.001, 0.008]]) {
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
  f.frequency.exponentialRampToValueAtTime(1100, t + len * 0.5);
  f.frequency.exponentialRampToValueAtTime(700, t + len);
  const g = ctx.createGain();
  g.gain.setValueAtTime(0.0001, t);
  g.gain.exponentialRampToValueAtTime(0.11, t + len * 0.35);
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
  note(t + 8 * beat, 523.25, 1.4, { type: 'sine', gain: 0.12, vibrato: 4 });
  note(t + 8 * beat, 783.99, 1.2, { type: 'sine', gain: 0.05 });
  return 8 * beat + 1.2;
}

// --- the pour: a spout's valve open (Sam, 2026-10-06) ----------------------
//
// "a gentle pshht sound" while a nozzle pours. A frozen carbonated drink is
// held under pressure and foams as it reaches the air (SECOND-ORDER-M7 row
// 1), so the sound is a soft airy psst as the valve opens, then a breathy
// hiss with a low, slowly wobbling body under it (the thick pour) and now and
// then a fine crackle of bubbles, fading as the valve closes. All of it is
// noise through filters, under the same low-pass as everything else, and
// quieter than a lid.
//
// When a valve is open is core's: each frame carries every spout's opening
// (4096 is fully open), and pourFrame follows it. A replay carries the same
// openings, so it sounds the same.

const FULL_OPEN = 4096;
const POUR_GAIN = 0.07;
const voices = new Map();   // "seat:spout" -> a voice
let foam = null;            // two seconds of hiss with crackle, shared, looped

function foamBuffer() {
  if (foam && foam.sampleRate === ctx.sampleRate) return foam;
  const n = Math.floor(ctx.sampleRate * 2);
  foam = ctx.createBuffer(1, n, ctx.sampleRate);
  const d = foam.getChannelData(0);
  let crackle = 0, brown = 0;
  for (let i = 0; i < n; i += 1) {
    // Sparse, quick ticks: bubbles breaking at the cup.
    if (Math.random() < 0.0009) crackle = 0.7 + Math.random() * 0.5;
    crackle *= 0.992;
    const white = Math.random() * 2 - 1;
    // A little brown noise under the white, so the hiss is breathy rather
    // than a bright shh.
    brown = (brown + white * 0.02) * 0.995;
    d[i] = white * (0.45 + crackle) + brown * 3;
  }
  return foam;
}

// One voice per spout: the hiss (a broad band near 1 kHz) and the body (a
// low band, its level wobbling a few times a second like a thick pour).
function voice() {
  const src = ctx.createBufferSource();
  src.buffer = foamBuffer();
  src.loop = true;
  const hiss = ctx.createBiquadFilter();
  hiss.type = 'bandpass';
  hiss.frequency.value = 1100;
  hiss.Q.value = 0.6;
  const body = ctx.createBiquadFilter();
  body.type = 'lowpass';
  body.frequency.value = 260;
  const bodyGain = ctx.createGain();
  bodyGain.gain.value = 0.55;
  const wobble = ctx.createOscillator();
  const wobbleDepth = ctx.createGain();
  wobble.frequency.value = 4 + Math.random() * 2.5;
  wobbleDepth.gain.value = 0.3;
  wobble.connect(wobbleDepth).connect(bodyGain.gain);
  const out = ctx.createGain();
  out.gain.value = 0;
  src.connect(hiss).connect(out);
  src.connect(body).connect(bodyGain).connect(out);
  out.connect(master);
  const t = ctx.currentTime;
  src.start(t, Math.random() * 1.9);
  wobble.start(t);
  return { src, wobble, out, open: false, stopAt: 0, level: 0 };
}

// The psst as a valve opens: a short burst of noise that falls from a
// brighter band into the hiss.
function psst(t) {
  const len = 0.28;
  const n = Math.floor(ctx.sampleRate * len);
  const buf = ctx.createBuffer(1, n, ctx.sampleRate);
  const d = buf.getChannelData(0);
  for (let i = 0; i < n; i += 1) d[i] = Math.random() * 2 - 1;
  const src = ctx.createBufferSource();
  src.buffer = buf;
  const f = ctx.createBiquadFilter();
  f.type = 'bandpass';
  f.Q.value = 0.8;
  f.frequency.setValueAtTime(1900, t);
  f.frequency.exponentialRampToValueAtTime(900, t + len);
  const g = ctx.createGain();
  g.gain.setValueAtTime(0.0001, t);
  g.gain.exponentialRampToValueAtTime(POUR_GAIN * 1.4, t + 0.012);
  g.gain.exponentialRampToValueAtTime(0.0001, t + len);
  src.connect(f).connect(g).connect(master);
  src.start(t);
  src.stop(t + len + 0.02);
}

function retire(key, v, t) {
  try { v.src.stop(t); v.wobble.stop(t); } catch { /* already stopped */ }
  voices.delete(key);
}

// Follow the openings in a frame core sent.
export function pourFrame(frame) {
  if (!ctx || ctx.state === 'closed') return;
  const t = ctx.currentTime;
  const seen = new Set();
  for (const L of frame.lines) {
    L.spouts.forEach((s, j) => {
      const key = `${L.seat}:${j}`;
      const open = Math.max(0, Math.min(1, s.opening / FULL_OPEN));
      let v = voices.get(key);
      if (open > 0) {
        seen.add(key);
        if (!v) { v = voice(); voices.set(key, v); }
        if (!v.open && volume > 0) psst(t);
        const level = POUR_GAIN * Math.sqrt(open);
        // About 30 ms to follow the handle, so a change never clicks. Only
        // when the level moves: a held handle would otherwise schedule an
        // event every frame, which an engine with no sound device, whose
        // clock stands still, never clears.
        if (!v.open || Math.abs(level - v.level) > POUR_GAIN * 0.02) {
          v.out.gain.setTargetAtTime(level, t, 0.03);
          v.level = level;
        }
        v.open = true;
        v.stopAt = 0;
      }
    });
  }
  // A valve that has closed: a soft tail, then the voice goes.
  for (const [key, v] of voices) {
    if (seen.has(key)) continue;
    if (v.open) {
      v.open = false;
      v.out.gain.setTargetAtTime(0, t, 0.12);
      v.stopAt = t + 0.8;
    } else if (v.stopAt && t >= v.stopAt) {
      retire(key, v, t);
    }
  }
}

// Every pour voice off at once: the run ended, the page was left or hidden.
export function pourStop() {
  if (!ctx || ctx.state === 'closed') { voices.clear(); return; }
  const t = ctx.currentTime;
  for (const [key, v] of voices) {
    v.out.gain.cancelScheduledValues(t);
    v.out.gain.setTargetAtTime(0, t, 0.04);
    retire(key, v, t + 0.25);
  }
}

// How many pour voices are sounding: for the gate, which cannot hear.
export function pourVoices() {
  let n = 0;
  for (const v of voices.values()) if (v.open) n += 1;
  return n;
}
