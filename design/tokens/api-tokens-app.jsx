/* ═══════════════════════════════════════════════════
   API TOKENS — list page + right detail panel (on AppShell)
   api-tokens-app.jsx   (programmatic tokens you create for API access)   (load after bodhi-app-shell.jsx + bodhi-list.jsx)
   Click a row → details open in the right sidepanel (rail). The primary
   control (Enabled/Disabled toggle) stays in the row and is repeated in
   the panel; Delete lives only in the panel.
═══════════════════════════════════════════════════ */
const { useState, useEffect } = React;
const Ic = ShellIcon;

const SAMPLE_TOKENS = [
  { id:'tok_1', name:'prod-inference',     scope:'user',  models:'all',      mcps:'all',      created:'Nov 15, 2025', lastUsed:'2 hours ago',   status:'active' },
  { id:'tok_2', name:'research-copilot',   scope:'user',  models:'specific', modelCount:3,    mcps:'specific', mcpCount:2, created:'Oct 22, 2025', lastUsed:'Yesterday',     status:'active' },
  { id:'tok_3', name:'automation-pipeline',scope:'power', models:'all',      mcps:'all',      created:'Sep 5, 2025',  lastUsed:'3 weeks ago',   status:'active' },
  { id:'tok_4', name:'dev-test',           scope:'user',  models:'specific', modelCount:1,    mcps:'none',     mcpCount:0, created:'Aug 12, 2025', lastUsed:'Never',         status:'inactive' },
  { id:'tok_5', name:null,                 scope:'user',  models:'all',      mcps:'specific', mcpCount:1,      created:'Dec 1, 2025',  lastUsed:'5 min ago',     status:'active' },
];

function ModelsSummary({ models, modelCount }) {
  if (models === 'all') return <span className="access-pill"><Ic name="cpu" size={11} /> All models</span>;
  if (models === 'specific') return <span className="access-pill"><Ic name="cpu" size={11} /> {modelCount} model{modelCount !== 1 ? 's' : ''}</span>;
  return null;
}

function McpSummary({ mcps, mcpCount }) {
  if (mcps === 'all')      return <span className="access-pill"><Ic name="plug" size={11} /> All MCPs</span>;
  if (mcps === 'none')     return <span className="access-pill" style={{opacity:.5}}><Ic name="plug" size={11} /> No MCPs</span>;
  if (mcps === 'specific') return <span className="access-pill"><Ic name="plug" size={11} /> {mcpCount} MCP{mcpCount !== 1 ? 's' : ''}</span>;
  return null;
}

function scopeLabel(scope) { return scope === 'power' ? 'scope_token_power_user' : 'scope_token_user'; }
function modelsText(t) { return t.models === 'all' ? 'All models' : `${t.modelCount} model${t.modelCount !== 1 ? 's' : ''}`; }
function mcpsText(t) { return t.mcps === 'all' ? 'All MCPs' : t.mcps === 'none' ? 'No MCPs' : `${t.mcpCount} MCP${t.mcpCount !== 1 ? 's' : ''}`; }

function TokenToggle({ token, onToggle, size = 'sm' }) {
  const on = token.status === 'active';
  return (
    <button
      className={`tk-switch tk-switch-${size}${on ? ' is-on' : ''}`}
      role="switch" aria-checked={on}
      title={on ? 'Disable token' : 'Enable token'}
      onClick={e => { e.stopPropagation(); onToggle(token.id); }}
    >
      <span className="tk-switch-label">{on ? 'Active' : 'Inactive'}</span>
      <span className="tk-switch-track"><span className="tk-switch-thumb" /></span>
    </button>
  );
}

