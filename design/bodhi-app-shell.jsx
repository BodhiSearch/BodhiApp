/* ═══════════════════════════════════════════════════════════════
   Bodhi App Shell — canonical React layout system
   bodhi-app-shell.jsx

   Load order (after React + lucide, before your page script):
     <link rel="stylesheet" href="bodhi-app-shell.css">
     <script type="text/babel" src="bodhi-app-shell.jsx"></script>

   <AppShell
     section="models" subPage="my-models"   primary-nav highlight + sub-pages
     user={{ initials, name, role }}
     resizeKey="models"                      localStorage namespace for column widths

     sidebarWidth={240} railWidth={340} headerHeight={56} bandHeight={52}

     breadcrumb={[{label,href},{label,current}]}
     headerActions={<…/>}                    right of the header band (main)

     sidebar={<…/>}                          page body below the nav (filters, etc.)
     footer={<…/>}                           override user chip (optional)

     toolbar / sidebarToolbar / railToolbar   shared toolbar band cells (optional)
     banner={<…/>}                           main-column alert sub-band (optional)

     rail={<…/>} railHeader={<…/>}           right panel (optional → 3rd column)
     railDefaultOpen={true}                  start with the rail showing (desktop)

     contentClass="narrow|wide|flush"
     mainScroll={true}  railScroll={true}    set false to manage your own scroll region
   >
     {main content}
   </AppShell>

   COLLAPSE  — the breadcrumb's leading toggle collapses the sidebar to a
   60px icon rail (desktop/tablet) or opens it as a drawer (mobile). Collapse
   state is intentionally NOT persisted.

   Building blocks for the `sidebar` slot (they read collapse from context and
   render an icon + popover when the rail is collapsed):
     <ShellModeSwitch value onChange options={[{id,label,sub,icon}]} />
     <ShellFilterGroup icon label chips={[{label,color,defaultOn}]} note clearable />

   Exports: AppShell, ShellNav, ShellIcon, ShellModeSwitch, ShellFilterGroup,
            useShell, ShellContext, SHELL_NAV
═══════════════════════════════════════════════════════════════ */

const SHELL_NAV = (typeof window !== 'undefined' && window.BSB_NAV) || [
  { id: 'chat', label: 'Chat', icon: 'message-circle', href: 'Bodhi Chat.html', subPages: [] },
  {
    id: 'models', label: 'Models', icon: 'cpu', href: 'Bodhi Models.html', badge: '14',
    subPages: [
      { id: 'my-models',          label: 'My Models',            icon: 'layers',      href: 'Bodhi Models.html' },
      { id: 'explore-local',      label: 'Explore · Local Models', icon: 'hard-drive', href: 'Bodhi Models Local.html' },
      { id: 'explore-api',        label: 'Explore · API Models',   icon: 'at-sign',    href: 'Bodhi Models API.html' },
      { id: 'new-local-model',    label: 'New Local Model',    icon: 'plus-circle', href: 'Create New Local Model v4.html' },
      { id: 'new-api-model',      label: 'New API Model',      icon: 'plug-zap',    href: 'Create API Model.html' },
      { id: 'new-fallback-model', label: 'New Model Router', icon: 'route',       href: 'Create Fallback Model.html' },
    ],
  },
  {
    id: 'mcp', label: 'MCP', icon: 'plug', href: 'Bodhi MCP Discover v2.html',
    subPages: [
      { id: 'discover',     label: 'All MCPs',     icon: 'compass',     href: 'Bodhi MCP Discover v2.html' },
      { id: 'new-mcp',      label: 'New Instance', icon: 'plus-circle', href: 'Bodhi MCP New Instance.html' },
    ],
  },
  {
    id: 'api-keys', label: 'Access Tokens', icon: 'key-round', href: 'API Tokens.html',
    subPages: [
      { id: 'api-tokens',          label: 'API Tokens',          icon: 'key-round',    href: 'API Tokens.html' },
      { id: 'new-token',           label: 'New API Token',       icon: 'plus-circle',  href: 'New App Token.html' },
      { id: 'app-tokens',          label: 'App Tokens',          icon: 'layout-grid',  href: 'App Tokens.html' },
    ],
  },
  {
    id: 'users', label: 'Users', icon: 'users', href: 'User Access Requests.html',
    subPages: [
      { id: 'user-access-requests', label: 'User Access Requests', icon: 'user-check', href: 'User Access Requests.html' },
      { id: 'all-users',            label: 'All Users',            icon: 'users',       href: 'Manage Users.html' },
    ],
  },
  {
    id: 'settings', label: 'Settings', icon: 'settings', href: 'Bodhi App Settings.html',
    subPages: [],
  },
];

