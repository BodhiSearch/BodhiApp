/* ═══════════════════════════════════════════════════
   Bodhi Setup Flow — shared shell
   setup-shell.jsx   ·   load AFTER tweaks-panel.jsx, before page apps

   Owns the wizard chrome: lotus logo, top stepper, step
   counter, and the on-page theme toggle (shared bodhi-theme.js).
   Each page renders <SetupShell current={n} wide?>…</SetupShell>.
   Exports SetupShell + Icon to window.
═══════════════════════════════════════════════════ */

const SU_STEPS = [
  { label: 'Get Started',   href: 'setup-1-get-started.html' },
  { label: 'Login & Setup', href: 'setup-2-login.html' },
  { label: 'Local Models',  href: 'setup-3-local-models.html' },
  { label: 'API Models',    href: 'setup-4-api-models.html' },
  { label: 'Extension',     href: 'setup-5-extension.html' },
  { label: 'All Done',      href: 'setup-6-complete.html' },
];

/* Lucide icon helper — paints once per name change. */
function Icon({ name, size = 16, strokeWidth = 2, style = {} }) {
  const ref = React.useRef(null);
  React.useEffect(() => {
    if (!ref.current || !window.lucide) return;
    ref.current.innerHTML = '';
    const el = document.createElement('i');
    el.setAttribute('data-lucide', name);
    ref.current.appendChild(el);
    window.lucide.createIcons({
      nodes: [el],
      attrs: { width: size, height: size, 'stroke-width': strokeWidth },
    });
  }, [name, size, strokeWidth]);
  return <span ref={ref} style={{ display: 'inline-flex', ...style }} />;
}

function SetupStepper({ current }) {
  return (
    <nav className="su-stepper" aria-label="Setup progress">
      {SU_STEPS.map((s, i) => {
        const done = i < current;
        const cur = i === current;
        const cls = `su-step${done ? ' is-done' : ''}${cur ? ' is-current' : ''}`;
        return (
          <React.Fragment key={s.label}>
            {i > 0 && <span className={`su-conn${i <= current ? ' is-filled' : ''}`} />}
            <a className={cls} href={s.href} aria-current={cur ? 'step' : undefined}>
              <span className="su-step-node">
                {(done || i === SU_STEPS.length - 1)
                  ? <Icon name="check" size={16} strokeWidth={3} />
                  : i + 1}
              </span>
              <span className="su-step-label">{s.label}</span>
            </a>
          </React.Fragment>
        );
      })}
    </nav>
  );
}

function SetupShell({ current, wide = false, children }) {
  // Theme is owned app-wide by bodhi-theme.js — mirror it for the toggle icon.
  const [resolved, setResolved] = React.useState(
    () => (window.bodhiTheme ? window.bodhiTheme.resolved : 'light'));
  React.useEffect(() => {
    if (!window.bodhiTheme) return;
    setResolved(window.bodhiTheme.resolved);
    return window.bodhiTheme.subscribe((m, r) => setResolved(r));
  }, []);

  // repaint any icons rendered by page children after each commit
  React.useEffect(() => { if (window.lucide) window.lucide.createIcons(); });

  return (
    <div className="su-page">
      <button
        type="button"
        className="su-theme-toggle"
        onClick={() => window.bodhiTheme && window.bodhiTheme.toggle()}
        aria-label={resolved === 'dark' ? 'Switch to light theme' : 'Switch to dark theme'}
        title={resolved === 'dark' ? 'Switch to light theme' : 'Switch to dark theme'}
      >
        <Icon name={resolved === 'dark' ? 'sun' : 'moon'} size={18} />
      </button>

      <div className={`su-wrap${wide ? ' is-wide' : ''}`}>
        <img className="su-logo" src="assets/bodhi-logo-240.svg" alt="Bodhi" />
        <div className="su-rise">
          <SetupStepper current={current} />
        </div>
        {children}
      </div>
    </div>
  );
}

Object.assign(window, { SetupShell, Icon, SU_STEPS });
