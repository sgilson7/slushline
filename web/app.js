// The page: screens and the clock. It draws numbers core sent and decides
// nothing (CLAUDE.md). Every word it shows comes through t().
import init, * as core from './pkg/slushline_wasm.js';
import { Stage } from './draw.js';
import * as keys from './keys.js';
import { download, pick } from './files.js';

const BUILD = '__BUILD__';
let COPY, NUM, PAL, FLAVORS, CONTROLS, ACTION_BITS;
let stage = null;
let run = null;          // { game, kind, ... } while a run is on screen
let bindings = null;     // action -> KeyboardEvent.code

const $ = (id) => document.getElementById(id);

// A copy string by key, with {placeholders} filled from `values`.
export function t(key, values = {}) {
  let v = COPY;
  for (const k of key.split('.')) v = v?.[k];
  if (typeof v !== 'string') throw new Error(`no copy string ${key}`);
  return v.replace(/\{([a-z0-9_.]+)\}/g, (m, name) => {
    if (name === 'game') return COPY.game.name;
    if (name in values) return String(values[name]);
    throw new Error(`no value for {${name}} in ${key}`);
  });
}

function el(tag, attrs = {}, ...kids) {
  const e = document.createElement(tag);
  for (const [k, v] of Object.entries(attrs)) {
    if (k === 'on') for (const [ev, fn] of Object.entries(v)) e.addEventListener(ev, fn);
    else if (k === 'class') e.className = v;
    else if (v === true) e.setAttribute(k, '');
    else if (v !== false && v != null) e.setAttribute(k, v);
  }
  for (const c of kids.flat()) if (c != null) e.append(c);
  return e;
}

function button(label, onClick, attrs = {}) {
  return el('button', { type: 'button', ...attrs, on: { click: onClick } }, label);
}

// --- tuning: ?tuning=0|1|2, as Vagrancy's motor tunings were ---------------
function tuning() {
  const q = new URLSearchParams(location.search).get('tuning');
  const n = Number.parseInt(q ?? '', 10);
  return Number.isInteger(n) && n >= 0 && n < NUM.tunings ? n : NUM.default_tuning;
}

// --- screens -------------------------------------------------------------

function screen(...kids) {
  stopRun();
  const s = $('screen');
  s.replaceChildren(...kids);
  s.hidden = false;
  $('stage').hidden = true;
  $('hud').hidden = true;
  const first = s.querySelector('button');
  if (first) first.focus();
}

function menu() {
  const item = (key, onClick, desc = true) =>
    el('div', { class: 'item' }, button(t(`menu.${key}.label`), onClick, { id: `menu-${key}` }), desc ? el('p', {}, t(`menu.${key}.desc`)) : null);
  screen(
    el('nav', { class: 'menu' },
      item('missions', () => startPour()),
      item('replay', () => loadReplay()),
    ),
    el('p', { id: 'notice', role: 'alert', hidden: true }),
  );
}

function notice(sentence) {
  const n = $('notice');
  if (!n) return;
  n.textContent = sentence;
  n.hidden = false;
}

// --- a run ---------------------------------------------------------------

function keyNames() {
  return NUM.actions.map(([action]) => keys.keyName(bindings[action] ?? ''));
}

function startPour() {
  const game = core.Game.standing(Date.now() % 100000, tuning());
  begin({ game, kind: 'pour' });
}

function begin(r) {
  stopRun();
  run = { ...r, acc: 0, last: performance.now(), frame: null };
  const s = $('screen');
  s.hidden = false;
  $('stage').hidden = false;
  $('hud').hidden = false;
  const controls = r.game.is_replay()
    ? [el('p', { id: 'replay-note' }, t('replay.playing')), button(t('replay.stop.label'), () => menu(), { id: 'replay-stop' })]
    : [button(t('replay.download.label'), () => saveReplay(), { id: 'download-replay' }), button(t('menu.back.label'), () => menu(), { id: 'back-to-menu' })];
  s.replaceChildren(el('div', { class: 'controls' }, ...controls));
  hud();
  draw();
  run.raf = requestAnimationFrame(loop);
}

