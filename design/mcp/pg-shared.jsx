/* ═══════════════════════════════════════════════════════════════
   BODHI MCP — PLAYGROUND SHARED CHROME
   mcp/pg-shared.jsx   (load AFTER pg-data.jsx, BEFORE pg-render)

   The left-rail instance picker + capability nav, the Developer-mode
   context, and the small primitives every view reuses. Declares the
   React hook aliases ONCE for the whole page (the later playground
   scripts use them bare). Published to window.
═══════════════════════════════════════════════════════════════ */
const { useState, useEffect, useRef, useMemo } = React;

/* ── tiny primitives ─────────────────────────────────────────── */
function PgSpinner({ size = 14 }) {
  return <span className="pg-spin" style={{ width: size, height: size }} />;
}
function prettyKey(k) {
  return String(k)
    .replace(/[_-]+/g, ' ')
    .replace(/([a-z0-9])([A-Z])/g, '$1 $2')
    .replace(/^./, c => c.toUpperCase());
}
function escapeHtml(str) {
  return String(str).replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
}
function syntaxHighlight(json) {
  if (typeof json !== 'string') json = JSON.stringify(json, null, 2);
  return escapeHtml(json).replace(
    /("(\\u[a-zA-Z0-9]{4}|\\[^u]|[^\\"])*"(\s*:)?|\b(true|false|null)\b|-?\d+(?:\.\d*)?(?:[eE][+\-]?\d+)?)/g,
    m => {
      if (/^"/.test(m)) return /:$/.test(m) ? `<span class="json-key">${m}</span>` : `<span class="json-str">${m}</span>`;
      if (/true|false/.test(m)) return `<span class="json-bool">${m}</span>`;
      if (/null/.test(m)) return `<span class="json-null">${m}</span>`;
      return `<span class="json-num">${m}</span>`;
    }
  );
}

/* ── Developer-mode context (raw JSON, request editor, log) ──── */
const DevContext = React.createContext({ dev: false, setDev: () => {} });
const useDev = () => React.useContext(DevContext);

/* ── server glyph (icon tile) ────────────────────────────────── */
function ServerGlyph({ s, size = 30, radius = 8 }) {
  if (!s) return <div className="pg-glyph" style={{ width: size, height: size, borderRadius: radius, background: 'hsl(var(--muted))' }} />;
  return (
    <div className="pg-glyph" style={{ width: size, height: size, borderRadius: radius, background: s.iconBg, color: s.iconColor, fontSize: size * 0.46 }}>
      {s.icon}
    </div>
  );
}

/* ── status dot for an instance ──────────────────────────────── */
function StatusDot({ status }) {
  const map = {
    connected: { c: 'var(--c-connected-text)', t: 'Connected' },
    pending:   { c: 'var(--c-pending-text)',   t: 'Authorizing' },
  };
  const m = map[status] || { c: 'hsl(var(--muted-foreground))', t: status || '' };
  return <span className="pg-dot" style={{ background: m.c }} title={m.t} />;
}

/* ══ INSTANCE COMBOBOX (left rail) ═══════════════════════════════
   Lists the user's connected instances, grouped connected → pending.
   Blank when nothing is selected. Footer links to New Instance. */
