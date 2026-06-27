/* ═══════════════════════════════════════════════════════════════
   Bodhi App Shell — APPSHELL layout + search
   shared/shell-app.jsx   (load last of the shell modules)

   The 3-column layout frame (sidebar / main / rail) with collapse,
   mobile drawers, and persisted column resizing — plus the shared
   <ShellSearch> input. Composes the pieces from shell-core /
   shell-chrome / shell-user.

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
   >{main content}</AppShell>

   Exports: AppShell, ShellSearch (plus everything from the other
   shell modules, all on window).
═══════════════════════════════════════════════════════════════ */
function AppShell({
  section = 'chat', subPage = null, user = {}, resizeKey = section,
  sidebarWidth = 240, railWidth = 340, headerHeight = 56, bandHeight = 52,
  sbMin = 190, sbMax = 380, railMin = 300, railMax = 560,
  breadcrumb, headerActions,
  brand, sidebar, footer, banner,
  toolbar, sidebarToolbar, railToolbar,
  rail, railHeader, railDefaultOpen = true, railCollapsible = true,
  navBase = '',
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
  const railIsCollapsed = railCollapsible ? railCollapsed : false;   // pinned-open when not collapsible (desktop)

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
    collapsed: effCollapsed, isMobile, openPop, setOpenPop, navBase,
    openRail: () => { setRailCollapsed(false); setRailOpen(true); },
    closeRail: () => setRailOpen(false),
    collapseRail: () => { setRailCollapsed(true); setRailOpen(false); },
  };

  React.useEffect(() => { if (typeof lucide !== 'undefined') lucide.createIcons(); });

  const shellClass = ['shell',
    effCollapsed ? 'sb-collapsed' : '',
    (railIsCollapsed && !isMobile) ? 'rail-collapsed' : '',
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
              {hasRail && (isMobile || railCollapsible) && (
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
        {!isMobile && hasRail && !railIsCollapsed && (
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

Object.assign(window, { AppShell, ShellSearch });
