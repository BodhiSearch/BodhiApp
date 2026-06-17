/* ═══════════════════════════════════════════════════
   BODHI MCP — NEW INSTANCE (form, on AppShell)
   bodhi-mcp-new-instance-app.jsx  (load after bodhi-app-shell.jsx)
═══════════════════════════════════════════════════ */
const { useState, useEffect, useRef } = React;
const Ic = ShellIcon;

const SERVERS = [
  { id:'notion', name:'Notion', publisher:'Notion Labs', icon:'N', iconBg:'#000', iconColor:'#fff',
    url:'https://mcp.notion.com/mcp',
    authConfigs:[{type:'oauth',name:'oauth-default',detail:'Dynamic registration · mcp.notion.com', authEndpoint:'https://mcp.notion.com/authorize', tokenEndpoint:'https://mcp.notion.com/token'}] },
  { id:'linear', name:'Linear', publisher:'Linear', icon:'L', iconBg:'#5E6AD2', iconColor:'#fff',
    url:'https://mcp.linear.app/mcp',
    authConfigs:[{type:'oauth',name:'oauth-default',detail:'Dynamic registration · mcp.linear.app', authEndpoint:'https://mcp.linear.app/authorize', tokenEndpoint:'https://mcp.linear.app/token'}] },
  { id:'slack', name:'Slack', publisher:'Slack', icon:'S', iconBg:'#4A154B', iconColor:'#fff',
    url:'https://mcp.slack.com/mcp',
    authConfigs:[{type:'oauth',name:'oauth-default',detail:'Dynamic registration · mcp.slack.com', authEndpoint:'https://mcp.slack.com/authorize', tokenEndpoint:'https://mcp.slack.com/token'}] },
  { id:'exa', name:'Exa Search', publisher:'Exa Labs', icon:'E', iconBg:'#1C1C1C', iconColor:'#fff',
    url:'https://mcp.exa.ai/mcp',
    authConfigs:[{type:'key',name:'apikey-default',detail:'Header: x-api-key', keyName:'x-api-key', keyPlaceholder:'exa-sk-...'}] },
  { id:'github', name:'GitHub', publisher:'GitHub', icon:'G', iconBg:'#24292e', iconColor:'#fff',
    url:'https://mcp.github.com/mcp',
    authConfigs:[{type:'oauth',name:'oauth-default',detail:'Dynamic registration · mcp.github.com', authEndpoint:'https://mcp.github.com/authorize', tokenEndpoint:'https://mcp.github.com/token'}] },
  { id:'supabase', name:'Supabase', publisher:'Supabase', icon:'▲', iconBg:'#3ECF8E', iconColor:'#000',
    url:'https://mcp.supabase.com/mcp',
    authConfigs:[{type:'key',name:'apikey-default',detail:'Header: Authorization', keyName:'Authorization', keyPlaceholder:'Bearer sbp_...'}] },
  { id:'deepwiki', name:'DeepWiki', publisher:'Dexa', icon:'D', iconBg:'#6C47FF', iconColor:'#fff',
    url:'https://mcp.deepwiki.com/mcp',
    authConfigs:[{type:'none',name:'public',detail:'No authentication required'}] },
  { id:'gmail', name:'Gmail', publisher:'Google', icon:'G', iconBg:'#EA4335', iconColor:'#fff',
    url:'https://mcp.google.com/gmail',
    authConfigs:[{type:'oauth',name:'oauth-default',detail:'Dynamic registration · mcp.google.com', authEndpoint:'https://mcp.google.com/authorize', tokenEndpoint:'https://mcp.google.com/token'}] },
];

const slugify = str => str.toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/^-|-$/g, '');

function AuthBadge({ type }) {
  if (type === 'oauth') return <span className="auth-badge auth-oauth"><Ic name="lock" size={10} />OAuth</span>;
  if (type === 'key')   return <span className="auth-badge auth-key"><Ic name="key" size={10} />Key</span>;
  return <span className="auth-badge auth-none"><Ic name="unlock" size={10} />Public</span>;
}