function stopRun() {
  if (run?.raf) cancelAnimationFrame(run.raf);
  run = null;
}

function loop(now) {
  if (!run) return;
  const dt = 1000 / NUM.ticks_per_second;
  run.acc += Math.min(now - run.last, 250);
  run.last = now;
  let steps = 0;
  while (run.acc >= dt && steps < 6) {
    run.acc -= dt;
    steps += 1;
    if (run.game.is_replay()) {
      if (run.game.replay_done()) break;
      run.game.step(0, 0);
    } else {
      run.game.step(keys.bits(bindings, ACTION_BITS), 0);
    }
  }
  if (steps) draw();
  run.raf = requestAnimationFrame(loop);
}

function draw() {
  run.frame = JSON.parse(run.game.frame());
  stage.draw(run.frame, run.game.units(), keyNames());
  document.body.dataset.tick = String(run.game.tick());
}

// The HUD: each spout's label and key, in spout order. Every line is a copy
// string; the key names and flavor names are values filled into it.
function hud() {
  const f = JSON.parse(run.game.frame());
  const line = f.lines[0];
  const names = keyNames();
  const rows = line.spouts.map((s, i) => el('li', {}, spoutLabel(s.pours), ' ', el('strong', {}, t('spout.key', { key: names[i] }))));
  $('hud').replaceChildren(
    el('p', {}, t('hud.keys', { keys_upper: names.slice(0, line.spouts.length).join(', ') })),
    el('ul', { class: 'spouts' }, ...rows),
  );
}

function flavorOf(f) { return FLAVORS[f].id; }

function spoutLabel(pours) {
  const ids = [...new Set(pours)];
  if (ids.length === 1) {
    const id = flavorOf(ids[0]);
    return t('spout.single', { flavor: t(`flavors.${id}.name`), short: t(`flavors.${id}.short`), pattern: t(`patterns.${FLAVORS[ids[0]].pattern}`) });
  }
  return ids.map((f) => t(`flavors.${flavorOf(f)}.name`)).join(', ');
}

// --- replays -----------------------------------------------------------

function saveReplay() {
  if (!run) return;
  download(run.game.replay_bytes(), `slush-${run.kind}-${run.game.tick()}.replay`);
}

async function loadReplay() {
  const f = await pick('.replay,application/octet-stream');
  if (!f) return;
  let game;
  try {
    game = core.Game.load_replay(f.bytes);
  } catch (e) {
    const why = JSON.parse(typeof e === 'string' ? e : String(e));
    menu();
    notice(t(why.key, why.values));
    return;
  }
  begin({ game, kind: 'replay' });
}

// --- start ------------------------------------------------------------

async function main() {
  try {
    await init();
  } catch (e) {
    $('status').hidden = true;
    $('loading-error').hidden = false;
    throw e;
  }
  COPY = JSON.parse(core.copy_json());
  NUM = JSON.parse(core.numbers());
  PAL = JSON.parse(core.palette_json());
  FLAVORS = JSON.parse(core.flavors_json()).flavors;
  CONTROLS = JSON.parse(core.controls_json());
  ACTION_BITS = Object.fromEntries(NUM.actions);
  bindings = { ...CONTROLS.solo };
  stage = new Stage($('stage'), NUM, PAL, FLAVORS);
  stage.codes = FLAVORS.map((f) => t(`flavors.${f.id}.short`));
  keys.listen((code) => Object.values(bindings).includes(code));
  // Hooks for the gate (testing/drive.py). They read core; they decide nothing.
  window.slushline = {
    scriptChecksum: (ticks) => core.script_checksum(ticks),
    checksum: () => run?.game.checksum(),
    recordedChecksum: () => run?.game.recorded_checksum(),
    tick: () => run?.game.tick(),
    replayDone: () => run?.game.replay_done(),
    frame: () => run?.frame,
  };
  $('status').hidden = true;
  menu();
  document.body.dataset.build = BUILD;
  document.body.dataset.ready = '1';
}

main();
