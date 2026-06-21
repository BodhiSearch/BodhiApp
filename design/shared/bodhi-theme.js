/* ═══════════════════════════════════════════════════════════════
   Bodhi theme controller — bodhi-theme.js
   Load SYNCHRONOUSLY in <head> (right after colors_and_type.css) so
   the saved theme is applied before first paint — no flash.

   Stored value (localStorage 'bodhi.theme'): 'light' | 'dark' | 'system'.
   Default when nothing is stored: 'light' (matches the app's base theme).
   'system' resolves live against prefers-color-scheme.

   API (window.bodhiTheme):
     .mode       → current stored mode ('light' | 'dark' | 'system')
     .resolved   → the theme actually applied ('light' | 'dark')
     .set(mode)  → persist + apply + notify
     .toggle()   → flip resolved light↔dark (sets an explicit mode)
     .subscribe(fn) → fn(mode, resolved) on change; returns unsubscribe
═══════════════════════════════════════════════════════════════ */
(function () {
  var KEY = 'bodhi.theme';
  var mq = window.matchMedia('(prefers-color-scheme: dark)');
  var listeners = new Set();

  function stored() {
    try { return localStorage.getItem(KEY); } catch (e) { return null; }
  }
  function mode() {
    var m = stored();
    return (m === 'light' || m === 'dark' || m === 'system') ? m : 'light';
  }
  function resolved(m) {
    m = m || mode();
    if (m === 'system') return mq.matches ? 'dark' : 'light';
    return m;
  }
  function apply() {
    document.documentElement.setAttribute('data-theme', resolved());
  }
  function notify() {
    var m = mode(), r = resolved(m);
    listeners.forEach(function (fn) { try { fn(m, r); } catch (e) {} });
  }
  function set(m) {
    if (m !== 'light' && m !== 'dark' && m !== 'system') return;
    try { localStorage.setItem(KEY, m); } catch (e) {}
    apply();
    notify();
  }
  function toggle() {
    set(resolved() === 'dark' ? 'light' : 'dark');
  }

  var onMq = function () { if (mode() === 'system') { apply(); notify(); } };
  if (mq.addEventListener) mq.addEventListener('change', onMq);
  else if (mq.addListener) mq.addListener(onMq);

  // Apply immediately (runs in <head>, before body paints).
  apply();

  // Stay in sync across tabs / page loads sharing localStorage.
  window.addEventListener('storage', function (e) {
    if (e.key === KEY) { apply(); notify(); }
  });

  window.bodhiTheme = {
    get mode() { return mode(); },
    get resolved() { return resolved(); },
    set: set,
    toggle: toggle,
    subscribe: function (fn) { listeners.add(fn); return function () { listeners.delete(fn); }; },
  };
})();
