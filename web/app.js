// The page: screens and the clock. It draws numbers core sent and decides
// nothing (CLAUDE.md). Every word it shows comes through t().
import init, * as core from './pkg/slushline_wasm.js';

const BUILD = '__BUILD__';
let COPY = null;

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

async function main() {
  try {
    await init();
  } catch (e) {
    document.getElementById('status').hidden = true;
    document.getElementById('loading-error').hidden = false;
    throw e;
  }
  COPY = JSON.parse(core.copy_json());
  document.getElementById('status').hidden = true;
  document.body.dataset.build = BUILD;
  document.body.dataset.ready = '1';
}

main();
