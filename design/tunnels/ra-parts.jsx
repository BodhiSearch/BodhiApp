/* ═══════════════════════════════════════════════════
   REMOTE ACCESS — shared parts
   tunnels/ra-parts.jsx  (load after shell-app.jsx)
   Components used by Remote-Access.html. Everything is
   exported to window at the bottom (Babel scripts don't share scope).
═══════════════════════════════════════════════════ */
const RA_Ic = ShellIcon;
const RA_HOST = 'bodhi.acme-labs.com';
const RA_ZONE = 'acme-labs.com';


const RA_BC = [
  { label: 'Bodhi', href: 'Chat.html' },
  { label: 'Settings', href: 'Settings.html' },
  { label: 'Remote Access', current: true },
];

/* ── Status pill ── */
function RAPill({ tone = 'idle', label, icon, spin }) {
  const ic = icon || { idle: 'circle-dashed', checking: 'refresh-cw', ok: 'check', live: 'radio', attn: 'alert-triangle', fail: 'x' }[tone];
  return (
    <span className={`ra-pill t-${tone}`}>
      <span className={spin ? 'ra-spin' : undefined} style={{ display: 'inline-flex' }}><RA_Ic name={ic} size={11} /></span>
      {label}
    </span>
  );
}

/* ── Note / callout ── */
function RANote({ tone = 'info', icon, title, children }) {
  const ic = icon || { info: 'info', warn: 'shield-alert', ok: 'check-circle-2', fail: 'alert-octagon', checking: 'refresh-cw' }[tone];
  return (
    <div className={`ra-note t-${tone}`}>
      <RA_Ic name={ic} size={15} />
      <div>{title && <span className="ra-note-t">{title}</span>}{children}</div>
    </div>
  );
}

/* ── One rung of the setup ladder ──
   A finished step folds to a single line so focus stays on the current
   one. `defaultFolded` folds it as soon as later steps are in play;
   `lockFolded` keeps it shut (the tunnel is live and can't be re-cut).
   `action` renders on the right of the folded line. ── */
function RAStep({ n, tone = 'idle', title, pill, spin, status, children, collapsible, summary, defaultFolded, lockFolded, action }) {
  const done = tone === 'ok';
  const [open, setOpen] = React.useState(false);
  React.useEffect(() => { setOpen(false); }, [defaultFolded, lockFolded]);
  const folded = collapsible && done && (lockFolded || (defaultFolded && !open));
  if (folded) {
    return (
      <div className="ra-step t-ok is-folded">
        <div className="ra-node"><RA_Ic name="check" size={14} /></div>
        <button className="ra-fold" onClick={() => !lockFolded && setOpen(true)}>
          <span className="ra-step-t">{title}</span>
          {summary && <span className="ra-fold-s">{summary}</span>}
          <span className="ra-fold-act">{action}</span>
        </button>
      </div>);

  }
  return (
    <div className={`ra-step t-${tone}${tone === 'idle' ? ' is-muted' : ''}`}>
      <div className="ra-node">{done ? <RA_Ic name="check" size={14} /> : n}</div>
      <div>
        <div className="ra-step-head">
          <span className="ra-step-t">{title}</span>
          {pill && <RAPill tone={tone} label={pill} spin={spin} />}
          {collapsible && done && (defaultFolded || open) &&
          <button className="ra-fold-x" onClick={() => setOpen(false)}><RA_Ic name="chevron-up" size={13} /> Collapse</button>}
        </div>
        {status && <div className="ra-step-s">{status}</div>}
        {children && <div className="ra-step-body">{children}</div>}
      </div>
    </div>
  );
}

/* ── A shell command, copyable ── */
function RACode({ children }) {
  const [done, setDone] = React.useState(false);
  const copy = () => { setDone(true); setTimeout(() => setDone(false), 1600); };
  return (
    <div className="ra-code">
      <code>{children}</code>
      <button className="ra-code-copy" onClick={copy} aria-label="Copy command">
        <RA_Ic name={done ? 'check' : 'copy'} size={13} /> {done ? 'Copied' : 'Copy'}
      </button>
    </div>
  );
}

/* ── Ambient exposure framing — appears wherever enabling is possible ── */
function RAExposure({ host, onMore }) {
  return (
    <RANote tone="warn" title="This opens the whole instance to the internet">
      The web UI and every API surface become reachable by anyone at <strong>https://{host || RA_HOST}</strong>.
      Sign-in is still required past the login page. Instance-wide, not just for you.
      {onMore && <> <a href="#faq-exposure" onClick={(e) => { e.preventDefault(); onMore(); }}>What exactly is exposed</a></>}
    </RANote>
  );
}

/* ═══════════ Sidebar: state switcher (review aid) ═══════════ */
function RASidebar({ states, active, onPick }) {
  const { collapsed } = useShell();
  if (collapsed) {
    return (
      <a href="Remote-Access.html" className="shell-railbtn shell-tip on" data-tip="Remote access">
        <RA_Ic name="waypoints" size={18} />
      </a>);

  }
  return (
    <div className="ra-side">
      <div className="ra-side-states">
        <div className="snav-label">States</div>
        {states.map(s => (
          <button key={s.id} className={`ra-state${s.id === active ? ' active' : ''}`} onClick={() => onPick(s.id)}>
            <span className={`ra-dot t-${s.tone}`} />
            <span className="ra-state-l">{s.label}</span>
            <span className="ra-state-n">{s.n}</span>
          </button>
        ))}
      </div>
      <div className="ra-side-foot">
        This state list is a review aid for the mock. In the app, the page shows whichever state is true.
      </div>
    </div>
  );
}

/* ── Page header ── */
function RAPageHeader({ note }) {
  return (
    <div className="page-header">
      <div className="page-header-text">
        <div className="page-title">Remote Access</div>
        <div className="page-subtitle">
          Reach this instance from outside your network through a Cloudflare tunnel on your own domain.
          {note ? ' ' + note : ''}
        </div>
      </div>
    </div>
  );
}

Object.assign(window, {
  RA_Ic, RA_HOST, RA_ZONE, RA_BC,
  RAPill, RANote, RAStep, RAExposure, RACode,
  RASidebar, RAPageHeader,
});
