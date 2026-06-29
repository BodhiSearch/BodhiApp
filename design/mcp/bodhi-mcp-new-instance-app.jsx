/* ═══════════════════════════════════════════════════
   BODHI MCP — NEW / EDIT INSTANCE (form, on AppShell)
   bodhi-mcp-new-instance-app.jsx  (load after shell modules)

   Create or edit a personal MCP instance against a REGISTERED server.
   The Auth Configuration dropdown always offers Public (built-in, no
   setup) plus every configured oauth / key mechanism — a server may have
   several, including several of one type. Picking one shows: Connect
   (oauth) · key value field (header/query) · nothing (public).

   Modes (URL params):
     • create — ?server=<id>&auth=<mechanism-name>  (auth optional)
     • edit   — ?instance=<id>&edit=1&server=<id>&name=&auth=
                server is locked; name/auth/desc editable
═══════════════════════════════════════════════════ */
const { useState, useEffect, useRef } = React;

/* CONNECTABLE_SERVERS (registered & enabled), connectMechs (auth options,
   Public last), PUBLIC_AC, AuthBadge and slugify all come from
   mcp-catalog.jsx — the single source of truth loaded before this. */

/* ── Server combobox (locked in edit mode) ── */
function ServerCombobox({ selected, onSelect, locked }) {
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

  if (locked) {
    return (
      <div className="server-combobox">
        <button type="button" className="server-trigger" disabled style={{ cursor: 'default', opacity: .85 }}>
          <div className="server-trigger-icon" style={{ background: selected ? selected.iconBg : 'hsl(var(--muted))' }}>
            {selected ? <span style={{ color: selected.iconColor, fontSize: 13, fontWeight: 800 }}>{selected.icon}</span> : <Ic name="server" size={13} />}
          </div>
          <span className="server-trigger-label">{selected ? selected.name : '—'}</span>
          <span className="server-trigger-url">{selected ? selected.url : ''}</span>
          <span style={{ marginLeft: 'auto', display: 'flex', gap: 4, alignItems: 'center', color: 'hsl(var(--muted-foreground))', fontSize: 11, fontWeight: 600 }}><Ic name="lock" size={11} /> locked</span>
        </button>
      </div>
    );
  }

  const filtered = CONNECTABLE_SERVERS.filter(s => {
    const ql = query.toLowerCase();
    return !ql || s.name.toLowerCase().includes(ql) || s.publisher.toLowerCase().includes(ql) || s.url.toLowerCase().includes(ql);
  });

  return (
    <div className="server-combobox" ref={boxRef}>
      <button type="button" className={'server-trigger' + (open ? ' open' : '')}
        onClick={e => { e.stopPropagation(); setOpen(o => !o); setQuery(''); }}>
        <div className="server-trigger-icon" style={{ background: selected ? selected.iconBg : 'hsl(var(--muted))' }}>
          {selected
            ? <span style={{ color: selected.iconColor, fontSize: 13, fontWeight: 800 }}>{selected.icon}</span>
            : <Ic name="server" size={13} />}
        </div>
        <span className={'server-trigger-label' + (selected ? '' : ' placeholder')}>{selected ? selected.name : 'Select a server…'}</span>
        <span className="server-trigger-url">{selected ? selected.url : ''}</span>
        <span style={{ marginLeft: 'auto', display: 'flex' }}><Ic name="chevron-down" size={14} /></span>
      </button>
      {open && (
        <div className="server-dropdown" onClick={e => e.stopPropagation()}>
          <div className="server-search">
            <Ic name="search" size={13} />
            <input ref={inputRef} type="text" placeholder="Search by name, publisher…" value={query}
              onChange={e => setQuery(e.target.value)} autoComplete="off" />
          </div>
          <div className="server-list">
            {filtered.length === 0 ? (
              <div className="server-no-results">No servers match "<strong>{query}</strong>"</div>
            ) : filtered.map(s => {
              const explicit = (s.authConfigs || []).filter(c => c.type !== 'none');
              const primary = explicit[0] || PUBLIC_AC;
              return (
                <div key={s.id} className={'server-opt' + (selected && selected.id === s.id ? ' selected' : '')}
                  onClick={() => { onSelect(s); setOpen(false); }}>
                  <div className="server-opt-icon" style={{ background: s.iconBg, color: s.iconColor }}>{s.icon}</div>
                  <div className="server-opt-body">
                    <div className="server-opt-name">{s.name}</div>
                    <div className="server-opt-url">{s.url}</div>
                  </div>
                  <div className="server-opt-badges">
                    <AuthBadge type={primary.type} />
                    {explicit.length > 1 && <span className="auth-badge auth-none">+{explicit.length - 1}</span>}
                  </div>
                </div>
              );
            })}
          </div>
        </div>
      )}
    </div>
  );
}

