// The soundtrack (Sam, 2026-10-06: "look at gear master 1s soundcloud
// integration, and have it play the following song from soundcloud as you
// play"). SoundCloud's own widget streams "Feel the Heat" by
// sunnybeatzproduction; nothing of it is copied here. After Gear Master's
// player: it starts when a run starts (a click, which browsers require
// before sound), repeats at its end, and keeps its level and whether it is
// folded away in this browser. If SoundCloud cannot be reached, the panel
// stays and the game plays on without it.

const LEVEL = 'slushline.music.level';
const FOLDED = 'slushline.music.folded';

let widget = null;
let ready = false;
let wanted = false;   // a run has asked for the music
let level = 25;
let muted = false;
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

function apply() {
  const $ = (id) => document.getElementById(id);
  $('music-volume').value = String(level);
  $('music-level').textContent = String(level);
  $('music-mute').textContent = muted ? words.unmute : words.mute;
  if (ready) widget.setVolume(muted ? 0 : level);
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
    muted = false;
    remember(LEVEL, level);
    apply();
  });
  $('music-mute').addEventListener('click', () => { muted = !muted; userPaused = false; apply(); if (!muted) play(); });
  apply();
  if (typeof window.SC === 'undefined') return;
  try {
    widget = window.SC.Widget($('music-player'));
  } catch {
    return;
  }
  const E = window.SC.Widget.Events;
  widget.bind(E.READY, () => {
    ready = true;
    apply();
    if (wanted && !muted) { askedAt = performance.now(); widget.play(); }
  });
  // A level set on READY is dropped before the sound loads (Gear Master's
  // finding), so it is set again each time the track plays.
  widget.bind(E.PLAY, () => { userPaused = false; apply(); });
  widget.bind(E.PLAY_PROGRESS, () => { $('music-blocked').hidden = true; });
  // A pause straight after the game asked to play is the browser's: Safari
  // starts a player in another site's frame only from a press on that
  // player (SECOND-ORDER-M9 row 3). The panel opens and says so. A pause
  // later is the player's own choice, and a new run does not undo it.
  widget.bind(E.PAUSE, () => {
    if (performance.now() - askedAt < 2000) {
      $('music-blocked').hidden = false;
      unfold();
      panel.scrollIntoView({ block: 'nearest', behavior: 'smooth' });
    } else {
      userPaused = true;
    }
  });
  // One track: at its end, back to the top.
  widget.bind(E.FINISH, () => { widget.seekTo(0); widget.play(); });
}

// A run has started: the music plays from where it is, and goes on between
// runs once begun.
export function play() {
  wanted = true;
  if (ready && !muted && !userPaused) {
    askedAt = performance.now();
    widget.play();
  }
}

// For the gate: whether the widget loaded and a run asked for the music.
export function state() {
  return { loaded: ready, wanted, level, muted, blocked: !document.getElementById('music-blocked').hidden };
}
