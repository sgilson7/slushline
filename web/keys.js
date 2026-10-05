// Keys to input bits. The bit for each action comes from core
// (sim::Input::ACTIONS through numbers()); the bindings from
// data/controls.json or the player's save. The page keeps no copy of either.
// Ported from Vagrancy's web/keys.js.

const held = new Set();

export function listen(isBound) {
  window.addEventListener('keydown', (e) => {
    if (isBound(e.code) && !e.target.closest?.('input, textarea')) e.preventDefault();
    held.add(e.code);
  });
  window.addEventListener('keyup', (e) => held.delete(e.code));
  window.addEventListener('blur', () => held.clear());
}

// The input bits for one seat: the OR of every held action's bit.
export function bits(binding, actionBits) {
  let b = 0;
  for (const [action, code] of Object.entries(binding)) {
    if (held.has(code) && action in actionBits) b |= actionBits[action];
  }
  return b;
}

// What a key code shows as in a sentence: "KeyD" reads as "D", "Semicolon"
// as ";". A key's name is a value filled into a sentence, like a file name,
// not a sentence of its own.
export function keyName(code) {
  const side = code.match(/^(Shift|Control|Alt|Meta)(Left|Right)$/);
  if (side) return `${side[2]} ${side[1] === 'Control' ? 'Ctrl' : side[1]}`;
  const punct = { Semicolon: ';', Comma: ',', Period: '.', Slash: '/', Quote: "'", BracketLeft: '[', BracketRight: ']', Minus: '-', Equal: '=', Backquote: '`', Backslash: '\\', Space: 'Space' };
  if (code in punct) return punct[code];
  return code.replace(/^Key/, '').replace(/^Digit/, '').replace(/^Arrow/, '').replace(/^Numpad/, 'Numpad ');
}
