/* ═══════════════════════════════════════════════════════════════
   Bodhi App Shell — USER (theme toggle + user menu + footer)
   shared/shell-user.jsx   (load after shell-core.jsx)

   The sidebar footer cluster: a light/dark/system theme switch, the
   user chip, and its popover (org switcher + logout) with toasts.
   Theme state comes from useTheme (shell-core); tenants from
   SHELL_TENANTS (shell-core), overridable per-page via user.tenants.
═══════════════════════════════════════════════════════════════ */

/* Quick light/dark switch that lives just above the user chip in the
   sidebar footer. Expanded → labelled sun/moon segmented pill.
   Collapsed icon-rail → a single button that flips the theme. */
function ShellThemeToggle({ collapsed }) {
  const { mode, setMode } = useTheme();
  if (collapsed) {
    const i = THEME_OPTS.findIndex(o => o.id === mode);
    const cur = THEME_OPTS[i < 0 ? 0 : i];
    const next = THEME_OPTS[(i + 1) % THEME_OPTS.length];
    return (
      <button className="shell-railbtn shell-tip shell-theme-rail"
              aria-label={'Theme: ' + cur.label}
              data-tip={'Theme: ' + cur.label + ' · switch to ' + next.label}
              onClick={() => setMode(next.id)}>
        <ShellIcon name={cur.icon} size={18} />
      </button>
    );
  }
  return (
    <div className="shell-theme-seg" role="group" aria-label="Theme">
      <span className="shell-theme-thumb" data-side={mode} />
      {THEME_OPTS.map(o => (
        <button key={o.id}
                className={'shell-theme-opt' + (mode === o.id ? ' on' : '')}
                aria-label={o.label} title={o.label} aria-pressed={mode === o.id}
                onClick={() => setMode(o.id)}>
          <ShellIcon name={o.icon} size={14} />
        </button>
      ))}
    </div>
  );
}

/* ── User menu popover (fixed; opens UP from the chip, RIGHT when collapsed) ── */
function UserMenuPop({ open, anchorRef, collapsed, onClose, children }) {
  const popRef = React.useRef(null);
  const [pos, setPos] = React.useState(null);

  React.useLayoutEffect(() => {
    if (!open || !anchorRef.current) { setPos(null); return; }
    const a = anchorRef.current.getBoundingClientRect();
    const ph = popRef.current ? popRef.current.offsetHeight : 280;
    const pw = popRef.current ? popRef.current.offsetWidth : 256;
    let top, left;
    if (collapsed) {
      left = a.right + 10;
      top = a.bottom - ph;
    } else {
      left = a.left;
      top = a.top - ph - 8;
    }
    if (top < 8) top = 8;
    if (left + pw > window.innerWidth - 8) left = Math.max(8, window.innerWidth - 8 - pw);
    setPos({ top, left });
  }, [open, collapsed]);

  React.useEffect(() => {
    if (!open) return;
    const h = () => onClose();
    const k = e => { if (e.key === 'Escape') onClose(); };
    document.addEventListener('click', h);
    document.addEventListener('keydown', k);
    return () => { document.removeEventListener('click', h); document.removeEventListener('keydown', k); };
  }, [open]);

  if (!open) return null;
  return (
    <div ref={popRef} className="shell-usermenu" style={{ top: pos ? pos.top : -9999, left: pos ? pos.left : -9999 }}
         onClick={e => e.stopPropagation()}>
      {children}
    </div>
  );
}