const clamp = (v, a, b) => Math.max(a, Math.min(b, v));

/* ── Context ─────────────────────────────────────────────────── */
const ShellContext = React.createContext({
  collapsed: false, isMobile: false, openRail: () => {}, closeRail: () => {},
  openPop: null, setOpenPop: () => {},
});
const useShell = () => React.useContext(ShellContext);

/* ── Lucide icon helper ─────────────────────────────────────── */
function ShellIcon({ name, size = 14, color, strokeWidth }) {
  const ref = React.useRef(null);
  React.useEffect(() => {
    if (!ref.current) return;
    ref.current.innerHTML = '';
    const el = document.createElement('i');
    el.setAttribute('data-lucide', name);
    if (color) el.style.color = color;
    if (strokeWidth) el.setAttribute('stroke-width', strokeWidth);
    ref.current.appendChild(el);
    if (typeof lucide !== 'undefined') lucide.createIcons({ nodes: [el] });
  }, [name, color, strokeWidth]);
  return <span ref={ref} style={{
    display: 'inline-flex', width: size, height: size,
    alignItems: 'center', justifyContent: 'center', flexShrink: 0,
  }} />;
}

/* ── Global tooltip (fixed-position; escapes sidebar overflow) ──
   One listener delegates over `.shell-tip` elements only — i.e. the
   COLLAPSED icon-rail buttons (and the avatar). Expanded sidebar items
   already show their label inline, and popover/popup options carry no
   tooltip, so neither fires a hint. Flips left near the viewport edge. */
function GlobalTooltip() {
  const [tip, setTip] = React.useState(null);
  const ref = React.useRef(null);
  React.useEffect(() => {
    let cur = null, timer = null;
    const onOver = e => {
      const el = e.target.closest && e.target.closest('.shell-tip[data-tip]');
      if (el === cur) return;
      cur = el; clearTimeout(timer);
      if (!el) { setTip(null); return; }
      timer = setTimeout(() => {
        if (cur !== el) return;
        const r = el.getBoundingClientRect();
        setTip({ text: el.getAttribute('data-tip'), top: r.top + r.height / 2, aRight: r.right, aLeft: r.left });
      }, 150);
    };
    const onOut = e => {
      const el = e.target.closest && e.target.closest('.shell-tip[data-tip]');
      if (!el || el !== cur) return;
      if (e.relatedTarget && el.contains(e.relatedTarget)) return;
      cur = null; clearTimeout(timer); setTip(null);
    };
    document.addEventListener('mouseover', onOver, true);
    document.addEventListener('mouseout', onOut, true);
    return () => {
      document.removeEventListener('mouseover', onOver, true);
      document.removeEventListener('mouseout', onOut, true);
      clearTimeout(timer);
    };
  }, []);
  React.useLayoutEffect(() => {
    if (!tip || !ref.current) return;
    const w = ref.current.offsetWidth;
    let left = tip.aRight + 10;
    if (left + w > window.innerWidth - 8) left = Math.max(8, tip.aLeft - 10 - w);
    ref.current.style.left = left + 'px';
  }, [tip]);
  if (!tip || !tip.text) return null;
  return <div ref={ref} className="shell-tooltip" style={{ top: tip.top, left: tip.aRight + 10 }}>{tip.text}</div>;
}

/* ── Anchored popover (fixed-position, escapes overflow) ────── */
function AnchoredPopover({ open, anchorRef, onClose, children }) {
  const popRef = React.useRef(null);
  const [pos, setPos] = React.useState(null);

  React.useLayoutEffect(() => {
    if (!open || !anchorRef.current) { setPos(null); return; }
    const a = anchorRef.current.getBoundingClientRect();
    const ph = popRef.current ? popRef.current.offsetHeight : 260;
    let top = a.top;
    if (top + ph > window.innerHeight - 8) top = Math.max(8, window.innerHeight - 8 - ph);
    setPos({ top, left: a.right + 8 });
  }, [open]);

  React.useEffect(() => {
    if (!open) return;
    const h = () => onClose();
    const k = e => { if (e.key === 'Escape') onClose(); };
    document.addEventListener('click', h);
    document.addEventListener('keydown', k);
    return () => { document.removeEventListener('click', h); document.removeEventListener('keydown', k); };
  }, [open]);

  if (!open) return null;
  return ReactDOM.createPortal(
    <div ref={popRef} className="shell-pop"
         style={{ top: (pos ? pos.top : -9999), left: (pos ? pos.left : -9999) }}
         onClick={e => e.stopPropagation()}>
      {children}
    </div>,
    document.body
  );
}

