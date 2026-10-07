// The page: screens and the clock. It draws numbers core sent and decides
// nothing (CLAUDE.md). Every word it shows comes through t(), or is a
// sentence core filled from the copy file.
import init, * as core from './pkg/slushline_wasm.js';
import { Stage } from './draw.js';
import * as keys from './keys.js';
import { download, pick } from './files.js';
import * as sound from './sound.js';
import * as grooveArt from './groove.js';
import { blendFrame, blendUnits } from './blend.js';
import * as music from './music.js';

const BUILD = '__BUILD__';
const STORE = 'slushline.save';
let COPY, NUM, PAL, FLAVORS, ACTION_BITS, LOWER_BITS, BELT_BITS;
let stage = null;
let run = null;          // the run on screen
let save = null;         // the save, as core's JSON
let rebinding = null;    // the action waiting for a key

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
  for (const c of kids.flat()) if (c != null && c !== false) e.append(c);
  return e;
}

function button(label, onClick, attrs = {}) {
  return el('button', { type: 'button', ...attrs, on: { click: onClick } }, label);
}

// A key's name, as a value inside a sentence rather than a sentence.
function keyValue(code) {
  return el('span', { class: 'key', 'data-value': '1' }, keys.keyName(code));
}

// --- the save ------------------------------------------------------------

function bindings() { return JSON.parse(save).keys; }
function options() { return JSON.parse(save).options; }

// The convenience copy in this browser. The save file the player downloads
// is the real one (settings.save.autosave says so).
function keep(json) {
  save = json;
  try { localStorage.setItem(STORE, json); } catch { /* storage refused */ }
  stage.showCodes = options().short_codes;
  sound.setVolume(options().sound_volume / 100);
}

function restore() {
  let stored = null;
  try { stored = localStorage.getItem(STORE); } catch { /* storage refused */ }
  if (stored) {
    try {
      save = core.save_load(new TextEncoder().encode(stored));
      return;
    } catch { /* a stale copy is dropped, not shown */ }
  }
  save = core.save_default();
}

function keyNames() {
  const b = bindings();
  return NUM.actions.map(([action]) => keys.keyName(b[action] ?? ''));
}

// The lower line's spout keys, when a mission has two lines.
function lowerKeyNames() {
  const b = bindings();
  return NUM.lower_actions.map(([action]) => keys.keyName(b[action] ?? ''));
}

function keyValues() {
  const n = keyNames();
  const v = {};
  NUM.actions.forEach(([action], i) => { v[`key.${action}`] = n[i]; });
  return v;
}

// What Settings calls an action: the spouts by number, the belt by name.
function actionLabel(action) {
  const spout = action.match(/^spout_(\d)$/);
  if (spout) return t('settings.keys.actions.spout', { n: spout[1] });
  const lower = action.match(/^lower_(\d)$/);
  if (lower) return t('settings.keys.actions.lower', { n: lower[1] });
  return t(`settings.keys.actions.${action}`);
}

// --- tuning: ?tuning=0|1|2 -------------------------------------------------
function tuning() {
  const q = new URLSearchParams(location.search).get('tuning');
  const n = Number.parseInt(q ?? '', 10);
  return Number.isInteger(n) && n >= 0 && n < NUM.tunings ? n : NUM.default_tuning;
}

// --- screens -------------------------------------------------------------