/* ── Auth section ── */
function AuthSection({ server, activeAc, onConfigChange, serverName }) {
  const [oauthState, setOauthState] = useState('idle'); // idle | connecting | connected
  const [keyVisible, setKeyVisible] = useState(false);
  const [apiKey, setApiKey] = useState('');

  useEffect(() => { setOauthState('idle'); setKeyVisible(false); setApiKey(''); }, [activeAc && activeAc.name, server && server.id]);

  if (!server || !activeAc) return null;
  const list = connectMechs(server);

  const acDesc = activeAc.type === 'oauth'
    ? 'OAuth authentication is required. Click Connect to authorize.'
    : activeAc.type === 'key'
      ? 'A pre-configured ' + (activeAc.injectVia === 'query' ? 'query parameter' : 'header') + ' will be sent with every request to this MCP server.'
      : 'No authentication required — this server is publicly accessible.';

  const meta = AUTH_META[activeAc.type] || AUTH_META.none;
  const headIcon = meta.icon;
  const headBg = meta.iconBg;
  const headColor = meta.iconColor;
  const headLabel = activeAc.type === 'key' ? 'Header / Query' : meta.label;
  const authHost = activeAc.authEndpoint ? new URL(activeAc.authEndpoint).hostname : '';
  const viaLabel = activeAc.injectVia === 'query' ? 'query param' : 'header';

  const simulateOAuth = () => { setOauthState('connecting'); setTimeout(() => setOauthState('connected'), 1800); };

  return (
    <div className="bf-section">
      <div className="bf-section-title">Authentication</div>
      <div className="bf-field">
        <label className="bf-label"><span className="bf-label-text">Auth Configuration</span></label>
        <select className="bf-select" value={activeAc.name} onChange={e => onConfigChange(e.target.value)}>
          {list.map(cfg => {
            const typeLabel = cfg.type === 'oauth' ? 'OAuth' : cfg.type === 'key' ? 'Header / Query Key' : 'Public / No Auth';
            return <option key={cfg.name} value={cfg.name}>{cfg.builtin ? 'Public — no auth' : `${cfg.name} · ${typeLabel}`}</option>;
          })}
        </select>
        <div className="bf-hint">{acDesc}</div>
      </div>
      <div className="auth-detail-box">
        <div className="adb-header">
          <div className="adb-header-icon" style={{ background: headBg }}>
            <Ic name={headIcon} size={13} color={headColor} />
          </div>
          <span className="adb-header-name">{activeAc.builtin ? 'public' : activeAc.name}</span>
          <span className={'auth-badge ' + meta.cls}>
            <Ic name={headIcon} size={10} />{headLabel}
          </span>
        </div>
        <div className="adb-body">
          {activeAc.type === 'oauth' && (<>
            <div className="adb-meta">
              <div className="adb-meta-row"><span className="adb-meta-key">Config:</span><span className="adb-meta-val">{activeAc.name}</span></div>
              <div className="adb-meta-row"><span className="adb-meta-key">Type:</span><span className="adb-meta-val">OAuth</span></div>
              <div className="adb-meta-row"><span className="adb-meta-key">Auth Server:</span><span className="adb-meta-val">{authHost}</span></div>
            </div>
            {oauthState === 'idle' && (
              <button className="bf-btn bf-btn-primary" onClick={simulateOAuth} style={{ gap: 8 }}><Ic name="external-link" size={15} /> Connect</button>
            )}
            {oauthState === 'connecting' && (
              <div className="oauth-connecting"><div className="oauth-spinner"></div><span style={{ fontSize: 13, fontWeight: 500 }}>Redirecting to authorization…</span></div>
            )}
            {oauthState === 'connected' && (
              <div className="oauth-connected-state">
                <div className="ocs-icon"><Ic name="check" size={15} /></div>
                <div className="ocs-body">
                  <div className="ocs-name">Authorized successfully</div>
                  <div className="ocs-sub">Bodhi is connected to {serverName} on your behalf</div>
                </div>
                <button className="ocs-revoke" onClick={() => setOauthState('idle')}>Revoke</button>
              </div>
            )}
          </>)}

          {activeAc.type === 'key' && (<>
            <div className="adb-meta" style={{ marginBottom: 12 }}>
              <div className="adb-meta-row"><span className="adb-meta-key">Config:</span><span className="adb-meta-val">{activeAc.name}</span></div>
              <div className="adb-meta-row"><span className="adb-meta-key">Inject via:</span><span className="adb-meta-val">{viaLabel}</span></div>
            </div>
            <div className="bf-field" style={{ marginBottom: 0 }}>
              <div className="bf-label" style={{ marginBottom: 5 }}>
                <span className="bf-label-text" style={{ fontWeight: 500, color: 'hsl(var(--muted-foreground))' }}>
                  {activeAc.keyName || 'x-api-key'} <span style={{ fontWeight: 400, opacity: .7 }}>({viaLabel})</span>
                </span>
              </div>
              <div className="key-input-wrap">
                <input type={keyVisible ? 'text' : 'password'} placeholder={activeAc.keyPlaceholder || `Enter ${activeAc.keyName || 'API key'} value`}
                  value={apiKey} onChange={e => setApiKey(e.target.value)} />
                <button className="key-toggle" onClick={() => setKeyVisible(v => !v)} title="Show/hide">
                  <Ic name={keyVisible ? 'eye-off' : 'eye'} size={14} />
                </button>
              </div>
            </div>
          </>)}

          {activeAc.type === 'none' && (<>
            <div className="adb-meta">
              <div className="adb-meta-row"><span className="adb-meta-key">Config:</span><span className="adb-meta-val">public</span></div>
              <div className="adb-meta-row"><span className="adb-meta-key">Type:</span><span className="adb-meta-val">Public / No Auth</span></div>
            </div>
            <div className="public-card" style={{ marginTop: 0 }}>
              <Ic name="unlock" size={16} />
              <div className="public-card-text">No authentication required. If the server rejects public access, pick another mechanism.</div>
            </div>
          </>)}
        </div>
      </div>
    </div>
  );
}