/* ── Primary section nav (expanded dropdown OR collapsed icon rail) ── */
function ShellNav({ section = 'chat', subPage = null }) {
  const { collapsed, openPop, setOpenPop } = useShell();
  const open = openPop === 'nav';
  const anchorRef = React.useRef(null);
  const cur = SHELL_NAV.find(n => n.id === section) || SHELL_NAV[0];

  React.useEffect(() => {
    if (!open) return;
    const h = () => setOpenPop(null);
    document.addEventListener('click', h);
    return () => document.removeEventListener('click', h);
  }, [open]);

  const menuItems = SHELL_NAV.map(item => (
    <a key={item.id} href={item.href || '#'} className={'shell-nav-item' + (item.id === section ? ' on' : '')}>
      <ShellIcon name={item.icon} color={item.id === section ? '#DB456C' : 'currentColor'} />
      {item.label}
      {item.badge && <span className="shell-nav-badge">{item.badge}</span>}
    </a>
  ));

  if (collapsed) {
    return (
      <>
        <button ref={anchorRef} className={'shell-railbtn shell-tip on'} data-tip={cur.label + ' · switch section'}
                onClick={e => { e.stopPropagation(); setOpenPop(open ? null : 'nav'); }}>
          <ShellIcon name={cur.icon} size={18} />
        </button>
        <AnchoredPopover open={open} anchorRef={anchorRef} onClose={() => setOpenPop(null)}>
          <div className="shell-pop-title">Go to section</div>
          {menuItems}
        </AnchoredPopover>
        {cur.subPages && cur.subPages.length > 0 && <div className="shell-iconrail-div" />}
        {cur.subPages && cur.subPages.map(sp => (
          <a key={sp.id} href={sp.href || '#'}
             className={'shell-railbtn shell-tip' + (sp.id === subPage ? ' on' : '')} data-tip={sp.label}>
            <ShellIcon name={sp.icon || 'circle'} size={17} />
            {sp.badge && <span className="rb-badge">{sp.badge}</span>}
          </a>
        ))}
      </>
    );
  }

  return (
    <div className="shell-nav-block">
      <div className={'shell-nav' + (open ? ' open' : '')} onClick={e => e.stopPropagation()}>
        <button className="shell-nav-trigger" data-tip="Switch section" onClick={e => { e.stopPropagation(); setOpenPop(open ? null : 'nav'); }}>
          <span className="lead"><ShellIcon name={cur.icon} size={15} color="#DB456C" /></span>
          <span className="lbl">{cur.label}</span>
          <span className="chev"><ShellIcon name="chevron-down" /></span>
        </button>
        {open && <div className="shell-nav-menu">{menuItems}</div>}
      </div>
      {cur.subPages && cur.subPages.length > 0 && (
        <div className="shell-sub">
          {cur.subPages.map(sp => (
            <a key={sp.id} href={sp.href || '#'} data-tip={sp.label} className={'shell-sub-item' + (sp.id === subPage ? ' on' : '')}>
              <ShellIcon name={sp.icon || 'circle'} size={13} color={sp.id === subPage ? '#DB456C' : 'currentColor'} />
              {sp.label}
              {sp.badge && <span className="shell-sub-badge">{sp.badge}</span>}
            </a>
          ))}
        </div>
      )}
    </div>
  );
}

/* ── Mode switch (page control) ─────────────────────────────── */
function ShellModeSwitch({ value, onChange, options = [], label }) {
  const { collapsed } = useShell();
  if (collapsed) {
    return (
      <>
        {options.map(o => (
          <button key={o.id} className={'shell-railbtn shell-tip' + (o.id === value ? ' on' : '')}
                  data-tip={o.label} onClick={() => onChange(o.id)}>
            <ShellIcon name={o.icon || 'circle'} size={18} />
          </button>
        ))}
      </>
    );
  }
  return (
    <div>
      {label && <div className="shell-fg-label" style={{ paddingTop: 10 }}><span className="fg-name">{label}</span></div>}
      <div className="shell-modeswitch">
        {options.map(o => (
          <button key={o.id} data-tip={o.label + (o.sub ? ' — ' + o.sub : '')} className={'shell-mode-card' + (o.id === value ? ' on' : '')} onClick={() => onChange(o.id)}>
            <span className="shell-mode-ico"><ShellIcon name={o.icon || 'circle'} size={16} /></span>
            <span>
              <span className="shell-mode-label" style={{ display: 'block' }}>{o.label}</span>
              {o.sub && <span className="shell-mode-sub" style={{ display: 'block' }}>{o.sub}</span>}
            </span>
          </button>
        ))}
      </div>
    </div>
  );
}