function screen(...kids) {
  stopRun();
  rebinding = null;
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
      item('missions', () => path()),
      item('replay', () => loadReplay()),
      item('how', () => how(), false),
      item('settings', () => settings(), false),
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

function back() {
  return button(t('menu.back.label'), () => menu(), { id: 'back-to-menu', class: 'quiet' });
}

// --- the path ------------------------------------------------------------

function missions() { return JSON.parse(core.path_json(save)); }

// The missions are a tree (Sam, 2026-10-05), after Vagrancy's road: one row
// per number of requirements, a line from each requirement down to the
// mission it opens, lit while the pointer is on either end. Picking a
// mission shows its card above the tree. Which missions are open and which
// requirements are met is core's (path_json); the page lays them out.
const SVG = 'http://www.w3.org/2000/svg';

// Which view of the missions: the tree or the lanes. Remembered in this
// browser only, as a convenience.
function missionView() {
  try { const v = localStorage.getItem('slushline.view'); if (v === 'tree' || v === 'chart') return v; } catch { /* storage off */ }
  return 'tree';
}

function path(pickId = null) {
  const view = missionView();
  const chart = view === 'chart';
  const ms = missions();
  const byId = new Map(ms.map((m) => [m.id, m]));
  const cardBox = el('section', { id: 'mission-card' });
  const tree = el('div', { id: 'tree', class: chart ? 'chart' : 'rows' });
  const wires = document.createElementNS(SVG, 'svg');
  wires.setAttribute('class', 'wires');
  wires.setAttribute('aria-hidden', 'true');
  tree.append(wires);
  const buttons = new Map();
  const node = (m) => {
    const state = m.passed ? 'passed' : m.open ? 'open' : 'locked';
    const b = el('button', { type: 'button', id: `mission-${m.id}`, class: `node ${state}`, 'aria-pressed': 'false',
      on: { click: () => pick(m.id), mouseenter: () => light(m.id), focus: () => light(m.id), mouseleave: () => light(null), blur: () => light(null) } }, m.name);
    buttons.set(m.id, b);
    return b;
  };
  // Rows are depths (2026-10-06): how many missions lie between a mission
  // and the start.
  const tierLabel = (l) => (l === 0 ? t('missions.tier.start') : l === 1 ? t('missions.tier.step_one') : t('missions.tier.step', { n: l }));
  const depth = Math.max(...ms.map((m) => m.level)) + 1;
  if (chart) {
    // The lanes (Sam, 2026-10-05: "a hasse diagram with seperate lanes /
    // paths of missions"): a column for each chapter, a row for each number
    // of requirements, and only the Hasse diagram's lines, which core chose.
    const lanes = JSON.parse(core.chapters_json());
    tree.style.gridTemplateColumns = `7.5em repeat(${lanes.length}, minmax(0, 1fr))`;
    tree.style.gridTemplateRows = `auto repeat(${depth}, minmax(64px, auto))`;
    lanes.forEach((lane, i) => {
      const bg = el('div', { class: `lane lane-${i % 2}` });
      bg.style.gridColumn = String(i + 2);
      bg.style.gridRow = `1 / span ${depth + 1}`;
      tree.append(bg);
      const head = el('h4', { class: 'lane-name' }, lane.name);
      head.style.gridColumn = String(i + 2);
      head.style.gridRow = '1';
      tree.append(head);
    });
    for (let l = 0; l < depth; l += 1) {
      const label = el('p', { class: 'tier-label' }, tierLabel(l));
      label.style.gridColumn = '1';
      label.style.gridRow = String(l + 2);
      tree.append(label);
    }
    const cells = new Map();
    for (const m of ms) {
      const key = `${m.chapter}:${m.level}`;
      if (!cells.has(key)) {
        const cell = el('div', { class: 'cell' });
        cell.style.gridColumn = String(lanes.findIndex((x) => x.id === m.chapter) + 2);
        cell.style.gridRow = String(m.level + 2);
        cells.set(key, cell);
        tree.append(cell);
      }
      cells.get(key).append(node(m));
    }
  } else {
    // The tree, after Vagrancy's road: within a row, by where the
    // requirements sit in the rows above, which keeps lines from crossing
    // more than they must.
    const rows = [];
    for (const m of ms) (rows[m.level] ||= []).push(m);
    const place = new Map();
    rows.forEach((row, l) => {
      if (!row) return;
      if (l > 0) {
        const at = (m) => m.requires.reduce((a, r) => a + (place.get(r.from) ?? 0.5), 0) / Math.max(1, m.requires.length);
        row.sort((a, b) => at(a) - at(b));
      }
      row.forEach((m, i) => place.set(m.id, (i + 0.5) / row.length));
    });
    rows.forEach((row, l) => {
      if (row) tree.append(el('div', { class: 'level' }, el('p', { class: 'tier-label' }, tierLabel(l)), el('div', { class: 'tier' }, ...row.map(node))));
    });
  }
  const paths = [];
  function wire() {
    wires.replaceChildren();
    paths.length = 0;
    const box = tree.getBoundingClientRect();
    wires.setAttribute('viewBox', `0 0 ${box.width} ${box.height}`);
    for (const m of ms) {
      const froms = chart ? m.hasse : m.requires.map((r) => r.from);
      for (const from of froms) {
        const met = m.requires.find((r) => r.from === from)?.met;
        const a = buttons.get(from).getBoundingClientRect();
        const c = buttons.get(m.id).getBoundingClientRect();
        const x1 = a.left - box.left + a.width / 2, y1 = a.bottom - box.top;
        const x2 = c.left - box.left + c.width / 2, y2 = c.top - box.top;
        const k = (y2 - y1) / 2;
        const line = document.createElementNS(SVG, 'path');
        line.setAttribute('d', `M ${x1} ${y1} C ${x1} ${y1 + k}, ${x2} ${y2 - k}, ${x2} ${y2}`);
        line.setAttribute('class', met ? 'met' : 'unmet');
        wires.append(line);
        paths.push({ from, to: m.id, line });
      }
    }
    tree.dataset.lines = String(paths.length);
  }
  function light(id) {
    for (const p of paths) p.line.classList.toggle('hot', id !== null && (p.to === id || p.from === id));
  }
  function pick(id) {
    for (const [k, b] of buttons) { b.classList.toggle('picked', k === id); b.setAttribute('aria-pressed', String(k === id)); }
    fillCard(cardBox, byId.get(id));
  }
  const pickView = (v) => { try { localStorage.setItem('slushline.view', v); } catch { /* storage off */ } path(pickId); };
  const switcher = el('div', { class: 'controls view-switch' },
    button(t('missions.view.tree.label'), () => pickView('tree'), { id: 'view-tree', class: chart ? 'quiet' : '', 'aria-pressed': String(!chart) }),
    button(t('missions.view.chart.label'), () => pickView('chart'), { id: 'view-chart', class: chart ? '' : 'quiet', 'aria-pressed': String(chart) }));
  screen(el('h2', {}, t('menu.missions.label')), cardBox, switcher, tree, el('div', { class: 'controls' }, back()));
  const first = pickId ?? (ms.find((m) => m.open && !m.passed) ?? ms[0]).id;
  pick(first);
  wire();
  new ResizeObserver(() => { if (tree.isConnected) wire(); }).observe(tree);
}

// The card above the tree: what the mission asks, what opens it, and the
// button to start it if it is open.
function fillCard(box, m) {
  const max = NUM.max_score;
  box.replaceChildren(...[
    el('h3', { id: 'card-name' }, m.name),
    el('p', {}, m.order),
    el('p', {}, m.pass),
    ...m.conditions.map((c) => el('p', { class: 'condition' }, el('strong', {}, c.name), ' ', c.desc)),
    m.best != null ? el('p', {}, t('missions.best', { avg: m.best, max_score: max })) : null,
    m.requires.length ? el('p', {}, t('missions.needs')) : null,
    m.requires.length ? el('ul', { class: 'needs' }, ...m.requires.map((r) => el('li', { class: r.met ? 'met' : 'unmet' },
      el('span', { class: 'mark' }, r.met ? t('missions.met') : t('missions.unmet')), ' ', r.sentence))) : null,
    m.open ? el('div', { class: 'controls' }, button(t('missions.start.label'), () => card(m.id), { id: 'open-mission' })) : null,
  ].filter(Boolean));
}

function card(id) {
  const m = missions().find((x) => x.id === id);
  const kv = keyValues();
  const upper = keyNames().slice(0, m.spouts[0]).join(', ');
  const lower = m.lines > 1 ? lowerKeyNames().slice(0, m.spouts[1]).join(', ') : '';
  const names = (seat) => (seat === 0 ? keyNames() : lowerKeyNames());
  const lineList = (labels, seat) => el('ul', { class: 'spouts' }, ...labels.map((label, i) => el('li', {}, label, ' ', el('strong', {}, t('spout.key', { key: names(seat)[i] })))));
  screen(
    el('h2', { id: 'card-name' }, m.name),
    el('p', {}, m.order),
    el('p', {}, m.pass),
    ...m.conditions.map((c) => el('p', { class: 'condition' }, el('strong', {}, c.name), ' ', c.desc)),
    el('p', { class: 'try' }, t(m.try_key, { ...kv, keys_upper: upper, keys_lower: lower })),
    ...m.spout_labels.flatMap((labels, seat) => [
      m.lines > 1 ? el('h4', {}, t(seat === 0 ? 'lines.upper.name' : 'lines.lower.name')) : null,
      lineList(labels, seat),
    ]),
    el('p', {}, t('hud.belt_keys', kv)),
    el('div', { class: 'controls' },
      button(t('missions.start.label'), () => startMission(id), { id: 'start-mission' }),
      button(t('results.to_missions.label'), () => path(id), { class: 'quiet', id: 'to-missions' })),
  );
}

// --- a run ---------------------------------------------------------------

function seed() { return (Date.now() % 1000000) >>> 0; }

function startMission(id) {
  sound.wake();
  const game = core.Game.mission(id, seed(), tuning());
  begin({ game, kind: id });
}

function begin(r) {
  stopRun();
  music.play();
  rebinding = null;
  const m = r.game.mission_id() ? missions().find((x) => x.id === r.game.mission_id()) : null;
  run = { ...r, acc: 0, last: performance.now(), frame: null, mission: m, finished: false, lines: JSON.parse(r.game.frame()).lines.length };
  const s = $('screen');
  s.hidden = false;
  $('stage').hidden = false;
  $('hud').hidden = false;
  const controls = r.game.is_replay()
    ? [el('p', { id: 'replay-note' }, t('replay.playing')), button(t('replay.stop.label'), () => menu(), { id: 'replay-stop' })]
    : [button(t('results.to_missions.label'), () => path(), { id: 'leave-run', class: 'quiet' })];
  s.replaceChildren(el('div', { class: 'controls' }, ...controls));
  hudStatic();
  draw();
  run.raf = requestAnimationFrame(loop);
}

function stopRun() {
  if (run?.raf) cancelAnimationFrame(run.raf);
  run = null;
  sound.pourStop();
  const j = $('judge');
  if (j) j.hidden = true;
  grooveArt.stop($('groove'));
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
    // The tick before the newest is what the screen draws from (blend.js).
    if (run.acc < dt) run.prev = run.cur && run.cur.tick === run.game.tick() ? run.cur : snapshot();
    if (run.game.is_replay()) {
      if (run.game.replay_done()) break;
      run.game.step(0, 0);
    } else {
      if (run.game.done()) break;
      // Seat 0 is the only line, or the upper one; seat 1 the lower one,
      // from its own spout keys, with the same belt keys.
      const upper = keys.bits(bindings(), ACTION_BITS);
      const lower = run.lines > 1 ? keys.bits(bindings(), LOWER_BITS) | (upper & BELT_BITS) : 0;
      run.game.step(upper, lower);
    }
  }
  if (steps) {
    take();
    hudLive();
    for (const j of JSON.parse(run.game.take_judged())) judge(j.word, j.groove);
  }
  const over = run.game.is_replay() ? run.game.replay_done() : run.game.done();
  if (over && !run.finished) sound.pourStop();
  if (over && !run.finished && run.mission) {
    run.finished = true;
    // Let the last lid close on screen for a moment before the result.
    setTimeout(() => { if (run?.finished) results(); }, 700);
  }
  // Every screen refresh draws, at the share of a tick the clock has run
  // past the newest one.
  if (run) {
    paint(Math.min(1, run.acc / dt));
    run.raf = requestAnimationFrame(loop);
  }
}

