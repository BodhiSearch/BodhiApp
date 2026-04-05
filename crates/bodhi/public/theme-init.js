// Theme initialization - must run before first paint to avoid FOUC
(function () {
  var theme = window.localStorage.getItem('bodhi-ui-theme');
  if (!theme) {
    theme = window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
  }
  document.documentElement.classList.add(theme);
})();
