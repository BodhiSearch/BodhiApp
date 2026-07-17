// Runs before first paint to avoid FOUC. Kept as an external asset because the
// server's CSP omits 'unsafe-inline' for script-src.
(function () {
  var theme = window.localStorage.getItem('bodhi-ui-theme');
  if (!theme) {
    theme = window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
  }
  document.documentElement.classList.add(theme);
})();