function snapshot() {
  return { tick: run.game.tick(), frame: JSON.parse(run.game.frame()), units: run.game.units() };
}

// The newest tick core has stepped to: kept for drawing, and for the sound.
function take() {
  run.cur = snapshot();
  run.frame = run.cur.frame;
  // The pour's sound follows the openings core sent in this frame.
  const over = run.game.is_replay() ? run.game.replay_done() : run.game.done();
  if (!over) sound.pourFrame(run.frame);
  document.body.dataset.tick = String(run.cur.tick);
}

// Draw `t` of the way from the tick before the newest to the newest. The
// canvas is sized from the newest frame as core sent it, so a blended handle
// never nudges its height.
function paint(t) {
  const { cur, prev } = run;
  const p = prev && prev.tick === cur.tick - 1 ? prev : null;
  stage.draw(blendFrame(p?.frame, cur.frame, t), blendUnits(p?.units, cur.units, t), keyNames(), lowerKeyNames(), cur.frame);
}

// Step outside the clock (a new run, the gate's hooks): draw the newest tick.
function draw() {
  take();
  run.prev = null;
  paint(1);
}

// A judged cup: the lid's sound and a big word over the line, after Dance
// Dance Revolution (Sam, 2026-10-05). The word and the phrase are the ones
// core chose for the score; the page only performs them.
function judge(word, groove = false) {
  sound.judged(word);
  if (groove) setTimeout(() => playGroove(), 650);
  const j = $('judge');
  const fresh = j.cloneNode(false);
  fresh.className = `judge-${word}`;
  fresh.textContent = t(`judge.${word}`);
  fresh.hidden = false;
  fresh.dataset.word = word;
  j.replaceWith(fresh);
  clearTimeout(judge.timer);
  judge.timer = setTimeout(() => { $('judge').hidden = true; }, 1300);
}