function InstanceCombobox({ instances, selected, onSelect }) {
  const [open, setOpen] = useState(false);
  const [query, setQuery] = useState('');
  const boxRef = useRef(null);
  const inputRef = useRef(null);

  useEffect(() => {
    if (!open) return;
    const h = e => { if (boxRef.current && !boxRef.current.contains(e.target)) setOpen(false); };
    document.addEventListener('click', h);
    return () => document.removeEventListener('click', h);
  }, [open]);
  useEffect(() => { if (open && inputRef.current) inputRef.current.focus(); }, [open]);

  const ql = query.toLowerCase();
  const filtered = instances.filter(i =>
    !ql || i.instName.toLowerCase().includes(ql) || i.server.name.toLowerCase().includes(ql));
  const connected = filtered.filter(i => i.status === 'connected');
  const pending = filtered.filter(i => i.status !== 'connected');

  const renderOpt = i => (
    <button type="button" key={i.instId}
      className={'pg-inst-opt' + (selected && selected.instId === i.instId ? ' selected' : '')}
      onClick={() => { onSelect(i); setOpen(false); setQuery(''); }}>
      <ServerGlyph s={i.server} size={26} radius={7} />
      <span className="pg-inst-opt-body">
        <span className="pg-inst-opt-name">{i.instName}</span>
        <span className="pg-inst-opt-sub">{i.server.name}</span>
      </span>
      <StatusDot status={i.status} />
    </button>
  );

  return (
    <div className="pg-inst" ref={boxRef}>
      <div className="pg-rail-label">Active MCP</div>
      <button type="button" className={'pg-inst-trigger' + (open ? ' open' : '') + (selected ? '' : ' placeholder')}
        onClick={e => { e.stopPropagation(); setOpen(o => !o); setQuery(''); }}>
        {selected ? (
          <>
            <ServerGlyph s={selected.server} size={28} radius={7} />
            <span className="pg-inst-trigger-body">
              <span className="pg-inst-trigger-name">{selected.instName}</span>
              <span className="pg-inst-trigger-sub">{selected.server.name}</span>
            </span>
            <StatusDot status={selected.status} />
          </>
        ) : (
          <>
            <span className="pg-inst-trigger-ph"><Ic name="mouse-pointer-click" size={15} /></span>
            <span className="pg-inst-trigger-body"><span className="pg-inst-trigger-name">Select an MCP…</span></span>
          </>
        )}
        <span className="pg-inst-chev"><Ic name="chevrons-up-down" size={14} /></span>
      </button>

      {open && (
        <div className="pg-inst-pop" onClick={e => e.stopPropagation()}>
          <div className="pg-inst-search">
            <Ic name="search" size={13} />
            <input ref={inputRef} value={query} onChange={e => setQuery(e.target.value)}
              placeholder="Search your MCPs…" autoComplete="off" />
          </div>
          <div className="pg-inst-list">
            {filtered.length === 0 && <div className="pg-inst-none">No MCPs match “{query}”</div>}
            {connected.length > 0 && <div className="pg-inst-grouplabel">Connected</div>}
            {connected.map(renderOpt)}
            {pending.length > 0 && <div className="pg-inst-grouplabel">Authorizing</div>}
            {pending.map(renderOpt)}
          </div>
          <a className="pg-inst-foot" href="Bodhi MCP New Instance.html">
            <Ic name="plus" size={13} /> Connect a new MCP
          </a>
        </div>
      )}
    </div>
  );
}

/* ══ CAPABILITY NAV (left rail) ══════════════════════════════════
   Overview + the four MCP capability surfaces, each with a live count.
   Disabled until an instance is chosen. */
const PG_CAPS = [
  { id: 'overview',  label: 'Overview',  icon: 'compass',     countKey: null },
  { id: 'tools',     label: 'Tools',     icon: 'wrench',      countKey: 'tools' },
  { id: 'prompts',   label: 'Prompts',   icon: 'message-square-quote', countKey: 'prompts' },
  { id: 'resources', label: 'Resources', icon: 'folder-open', countKey: 'resources' },
  { id: 'templates', label: 'Templates', icon: 'layout-template', countKey: 'templates' },
];

function CapabilityNav({ counts, active, onSelect, disabled }) {
  return (
    <div className="pg-capnav">
      <div className="pg-rail-label">Explore</div>
      {PG_CAPS.map(c => {
        const n = c.countKey ? (counts ? counts[c.countKey] : 0) : null;
        return (
          <button key={c.id} type="button" disabled={disabled}
            className={'pg-cap' + (active === c.id ? ' on' : '')}
            onClick={() => onSelect(c.id)}>
            <span className="pg-cap-ico"><Ic name={c.icon} size={15} /></span>
            <span className="pg-cap-label">{c.label}</span>
            {n != null && <span className="pg-cap-count">{disabled ? '—' : n}</span>}
          </button>
        );
      })}
    </div>
  );
}

Object.assign(window, {
  useState, useEffect, useRef, useMemo,
  PgSpinner, prettyKey, escapeHtml, syntaxHighlight,
  DevContext, useDev, ServerGlyph, StatusDot,
  InstanceCombobox, CapabilityNav, PG_CAPS,
});
