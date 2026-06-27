/* ═══════════════════════════════════════════════════════════════
   BODHI MCP — PLAYGROUND (root, on AppShell)
   mcp/bodhi-mcp-playground-app.jsx   (load LAST of the playground scripts)

   A re-imagined, non-technical Playground. The left rail picks the
   active MCP instance (URL-param driven) and the capability to explore;
   the main area is the friendly Overview or a runnable Tools / Prompts /
   Resources / Templates surface. Nothing is shown until a valid instance
   is selected. A Developer toggle reveals raw JSON, request bodies and
   the JSON input editor for the technically inclined.
═══════════════════════════════════════════════════════════════ */

/* ── left rail: instance picker + capability nav (collapse-aware) ── */
function PlaygroundSidebar({ instances, selected, onSelectInstance, counts, activeCap, onSelectCap }) {
  const { collapsed } = useShell();
  if (collapsed) {
    return (
      <>
        {selected
          ? <button className="shell-railbtn shell-tip on" data-tip={selected.instName + ' · ' + selected.server.name}><ServerGlyph s={selected.server} size={20} radius={6} /></button>
          : <button className="shell-railbtn shell-tip" data-tip="Select an MCP"><Ic name="plug" size={17} /></button>}
        <div className="shell-iconrail-div" />
        {PG_CAPS.map(c => (
          <button key={c.id} className={'shell-railbtn shell-tip' + (activeCap === c.id ? ' on' : '')}
            data-tip={c.label} disabled={!selected} onClick={() => onSelectCap(c.id)}>
            <Ic name={c.icon} size={17} />
          </button>
        ))}
      </>
    );
  }
  return (
    <div className="pg-rail">
      <InstanceCombobox instances={instances} selected={selected} onSelect={onSelectInstance} />
      <div className="pg-rail-div" />
      <CapabilityNav counts={counts} active={activeCap} onSelect={onSelectCap} disabled={!selected} />
    </div>
  );
}

/* ── blank state (no instance chosen) ────────────────────────── */
function BlankState({ instances, onPick }) {
  const connected = instances.filter(i => i.status === 'connected');
  return (
    <div className="pg-blank">
      <div className="pg-blank-inner">
        <div className="pg-blank-mark"><Ic name="flask-conical" size={30} /></div>
        <h1 className="pg-blank-title">MCP Playground</h1>
        <p className="pg-blank-sub">A calm place to explore what a connected MCP can do — run its tools, preview prompts, and read its data, no code required. Pick one of your MCPs to begin.</p>
        <div className="pg-blank-grid">
          {connected.map(i => (
            <button key={i.instId} className="pg-blank-card" onClick={() => onPick(i)}>
              <ServerGlyph s={i.server} size={34} radius={9} />
              <span className="pg-blank-card-body">
                <span className="pg-blank-card-name">{i.instName}</span>
                <span className="pg-blank-card-sub">{i.server.name}</span>
              </span>
              <Ic name="arrow-right" size={15} />
            </button>
          ))}
          <a className="pg-blank-new" href="Bodhi MCP New Instance.html"><Ic name="plus" size={14} /> Connect a new MCP</a>
        </div>
      </div>
    </div>
  );
}

/* ── routes the active capability to its view ────────────────── */
function CapabilityRouter({ inst, counts, status, activeCap, onSelectCap }) {
  const serverId = inst.serverId;
  if (activeCap === 'overview') return <OverviewView inst={inst} counts={counts} status={status} onSelect={onSelectCap} />;
  if (activeCap === 'tools') return <ToolsView serverId={serverId} inst={inst} />;
  if (activeCap === 'prompts') return <PromptsView serverId={serverId} inst={inst} />;
  if (activeCap === 'resources') return <ResourcesView serverId={serverId} />;
  if (activeCap === 'templates') return <TemplatesView serverId={serverId} />;
  return null;
}

/* ── root ────────────────────────────────────────────────────── */
function PlaygroundApp() {
  const instances = useMemo(() => playgroundInstances(), []);
  const params = useMemo(() => new URLSearchParams(window.location.search), []);

  const initial = useMemo(() => {
    const instId = params.get('instance');
    const serverId = params.get('server');
    if (!instId && !serverId) return null;
    return findInstance(instId, serverId);
  }, []);

  const [selected, setSelected] = useState(initial);
  const [activeCap, setActiveCap] = useState('overview');
  const [status, setStatus] = useState(initial ? 'connecting' : 'idle');
  const [dev, setDev] = useState(false);

  const counts = useMemo(() => selected ? capabilityCounts(selected.serverId) : null, [selected]);

  /* simulate connecting whenever the instance changes */
  useEffect(() => {
    if (!selected) { setStatus('idle'); return; }
    setStatus('connecting');
    const t = setTimeout(() => setStatus(selected.status === 'pending' ? 'connecting' : 'connected'), 850);
    return () => clearTimeout(t);
  }, [selected && selected.instId]);

  useEffect(() => {
    document.title = selected ? `Bodhi · ${selected.instName} · Playground` : 'Bodhi · MCP Playground';
  }, [selected]);

  const selectInstance = inst => {
    setSelected(inst);
    setActiveCap('overview');
    const q = new URLSearchParams({ instance: inst.instId, name: inst.instName, server: inst.serverId });
    window.history.replaceState(null, '', 'Bodhi MCP Playground.html?' + q.toString());
  };

  const reconnect = () => { setStatus('connecting'); setTimeout(() => setStatus('connected'), 900); };

  const headerActions = (
    <>
      <div className="pg-head-title">
        <span className="pg-head-name">{selected ? selected.instName : 'Playground'}</span>
        {selected && (
          status === 'connected'
            ? <span className="pg-pill ok"><Ic name="circle-check" size={11} /> Connected</span>
            : <span className="pg-pill warn"><PgSpinner size={11} /> Connecting…</span>
        )}
      </div>
      <button className={'pg-dev-toggle' + (dev ? ' on' : '')} onClick={() => setDev(d => !d)}
        title="Show raw JSON, request bodies and the JSON editor">
        <Ic name="code-2" size={13} /> Developer
        <span className={'pg-dev-sw' + (dev ? ' on' : '')} />
      </button>
      {selected && <button className="icon-btn" title="Reconnect" onClick={reconnect}><Ic name="refresh-cw" size={14} /></button>}
    </>
  );

  return (
    <DevContext.Provider value={{ dev, setDev }}>
      <AppShell
        section="mcp" subPage="playground" resizeKey="mcp"
        sidebarWidth={264} sbMin={232} sbMax={360}
        breadcrumb={[
          { label: 'MCP', href: 'Bodhi MCP My MCPs.html' },
          { label: 'Playground', current: true },
        ]}
        headerActions={headerActions}
        sidebar={<PlaygroundSidebar instances={instances} selected={selected} onSelectInstance={selectInstance}
          counts={counts} activeCap={activeCap} onSelectCap={setActiveCap} />}
        contentClass="flush" mainScroll={false}
      >
        {selected
          ? <CapabilityRouter inst={selected} counts={counts} status={status} activeCap={activeCap} onSelectCap={setActiveCap} />
          : <BlankState instances={instances} onPick={selectInstance} />}
      </AppShell>
    </DevContext.Provider>
  );
}

ReactDOM.createRoot(document.getElementById('root')).render(<PlaygroundApp />);
