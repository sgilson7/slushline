// Fingers and the mouse to actions (Sam, 2026-10-10: touch the nozzles to
// pour, in two modes: "single tap to open, single tap to close", or "hold to
// hold open, closes when released").
//
// The same shape as keys.js and the newer Builder repos' input.js: a device
// becomes a set of actions, and only actions become core's input bits. The
// page says which action is under a point (`at`); this module says nothing
// about the game. Pointer Events cover touch, pen and mouse with one path
// (after CirqueGame's web/input.js), and a pointer is captured, so a finger
// that slides off the nozzle still lets go where it lifts.
//
// Two modes. Hold: an action is on while a pointer that pressed it is down.
// Tap: a press turns its action on, and the next press on it turns it off;
// the latch is the input device's, like a sticky key, so core still sees
// only which bits are held each tick, and a replay records the same.

export function makeTouch() {
  const pressed = new Map();   // pointerId -> action, while the pointer is down
  const latched = new Set();   // actions on until tapped again (tap mode)
  let mode = 'hold';
  let used = false;            // a finger has touched the page

  function press(e, action, tappable) {
    if (!action) return false;
    e.preventDefault();
    if (e.pointerType === 'touch') used = true;
    if (tappable && mode === 'tap') {
      if (latched.has(action)) latched.delete(action);
      else latched.add(action);
      return true;
    }
    pressed.set(e.pointerId, action);
    // Capture, so the release reaches us wherever the finger lifts. A
    // pointer the browser no longer counts as down cannot be captured.
    try { e.currentTarget.setPointerCapture?.(e.pointerId); } catch { /* not capturable */ }
    return true;
  }
  const release = (e) => pressed.delete(e.pointerId);

  // An area whose actions come from where it is pressed (the canvas and its
  // nozzles). Tap mode applies here.
  function area(el, at) {
    el.addEventListener('pointerdown', (e) => press(e, at(e.clientX, e.clientY), true));
    for (const ev of ['pointerup', 'pointercancel', 'lostpointercapture']) el.addEventListener(ev, release);
  }

  // A button held for one action (the belt's throttle). Always held, never
  // latched: a throttle left on would run away.
  function holdButton(el, action) {
    el.addEventListener('pointerdown', (e) => press(e, action, false));
    for (const ev of ['pointerup', 'pointercancel', 'lostpointercapture']) el.addEventListener(ev, release);
    el.addEventListener('contextmenu', (e) => e.preventDefault());
  }

  // The input bits for the actions on now, by core's bit for each action.
  function bits(actionBits) {
    let b = 0;
    for (const a of [...pressed.values(), ...latched]) if (a in actionBits) b |= actionBits[a];
    return b;
  }

  return {
    area,
    holdButton,
    bits,
    isLatched: (action) => latched.has(action),
    setMode(m) { mode = m === 'tap' ? 'tap' : 'hold'; latched.clear(); },
    mode: () => mode,
    used: () => used,
    // Everything off: a run ended, the page was hidden.
    clear() { pressed.clear(); latched.clear(); },
  };
}
