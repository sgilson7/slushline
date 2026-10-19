// The soundtrack (Sam, 2026-10-06: "look at gear master 1s soundcloud
// integration, and have it play the following song from soundcloud as you
// play"). SoundCloud's own widget streams "Feel the Heat" by
// sunnybeatzproduction; nothing of it is copied here. After Gear Master's
// player: it repeats at its end, goes on between runs, and keeps its level
// and whether it is folded away in this browser. If SoundCloud cannot be
// reached, the panel stays and the game plays on without it.
//
// 2026-10-19, for schools: the widget's frame is made only when the player
// presses "Play the soundtrack", so a page that is loaded and left asks
// SoundCloud for nothing. SoundCloud's api.js is not loaded: it would run
// SoundCloud's code on this site's origin, beside every other tool's saved
// data there. The page talks to the frame by postMessage instead, the same
// messages api.js sends (play, setVolume, seekTo, addEventListener), and
// reads only messages from that frame and SoundCloud's origin.

const SC_ORIGIN = 'https://w.soundcloud.com';
const SRC = `${SC_ORIGIN}/player/?url=https%3A%2F%2Fapi.soundcloud.com%2Ftracks%2F78084887&auto_play=false&hide_related=true&show_comments=false&show_user=true&show_reposts=false&show_teaser=false&visual=false`;
const EVENTS = ['play', 'playProgress', 'pause', 'finish'];

const LEVEL = 'slushline.music.level';
const FOLDED = 'slushline.music.folded';

let frame = null;     // the widget's frame, once the player asks for music
let ready = false;
let wanted = false;   // the player has asked for the music
let level = 25;
let muted = true;     // until the player presses "Play the soundtrack"
let words = null;
let askedAt = -1e9;   // when the game last asked the widget to play
let userPaused = false;
let unfold = null;

function remembered(key, fallback) {
  try { const v = localStorage.getItem(key); return v === null ? fallback : v; } catch { return fallback; }
}
function remember(key, value) {
  try { localStorage.setItem(key, String(value)); } catch { /* storage off */ }
}

// One command to the widget, as api.js words it.
function send(method, value) {
  if (!frame || !frame.contentWindow) return;
  frame.contentWindow.postMessage(JSON.stringify(value === undefined ? { method } : { method, value }), SC_ORIGIN);
}

function apply() {
  const $ = (id) => document.getElementById(id);
  $('music-volume').value = String(level);
  $('music-level').textContent = String(level);
  $('music-mute').textContent = muted ? words.unmute : words.mute;
  if (ready) send('setVolume', muted ? 0 : level);
}

// `t` gives copy strings by key: the panel's words change as it is used.
export function setup(t) {
  words = { mute: t('music.mute'), unmute: t('music.unmute'), fold: t('music.fold'), unfold: t('music.unfold') };
  const $ = (id) => document.getElementById(id);
  const panel = $('music');
  const fold = $('music-fold');
  const n = Number.parseInt(remembered(LEVEL, '25'), 10);
  // A quiet default, as Gear Master's: nobody is shouted at.
  level = Number.isFinite(n) ? Math.max(0, Math.min(100, n)) : 25;
  const setFolded = (f) => {
    panel.classList.toggle('folded', f);
    fold.textContent = f ? words.unfold : words.fold;
    fold.setAttribute('aria-expanded', String(!f));
    remember(FOLDED, f ? '1' : '0');
  };
  setFolded(remembered(FOLDED, '0') === '1');
  unfold = () => setFolded(false);
  fold.addEventListener('click', () => setFolded(!panel.classList.contains('folded')));
  $('music-volume').addEventListener('input', (e) => {
    level = Number.parseInt(e.target.value, 10) || 0;
    // Turning the level is not asking for the music to begin.
    if (frame) muted = false;
    remember(LEVEL, level);
    apply();
  });
  $('music-mute').addEventListener('click', () => {
    if (!frame) { choose(); return; }
    muted = !muted; userPaused = false; apply(); if (!muted) play();
  });
  apply();
}

// The player has pressed "Play the soundtrack": only now is SoundCloud asked
// for anything.
function choose() {
  const $ = (id) => document.getElementById(id);
  const panel = $('music');
  frame = document.createElement('iframe');
  frame.id = 'music-player';
  frame.title = $('music').getAttribute('aria-label');
  frame.allow = 'autoplay; encrypted-media';
  frame.setAttribute('scrolling', 'no');
  frame.setAttribute('frameborder', 'no');
  frame.src = SRC;
  $('music-blocked').before(frame);
  muted = false;
  wanted = true;
  apply();
  window.addEventListener('message', (e) => {
    if (e.source !== frame.contentWindow || e.origin !== SC_ORIGIN) return;
    let m;
    try { m = typeof e.data === 'string' ? JSON.parse(e.data) : e.data; } catch { return; }
    if (!m || typeof m.method !== 'string') return;
    heard(m.method, panel);
  });
}

function heard(method, panel) {
  const $ = (id) => document.getElementById(id);
  if (method === 'ready') {
    ready = true;
    for (const ev of EVENTS) send('addEventListener', ev);
    apply();
    if (wanted && !muted) { askedAt = performance.now(); send('play'); }
  } else if (method === 'play') {
    // A level set on READY is dropped before the sound loads (Gear Master's
    // finding), so it is set again each time the track plays.
    userPaused = false; apply();
  } else if (method === 'playProgress') {
    $('music-blocked').hidden = true;
  } else if (method === 'pause') {
    // A pause straight after the page asked to play is the browser's: Safari
    // starts a player in another site's frame only from a press on that
    // player (SECOND-ORDER-M9 row 3). The panel opens and says so. A pause
    // later is the player's own choice, and a new run does not undo it.
    if (performance.now() - askedAt < 2000) {
      $('music-blocked').hidden = false;
      unfold();
      panel.scrollIntoView({ block: 'nearest', behavior: 'smooth' });
    } else {
      userPaused = true;
    }
  } else if (method === 'finish') {
    // One track: at its end, back to the top.
    send('seekTo', 0); send('play');
  }
}

// A run has started: once the player has asked for it, the music plays from
// where it is, and goes on between runs.
export function play() {
  if (!frame) return;
  wanted = true;
  if (ready && !muted && !userPaused) {
    askedAt = performance.now();
    send('play');
  }
}

// For the gate: whether the player asked for the music, and whether the
// widget loaded.
export function state() {
  return { chosen: frame !== null, loaded: ready, wanted, level, muted, blocked: !document.getElementById('music-blocked').hidden };
}