/* ── Dual-handle range slider ───────────────────────────────── */
function ShellRangeSlider({ min = 0, max = 100, step = 1, unit = '', defaultMin, defaultMax }) {
  const [lo, setLo] = React.useState(defaultMin != null ? defaultMin : min);
  const [hi, setHi] = React.useState(defaultMax != null ? defaultMax : max);
  const pct = v => ((v - min) / (max - min)) * 100;
  const fmt = v => v + (v >= max ? '+' : '') + unit;

  const onLo = e => setLo(Math.min(Number(e.target.value), hi - step));
  const onHi = e => setHi(Math.max(Number(e.target.value), lo + step));

  return (
    <div className="shell-range">
      <div className="shell-range-vals">
        <span>{fmt(lo)}</span>
        <span>{fmt(hi)}</span>
      </div>
      <div className="shell-range-track">
        <div className="shell-range-rail" />
        <div className="shell-range-fill" style={{ left: pct(lo) + '%', right: (100 - pct(hi)) + '%' }} />
        <input type="range" min={min} max={max} step={step} value={lo} onChange={onLo}
               className="shell-range-input" style={{ zIndex: lo > max - step ? 5 : 3 }} aria-label="Minimum" />
        <input type="range" min={min} max={max} step={step} value={hi} onChange={onHi}
               className="shell-range-input" style={{ zIndex: 4 }} aria-label="Maximum" />
      </div>
    </div>
  );
}

/* ── Filter group (page control) ────────────────────────────── */
function ShellFilterGroup({ icon = 'filter', label, chips = [], note, clearable, range }) {
  const { collapsed, openPop, setOpenPop } = useShell();
  const popId = 'fg:' + label;
  const open = openPop === popId;
  const [sel, setSel] = React.useState(() => new Set(chips.filter(c => c.defaultOn).map(c => c.label)));
  const anchorRef = React.useRef(null);

  const toggle = lbl => setSel(prev => {
    const next = new Set(prev);
    next.has(lbl) ? next.delete(lbl) : next.add(lbl);
    return next;
  });
  const clear = () => setSel(new Set());

  const chipEls = chips.map(c => (
    <button key={c.label} data-tip={c.label}
            className={'shell-fc fc-' + (c.color || 'neutral') + (sel.has(c.label) ? ' on' : '')}
            onClick={() => toggle(c.label)}>{c.label}</button>
  ));

  if (collapsed) {
    return (
      <>
        <button ref={anchorRef} className={'shell-railbtn shell-tip' + (sel.size ? ' on' : '')} data-tip={label}
                onClick={e => { e.stopPropagation(); setOpenPop(open ? null : popId); }}>
          <ShellIcon name={icon} size={17} />
          {sel.size > 0 && <span className="rb-badge">{sel.size}</span>}
        </button>
        <AnchoredPopover open={open} anchorRef={anchorRef} onClose={() => setOpenPop(null)}>
          <div className="shell-pop-title">
            <span>{label}</span>
            {clearable && sel.size > 0 && <button className="fg-clear" onClick={clear}>Clear</button>}
          </div>
          {range ? <div className="shell-pop-chips"><ShellRangeSlider {...range} /></div>
                 : <div className="shell-pop-chips">{chipEls}</div>}
        </AnchoredPopover>
      </>
    );
  }

  return (
    <div className="shell-filtergroup">
      <div className="shell-fg-label">
        <span className="fg-ico"><ShellIcon name={icon} size={13} /></span>
        <span className="fg-name">{label}{note && <span className="fg-note"> {note}</span>}</span>
        {clearable && sel.size > 0 && <button className="fg-clear" onClick={clear}>Clear</button>}
      </div>
      {range ? <ShellRangeSlider {...range} /> : <div className="shell-fc-row">{chipEls}</div>}
    </div>
  );
}

