/* ═══════════════════════════════════════════════════════════════
   Bodhi Auth — shared page behaviors
   bodhi-auth.js   ·   load at end of <body> on every auth page

   Pure UX niceties — all client-side, safe to keep or drop on the
   Keycloak port:
     • password show / hide toggle
     • password-strength meter (register / update-password)
     • in-page light / dark theme toggle (uses window.bodhiTheme)
     • lucide icon init
═══════════════════════════════════════════════════════════════ */
(function () {
  function initIcons() { if (window.lucide) lucide.createIcons(); }

  /* ── Password show / hide ──────────────────────────────────── */
  document.addEventListener('click', function (e) {
    var btn = e.target.closest('[data-pw-toggle]');
    if (!btn) return;
    var input = btn.parentElement.querySelector('input');
    var reveal = input.type === 'password';
    input.type = reveal ? 'text' : 'password';
    btn.querySelector('i').setAttribute('data-lucide', reveal ? 'eye-off' : 'eye');
    initIcons();
  });

  /* ── Password-strength meter ───────────────────────────────── */
  function score(v) {
    var s = 0;
    if (v.length >= 8) s++;
    if (/[A-Z]/.test(v) && /[a-z]/.test(v)) s++;
    if (/\d/.test(v)) s++;
    if (/[^A-Za-z0-9]/.test(v)) s++;
    return Math.min(s, 4);
  }
  document.querySelectorAll('[data-meter]').forEach(function (input) {
    var wrap = input.closest('.ba-group');
    var segs = wrap.querySelectorAll('[data-meter-bar] .ba-meter-seg');
    var label = wrap.querySelector('[data-meter-label]');
    var base = label ? label.textContent : '';
    input.addEventListener('input', function () {
      var sc = score(input.value);
      var cls = sc <= 1 ? 'on-weak' : sc <= 3 ? 'on-medium' : 'on-strong';
      segs.forEach(function (seg, i) { seg.className = 'ba-meter-seg' + (i < sc ? ' ' + cls : ''); });
      if (label) {
        label.textContent = !input.value ? base
          : sc <= 1 ? 'Weak password'
          : sc <= 3 ? 'Getting stronger…'
          : 'Strong password';
      }
    });
  });

  /* ── In-page theme toggle ──────────────────────────────────── */
  var themeBtn = document.getElementById('ba-theme');
  if (themeBtn && window.bodhiTheme) {
    var sync = function () {
      var dark = window.bodhiTheme.resolved === 'dark';
      themeBtn.querySelector('i').setAttribute('data-lucide', dark ? 'sun' : 'moon');
      var label = themeBtn.querySelector('.ba-tt-label');
      if (label) label.textContent = dark ? 'Light' : 'Dark';
      initIcons();
    };
    themeBtn.addEventListener('click', function () { window.bodhiTheme.toggle(); sync(); });
    window.bodhiTheme.subscribe(sync);
    sync();
  }

  initIcons();
})();
