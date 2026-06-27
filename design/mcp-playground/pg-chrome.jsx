/* ═══════════════════════════════════════════════════════════════
   BODHI MCP — PLAYGROUND PAGE SHELL  (multi-page)
   mcp-playground/pg-chrome.jsx   (load LAST, after pg-views.jsx)

   Each capability is its own page. This file wires the shared shell for
   every page on the Bodhi AppShell:

     • LEFT sidebar  — instance picker (Active MCP) + capability nav.
       The nav links ACROSS pages (Overview / Tools / Prompts / …),
       carrying the active instance in the URL.
     • RIGHT rail    — the item list for this capability, PINNED open
       (railCollapsible=false). Selecting a row updates the centre.
     • CENTRE main   — the selected item's form + readable result.
       Developer (header toggle) reveals Raw / Request inline.

   A page sets window.PG_PAGE_CAP ('overview'|'tools'|…) before this
   script; we auto-mount it.
═══════════════════════════════════════════════════════════════ */

/* capability nav model — each links to its own page, instance in tow */
const PG_CAPS = [
  { id: 'overview',  label: 'Overview',  icon: 'compass',              file: 'MCP-Playground-Overview.html',  countKey: null },
  { id: 'tools',     label: 'Tools',     icon: 'wrench',               file: 'MCP-Playground-Tools.html',     countKey: 'tools' },
  { id: 'prompts',   label: 'Prompts',   icon: 'message-square-quote', file: 'MCP-Playground-Prompts.html',   countKey: 'prompts' },
  { id: 'resources', label: 'Resources', icon: 'folder-open',          file: 'MCP-Playground-Resources.html', countKey: 'resources' },
  { id: 'templates', label: 'Templates', icon: 'layout-template',      file: 'MCP-Playground-Templates.html', countKey: 'templates' },
];
const capFile = id => (PG_CAPS.find(c => c.id === id) || PG_CAPS[0]).file;
function capHref(id, inst) {
  return capFile(id) + (inst ? '?' + instQS(inst) : '');
}

/* ══ INSTANCE PICKER (left, navigates within the current capability) ══ */
function InstancePicker({ instances, selected, currentCap }) {
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

  const pick = i => { window.location.href = capHref(currentCap, i); };

  const ql = query.toLowerCase();
  const filtered = instances.filter(i => !ql || i.instName.toLowerCase().includes(ql) || i.server.name.toLowerCase().includes(ql));
  const connected = filtered.filter(i => i.status === 'connected');
  const pending = filtered.filter(i => i.status !== 'connected');
  const renderOpt = i => (
    <button type="button" key={i.instId}
      className={'pg-inst-opt' + (selected && selected.instId === i.instId ? ' selected' : '')}
      onClick={() => pick(i)}>
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
            <input ref={inputRef} value={query} onChange={e => setQuery(e.target.value)} placeholder="Search your MCPs…" autoComplete="off" />
          </div>
          <div className="pg-inst-list">
            {filtered.length === 0 && <div className="pg-inst-none">No MCPs match “{query}”</div>}
            {connected.length > 0 && <div className="pg-inst-grouplabel">Connected</div>}
            {connected.map(renderOpt)}
            {pending.length > 0 && <div className="pg-inst-grouplabel">Authorizing</div>}
            {pending.map(renderOpt)}
          </div>
          <a className="pg-inst-foot" href="Bodhi MCP New Instance.html"><Ic name="plus" size={13} /> Connect a new MCP</a>
        </div>
      )}
    </div>
  );
}