/* ── Server combobox ── */
function ServerCombobox({ selected, onSelect }) {
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

  const filtered = SERVERS.filter(s => {
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
            ) : filtered.map(s => (
              <div key={s.id} className={'server-opt' + (selected && selected.id === s.id ? ' selected' : '')}
                onClick={() => { onSelect(s); setOpen(false); }}>
                <div className="server-opt-icon" style={{ background: s.iconBg, color: s.iconColor }}>{s.icon}</div>
                <div className="server-opt-body">
                  <div className="server-opt-name">{s.name}</div>
                  <div className="server-opt-url">{s.url}</div>
                </div>
                <div className="server-opt-badges"><AuthBadge type={s.authConfigs[0].type} /></div>
              </div>
            ))}
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

  // reset transient auth state when the config changes
  useEffect(() => { setOauthState('idle'); setKeyVisible(false); setApiKey(''); }, [activeAc && activeAc.name, server && server.id]);

  if (!server || !activeAc) return null;

  const acDesc = activeAc.type === 'oauth'
    ? 'OAuth authentication is required. Click Connect to authorize.'
    : activeAc.type === 'key'
      ? 'A pre-configured header will be sent with every request to this MCP server.'
      : 'No authentication required. This server is publicly accessible.';

  const headIcon = activeAc.type === 'oauth' ? 'lock' : activeAc.type === 'key' ? 'key' : 'unlock';
  const headBg = activeAc.type === 'oauth' ? 'var(--c-indigo-bg)' : activeAc.type === 'key' ? 'var(--c-saffron-bg)' : 'var(--c-leaf-bg)';
  const headColor = activeAc.type === 'oauth' ? 'var(--c-indigo-text)' : activeAc.type === 'key' ? 'var(--c-saffron-text)' : 'var(--c-leaf-text)';
  const headLabel = activeAc.type === 'oauth' ? 'OAuth' : activeAc.type === 'key' ? 'Header / Query Params' : 'Public';
  const authHost = activeAc.authEndpoint ? new URL(activeAc.authEndpoint).hostname : '';

  const simulateOAuth = () => {
    setOauthState('connecting');
    setTimeout(() => setOauthState('connected'), 1800);
  };

  return (
    <div className="bf-section">
      <div className="bf-section-title">Authentication</div>
      <div className="bf-field">
        <label className="bf-label"><span className="bf-label-text">Auth Configuration</span></label>
        <select className="bf-select" value={activeAc.name} onChange={e => onConfigChange(e.target.value)}>
          {server.authConfigs.map(cfg => {
            const typeLabel = cfg.type === 'oauth' ? 'OAuth' : cfg.type === 'key' ? 'Header / Query Params' : 'Public / No Auth';
            return <option key={cfg.name} value={cfg.name}>{cfg.name} [{typeLabel}]</option>;
          })}
        </select>
        <div className="bf-hint">{acDesc}</div>
      </div>
      <div className="auth-detail-box">
        <div className="adb-header">
          <div className="adb-header-icon" style={{ background: headBg }}>
            <Ic name={headIcon} size={13} color={headColor} />
          </div>
          <span className="adb-header-name">{activeAc.name}</span>
          <span className={'auth-badge ' + (activeAc.type === 'oauth' ? 'auth-oauth' : activeAc.type === 'key' ? 'auth-key' : 'auth-none')}>
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
            </div>
            <div className="bf-field" style={{ marginBottom: 0 }}>
              <div className="bf-label" style={{ marginBottom: 5 }}>
                <span className="bf-label-text" style={{ fontWeight: 500, color: 'hsl(var(--muted-foreground))' }}>
                  {activeAc.keyName || 'x-api-key'} <span style={{ fontWeight: 400, opacity: .7 }}>(header)</span>
                </span>
              </div>
              <div className="key-input-wrap">
                <input type={keyVisible ? 'text' : 'password'} placeholder={`Enter ${activeAc.keyName || 'API key'} value`}
                  value={apiKey} onChange={e => setApiKey(e.target.value)} />
                <button className="key-toggle" onClick={() => setKeyVisible(v => !v)} title="Show/hide">
                  <Ic name={keyVisible ? 'eye-off' : 'eye'} size={14} />
                </button>
              </div>
            </div>
          </>)}

          {activeAc.type === 'none' && (<>
            <div className="adb-meta">
              <div className="adb-meta-row"><span className="adb-meta-key">Config:</span><span className="adb-meta-val">{activeAc.name}</span></div>
              <div className="adb-meta-row"><span className="adb-meta-key">Type:</span><span className="adb-meta-val">Public / No Auth</span></div>
            </div>
            <div className="public-card" style={{ marginTop: 0 }}>
              <Ic name="unlock" size={16} />
              <div className="public-card-text">No authentication required.</div>
            </div>
          </>)}
        </div>
      </div>
    </div>
  );
}

