/* ═══════════════════════════════════════════════════
   APP TOKENS — tokens issued to approved third-party apps (on AppShell)
   app-tokens-app.jsx   (load after bodhi-app-shell.jsx + bodhi-list.jsx)
   These are the live access grants minted when an App Access Request is
   approved. Click a row → details open in the right sidepanel (rail).
   The active/revoked toggle stays in the row and repeats in the panel;
   Revoke (permanent) lives only in the panel.
═══════════════════════════════════════════════════ */
const { useState, useEffect } = React;
const Ic = ShellIcon;

const SAMPLE_APP_TOKENS = [
  { id:'app_1', name:'Research Copilot', initial:'R',  color:'#3E4AA8', verified:true,  scope:'user',  models:'specific', modelNames:['GPT-4o', 'Claude 3.5 Sonnet', 'Llama 3.1 70B'], mcps:'specific', mcpNames:['Filesystem', 'GitHub'], issued:'Nov 18, 2025', lastUsed:'2 hours ago', status:'active' },
  { id:'app_2', name:'DataSync Pro',     initial:'D',  color:'#5E6AD2', verified:false, scope:'user',  models:'specific', modelNames:['GPT-4o', 'Claude 3.5 Haiku'], mcps:'specific', mcpNames:['Postgres'], issued:'Oct 25, 2025', lastUsed:'Yesterday',   status:'active' },
  { id:'app_3', name:'AutoReporter',     initial:'A',  color:'#2F7D1F', verified:true,  scope:'user',  models:'specific', modelNames:['Llama 3.1 8B'], mcps:'none',     mcpNames:[], issued:'Sep 8, 2025',  lastUsed:'3 weeks ago', status:'active' },
  { id:'app_4', name:'DevBot',           initial:'DV', color:'#0F6F67', verified:true,  scope:'power', models:'all',      mcps:'all',      modelNames:[], mcpNames:[],          issued:'Aug 14, 2025', lastUsed:'Never',       status:'revoked' },
];

function ModelsSummary({ models, modelNames }) {
  if (models === 'all') return <span className="access-pill"><Ic name="cpu" size={11} /> All models</span>;
  if (models === 'specific') { const n = (modelNames || []).length; return <span className="access-pill"><Ic name="cpu" size={11} /> {n} model{n !== 1 ? 's' : ''}</span>; }
  return null;
}

function McpSummary({ mcps, mcpNames }) {
  if (mcps === 'all')      return <span className="access-pill"><Ic name="plug" size={11} /> All MCPs</span>;
  if (mcps === 'none')     return <span className="access-pill" style={{opacity:.5}}><Ic name="plug" size={11} /> No MCPs</span>;
  if (mcps === 'specific') { const n = (mcpNames || []).length; return <span className="access-pill"><Ic name="plug" size={11} /> {n} MCP{n !== 1 ? 's' : ''}</span>; }
  return null;
}

function scopeLabel(scope) { return scope === 'power' ? 'scope_token_power_user' : 'scope_token_user'; }
function modelsText(t) { return t.models === 'all' ? 'All models' : `${(t.modelNames||[]).length} model${(t.modelNames||[]).length !== 1 ? 's' : ''}`; }
function mcpsText(t) { return t.mcps === 'all' ? 'All MCPs' : t.mcps === 'none' ? 'No MCPs' : `${(t.mcpNames||[]).length} MCP${(t.mcpNames||[]).length !== 1 ? 's' : ''}`; }

function TokenToggle({ token, onToggle, size = 'sm' }) {
  const on = token.status === 'active';
  return (
    <button
      className={`tk-switch tk-switch-${size}${on ? ' is-on' : ''}`}
      role="switch" aria-checked={on}
      title={on ? 'Set inactive' : 'Set active'}
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
             onSelect={() => onSelect(token.id)} label={`Open token ${token.name}`}>
      <div className="tk-icon">
        <div className="app-token-avatar" style={{ background: token.color }}>{token.initial}</div>
      </div>

      <div className="tk-id">
        <div className="app-token-name-row">
          <span className="app-token-name">{token.name}</span>
          {token.verified && <span className="tag tag-leaf">✓ verified</span>}
        </div>
        <div className="token-meta">
          <span className={token.scope === 'power' ? 'scope-power' : 'scope-user'}>{scopeLabel(token.scope)}</span>
          <span className="tk-dot">·</span>
          <ModelsSummary models={token.models} modelNames={token.modelNames} />
          <span className="tk-dot">·</span>
          <McpSummary mcps={token.mcps} mcpNames={token.mcpNames} />
        </div>
      </div>

      <div className="tk-created"><span className="tk-date-lbl">Issued</span><span className="tk-date-val">{token.issued}</span></div>
      <div className="tk-used"><span className="tk-date-lbl">Last used</span><span className="tk-date-val">{token.lastUsed}</span></div>

      <div className="tk-act">
        <TokenToggle token={token} onToggle={onToggle} />
      </div>
    </ListRow>
  );
}

/* ── Rail header (railHeader slot) ── */
function TokenDetailHeader({ token, onClose }) {
  return (
    <div className="dp-head">
      <div className="dp-head-icon" style={{ background: token.color }}>{token.initial}</div>
      <div className="dp-head-body">
        <div className="dp-head-title">{token.name}</div>
        <div className="dp-head-sub">{token.verified ? '✓ Verified · ' : ''}3rd-party app</div>
      </div>
      <button className="dp-close" onClick={onClose} title="Close"><Ic name="x" size={15} /></button>
    </div>
  );
}