/* ── Main app ── */
function NewInstanceApp() {
  const params = new URLSearchParams(window.location.search);
  const isEdit = params.get('edit') === '1';

  const [selectedServer, setSelectedServer] = useState(null);
  const [activeAcName, setActiveAcName] = useState(null);
  const [name, setName] = useState('');
  const [slug, setSlug] = useState('');
  const [desc, setDesc] = useState('');
  const [enabled, setEnabled] = useState(true);
  const [slugDirty, setSlugDirty] = useState(false);
  const [nameError, setNameError] = useState(false);
  const [submitting, setSubmitting] = useState(false);

  useEffect(() => {
    const serverId = params.get('server');
    const authName = params.get('auth');
    const presetName = params.get('name');
    if (!serverId) return;
    const s = CONNECTABLE_SERVERS.find(x => x.id === serverId);
    if (!s) return;
    const list = connectMechs(s);
    const ac = list.find(c => c.name === authName) || list.find(c => c.type === authName) || list[0];
    setSelectedServer(s);
    setActiveAcName(ac.name);
    const n = presetName || s.id;
    setName(n);
    setSlug(slugify(n));
  }, []);

  const activeAc = selectedServer
    ? (connectMechs(selectedServer).find(c => c.name === activeAcName) || connectMechs(selectedServer)[0])
    : null;

  function applySelection(s, acName) {
    setSelectedServer(s);
    setActiveAcName(acName || connectMechs(s)[0].name);
    setName(prev => prev || s.id);
    setSlug(prev => (slugDirty ? prev : (prev || s.id)));
  }
  const onSelectServer = s => applySelection(s, connectMechs(s)[0].name);

  const onNameInput = v => { setName(v); setNameError(false); if (!slugDirty) setSlug(slugify(v)); };

  const canSubmit = !!selectedServer && !!name.trim() && !!slug.trim();

  const handleSubmit = () => {
    if (!name.trim() || !slug.trim()) { setNameError(true); return; }
    if (!selectedServer) return;
    setSubmitting(true);
    setTimeout(() => {
      window.location.href = `MCP-Playground-Overview.html?instance=${isEdit ? params.get('instance') : 'new-' + selectedServer.id}&name=${encodeURIComponent(name)}&server=${selectedServer.id}`;
    }, 1200);
  };

  return (
    <AppShell
      section="mcp" subPage="new-mcp" resizeKey="mcp"
      breadcrumb={[
        { label: 'Bodhi', href: 'Chat.html' },
        { label: 'MCP', href: 'MCP-My-MCPs.html' },
        { label: 'My MCPs', href: 'MCP-My-MCPs.html' },
        { label: isEdit ? 'Edit Instance' : 'New Instance', current: true },
      ]}
      contentClass="flush" mainScroll={false}
    >
      <div className="bf-scroll">
        <div className="bf-container">
          <div className="bf-card">
            <div className="bf-card-head">
              <h1 className="bf-card-title">{isEdit ? 'Edit MCP Instance' : 'New MCP Instance'}</h1>
              <p className="bf-card-sub">{isEdit ? 'Update this instance’s name, authentication, or description.' : 'Connect to an MCP server and set up your personal instance.'}</p>
            </div>
            <div className="bf-card-body">
              <div className="bf-field">
                <label className="bf-label"><span className="bf-label-text">MCP Server</span><span className="bf-req">*</span></label>
                <ServerCombobox selected={selectedServer} onSelect={onSelectServer} locked={isEdit} />
                <div className="bf-hint">{isEdit ? 'The server can’t be changed for an existing instance.' : 'Choose an MCP server to connect to'}</div>
              </div>

              <div className="bf-divider"></div>

              <div className="bf-section">
                <div className="bf-section-title">Instance Details</div>

                <div className="bf-field">
                  <label className="bf-label"><span className="bf-label-text">Name</span><span className="bf-req">*</span></label>
                  <input className={'bf-input' + (nameError ? ' is-error' : '')} placeholder="e.g. my-notion"
                    value={name} onChange={e => onNameInput(e.target.value)} />
                  <div className="bf-hint">A friendly name to identify this connection</div>
                </div>

                <div className="bf-field">
                  <label className="bf-label"><span className="bf-label-text">Slug</span><span className="bf-req">*</span></label>
                  <input className="bf-input bf-input-mono" placeholder="auto-derived from name"
                    value={slug} onChange={e => { setSlug(e.target.value); setSlugDirty(true); }} />
                  <div className="bf-hint">Unique identifier — letters, numbers, and hyphens only</div>
                </div>

                <div className="bf-field">
                  <label className="bf-label"><span className="bf-label-text">Description</span><span className="bf-optional">Optional</span></label>
                  <textarea className="bf-textarea" placeholder="Describe what this MCP instance is used for…"
                    value={desc} onChange={e => setDesc(e.target.value)}></textarea>
                </div>

                <div className="bf-toggle-row">
                  <div className="bf-toggle-body">
                    <div className="bf-toggle-label">Enable MCP</div>
                    <div className="bf-toggle-desc">Make this MCP instance available for use</div>
                  </div>
                  <div className={'bf-switch' + (enabled ? ' on' : '')} onClick={() => setEnabled(v => !v)}></div>
                </div>
              </div>

              {selectedServer && <div className="bf-divider"></div>}

              <AuthSection server={selectedServer} activeAc={activeAc}
                serverName={selectedServer ? selectedServer.name : ''}
                onConfigChange={setActiveAcName} />
            </div>

            <div className="bf-footer">
              <div className="bf-footer-spacer"></div>
              <button className="bf-btn bf-btn-ghost" onClick={() => history.back()}>Cancel</button>
              <button className="bf-btn bf-btn-primary" onClick={handleSubmit} disabled={submitting || !canSubmit}>
                {submitting
                  ? <><svg style={{ width: 14, height: 14, animation: 'mcp-spin 0.8s linear infinite' }} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><path d="M21 12a9 9 0 1 1-6.219-8.56"/></svg> {isEdit ? 'Saving…' : 'Creating…'}</>
                  : <><Ic name={isEdit ? 'check' : 'plug'} size={13} /> {isEdit ? 'Save Changes' : 'Create MCP'}</>}
              </button>
            </div>
          </div>
        </div>
      </div>
    </AppShell>
  );
}

ReactDOM.createRoot(document.getElementById('root')).render(<NewInstanceApp />);
