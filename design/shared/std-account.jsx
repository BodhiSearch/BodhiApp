/* ═══════════════════════════════════════════════════════════════
   Bodhi — <StdAccountMenu /> : top-bar account chip
   shared/std-account.jsx

   A DROP-IN header control: shows who you're acting as, switches
   organization, and signs in as a different user. Built for the
   standalone pages (.std-topbar) but has no dependency on them —
   it can sit in any header, including the app shell's.

   ── Integrating in another page ───────────────────────────────
     <link rel="stylesheet" href="shared/std-account.css">   (after colors_and_type.css)
     <script type="text/babel" src="shared/std-account.jsx"></script>
   Then, anywhere in a React tree:
     <StdAccountMenu
       user={{ initials:'YO', name:'Yogesh', email:'yogesh@email.com', currentTenantId:'acme' }}
       tenants={[{ id, name, role, plan }, …]}   // optional; falls back to
                                                 // window.BSB_TENANTS → SHELL_TENANTS → built-in demo list
       onSwitchOrg={org => …}     // fires AFTER the switch; org = {id,name,role,plan}
       onSwitchUser={() => …}     // host decides where login goes
       switchUserNote="You'll come back to this request"   // omit for no sub-line
       align="right"              // "right" (default) | "left"
     />

   Dependencies: React + lucide only (both already global on these
   pages). Uses its own icon helper and its own CSS namespace, so it
   works with or without bodhi-app-shell.css loaded.
   State: the active org is held internally but reported upward via
   onSwitchOrg — lift it if the page's body depends on org context
   (App-Access-Review does: the consent card's role pill follows it).
═══════════════════════════════════════════════════════════════ */

/* local lucide helper — avoids depending on shell-core.jsx's ShellIcon */
function StdAmIcon({ name, size = 14 }) {
  const ref = React.useRef(null);
  React.useEffect(() => {
    if (!ref.current) return;
    ref.current.innerHTML = '';
    const el = document.createElement('i');
    el.setAttribute('data-lucide', name);
    ref.current.appendChild(el);
    if (window.lucide) window.lucide.createIcons({ nodes: [el] });
  }, [name]);
  return <span ref={ref} style={{ display: 'inline-flex', width: size, height: size, alignItems: 'center', justifyContent: 'center', flexShrink: 0 }} />;
}

const STD_ACCOUNT_TENANTS = [
  { id: 'acme',      name: 'Acme Corp',         role: 'Admin',      plan: 'Enterprise' },
  { id: 'northwind', name: 'Northwind Trading', role: 'Power User', plan: 'Team' },
  { id: 'initech',   name: 'Initech Labs',      role: 'User',       plan: 'Free' },
];