// Three of the top word in a row (core decides when): the groove and its
// animation, over the middle of the line.
function playGroove() {
  if (!run) return;
  const seconds = sound.groove();
  const L = run.frame.lines[0];
  // Centered on the line rather than the lid, which is near the edge.
  grooveArt.play($('groove'), NUM, PAL, FLAVORS, $('groove').width / 2, stage.sy(L.rim), seconds, sound.GROOVE_BPM);
  document.body.dataset.groove = String(Number(document.body.dataset.groove ?? 0) + 1);
}

// The HUD: the keys and spouts once, then the live numbers core worked out.
function hudStatic() {
  const f = JSON.parse(run.game.frame());
  const two = f.lines.length > 1;
  const labels = run.mission ? run.mission.spout_labels : f.lines.map((L) => L.spouts.map(() => ''));
  const kids = [el('div', { id: 'hud-live' })];
  f.lines.forEach((L, seat) => {
    const names = seat === 0 ? keyNames() : lowerKeyNames();
    const list = names.slice(0, L.spouts.length).join(', ');
    kids.push(el('p', { class: 'hud-keys' }, seat === 0 ? t('hud.keys', { keys_upper: list }) : t('hud.keys_lower', { keys_lower: list })));
    kids.push(el('ul', { class: 'spouts' },
      two ? el('li', {}, el('strong', {}, t(seat === 0 ? 'lines.upper.name' : 'lines.lower.name'))) : null,
      ...L.spouts.map((s, i) => el('li', {}, labels[seat]?.[i] || null, labels[seat]?.[i] ? ' ' : null, el('strong', {}, t('spout.key', { key: names[i] }))))));
  });
  if (!run.game.is_replay()) kids.push(el('p', { id: 'hud-belt-keys' }, t('hud.belt_keys', keyValues())));
  $('hud').replaceChildren(...kids);
  hudLive();
}

