/* ═══════════════════════════════════════════════════════════════
   Bodhi App Shell — CORE primitives
   shared/shell-core.jsx   (load 1st of the shell modules)

   Context, the lucide icon helper, the two fixed-position overlays
   (tooltip + popover), the nav/tenant data, and the theme hook.
   Everything here is published to window so the other shell modules
   (shell-chrome, shell-user, shell-app) and page apps can use it.

   Load order on every page:
     <script type="text/babel" src="shared/shell-core.jsx"></script>
     <script type="text/babel" src="shared/shell-chrome.jsx"></script>
     <script type="text/babel" src="shared/shell-user.jsx"></script>
     <script type="text/babel" src="shared/shell-app.jsx"></script>
═══════════════════════════════════════════════════════════════ */

const SHELL_NAV = (typeof window !== 'undefined' && window.BSB_NAV) || [
  { id: 'chat', label: 'Chat', icon: 'message-circle', href: 'Bodhi Chat.html', subPages: [] },
  {
    id: 'models', label: 'Models', icon: 'cpu', href: 'Bodhi Models.html', badge: '14',
    subPages: [
      { id: 'my-models',          label: 'My Models',            icon: 'layers',      href: 'Bodhi Models.html' },
      { id: 'explore-local',      label: 'Explore · Local Models', icon: 'hard-drive', href: 'Bodhi Models Local.html' },
      { id: 'explore-api',        label: 'Explore · API Providers', icon: 'at-sign',    href: 'Bodhi Models API.html' },
      { id: 'explore-api-catalog', label: 'Explore · API Models',   icon: 'sparkles',   href: 'Bodhi Models API Catalog.html' },
      { id: 'new-local-model',    label: 'New Local Model',    icon: 'plus-circle', href: 'Create New Local Model v4.html' },
      { id: 'new-api-model',      label: 'New API Model',      icon: 'plug-zap',    href: 'Create API Model.html' },
      { id: 'new-fallback-model', label: 'New Model Router', icon: 'route',       href: 'Create Fallback Model.html' },
    ],
  },
  {
    id: 'mcp', label: 'MCP', icon: 'plug', href: 'Bodhi MCP My MCPs.html',
    subPages: [
      { id: 'my-mcps',      label: 'My MCPs',      icon: 'layers',      href: 'Bodhi MCP My MCPs.html' },
      { id: 'explore',      label: 'Explore MCPs',      icon: 'compass',     href: 'Bodhi MCP Discover v2.html' },
      { id: 'playground',   label: 'Playground',   icon: 'flask-conical', href: 'MCP-Playground-Overview.html' },
      { id: 'new-server',   label: 'New MCP Server',   icon: 'server-cog',  href: 'Bodhi MCP New Server.html' },
      { id: 'new-mcp',      label: 'New MCP Instance', icon: 'plus-circle', href: 'Bodhi MCP New Instance.html' },
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

Object.assign(window, {
  SHELL_NAV, clamp, ShellContext, useShell, ShellIcon,
  GlobalTooltip, AnchoredPopover, SHELL_TENANTS, THEME_OPTS, useTheme,
});