/* ══ CAPABILITY NAV (left, cross-page links) ══ */
function CapabilityNav({ counts, active, inst }) {
  return (
    <div className="pg-capnav">
      <div className="pg-rail-label">Explore</div>
      {PG_CAPS.map(c => {
        const n = c.countKey ? (counts ? counts[c.countKey] : 0) : null;
        const muted = !inst || (c.countKey && n === 0);
        const isActive = active === c.id;
        const cls = 'pg-cap' + (isActive ? ' on' : '') + (muted && !isActive ? ' muted' : '');
        const inner = (
          <>
            <span className="pg-cap-ico"><Ic name={c.icon} size={15} /></span>
            <span className="pg-cap-label">{c.label}</span>
            {n != null && <span className="pg-cap-count">{!inst ? '—' : n}</span>}
          </>
        );
        if (muted && !isActive) return <span key={c.id} className={cls} aria-disabled="true">{inner}</span>;
        return <a key={c.id} className={cls} href={capHref(c.id, inst)}>{inner}</a>;
      })}
    </div>
  );
}

/* ══ LIST RAIL (right, pinned) — the master list for this capability ══ */
function ListRail({ config, items, activeId, onSelect }) {
  const [q, setQ] = useState('');
  if (!items || items.length === 0) {
    return (
      <div className="pg-cap-empty">
        <Ic name={config.empty.icon} size={34} />
        <div className="pg-cap-empty-t">{config.empty.title}</div>
        <div className="pg-cap-empty-s">{config.empty.desc}</div>
      </div>
    );
  }
  const ql = q.toLowerCase();
  const filtered = ql ? items.filter(it => config.searchKeys.some(k => String(it[k] || '').toLowerCase().includes(ql))) : items;
  return (
    <div className="pg-listrail">
      <div className="pg-md-search">
        <ShellSearch size="sm" value={q} onChange={setQ} placeholder={config.searchPlaceholder} />
      </div>
      <div className="pg-md-rows">
        {filtered.length === 0 && <div className="pg-md-none">No matches</div>}
        {filtered.map(it => (
          <button key={config.getId(it)} type="button"
            className={'pg-row' + (config.getId(it) === activeId ? ' on' : '')}
            onClick={() => onSelect(config.getId(it))}>
            {config.renderRow(it)}
          </button>
        ))}
      </div>
    </div>
  );
}

/* ══ BLANK STATE (no instance chosen) ══ */
function BlankState({ instances }) {
  const connected = instances.filter(i => i.status === 'connected');
  return (
    <div className="pg-blank">
      <div className="pg-blank-inner">
        <div className="pg-blank-mark"><Ic name="flask-conical" size={30} /></div>
        <h1 className="pg-blank-title">MCP Playground</h1>
        <p className="pg-blank-sub">A calm place to explore what a connected MCP can do — run its tools, preview prompts, and read its data, no code required. Pick one of your MCPs to begin.</p>
        <div className="pg-blank-grid">
          {connected.map(i => (
            <a key={i.instId} className="pg-blank-card" href={capHref('overview', i)}>
              <ServerGlyph s={i.server} size={34} radius={9} />
              <span className="pg-blank-card-body">
                <span className="pg-blank-card-name">{i.instName}</span>
                <span className="pg-blank-card-sub">{i.server.name}</span>
              </span>
              <Ic name="arrow-right" size={15} />
            </a>
          ))}
          <a className="pg-blank-new" href="Bodhi MCP New Instance.html"><Ic name="plus" size={14} /> Connect a new MCP</a>
        </div>
      </div>
    </div>
  );
}