function hudLive() {
  if (!run?.mission) return;
  const h = JSON.parse(run.game.hud());
  const live = $('hud-live');
  if (!live) return;
  const groups = h.lines.map((L) => [L.name, L.cup, L.order].filter(Boolean));
  const tail = [h.score, h.waste, h.belt].filter(Boolean);
  const text = JSON.stringify([groups, tail]);
  if (live.dataset.text === text) return;
  live.dataset.text = text;
  live.replaceChildren(
    ...groups.map((g) => el('span', { class: 'hud-line' }, ...g.map((p) => el('span', {}, p)))),
    ...tail.map((p) => el('span', {}, p)),
  );
}

// --- results -------------------------------------------------------------

function results() {
  const game = run.game;
  const m = run.mission;
  const o = JSON.parse(game.outcome());
  const replay = game.is_replay();
  if (!replay) keep(core.save_record(save, m.id, o.average, o.passed, o.waste_pct));
  const ms = missions();
  // The first mission this run has just opened, if any.
  const next = ms.find((x) => m.opens.includes(x.id) && x.open) ?? null;
  const bytes = game.replay_bytes();
  const name = `slushline-${m.id}-${o.average}.replay`;
  const buttons = [
    o.passed && next?.open && !replay ? button(t('results.next.label'), () => card(next.id), { id: 'next-mission' }) : null,
    replay ? null : button(t('results.again.label'), () => startMission(m.id), { id: 'run-again' }),
    button(t('results.to_missions.label'), () => path(m.id), { id: 'to-missions', class: 'quiet' }),
    button(t('results.replay.label'), () => download(bytes, name), { id: 'download-replay', class: 'quiet' }),
  ].filter(Boolean);
  const cups = [];
  for (let k = 0; k < o.lines.length - (o.cause ? 2 : 1); k += 2) {
    cups.push(el('li', {}, el('strong', {}, o.lines[k]), ' ', o.lines[k + 1]));
  }
  const tail = o.lines.slice(cups.length * 2);
  screen(
    el('h2', {}, m.name),
    ...tail.map((l, i) => el('p', { class: i === 0 ? 'verdict' : '' }, l)),
    el('ul', { class: 'cups' }, ...cups),
    el('div', { class: 'controls' }, ...buttons),
    el('p', { class: 'hint' }, t('results.key_hint', { key: keys.keyName('Enter') })),
  );
  document.body.dataset.result = o.passed ? 'pass' : 'fail';
  $('screen').dataset.average = String(o.average);
}