function StdAccountMenu({
  user = {}, tenants, onSwitchOrg, onSwitchUser, onAddOrg,
  switchUserNote = "You'll come back to this request", align = 'right',
}) {
  const u = {
    initials: user.initials || 'YO',
    name: user.name || 'Yogesh',
    email: user.email || 'yogesh@email.com',
  };
  const orgs = tenants
    || (typeof window !== 'undefined' && window.BSB_TENANTS)
    || (typeof SHELL_TENANTS !== 'undefined' ? SHELL_TENANTS : null)
    || STD_ACCOUNT_TENANTS;

  const [open, setOpen] = React.useState(false);
  const [flyout, setFlyout] = React.useState(false);
  const [curId, setCurId] = React.useState(user.currentTenantId || (orgs[0] && orgs[0].id));
  const [toast, setToast] = React.useState(null);
  const wrapRef = React.useRef(null);
  const toastTimer = React.useRef(null);
  const flyoutTimer = React.useRef(null);
  const cur = orgs.find(t => t.id === curId) || orgs[0] || { name: '—', role: '', plan: '' };

  const openFlyout = () => { clearTimeout(flyoutTimer.current); setFlyout(true); };
  const scheduleCloseFlyout = () => {
    clearTimeout(flyoutTimer.current);
    flyoutTimer.current = setTimeout(() => setFlyout(false), 260);
  };
  React.useEffect(() => { if (!open) setFlyout(false); }, [open]);
  React.useEffect(() => {
    if (!open) return;
    const away = e => { if (!wrapRef.current || !wrapRef.current.contains(e.target)) setOpen(false); };
    const key = e => { if (e.key === 'Escape') setOpen(false); };
    document.addEventListener('mousedown', away);
    document.addEventListener('keydown', key);
    return () => { document.removeEventListener('mousedown', away); document.removeEventListener('keydown', key); };
  }, [open]);
  React.useEffect(() => () => { clearTimeout(toastTimer.current); clearTimeout(flyoutTimer.current); }, []);

  const fire = t => {
    clearTimeout(toastTimer.current);
    setToast(t);
    toastTimer.current = setTimeout(() => setToast(null), 2800);
  };
  const switchOrg = t => {
    setCurId(t.id); setOpen(false);
    fire({ kind: 'org', name: t.name, role: t.role });
    onSwitchOrg && onSwitchOrg(t);
  };
  const addOrg = () => { setOpen(false); fire({ kind: 'add' }); onAddOrg && onAddOrg(); };
  const switchUser = () => { setOpen(false); fire({ kind: 'user' }); onSwitchUser && onSwitchUser(); };

  const menuStyle = align === 'left' ? { right: 'auto', left: 0 } : null;

  return (
    <div className="std-account" ref={wrapRef}>
      <button className={'std-account-chip' + (open ? ' on' : '')} onClick={() => setOpen(o => !o)}
              aria-haspopup="menu" aria-expanded={open}>
        <span className="std-account-av">{u.initials}</span>
        <span className="std-account-meta">
          <span className="std-account-name">{u.name}</span>
          <span className="std-account-org"><StdAmIcon name="building-2" size={11} />{cur.name}</span>
        </span>
        <span className="std-account-chev"><StdAmIcon name="chevrons-up-down" size={14} /></span>
      </button>

      {open && (
        <div className="std-account-menu" style={menuStyle} role="menu">
          <div className="std-am-head">
            <span className="std-account-av">{u.initials}</span>
            <span className="std-am-id">
              <span className="std-am-name">{u.name}</span>
              <span className="std-am-email">{u.email}</span>
            </span>
          </div>

          <div className="std-am-org">
            <span className="std-am-org-lbl">Current organization</span>
            <div className="std-am-switch" onMouseLeave={scheduleCloseFlyout}>
              <button className={'std-am-org-row' + (flyout ? ' on' : '')}
                      onClick={() => setFlyout(f => !f)} onMouseEnter={openFlyout}>
                <span className="std-am-org-ico"><StdAmIcon name="building-2" size={13} /></span>
                <span className="std-am-org-body">
                  <span className="std-am-org-name">{cur.name}</span>
                  <span className="std-am-org-sub">{cur.role} · {cur.plan}</span>
                </span>
                <span className="std-am-org-chev"><StdAmIcon name="chevron-right" size={14} /></span>
              </button>
              {flyout && (
                <div className="std-am-flyout" onMouseEnter={openFlyout}>
                  <div className="std-am-flyout-title">Switch organization</div>
                  {orgs.map(t => (
                    <button key={t.id} className={'std-am-tenant' + (t.id === curId ? ' on' : '')}
                            onClick={() => switchOrg(t)}>
                      <span className="std-am-tenant-mark">
                        {t.id === curId ? <StdAmIcon name="check" size={13} /> : <span className="std-am-tenant-dot" />}
                      </span>
                      <span className="std-am-tenant-body">
                        <span className="std-am-tenant-name">{t.name}</span>
                        <span className="std-am-tenant-sub">{t.role} · {t.plan}</span>
                      </span>
                    </button>
                  ))}
                  <div className="std-am-div" />
                  <button className="std-am-tenant std-am-addorg" onClick={addOrg}>
                    <span className="std-am-tenant-mark"><StdAmIcon name="plus" size={13} /></span>
                    <span className="std-am-tenant-body">
                      <span className="std-am-tenant-name">Add organization</span>
                      <span className="std-am-tenant-sub">Join or create a new one</span>
                    </span>
                  </button>
                </div>
              )}
            </div>
          </div>

          <div className="std-am-items">
            <button className="std-am-item" onClick={switchUser}>
              <StdAmIcon name="user-round-cog" size={14} />
              <span className="std-am-label">
                Sign in as a different user
                {switchUserNote && <span className="std-am-sub">{switchUserNote}</span>}
              </span>
            </button>
          </div>
        </div>
      )}

      {toast && (
        <div className="std-am-toast">
          <StdAmIcon name={toast.kind === 'org' ? 'check-circle-2' : toast.kind === 'add' ? 'plus' : 'user-round-cog'} size={14} />
          {toast.kind === 'org'
            ? <span>Switched to <strong>{toast.name}</strong> · {toast.role}</span>
            : toast.kind === 'add'
              ? <span>Add organization — join or create a new one</span>
              : <span>Signing out — you'll return to this request after login</span>}
        </div>
      )}
    </div>
  );
}

Object.assign(window, { StdAccountMenu, StdAmIcon, STD_ACCOUNT_TENANTS });
