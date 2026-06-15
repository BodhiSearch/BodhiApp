/* ═══════════════════════════════════════════════════
   APP TOKENS — List page React app
═══════════════════════════════════════════════════ */
const { useState, useEffect, useRef } = React;

const SAMPLE_TOKENS = [
  { id:'tok_1', name:'prod-inference',     scope:'user',  models:'all',      mcps:'all',      created:'Nov 15, 2025', lastUsed:'2 hours ago',   status:'active' },
  { id:'tok_2', name:'research-copilot',   scope:'user',  models:'specific', modelCount:3,    mcps:'specific', mcpCount:2, created:'Oct 22, 2025', lastUsed:'Yesterday',     status:'active' },
  { id:'tok_3', name:'automation-pipeline',scope:'power', models:'all',      mcps:'all',      created:'Sep 5, 2025',  lastUsed:'3 weeks ago',   status:'active' },
  { id:'tok_4', name:'dev-test',           scope:'user',  models:'specific', modelCount:1,    mcps:'none',     mcpCount:0, created:'Aug 12, 2025', lastUsed:'Never',         status:'revoked' },
  { id:'tok_5', name:null,                 scope:'user',  models:'all',      mcps:'specific', mcpCount:1,      created:'Dec 1, 2025',  lastUsed:'5 min ago',     status:'active' },
];

function Icon({ name, size = 14 }) {
  const ref = useRef(null);
  useEffect(() => {
    if (!ref.current) return;
    ref.current.innerHTML = '';
    const el = document.createElement('i');
    el.setAttribute('data-lucide', name);
    ref.current.appendChild(el);
    if (window.lucide) window.lucide.createIcons({ nodes: [el] });
  }, [name]);
  return <span ref={ref} style={{ display:'inline-flex', width:size, height:size, alignItems:'center', justifyContent:'center', flexShrink:0 }} />;
}

function ModelsSummary({ models, modelCount }) {
  if (models === 'all') return <span className="access-pill"><Icon name="cpu" size={11} /> All models</span>;
  if (models === 'specific') return <span className="access-pill"><Icon name="cpu" size={11} /> {modelCount} model{modelCount !== 1 ? 's' : ''}</span>;
  return null;
}

function McpSummary({ mcps, mcpCount }) {
  if (mcps === 'all')      return <span className="access-pill"><Icon name="plug" size={11} /> All MCPs</span>;
  if (mcps === 'none')     return <span className="access-pill" style={{opacity:.5}}><Icon name="plug" size={11} /> No MCPs</span>;
  if (mcps === 'specific') return <span className="access-pill"><Icon name="plug" size={11} /> {mcpCount} MCP{mcpCount !== 1 ? 's' : ''}</span>;
  return null;
}

function TokenCard({ token, onRevoke, onDelete }) {
  const [confirming, setConfirming] = useState(false);

  useEffect(() => { if (window.lucide) window.lucide.createIcons(); });

  return (
    <div className={`token-card${token.status === 'revoked' ? ' revoked' : ''}`} style={{flexDirection:'column',alignItems:'stretch',gap:0,padding:0}}>
      <div style={{display:'flex',alignItems:'center',gap:14,padding:'14px 16px'}}>
        {/* Key icon */}
        <div className="token-key-icon">
          <Icon name={token.status === 'revoked' ? 'key' : 'key-round'} size={16} />
        </div>

        {/* Identity */}
        <div className="token-identity">
          <div className={`token-name${!token.name ? ' unnamed' : ''}`}>
            {token.name || 'Unnamed token'}
          </div>
          <div className="token-meta">
            <span className={token.scope === 'power' ? 'scope-power' : 'scope-user'}>
              {token.scope === 'power' ? 'scope_token_power_user' : 'scope_token_user'}
            </span>
            <span style={{color:'hsl(var(--border))',fontSize:12}}>·</span>
            <ModelsSummary models={token.models} modelCount={token.modelCount} />
            <span style={{color:'hsl(var(--border))',fontSize:12}}>·</span>
            <McpSummary mcps={token.mcps} mcpCount={token.mcpCount} />
          </div>
        </div>

        {/* Divider */}
        <div className="token-divider"></div>

        {/* Dates */}
        <div className="token-dates">
          <div className="token-date-row">
            <span className="token-date-label">Created</span>
            <span>{token.created}</span>
          </div>
          <div className="token-date-row">
            <span className="token-date-label">Last used</span>
            <span>{token.lastUsed}</span>
          </div>
        </div>

        {/* Status + actions */}
        <div className="token-actions">
          <span className={`status-chip ${token.status === 'active' ? 'status-active' : 'status-revoked'}`}>
            {token.status === 'active' ? '● active' : '○ revoked'}
          </span>
          {token.status === 'active' && (
            <button className="btn-revoke" onClick={() => setConfirming(true)}>
              <Icon name="shield-off" size={12} /> Revoke
            </button>
          )}
          <button className="btn-delete" title="Delete token" onClick={() => onDelete(token.id)}>
            <Icon name="trash-2" size={13} />
          </button>
        </div>
      </div>

      {/* Inline revoke confirm */}
      {confirming && (
        <div className="revoke-confirm">
          <span className="revoke-confirm-text">Revoke this token? Any apps using it will lose access immediately.</span>
          <button className="btn-ghost" style={{height:28,fontSize:12}} onClick={() => setConfirming(false)}>Cancel</button>
          <button style={{height:28,padding:'0 12px',borderRadius:7,background:'var(--c-saffron-text)',color:'#fff',border:'none',fontSize:12,fontWeight:700,cursor:'pointer',fontFamily:'inherit'}}
            onClick={() => { onRevoke(token.id); setConfirming(false); }}>
            Yes, revoke
          </button>
        </div>
      )}
    </div>
  );
}