function ShellFooter({ user, collapsed }) {
  const u = {
    initials: user.initials || 'YO',
    name: user.name || 'Yogesh',
    email: user.email || 'yogesh@email.com',
    role: user.role || 'Admin',
  };
  const multi = !!user.multiTenant;
  const tenants = user.tenants || SHELL_TENANTS;

  const [open, setOpen] = React.useState(false);
  const [flyout, setFlyout] = React.useState(false);
  const [curId, setCurId] = React.useState(user.currentTenantId || tenants[0].id);
  const [toast, setToast] = React.useState(null);
  const anchorRef = React.useRef(null);
  const toastTimer = React.useRef(null);

  const cur = tenants.find(t => t.id === curId) || tenants[0];
  const flyoutTimer = React.useRef(null);
  const openFlyout = () => { clearTimeout(flyoutTimer.current); setFlyout(true); };
  const scheduleCloseFlyout = () => {
    clearTimeout(flyoutTimer.current);
    flyoutTimer.current = setTimeout(() => setFlyout(false), 260);
  };
  // In multi-tenant mode the role is contextual to the active org.
  const activeRole = multi ? cur.role : u.role;

  // Collapsing the menu when multi-tenant is toggled off keeps state clean.
  React.useEffect(() => { if (!multi) setFlyout(false); }, [multi]);
  React.useEffect(() => { if (!open) setFlyout(false); }, [open]);
  // Disable column resize handles while the menu is open — they sit at the
  // sidebar edge and otherwise conflict with the popup/flyout.
  React.useEffect(() => {
    document.body.classList.toggle('shell-menu-open', open);
    return () => document.body.classList.remove('shell-menu-open');
  }, [open]);

  function switchTenant(t) {
    setCurId(t.id);
    setOpen(false);
    clearTimeout(toastTimer.current);
    setToast({ name: t.name, role: t.role });
    toastTimer.current = setTimeout(() => setToast(null), 2600);
  }
  function logout() {
    setOpen(false);
    clearTimeout(toastTimer.current);
    setToast({ logout: true });
    toastTimer.current = setTimeout(() => setToast(null), 2600);
  }
  function addOrg() {
    setOpen(false);
    clearTimeout(toastTimer.current);
    setToast({ add: true });
    toastTimer.current = setTimeout(() => setToast(null), 2600);
  }

  const toggle = e => { e.stopPropagation(); setOpen(o => !o); };

  const chip = collapsed ? (
    <button ref={anchorRef} className={'shell-avatar shell-tip shell-userbtn-collapsed' + (open ? ' on' : '')}
            data-tip={u.name + (multi ? ' · ' + cur.name : ' · ' + activeRole)} onClick={toggle}>
      {u.initials}
    </button>
  ) : (
    <button ref={anchorRef} className={'shell-userbtn' + (open ? ' on' : '')} onClick={toggle}>
      <span className="shell-avatar">{u.initials}</span>
      <span className="shell-userbtn-meta">
        <span className="shell-foot-name">{u.name}</span>
        <span className="shell-foot-role">
          {multi && <ShellIcon name="building-2" size={11} />}
          {multi ? cur.name : activeRole}
        </span>
      </span>
      <span className="shell-userbtn-chev"><ShellIcon name={multi ? 'chevrons-up-down' : 'chevron-up'} size={14} /></span>
    </button>
  );

  return (
    <>
      <ShellThemeToggle collapsed={collapsed} />
      {chip}
      <UserMenuPop open={open} anchorRef={anchorRef} collapsed={collapsed} onClose={() => setOpen(false)}>
        <div className="shell-um-head">
          <span className="shell-avatar shell-um-avatar">{u.initials}</span>
          <span className="shell-um-id">
            <span className="shell-um-name">{u.name}</span>
            <span className="shell-um-email">{u.email}</span>
          </span>
        </div>

        {multi && (
          <div className="shell-um-org">
            <span className="shell-um-org-lbl">Current organization</span>
            <div className="shell-um-switch" onMouseLeave={scheduleCloseFlyout}>
              <button className={'shell-um-org-row' + (flyout ? ' on' : '')}
                      onClick={e => { e.stopPropagation(); setFlyout(f => !f); }}
                      onMouseEnter={openFlyout}>
                <span className="shell-um-org-ico"><ShellIcon name="building-2" size={13} /></span>
                <span className="shell-um-org-body">
                  <span className="shell-um-org-name">{cur.name}</span>
                  <span className="shell-um-org-sub">{cur.role} · {cur.plan}</span>
                </span>
                <span className="shell-um-org-chev"><ShellIcon name="chevron-right" size={14} /></span>
              </button>
              {flyout && (
                <div className="shell-um-flyout" onMouseEnter={openFlyout} onClick={e => e.stopPropagation()}>
                  <div className="shell-pop-title">Switch organization</div>
                  {tenants.map(t => (
                    <button key={t.id} className={'shell-um-tenant' + (t.id === curId ? ' on' : '')}
                            onClick={() => switchTenant(t)}>
                      <span className="shell-um-tenant-mark">
                        {t.id === curId ? <ShellIcon name="check" size={13} /> : <span className="shell-um-tenant-dot" />}
                      </span>
                      <span className="shell-um-tenant-body">
                        <span className="shell-um-tenant-name">{t.name}</span>
                        <span className="shell-um-tenant-sub">{t.role} · {t.plan}</span>
                      </span>
                    </button>
                  ))}
                  <div className="shell-um-flyout-div" />
                  <button className="shell-um-tenant shell-um-addorg" onClick={addOrg}>
                    <span className="shell-um-tenant-mark"><ShellIcon name="plus" size={13} /></span>
                    <span className="shell-um-tenant-body">
                      <span className="shell-um-tenant-name">Add organization</span>
                      <span className="shell-um-tenant-sub">Join or create a new tenant</span>
                    </span>
                  </button>
                </div>
              )}
            </div>
          </div>
        )}

        <div className={'shell-um-items' + (multi ? '' : ' shell-um-items-flush')}>
          <button className="shell-um-item shell-um-logout" onClick={logout}>
            <ShellIcon name="log-out" size={14} />
            <span className="shell-um-label">Log out</span>
          </button>
        </div>
      </UserMenuPop>

      {toast && (
        <div className="shell-usertoast">
          <ShellIcon name={toast.logout ? 'log-out' : toast.add ? 'plus' : 'check-circle-2'} size={14} />
          {toast.logout
            ? <span>Logged out</span>
            : toast.add
              ? <span>Add organization — connect or create a new tenant</span>
              : <span>Switched to <strong>{toast.name}</strong> · {toast.role}</span>}
        </div>
      )}
    </>
  );
}

Object.assign(window, { ShellThemeToggle, UserMenuPop, ShellFooter });
