/* ═══════════════════════════════════════════
   Bodhi Sidebar — React Component
   bodhi-sidebar-react.jsx

   Load AFTER React + lucide, BEFORE your app script:
     <script type="text/babel" src="bodhi-sidebar-react.jsx"></script>

   Usage:
     <BodhiSidebar section="models" subPage="new-local-model" />

   Exports to window: BodhiSidebar, BsbIcon, BSB_NAV
═══════════════════════════════════════════ */

const BSB_NAV = [
  {
    id: 'chat', label: 'Chat', icon: 'message-circle',
    href: 'Bodhi Chat.html', subPages: [],
  },
  {
    id: 'models', label: 'Models', icon: 'cpu',
    href: 'Bodhi Models.html', badge: '14',
    subPages: [
      { id: 'my-models',       label: 'My Models',       icon: 'layers',      href: 'Bodhi Models.html', badge: '14' },
      { id: 'all-models',      label: 'All Models',       icon: 'globe-2',     href: 'Bodhi Models.html' },
      { id: 'new-local-model',    label: 'New Local Model',    icon: 'plus-circle', href: 'Create New Local Model v4.html' },
      { id: 'new-api-model',      label: 'New API Model',      icon: 'plug-zap',    href: 'Create API Model.html' },
      { id: 'new-fallback-model', label: 'New Fallback Alias', icon: 'route',       href: 'Create Fallback Model.html' },
    ],
  },
  {
    id: 'mcp', label: 'MCP', icon: 'plug',
    href: 'Bodhi MCP Discover v2.html', badge: '3',
    subPages: [
      { id: 'discover',         label: 'Discover',         icon: 'compass',     href: 'Bodhi MCP Discover v2.html' },
      { id: 'my-instances',     label: 'My Instances',     icon: 'server',      href: '#', badge: '3' },
      { id: 'new-mcp-server',   label: 'New MCP Server',   icon: 'plus-circle', href: '#' },
      { id: 'new-mcp-instance', label: 'New MCP Instance', icon: 'plus',        href: '#' },
    ],
  },
  {
    id: 'api-keys', label: 'API Keys', icon: 'key-round', href: 'App Tokens.html',
    subPages: [
      { id: 'app-tokens',       label: 'App Tokens',       icon: 'key-round',    href: 'App Tokens.html' },
      { id: 'new-token',        label: 'New Token',        icon: 'plus-circle',  href: 'New App Token.html' },
      { id: 'access-requests',  label: 'Access Requests',  icon: 'shield-check', href: 'Access Requests.html' },
    ],
  },
  { id: 'settings', label: 'Settings', icon: 'settings',  href: '#', subPages: [] },
];

/* ── Lucide icon helper ─────────────────── */
function BsbIcon({ name, size = 14, color }) {
  const ref = React.useRef(null);
  React.useEffect(() => {
    if (!ref.current) return;
    ref.current.innerHTML = '';
    const el = document.createElement('i');
    el.setAttribute('data-lucide', name);
    if (color) el.style.color = color;
    ref.current.appendChild(el);
    if (typeof lucide !== 'undefined') lucide.createIcons({ nodes: [el] });
  }, [name, color]);
  return (
    <span
      ref={ref}
      style={{
        display: 'inline-flex', width: size, height: size,
        alignItems: 'center', justifyContent: 'center', flexShrink: 0,
      }}
    />
  );
}