/* ── Rail body (rail slot) ── */
function TokenDetailPanel({ token, onToggle, onDelete, onEdit }) {
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
          <div className="dp-sec-lbl">Models</div>
          {token.models === 'all' ? (
            <div className="dp-resource"><Ic name="cpu" size={14} /> All models</div>
          ) : (token.modelNames || []).length ? (
            <div className="dp-chips">
              {token.modelNames.map(m => <span key={m} className="dp-chip"><Ic name="cpu" size={12} /> {m}</span>)}
            </div>
          ) : (
            <div className="dp-resource" style={{ opacity:.55 }}><Ic name="cpu" size={14} /> No models</div>
          )}
        </div>

        <div className="dp-section">
          <div className="dp-sec-lbl">MCP servers</div>
          {token.mcps === 'all' ? (
            <div className="dp-resource"><Ic name="plug" size={14} /> All MCPs</div>
          ) : (token.mcpNames || []).length ? (
            <div className="dp-chips">
              {token.mcpNames.map(m => <span key={m} className="dp-chip"><Ic name="plug" size={12} /> {m}</span>)}
            </div>
          ) : (
            <div className="dp-resource" style={{ opacity:.55 }}><Ic name="plug" size={14} /> No MCPs</div>
          )}
        </div>

        <div className="dp-section">
          <div className="dp-sec-lbl">Details</div>
          <div className="dp-rows">
            <div className="dp-row"><span className="dp-row-k"><Ic name="shield" size={13} /> Role</span><span className="dp-row-v">{token.scope === 'power' ? 'Power User' : 'User'}</span></div>
            <div className="dp-row"><span className="dp-row-k"><Ic name="hash" size={13} /> Token ID</span><span className="dp-row-v mono">{token.id}</span></div>
            <div className="dp-row"><span className="dp-row-k"><Ic name="calendar" size={13} /> Issued</span><span className="dp-row-v">{token.issued}</span></div>
            <div className="dp-row"><span className="dp-row-k"><Ic name="activity" size={13} /> Last used</span><span className="dp-row-v">{token.lastUsed}</span></div>
          </div>
        </div>
      </div>

      <div className="dp-foot">
        <div className="dp-toggle-row">
          <div className="dp-toggle-copy">
            <div className="dp-toggle-title">{active ? 'Token active' : 'Token inactive'}</div>
            <div className="dp-toggle-sub">{active ? 'App can call the API.' : 'App requests are rejected.'}</div>
          </div>
          <TokenToggle token={token} onToggle={onToggle} size="lg" />
        </div>
        <button className="dp-btn dp-btn-outline" onClick={() => onEdit && onEdit(token.id)}><Ic name="pencil" size={14} /> Edit token</button>
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
  const select = id => { onSelect(id); openRail(); };

  const visible = tokens.filter(t => {
    if (filter === 'active'  && t.status !== 'active')  return false;
    if (filter === 'revoked' && t.status === 'active')  return false;
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
          { id: 'all',     label: 'All',     badge: counts.all },
          { id: 'active',  label: 'Active',  badge: counts.active },
          { id: 'revoked', label: 'Inactive', badge: counts.revoked },
        ]}
        category={filter} onCategory={setFilter}
        search={search} onSearch={setSearch} searchPlaceholder="Search apps by name or id…" />

      <div className="l-scroll">
        {visible.length === 0 ? (
          <div className="l-empty">
            <Ic name="layout-grid" size={32} />
            <div className="l-empty-t">{search ? 'No app tokens match your search' : 'No app tokens yet'}</div>
            <div className="l-empty-s">{search ? 'Try a different search term.' : 'App tokens are issued when an app is granted access.'}</div>
          </div>
        ) : (
          <ListView head={
            <>
              <div className="tk-icon"></div>
              <div className="tk-id l-lh">Application</div>
              <div className="tk-created l-lh">Issued</div>
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
  const [tokens, setTokens] = useState(SAMPLE_APP_TOKENS);
  const [filter, setFilter] = useState('all');
  const [search, setSearch] = useState('');
  const [selId,  setSelId]  = useState(null);

  useEffect(() => {
    if (!window.matchMedia('(max-width:767px)').matches) setSelId(SAMPLE_APP_TOKENS[0].id);
  }, []);

  const handleToggle = id => setTokens(p => p.map(t => t.id === id ? { ...t, status: t.status === 'active' ? 'revoked' : 'active' } : t));
  const handleDelete = id => { setTokens(p => p.filter(t => t.id !== id)); setSelId(s => s === id ? null : s); };
  const handleEdit = id => { window.location.href = 'New App Token.html'; };

  const counts = {
    all:     tokens.length,
    active:  tokens.filter(t => t.status === 'active').length,
    revoked: tokens.filter(t => t.status !== 'active').length,
  };

  const selected = tokens.find(t => t.id === selId) || null;

  return (
    <AppShell
      section="api-keys" subPage="app-tokens" resizeKey="api-keys"
      contentClass="flush" mainScroll={false} railScroll={false}
      breadcrumb={[
        { label: 'Bodhi', href: 'Bodhi Chat.html' },
        { label: 'Access Tokens', href: 'API Tokens.html' },
        { label: 'App Tokens', current: true },
      ]}
      rail={selected ? <TokenDetailPanel token={selected} onToggle={handleToggle} onDelete={handleDelete} onEdit={handleEdit} /> : null}
      railHeader={selected ? <TokenDetailHeader token={selected} onClose={() => setSelId(null)} /> : undefined}
    >
      <AppTokensMain tokens={tokens} filter={filter} setFilter={setFilter} search={search} setSearch={setSearch}
        counts={counts} selId={selId} onSelect={setSelId} onToggle={handleToggle} />
    </AppShell>
  );
}

ReactDOM.createRoot(document.getElementById('root')).render(<AppTokensApp />);