function TokenRow({ token, selected, onSelect, onToggle }) {
  const inactive = token.status !== 'active';
  return (
    <ListRow className={`tk-row${inactive ? ' is-dim' : ''}`} active={selected}
             onSelect={() => onSelect(token.id)} label={`Open token ${token.name || 'Unnamed token'}`}>
      <div className="tk-icon">
        <div className="token-key-icon"><Ic name={inactive ? 'key' : 'key-round'} size={16} /></div>
      </div>

      <div className="tk-id">
        <div className={`token-name${!token.name ? ' unnamed' : ''}`}>{token.name || 'Unnamed token'}</div>
        <div className="token-meta">
          <span className={token.scope === 'power' ? 'scope-power' : 'scope-user'}>{scopeLabel(token.scope)}</span>
          <span className="tk-dot">·</span>
          <ModelsSummary models={token.models} modelCount={token.modelCount} />
          <span className="tk-dot">·</span>
          <McpSummary mcps={token.mcps} mcpCount={token.mcpCount} />
        </div>
      </div>

      <div className="tk-created"><span className="tk-date-lbl">Created</span><span className="tk-date-val">{token.created}</span></div>
      <div className="tk-used"><span className="tk-date-lbl">Last used</span><span className="tk-date-val">{token.lastUsed}</span></div>

      <div className="tk-act">
        <TokenToggle token={token} onToggle={onToggle} />
      </div>
    </ListRow>
  );
}

/* ── Rail header (railHeader slot) ── */
function TokenDetailHeader({ token, onClose }) {
  const inactive = token.status !== 'active';
  return (
    <div className="dp-head">
      <div className="dp-head-icon" style={{ background: 'var(--c-indigo-text)' }}><Ic name={inactive ? 'key' : 'key-round'} size={15} /></div>
      <div className="dp-head-body">
        <div className={`dp-head-title${token.name ? ' mono' : ''}`}>{token.name || 'Unnamed token'}</div>
        <div className="dp-head-sub">{token.id}</div>
      </div>
      <button className="dp-close" onClick={onClose} title="Close"><Ic name="x" size={15} /></button>
    </div>
  );
}

/* ── Rail body (rail slot) ── */
function TokenDetailPanel({ token, onToggle, onDelete }) {
  const [confirmDelete, setConfirmDelete] = useState(false);
  useEffect(() => { setConfirmDelete(false); }, [token && token.id]);

  const active = token.status === 'active';
  return (
    <div className="dp-panel">
      <div className="dp-status-row">
        <span className={`status-chip ${active ? 'status-active' : 'status-revoked'}`}>{active ? '● active' : '○ inactive'}</span>
        <span className={token.scope === 'power' ? 'scope-power' : 'scope-user'}>{scopeLabel(token.scope)}</span>
      </div>

      <div className="dp-body">
        <div className="dp-section">
          <div className="dp-sec-lbl">Access</div>
          <div className="dp-resource"><Ic name="cpu" size={14} /> {modelsText(token)}</div>
          <div className="dp-resource" style={{ opacity: token.mcps === 'none' ? .55 : 1 }}><Ic name="plug" size={14} /> {mcpsText(token)}</div>
        </div>

        <div className="dp-section">
          <div className="dp-sec-lbl">Details</div>
          <div className="dp-rows">
            <div className="dp-row"><span className="dp-row-k"><Ic name="hash" size={13} /> Token ID</span><span className="dp-row-v mono">{token.id}</span></div>
            <div className="dp-row"><span className="dp-row-k"><Ic name="shield" size={13} /> Scope</span><span className="dp-row-v">{token.scope === 'power' ? 'Power User' : 'User'}</span></div>
            <div className="dp-row"><span className="dp-row-k"><Ic name="calendar" size={13} /> Created</span><span className="dp-row-v">{token.created}</span></div>
            <div className="dp-row"><span className="dp-row-k"><Ic name="activity" size={13} /> Last used</span><span className="dp-row-v">{token.lastUsed}</span></div>
          </div>
        </div>
      </div>

      <div className="dp-foot">
        <div className="dp-toggle-row">
          <div className="dp-toggle-copy">
            <div className="dp-toggle-title">{active ? 'Token active' : 'Token inactive'}</div>
            <div className="dp-toggle-sub">{active ? 'Accepting API requests.' : 'Requests are rejected while disabled.'}</div>
          </div>
          <TokenToggle token={token} onToggle={onToggle} size="lg" />
        </div>
        {confirmDelete ? (
          <button className="dp-btn dp-btn-danger" style={{ borderColor: 'hsl(var(--destructive))', background: 'rgba(220,38,38,.05)', color: 'hsl(var(--destructive))' }}
                  onClick={() => onDelete(token.id)}><Ic name="trash-2" size={14} /> Confirm delete</button>
        ) : (
          <button className="dp-btn dp-btn-danger" onClick={() => setConfirmDelete(true)}><Ic name="trash-2" size={14} /> Delete token</button>
        )}
        {confirmDelete && <div className="dp-field-hint" style={{ textAlign: 'center' }}>Deleting is permanent. Click again to confirm.</div>}
      </div>
    </div>
  );
}