/* ── Main app ── */
function NewInstanceApp() {
  const [selectedServer, setSelectedServer] = useState(null);
  const [activeAcName, setActiveAcName] = useState(null);
  const [name, setName] = useState('');
  const [slug, setSlug] = useState('');
  const [desc, setDesc] = useState('');
  const [enabled, setEnabled] = useState(true);
  const [slugDirty, setSlugDirty] = useState(false);
  const [nameError, setNameError] = useState(false);
  const [submitting, setSubmitting] = useState(false);

  /* read URL params once */
  useEffect(() => {
    const params = new URLSearchParams(window.location.search);
    const serverId = params.get('server');
    const authType = params.get('auth');
    if (serverId) {
      const s = SERVERS.find(x => x.id === serverId);
      if (s) {
        const ac = s.authConfigs.find(c => c.type === authType) || s.authConfigs[0];
        applySelection(s, ac.name);
      }
    }
  }, []);

  const activeAc = selectedServer
    ? (selectedServer.authConfigs.find(c => c.name === activeAcName) || selectedServer.authConfigs[0])
    : null;

  function applySelection(s, acName) {
    setSelectedServer(s);
    setActiveAcName(acName || s.authConfigs[0].name);
    setName(prev => prev || s.id);
    setSlug(prev => (slugDirty ? prev : (prev || s.id)));
  }

  const onSelectServer = s => applySelection(s, s.authConfigs[0].name);

  const onNameInput = v => {
    setName(v);
    setNameError(false);
    if (!slugDirty) setSlug(slugify(v));
  };

  const canSubmit = !!selectedServer && !!name.trim() && !!slug.trim();

  const handleSubmit = () => {
    if (!name.trim() || !slug.trim()) { setNameError(true); return; }
    if (!selectedServer) return;
    setSubmitting(true);
    setTimeout(() => {
      window.location.href = `Bodhi MCP Playground.html?instance=new-${selectedServer.id}&name=${encodeURIComponent(name)}&server=${selectedServer.id}`;
    }, 1200);
  };

  return (
    <>
    <AppShell
      section="mcp" subPage="new-mcp" resizeKey="mcp"
      breadcrumb={[
        { label: 'Bodhi', href: 'Bodhi Chat.html' },
        { label: 'MCP', href: 'Bodhi MCP Discover v2.html' },
        { label: 'Discover', href: 'Bodhi MCP Discover v2.html' },
        { label: 'New Instance', current: true },
      ]}
      contentClass="flush" mainScroll={false}
    >
      <div className="bf-scroll">
        <div className="bf-container">
          <div className="bf-page-head">
            <h1 className="bf-page-title">New MCP Instance</h1>
            <p className="bf-page-sub">Connect to an MCP server and configure your personal instance</p>
          </div>

          <div className="bf-card">
            <div className="bf-card-body">
              <div className="bf-field">
                <label className="bf-label"><span className="bf-label-text">MCP Server</span><span className="bf-req">*</span></label>
                <ServerCombobox selected={selectedServer} onSelect={onSelectServer} />
                <div className="bf-hint">Choose an MCP server to connect to</div>
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
            </div>{/* end card-body */}

            <div className="bf-footer">
              <div className="bf-footer-spacer"></div>
              <button className="bf-btn bf-btn-ghost" onClick={() => history.back()}>Cancel</button>
              <button className="bf-btn bf-btn-primary" onClick={handleSubmit} disabled={submitting || !canSubmit}>
                {submitting
                  ? <><svg style={{ width: 14, height: 14, animation: 'mcpspin 0.8s linear infinite' }} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><path d="M21 12a9 9 0 1 1-6.219-8.56"/></svg> Creating…</>
                  : <><Ic name="plug" size={13} /> Create MCP</>}
              </button>
            </div>
          </div>{/* end card */}
        </div>
      </div>
    </AppShell>
    </>
  );
}

ReactDOM.createRoot(document.getElementById('root')).render(<NewInstanceApp />);
