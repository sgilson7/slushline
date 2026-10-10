// Fullscreen (Sam, 2026-10-10: "some form of fullscreen mode or button").
// The standard Fullscreen API, and Safari's prefixed one, which iPadOS
// Safari supports for the whole page. On a phone, or a browser without
// either, `available()` is false and the page shows no button. A page
// started from the home screen (the web app manifest) already fills the
// screen, so it shows none either.

const root = () => document.documentElement;

export function available() {
  const r = root();
  return !!(r.requestFullscreen || r.webkitRequestFullscreen) && !standalone();
}

export function active() {
  return !!(document.fullscreenElement || document.webkitFullscreenElement);
}

// Started from the home screen, with no browser around it.
export function standalone() {
  return window.matchMedia?.('(display-mode: standalone)').matches || window.matchMedia?.('(display-mode: fullscreen)').matches || navigator.standalone === true;
}

// Into fullscreen or out of it. The browser may refuse (no user gesture, a
// policy); a refusal leaves the page as it was and is not an error.
export function toggle() {
  const r = root();
  try {
    if (active()) {
      const out = document.exitFullscreen?.() ?? document.webkitExitFullscreen?.();
      out?.catch?.(() => {});
    } else {
      const go = r.requestFullscreen?.() ?? r.webkitRequestFullscreen?.();
      go?.catch?.(() => {});
    }
  } catch { /* refused */ }
}

export function onChange(fn) {
  document.addEventListener('fullscreenchange', fn);
  document.addEventListener('webkitfullscreenchange', fn);
}