/* ── Default brand + footer + breadcrumb ────────────────────── */
function ShellBrand({ collapsed }) {
  return (
    <a href="Bodhi Chat.html" style={{ display: 'flex', alignItems: 'center', gap: 10 }}>
      <img src="assets/bodhi-logo-60.svg" alt="Bodhi" onError={e => { e.target.style.display = 'none'; }} />
      {!collapsed && (
        <span>
          <span className="shell-brand-t" style={{ display: 'block' }}>Bodhi</span>
          <span className="shell-brand-s" style={{ display: 'block' }}>AI Gateway</span>
        </span>
      )}
    </a>
  );
}

/* ── Shared tenant/org list (override via window.BSB_TENANTS or user.tenants) ── */
const SHELL_TENANTS = (typeof window !== 'undefined' && window.BSB_TENANTS) || [
  { id: 'acme',    name: 'Acme Corp',     role: 'Admin',      plan: 'Enterprise' },
  { id: 'northwind', name: 'Northwind Trading', role: 'Power User', plan: 'Team' },
  { id: 'initech', name: 'Initech Labs',  role: 'User',       plan: 'Free' },
];

/* ── Theme (light/dark/system) ─────────────────────────────────
   Backed by window.bodhiTheme (bodhi-theme.js, loaded in <head>).
   Persists to localStorage + applies data-theme on <html>. */
const THEME_OPTS = [
  { id: 'light',  label: 'Light',  icon: 'sun' },
  { id: 'dark',   label: 'Dark',   icon: 'moon' },
  { id: 'system', label: 'System', icon: 'monitor-smartphone' },
];
function useTheme() {
  const read = () => (typeof window !== 'undefined' && window.bodhiTheme)
    ? { mode: window.bodhiTheme.mode, resolved: window.bodhiTheme.resolved }
    : { mode: 'light', resolved: 'light' };
  const [state, setState] = React.useState(read);
  React.useEffect(() => {
    if (!window.bodhiTheme) return;
    setState(read());
    return window.bodhiTheme.subscribe((mode, resolved) => setState({ mode, resolved }));
  }, []);
  return {
    mode: state.mode,
    resolved: state.resolved,
    setMode: m => window.bodhiTheme && window.bodhiTheme.set(m),
    toggle: () => window.bodhiTheme && window.bodhiTheme.toggle(),
  };
}

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

        <div className="shell-um-items">
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

function ShellBreadcrumb({ items }) {
  if (!items) return null;
  if (!Array.isArray(items)) return <div className="shell-bc">{items}</div>;
  return (
    <div className="shell-bc">
      {items.map((it, i) => (
        <React.Fragment key={i}>
          {i > 0 && <ShellIcon name="chevron-right" size={11} />}
          {it.current
            ? <span className="shell-bc-current">{it.label}</span>
            : <a className="shell-bc-seg" href={it.href || '#'}>{it.label}</a>}
        </React.Fragment>
      ))}
    </div>
  );
}