/* ── BodhiSidebar component ─────────────── */
function BodhiSidebar({ section = 'chat', subPage = null, user = {}, children }) {
  const [navOpen, setNavOpen] = React.useState(false);

  const u = {
    initials: user.initials || 'YO',
    name:     user.name     || 'Yogesh',
    role:     user.role     || 'Admin',
  };

  const cur     = BSB_NAV.find(n => n.id === section) || BSB_NAV[0];
  const hasSub  = cur.subPages && cur.subPages.length > 0;
  const hasBody = Boolean(children);

  /* close dropdown on outside click */
  React.useEffect(() => {
    if (!navOpen) return;
    const h = () => setNavOpen(false);
    document.addEventListener('click', h);
    return () => document.removeEventListener('click', h);
  }, [navOpen]);

  /* ── colour tokens ── */
  const C = {
    lotus:     'rgba(255,164,184,.18)',
    lotusSub:  'rgba(255,164,184,.14)',
    badge:     'rgba(255,164,184,.2)',
    pink:      '#B02A52',
    pinkIcon:  '#DB456C',
    muted:     'hsl(var(--muted))',
    mutedFg:   'hsl(var(--muted-foreground))',
    border:    'hsl(var(--border))',
    fg:        'hsl(var(--foreground))',
    card:      'hsl(var(--card))',
    indigo:    '#3E4AA8',
  };

  const menuItemSt = on => ({
    display: 'flex', alignItems: 'center', gap: 9, padding: '8px 10px',
    borderRadius: 7, fontSize: 13, fontFamily: 'inherit', textDecoration: 'none',
    background: on ? C.lotus : 'none', color: on ? C.pink : C.mutedFg,
    fontWeight: on ? 600 : 400, cursor: 'pointer', border: 'none',
    width: '100%', textAlign: 'left', boxSizing: 'border-box',
  });

  const subItemSt = on => ({
    display: 'flex', alignItems: 'center', gap: 8, padding: '6px 11px',
    borderRadius: 8, fontSize: 13, fontFamily: 'inherit', textDecoration: 'none',
    background: on ? C.lotusSub : 'none', color: on ? C.pink : C.mutedFg,
    fontWeight: on ? 600 : 500, cursor: 'pointer', border: 'none',
    width: '100%', textAlign: 'left',
    transition: 'background 100ms, color 100ms', boxSizing: 'border-box',
  });

  return (
    <aside style={{
      background: C.card, borderRight: `1px solid ${C.border}`,
      display: 'flex', flexDirection: 'column',
      height: '100vh', width: 220, flexShrink: 0, overflow: 'hidden',
    }}>

      {/* Logo */}
      <div style={{
        display: 'flex', alignItems: 'center', gap: 10,
        padding: '13px 12px 12px', borderBottom: `1px solid ${C.border}`, flexShrink: 0,
      }}>
        <img src="assets/bodhi-logo-60.svg" alt="Bodhi" style={{ width: 28, height: 28, flexShrink: 0 }} />
        <div>
          <div style={{ fontSize: 15, fontWeight: 700, letterSpacing: '-.01em', lineHeight: 1 }}>Bodhi</div>
          <div style={{ fontFamily: 'var(--font-mono)', fontSize: 9, letterSpacing: '.14em', color: C.mutedFg, textTransform: 'uppercase', marginTop: 2 }}>
            AI Gateway
          </div>
        </div>
      </div>

      {/* Nav dropdown */}
      <div
        style={{ position: 'relative', padding: '10px 8px 4px', flexShrink: 0 }}
        onClick={e => e.stopPropagation()}
      >
        <button
          onClick={() => setNavOpen(o => !o)}
          style={{
            width: '100%', display: 'flex', alignItems: 'center', gap: 9,
            padding: '9px 11px', borderRadius: 10,
            background: C.muted, border: `1px solid ${C.border}`,
            cursor: 'pointer', fontSize: 13, fontWeight: 600,
            color: C.fg, fontFamily: 'inherit',
          }}
        >
          <span style={{ display: 'flex', alignItems: 'center', flexShrink: 0 }}>
            <BsbIcon name={cur.icon} size={15} color={C.pinkIcon} />
          </span>
          <span style={{ flex: 1, textAlign: 'left' }}>{cur.label}</span>
          <span style={{
            display: 'flex', alignItems: 'center', flexShrink: 0,
            color: C.mutedFg, transition: 'transform 150ms',
            transform: navOpen ? 'rotate(180deg)' : 'none',
          }}>
            <BsbIcon name="chevron-down" />
          </span>
        </button>

        {navOpen && (
          <div
            onClick={e => e.stopPropagation()}
            style={{
              position: 'absolute', top: 'calc(100% - 6px)', left: 8, right: 8,
              background: C.card, border: `1px solid ${C.border}`,
              borderRadius: 10, padding: 4,
              boxShadow: '0 8px 28px rgba(0,0,0,.12)', zIndex: 300,
            }}
          >
            {BSB_NAV.map(item => {
              const on = item.id === section;
              return (
                <a key={item.id} href={item.href || '#'} style={menuItemSt(on)}>
                  <BsbIcon name={item.icon} color={on ? C.pinkIcon : C.mutedFg} />
                  {item.label}
                  {item.badge && (
                    <span style={{ marginLeft: 'auto', fontSize: 10, fontWeight: 600, padding: '1px 6px', borderRadius: 99, background: C.badge, color: C.pink }}>
                      {item.badge}
                    </span>
                  )}
                </a>
              );
            })}
          </div>
        )}
      </div>

      {/* Sub-pages */}
      {hasSub && (
        <div style={{ padding: '4px 8px 0', flexShrink: 0 }}>
          {cur.subPages.map(sp => {
            const on = sp.id === subPage;
            return (
              <a key={sp.id} href={sp.href || '#'} style={subItemSt(on)}>
                <BsbIcon name={sp.icon || 'circle'} size={13} color={on ? C.pinkIcon : C.mutedFg} />
                {sp.label}
                {sp.badge && (
                  <span style={{ marginLeft: 'auto', fontSize: 10, fontWeight: 600, padding: '1px 6px', borderRadius: 99, background: C.badge, color: C.pink }}>
                    {sp.badge}
                  </span>
                )}
              </a>
            );
          })}
        </div>
      )}

      {/* Optional body slot (filters, history, etc.) */}
      {hasBody ? (
        <>
          <div style={{ height: 1, background: C.border, margin: '8px 10px 4px', flexShrink: 0 }} />
          <div style={{ flex: 1, overflowY: 'auto', overflowX: 'hidden' }}>
            {children}
          </div>
        </>
      ) : (
        <div style={{ flex: 1 }} />
      )}

      {/* Footer */}
      <div style={{
        borderTop: `1px solid ${C.border}`, padding: '10px 12px',
        display: 'flex', alignItems: 'center', gap: 9, flexShrink: 0,
      }}>
        <div style={{
          width: 30, height: 30, borderRadius: '50%',
          background: C.indigo, color: '#fff',
          display: 'flex', alignItems: 'center', justifyContent: 'center',
          fontSize: 11, fontWeight: 700, flexShrink: 0,
        }}>
          {u.initials}
        </div>
        <div style={{ minWidth: 0 }}>
          <div style={{ fontSize: 13, fontWeight: 500 }}>{u.name}</div>
          <div style={{ fontSize: 11, color: C.mutedFg }}>{u.role}</div>
        </div>
        <button
          title="Log out"
          style={{
            marginLeft: 'auto', width: 26, height: 26, borderRadius: 6,
            display: 'flex', alignItems: 'center', justifyContent: 'center',
            background: 'none', border: 'none', color: C.mutedFg, cursor: 'pointer',
          }}
        >
          <BsbIcon name="log-out" />
        </button>
      </div>
    </aside>
  );
}

Object.assign(window, { BodhiSidebar, BsbIcon, BSB_NAV });