function AppTokensMain({ tokens, filter, setFilter, search, setSearch, counts, selId, onSelect, onToggle }) {
  const { openRail } = useShell();
  useListKeyNav();
  const select = id => { onSelect(id); openRail(); };

  const visible = tokens.filter(t => {
    if (filter === 'active'   && t.status !== 'active')  return false;
    if (filter === 'inactive' && t.status === 'active')  return false;
    if (search) {
      const q = search.toLowerCase();
      if (!(t.name || '').toLowerCase().includes(q) && !t.id.includes(q)) return false;
    }
    return true;
  });

  return (
    <div className="l-page">
      <ListToolbar
        categories={[
          { id: 'all',      label: 'All',      badge: counts.all },
          { id: 'active',   label: 'Active',   badge: counts.active },
          { id: 'inactive', label: 'Inactive', badge: counts.inactive },
        ]}
        category={filter} onCategory={setFilter}
        search={search} onSearch={setSearch} searchPlaceholder="Search tokens by name or id…" />

      <div className="l-scroll">
        {visible.length === 0 ? (
          <div className="l-empty">
            <Ic name="key-round" size={32} />
            <div className="l-empty-t">{search ? 'No tokens match your search' : 'No tokens yet'}</div>
            <div className="l-empty-s">{search ? 'Try a different search term.' : 'Create your first token to get started.'}</div>
          </div>
        ) : (
          <ListView head={
            <>
              <div className="tk-icon"></div>
              <div className="tk-id l-lh">Token</div>
              <div className="tk-created l-lh">Created</div>
              <div className="tk-used l-lh">Last used</div>
              <div className="tk-act"></div>
            </>
          }>
            {visible.map(token => (
              <TokenRow key={token.id} token={token} selected={token.id === selId} onSelect={select} onToggle={onToggle} />
            ))}
          </ListView>
        )}
      </div>
    </div>
  );
}

function AppTokensApp() {
  const [tokens, setTokens] = useState(SAMPLE_TOKENS);
  const [filter, setFilter] = useState('all');
  const [search, setSearch] = useState('');
  const [selId,  setSelId]  = useState(null);

  /* default selection on desktop */
  useEffect(() => {
    if (!window.matchMedia('(max-width:767px)').matches) setSelId(SAMPLE_TOKENS[0].id);
  }, []);

  const handleToggle = id => setTokens(p => p.map(t => t.id === id ? { ...t, status: t.status === 'active' ? 'inactive' : 'active' } : t));
  const handleDelete = id => { setTokens(p => p.filter(t => t.id !== id)); setSelId(s => s === id ? null : s); };

  const counts = {
    all:      tokens.length,
    active:   tokens.filter(t => t.status === 'active').length,
    inactive: tokens.filter(t => t.status !== 'active').length,
  };

  const selected = tokens.find(t => t.id === selId) || null;

  return (
    <AppShell
      section="api-keys" subPage="api-tokens" resizeKey="api-keys"
      contentClass="flush" mainScroll={false} railScroll={false}
      breadcrumb={[
        { label: 'Bodhi', href: 'Bodhi Chat.html' },
        { label: 'Access Tokens', href: 'API Tokens.html' },
        { label: 'API Tokens', current: true },
      ]}
      rail={selected ? <TokenDetailPanel token={selected} onToggle={handleToggle} onDelete={handleDelete} /> : null}
      railHeader={selected ? <TokenDetailHeader token={selected} onClose={() => setSelId(null)} /> : undefined}
    >
      <AppTokensMain tokens={tokens} filter={filter} setFilter={setFilter} search={search} setSearch={setSearch}
        counts={counts} selId={selId} onSelect={setSelId} onToggle={handleToggle} />
    </AppShell>
  );
}

ReactDOM.createRoot(document.getElementById('root')).render(<AppTokensApp />);