function AppTokensApp() {
  const [tokens,  setTokens]  = useState(SAMPLE_TOKENS);
  const [filter,  setFilter]  = useState('all');
  const [search,  setSearch]  = useState('');

  useEffect(() => { if (window.lucide) window.lucide.createIcons(); });

  const handleRevoke = id => setTokens(p => p.map(t => t.id === id ? { ...t, status:'revoked' } : t));
  const handleDelete = id => setTokens(p => p.filter(t => t.id !== id));

  const counts = {
    all:     tokens.length,
    active:  tokens.filter(t => t.status === 'active').length,
    revoked: tokens.filter(t => t.status === 'revoked').length,
  };

  const visible = tokens.filter(t => {
    if (filter === 'active'  && t.status !== 'active')  return false;
    if (filter === 'revoked' && t.status !== 'revoked') return false;
    if (search) {
      const q = search.toLowerCase();
      if (!(t.name || '').toLowerCase().includes(q) && !t.id.includes(q)) return false;
    }
    return true;
  });

  return (
    <div className="app">
      <BodhiSidebar section="api-keys" subPage="app-tokens" />
      <main className="main">

        {/* Topbar */}
        <div className="topbar">
          <nav className="breadcrumb">
            <a className="bc-seg" href="#">Bodhi</a>
            <Icon name="chevron-right" size={10} />
            <span className="bc-current">API Keys</span>
            <Icon name="chevron-right" size={10} />
            <span className="bc-current">App Tokens</span>
          </nav>
          <div className="topbar-actions">
            <a href="New App Token.html">
              <button className="btn-accent">
                <Icon name="plus" size={14} /> New Token
              </button>
            </a>
          </div>
        </div>

        {/* Body */}
        <div className="page-body">
          <div className="page-body-inner">

            <div className="page-header">
              <div className="page-header-text">
                <div className="page-title">App Tokens</div>
                <div className="page-subtitle">Scoped API tokens for programmatic access to the Bodhi API.</div>
              </div>
            </div>

            {/* Toolbar */}
            <div className="toolbar">
              <div className="filter-tabs">
                {[
                  { id:'all',     label:'All',     count: counts.all },
                  { id:'active',  label:'Active',  count: counts.active },
                  { id:'revoked', label:'Revoked', count: counts.revoked },
                ].map(tab => (
                  <button key={tab.id} className={`filter-tab${filter === tab.id ? ' active' : ''}`} onClick={() => setFilter(tab.id)}>
                    {tab.label}
                    <span className="tab-count">{tab.count}</span>
                  </button>
                ))}
              </div>
              <div className="toolbar-spacer"></div>
              <div className="search-wrap">
                <span className="search-icon"><Icon name="search" size={12} /></span>
                <input className="search-input" placeholder="Search tokens…" value={search} onChange={e => setSearch(e.target.value)} />
              </div>
            </div>

            {/* List */}
            {visible.length === 0 ? (
              <div className="empty-state">
                <div className="empty-icon"><Icon name="key-round" size={32} /></div>
                <div className="empty-title">{search ? 'No tokens match your search' : 'No tokens yet'}</div>
                <div className="empty-sub">{search ? 'Try a different search term.' : 'Create your first token to get started.'}</div>
              </div>
            ) : (
              <div className="token-list">
                {visible.map(token => (
                  <TokenCard key={token.id} token={token} onRevoke={handleRevoke} onDelete={handleDelete} />
                ))}
              </div>
            )}

          </div>
        </div>

      </main>
    </div>
  );
}

ReactDOM.createRoot(document.getElementById('app-root')).render(<AppTokensApp />);