/* ════════════════════════════════════════════════════════════
   AppShell
════════════════════════════════════════════════════════════ */
function AppShell({
  section = 'chat', subPage = null, user = {}, resizeKey = section,
  sidebarWidth = 240, railWidth = 340, headerHeight = 56, bandHeight = 52,
  sbMin = 190, sbMax = 380, railMin = 300, railMax = 560,
  breadcrumb, headerActions,
  brand, sidebar, footer, banner,
  toolbar, sidebarToolbar, railToolbar,
  rail, railHeader, railDefaultOpen = true,
  contentClass = '', mainScroll = true, railScroll = true,
  children,
}) {
  const shellRef = React.useRef(null);
  const [collapsed, setCollapsed] = React.useState(false);      // icon rail (desktop/tablet)
  const [railCollapsed, setRailCollapsed] = React.useState(!railDefaultOpen);
  const [sbOpen, setSbOpen] = React.useState(false);            // mobile drawer
  const [railOpen, setRailOpen] = React.useState(false);        // mobile drawer
  const [dragging, setDragging] = React.useState(false);
  const [openPop, setOpenPop] = React.useState(null);   // which collapsed popover is open (only one)
  const [isMobile, setIsMobile] = React.useState(
    () => typeof window !== 'undefined' && window.matchMedia('(max-width:767px)').matches);

  React.useEffect(() => {
    const m = window.matchMedia('(max-width:767px)');
    const h = e => setIsMobile(e.matches);
    m.addEventListener('change', h);
    return () => m.removeEventListener('change', h);
  }, []);

  const hasRail = Boolean(rail);
  const hasBand = Boolean(toolbar || sidebarToolbar || railToolbar);
  const effCollapsed = collapsed && !isMobile;

  /* ── column resize (widths persist; collapse does not) ── */
  React.useEffect(() => {
    const shell = shellRef.current; if (!shell) return;
    const sw = parseFloat(localStorage.getItem(`bodhi.${resizeKey}.sideW`));
    if (!isNaN(sw)) shell.style.setProperty('--shell-sb-w-user', clamp(sw, sbMin, sbMax) + 'px');
    const rw = parseFloat(localStorage.getItem(`bodhi.${resizeKey}.railW`));
    if (!isNaN(rw)) shell.style.setProperty('--shell-rail-w-user', clamp(rw, railMin, railMax) + 'px');
  }, []);

  const startDrag = (side, e) => {
    e.preventDefault();
    const shell = shellRef.current; if (!shell) return;
    const isLeft = side === 'left';
    const colEl = shell.querySelector(isLeft ? '.shell-sidebar' : '.shell-rail');
    const varName = isLeft ? '--shell-sb-w-user' : '--shell-rail-w-user';
    const min = isLeft ? sbMin : railMin, max = isLeft ? sbMax : railMax;
    const startX = e.clientX;
    const startW = colEl.getBoundingClientRect().width;
    setDragging(true);
    const move = mv => {
      const dx = mv.clientX - startX;
      shell.style.setProperty(varName, clamp(isLeft ? startW + dx : startW - dx, min, max) + 'px');
    };
    const up = () => {
      window.removeEventListener('pointermove', move);
      window.removeEventListener('pointerup', up);
      setDragging(false);
      const v = parseFloat(shell.style.getPropertyValue(varName));
      if (!isNaN(v)) localStorage.setItem(`bodhi.${resizeKey}.${isLeft ? 'sideW' : 'railW'}`, String(Math.round(v)));
    };
    window.addEventListener('pointermove', move);
    window.addEventListener('pointerup', up);
  };
  const resetWidth = side => {
    const shell = shellRef.current; if (!shell) return;
    const isLeft = side === 'left';
    shell.style.removeProperty(isLeft ? '--shell-sb-w-user' : '--shell-rail-w-user');
    localStorage.removeItem(`bodhi.${resizeKey}.${isLeft ? 'sideW' : 'railW'}`);
  };

  const toggleSidebar = () => { setOpenPop(null); isMobile ? setSbOpen(o => !o) : setCollapsed(c => !c); };
  const toggleRail = () => { isMobile ? setRailOpen(o => !o) : setRailCollapsed(c => !c); };
  const ctx = {
    collapsed: effCollapsed, isMobile, openPop, setOpenPop,
    openRail: () => { setRailCollapsed(false); setRailOpen(true); },
    closeRail: () => setRailOpen(false),
    collapseRail: () => { setRailCollapsed(true); setRailOpen(false); },
  };

  React.useEffect(() => { if (typeof lucide !== 'undefined') lucide.createIcons(); });

  const shellClass = ['shell',
    effCollapsed ? 'sb-collapsed' : '',
    (railCollapsed && !isMobile) ? 'rail-collapsed' : '',
    !hasRail ? 'no-rail' : '',
    sbOpen ? 'sb-open' : '',
    railOpen ? 'rail-open' : '',
    dragging ? 'is-dragging' : '',
  ].filter(Boolean).join(' ');

  const shellStyle = {
    '--sb-w-cfg': sidebarWidth + 'px',
    '--rail-w-cfg': railWidth + 'px',
    '--header-h-cfg': headerHeight + 'px',
    '--band-h-cfg': bandHeight + 'px',
  };

  return (
    <ShellContext.Provider value={ctx}>
      <div className={shellClass} style={shellStyle} ref={shellRef}>

        {/* ══ SIDEBAR ══ */}
        <aside className={'shell-col shell-sidebar' + (effCollapsed ? ' is-collapsed' : '')}>
          <div className="shell-headrow shell-brand">{brand || <ShellBrand collapsed={effCollapsed} />}</div>
          {hasBand && !effCollapsed && <div className="shell-bandrow shell-sb-band">{sidebarToolbar}</div>}
          {effCollapsed ? (
            <div className="shell-iconrail">
              <ShellNav section={section} subPage={subPage} />
              {sidebar && (<><div className="shell-iconrail-div" />{sidebar}</>)}
            </div>
          ) : (
            <div className="shell-body shell-nav-body">
              <ShellNav section={section} subPage={subPage} />
              {sidebar && (<><div className="shell-nav-div" />{sidebar}</>)}
            </div>
          )}
          <div className="shell-foot">{footer || <ShellFooter user={user} collapsed={effCollapsed} />}</div>
        </aside>

        {/* ══ MAIN ══ */}
        <main className="shell-col shell-main">
          <div className="shell-headrow shell-header">
            <button className="shell-icon-btn shell-sb-toggle" onClick={toggleSidebar}
                    title={isMobile ? 'Open menu' : 'Collapse sidebar'}>
              <ShellIcon name="panel-left" size={16} />
            </button>
            <ShellBreadcrumb items={breadcrumb} />
            <div className="shell-head-actions">
              {headerActions}
              {hasRail && (
                <button className="shell-icon-btn shell-rail-toggle" onClick={toggleRail} title="Toggle detail panel">
                  <ShellIcon name="panel-right" size={16} />
                </button>
              )}
            </div>
          </div>

          {banner && <div className="shell-mainband">{banner}</div>}
          {hasBand && <div className="shell-bandrow shell-toolbar">{toolbar}</div>}

          <div className={'shell-body' + (mainScroll ? '' : ' is-fill')}>
            <div className={'shell-content ' + contentClass}>{children}</div>
          </div>
        </main>

        {/* ══ RAIL ══ */}
        {hasRail && (
          <aside className="shell-col shell-rail">
            {railHeader !== undefined && <div className="shell-headrow" style={{ padding: '0 8px 0 14px' }}>{railHeader}</div>}
            {hasBand && <div className="shell-bandrow" style={{ padding: '0 14px' }}>{railToolbar}</div>}
            <div className={'shell-body' + (railScroll ? '' : ' is-fill')}>{rail}</div>
          </aside>
        )}

        {/* ══ RESIZE HANDLES (hover-reveal) ══ */}
        {!isMobile && (
          <div className="shell-resize left" style={{ left: 'var(--shell-sb-track)', transform: 'translateX(-50%)' }}
               onPointerDown={e => startDrag('left', e)} onDoubleClick={() => resetWidth('left')}>
            <div className="shell-resize-grip" />
          </div>
        )}
        {!isMobile && hasRail && !railCollapsed && (
          <div className="shell-resize right" style={{ left: 'calc(100% - var(--shell-rail-track))', transform: 'translateX(-50%)' }}
               onPointerDown={e => startDrag('right', e)} onDoubleClick={() => resetWidth('right')}>
            <div className="shell-resize-grip" />
          </div>
        )}

        {/* ══ TOOLTIP + DRAWER SCRIM ══ */}
        <GlobalTooltip />
        <div className="shell-scrim" onClick={() => { setSbOpen(false); setRailOpen(false); }} />
      </div>
    </ShellContext.Provider>
  );
}