// --- replays -----------------------------------------------------------

async function loadReplay() {
  sound.wake();
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

// --- how to play ---------------------------------------------------------

function how() {
  const values = { ...keyValues(), swell_s: core.swell_seconds(tuning()), max_score: NUM.max_score };
  const topics = ['spout', 'lead', 'tail', 'swell', 'belt', 'order', 'score', 'pass', 'blend', 'files'];
  screen(
    el('h2', {}, t('menu.how.label')),
    ...topics.map((k) => el('section', { class: 'how' }, el('h3', {}, t(`how.${k}.title`)), el('p', {}, t(`how.${k}.body`, values)))),
    el('div', { class: 'controls' }, back()),
  );
}

// --- settings ------------------------------------------------------------

function settings(message = null) {
  const b = bindings();
  const rows = NUM.actions.map(([action]) => el('li', {},
    el('span', {}, actionLabel(action)), ' ',
    button(keys.keyName(b[action]), () => waitForKey(action), { id: `bind-${action}`, class: 'quiet key', 'data-value': '1' }),
  ));
  const lowerRows = NUM.lower_actions.map(([action]) => el('li', {},
    el('span', {}, actionLabel(action)), ' ',
    button(keys.keyName(b[action]), () => waitForKey(action), { id: `bind-${action}`, class: 'quiet key', 'data-value': '1' }),
  ));
  const vol = el('input', { type: 'range', id: 'sound-volume', min: '0', max: '100', step: '5', value: String(options().sound_volume) });
  vol.addEventListener('change', () => { keep(core.save_set_volume(save, Number(vol.value))); sound.wake(); sound.judged('great'); });
  const short = el('input', { type: 'checkbox', id: 'short-codes', checked: options().short_codes });
  short.addEventListener('change', () => keep(core.save_set_short_codes(save, short.checked)));
  screen(
    el('h2', {}, t('menu.settings.label')),
    el('section', {},
      el('h3', {}, t('settings.look.title')),
      el('label', { for: 'short-codes', class: 'switch' }, short, ' ', t('settings.look.short.label')),
      el('p', {}, t('settings.look.short.desc')),
      el('label', { for: 'sound-volume', class: 'switch' }, t('settings.sound.volume.label'), ' ', vol)),
    el('section', {},
      el('h3', {}, t('settings.keys.title')),
      el('p', {}, t('settings.keys.desc')),
      el('h4', {}, t('settings.keys.solo.heading')),
      el('ul', { class: 'bindings' }, ...rows),
      el('h4', {}, t('settings.keys.lower.heading')),
      el('ul', { class: 'bindings' }, ...lowerRows),
      el('p', { id: 'bind-note', role: 'alert', hidden: !message }, message ?? ''),
      button(t('settings.keys.reset.label'), () => { keep(core.save_reset_keys(save)); settings(); }, { id: 'reset-keys', class: 'quiet' })),
    el('section', {},
      el('h3', {}, t('settings.save.title')),
      el('p', {}, t('settings.save.download.desc')),
      el('div', { class: 'controls' },
        button(t('settings.save.download.label'), () => download(new TextEncoder().encode(save), 'slushline.save', 'application/json'), { id: 'download-save' }),
        button(t('settings.save.load.label'), () => loadSave(), { id: 'load-save', class: 'quiet' })),
      el('p', {}, t('settings.save.autosave')),
      el('p', { id: 'save-note', role: 'status', hidden: true })),
    el('div', { class: 'controls' }, back()),
  );
}

function waitForKey(action) {
  rebinding = action;
  const btn = $(`bind-${action}`);
  if (btn) btn.classList.add('waiting');
}

window.addEventListener('keydown', (e) => {
  if (!rebinding) {
    if (e.code === 'Enter' && !$('screen').hidden && !run) {
      const first = $('screen').querySelector('button');
      if (first && document.activeElement?.tagName !== 'BUTTON') { e.preventDefault(); first.click(); }
    }
    return;
  }
  e.preventDefault();
  const action = rebinding;
  rebinding = null;
  try {
    keep(core.save_rebind(save, action, e.code));
    settings();
  } catch (err) {
    const why = JSON.parse(typeof err === 'string' ? err : String(err));
    settings(t(why.key, { key: keys.keyName(e.code), action: actionLabel(why.other) }));
  }
}, true);

async function loadSave() {
  const f = await pick('.save,.json,application/json');
  if (!f) return;
  try {
    keep(core.save_load(f.bytes));
    settings();
    const n = $('save-note');
    n.textContent = t('settings.save.loaded');
    n.hidden = false;
  } catch (err) {
    const why = JSON.parse(typeof err === 'string' ? err : String(err));
    settings();
    const n = $('save-note');
    n.textContent = t(why.key, why.values);
    n.hidden = false;
  }
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
  ACTION_BITS = Object.fromEntries(NUM.actions);
  LOWER_BITS = Object.fromEntries(NUM.lower_actions);
  BELT_BITS = ACTION_BITS.belt_slower | ACTION_BITS.belt_faster;
  stage = new Stage($('stage'), NUM, PAL, FLAVORS);
  stage.onResize = (w, h) => { const gc = $('groove'); gc.width = w; gc.height = h; };
  stage.codes = FLAVORS.map((f) => t(`flavors.${f.id}.short`));
  restore();
  music.setup(t);
  stage.showCodes = options().short_codes;
  sound.setVolume(options().sound_volume / 100);
  keys.listen((code) => run && Object.values(bindings()).includes(code));
  // Any key a player presses lets the browser start sound, so the first
  // lid is not lost to a context still waking up.
  window.addEventListener('keydown', () => sound.wake(), { passive: true });
  // A hidden tab stops drawing frames, so its pours would hiss on unheard
  // by the frames that would close them.
  document.addEventListener('visibilitychange', () => { if (document.hidden) sound.pourStop(); });
  // Hooks for the gate (testing/drive.py). They read core; they decide nothing.
  window.slushline = {
    scriptChecksum: (ticks) => core.script_checksum(ticks),
    checksum: () => run?.game.checksum(),
    recordedChecksum: () => run?.game.recorded_checksum(),
    tick: () => run?.game.tick(),
    replayDone: () => run?.game.replay_done(),
    done: () => run?.game.done(),
    frame: () => run?.frame,
    save: () => save,
    // Step the run on screen as fast as the browser can, with the given
    // input, so the gate does not wait a mission's length in real time.
    // Step a replay being watched, as fast as the browser can.
    skipReplay: (ticks) => {
      for (let k = 0; k < ticks && run?.game.is_replay() && !run.game.replay_done(); k += 1) run.game.step(0, 0);
      if (run) draw();
    },
    // Where each unit was drawn, in canvas pixels, with its flavor.
    unitPixels: () => {
      if (!run) return [];
      const u = run.game.units();
      const out = [];
      // Whether each unit sits in a cup, from the cups core sent: the gate
      // samples the slush in cups, not slush passing behind a nozzle.
      const L = run.frame.lines;
      // The cups are flower pots: narrower than inner_half by the flare at
      // the floor, wider by it at the rim, straight between.
      const halfAt = (Ls, c, y) => Ls.inner_half - Ls.flare + (2 * Ls.flare * (y - c.floor)) / (c.rim - c.floor);
      const inCup = (seat, x, y) => L[seat]?.cups.some((c) => !c.judged && Math.abs(x - c.x) < halfAt(L[seat], c, y) && y > c.floor && y < c.rim);
      for (let k = 0; k < u.length; k += 5) out.push({ f: u[k + 4] & 255, x: stage.sx(u[k + 1]), y: stage.sy(u[k + 2]), r: stage.len(u[k + 3]), cup: !!inCup(u[k + 4] >> 8, u[k + 1], u[k + 2]) });
      return out;
    },
    autoplay: (ticks) => {
      if (!run) return;
      run.game.autoplay(ticks);
      draw();
      for (const j of JSON.parse(run.game.take_judged())) judge(j.word, j.groove);
    },
    // Play the groove on the run on screen, as three top cups in a row would.
    groove: () => playGroove(),
    grooveShowing: () => !$('groove').hidden,
    pourVoices: () => sound.pourVoices(),
    // Perform a judgement's word and sound, as a lid closing would.
    performJudgement: (word) => judge(word),
    voiceClips: () => sound.clipState(),
    music: () => music.state(),
    judgement: () => { const j = $('judge'); return j && !j.hidden ? [j.dataset.word, j.textContent] : null; },
  };
  $('status').hidden = true;
  menu();
  document.body.dataset.build = BUILD;
  document.body.dataset.ready = '1';
}

main();