/* ══ PAGE ══ */
function PlaygroundPage({ cap }) {
  const instances = useMemo(() => playgroundInstances(), []);
  const inst = useMemo(() => resolveURLInstance(), []);
  const [status, setStatus] = useState(inst ? 'connecting' : 'idle');

  const counts = useMemo(() => inst ? capabilityCounts(inst.serverId) : null, [inst]);
  const config = cap === 'overview' ? null : CAP_CONFIG[cap];

  /* resource link opened from a tool (Resources page reads ?open) */
  const openParam = useMemo(() => {
    const p = urlParams();
    if (cap !== 'resources' || !p.get('open')) return null;
    return { uri: p.get('open'), name: p.get('openName'), mimeType: p.get('openMime'), description: p.get('openDesc') };
  }, [cap]);

  const items = useMemo(() => {
    if (!inst || !config) return [];
    const base = config.getItems(inst.serverId);
    if (openParam && !base.find(r => r.uri === openParam.uri)) return [makeTransientResource(openParam), ...base];
    return base;
  }, [inst, cap]);

  const [activeId, setActiveId] = useState(() => openParam ? openParam.uri : (config && items[0] ? config.getId(items[0]) : null));

  useEffect(() => {
    if (!inst) { setStatus('idle'); return; }
    setStatus('connecting');
    const t = setTimeout(() => setStatus(inst.status === 'pending' ? 'connecting' : 'connected'), 800);
    return () => clearTimeout(t);
  }, []);
  useEffect(() => { document.title = inst ? `Bodhi · ${inst.instName} · ${cap[0].toUpperCase() + cap.slice(1)}` : 'Bodhi · MCP Playground'; }, []);

  const nav = useMemo(() => ({
    openResource: link => {
      const qs = instQS(inst) + '&open=' + encodeURIComponent(link.uri)
        + (link.name ? '&openName=' + encodeURIComponent(link.name) : '')
        + (link.mimeType ? '&openMime=' + encodeURIComponent(link.mimeType) : '')
        + (link.description ? '&openDesc=' + encodeURIComponent(link.description) : '');
      window.location.href = 'MCP-Playground-Resources.html?' + qs;
    },
  }), [inst]);

  const reconnect = () => { setStatus('connecting'); setTimeout(() => setStatus('connected'), 800); };

  const headerActions = (
    <>
      <div className="pg-head-title">
        <span className="pg-head-name">{inst ? inst.instName : 'Playground'}</span>
        {inst && (status === 'connected'
          ? <span className="pg-pill ok"><Ic name="circle-check" size={11} /> Connected</span>
          : <span className="pg-pill warn"><PgSpinner size={11} /> Connecting…</span>)}
      </div>
      {inst && <button className="icon-btn" title="Reconnect" onClick={reconnect}><Ic name="refresh-cw" size={14} /></button>}
    </>
  );

  const breadcrumb = [{ label: 'MCP', href: 'Bodhi MCP My MCPs.html' }];
  if (cap === 'overview') breadcrumb.push({ label: 'Playground', current: true });
  else breadcrumb.push({ label: 'Playground', href: capHref('overview', inst) }, { label: PG_CAPS.find(c => c.id === cap).label, current: true });

  const active = config && items.find(it => config.getId(it) === activeId);
  const centre = !inst
    ? <BlankState instances={instances} />
    : cap === 'overview'
      ? <OverviewView inst={inst} counts={counts} status={status} capHref={id => capHref(id, inst)} />
      : (active ? config.renderDetail(active, inst) : <PickSomething what={config.pick} />);

  const showRail = inst && cap !== 'overview';

  return (
    <PgNavContext.Provider value={nav}>
      <AppShell
          section="mcp" subPage="playground" resizeKey="mcp-pg"
          sidebarWidth={264} sbMin={232} sbMax={360}
          railWidth={320} railMin={264} railMax={420}
          railScroll={false} navBase=""
          breadcrumb={breadcrumb}
          headerActions={headerActions}
          sidebar={<div className="pg-rail">
            <InstancePicker instances={instances} selected={inst} currentCap={cap} />
            <div className="pg-rail-div" />
            <CapabilityNav counts={counts} active={cap} inst={inst} />
          </div>}
          rail={showRail ? <ListRail config={config} items={items} activeId={activeId} onSelect={setActiveId} /> : undefined}
          railHeader={showRail ? <div className="pg-listrail-head"><span>{config.listTitle}</span><span className="pg-listrail-count">{items.length}</span></div> : undefined}
          contentClass="flush" mainScroll={false}
        >
          {centre}
        </AppShell>
      </PgNavContext.Provider>
  );
}

Object.assign(window, { PG_CAPS, capHref, InstancePicker, CapabilityNav, ListRail, BlankState, PlaygroundPage });

if (window.PG_PAGE_CAP) {
  ReactDOM.createRoot(document.getElementById('root')).render(<PlaygroundPage cap={window.PG_PAGE_CAP} />);
}