/* ── Reusable search input (shared across pages) ──
   <ShellSearch value={q} onChange={setQ} placeholder="…" size="md|sm" kbd="⌘K" />
   onChange receives the new string value. Consistent height, centered
   icon, focus ring — use everywhere instead of hand-rolled search boxes. */
function ShellSearch({ value = '', onChange, placeholder, size = 'md', kbd, autoFocus, onKeyDown }) {
  const cls = 'shell-search' + (size === 'sm' ? ' sm' : '') + (kbd ? ' has-kbd' : '');
  return (
    <div className={cls}>
      <span className="ss-ico"><ShellIcon name="search" size={size === 'sm' ? 12 : 14} /></span>
      <input type="text" placeholder={placeholder} value={value} autoFocus={autoFocus}
             onKeyDown={onKeyDown}
             onChange={e => onChange && onChange(e.target.value)} />
      {kbd && <span className="ss-kbd">{kbd}</span>}
    </div>
  );
}

Object.assign(window, {
  AppShell, ShellNav, ShellIcon, ShellSearch, ShellModeSwitch, ShellFilterGroup,
  ShellBrand, ShellFooter, useShell, ShellContext, SHELL_NAV,
  AnchoredPopover, UserMenuPop, SHELL_TENANTS,
  useTheme, ShellThemeToggle, THEME_OPTS,
});
