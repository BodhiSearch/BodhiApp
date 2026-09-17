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
  { id:'tok_1', name:'prod-inference',      scope:'user',
    listModels:true,  models:'all',
    listMcps:true,    mcps:'all',
    created:'Nov 15, 2025', lastUsed:'2 hours ago', status:'active' },

  { id:'tok_2', name:'research-copilot',    scope:'user',
    listModels:true,  models:'specific', modelNames:['llama3.1:70b','qwen2.5:7b','nomic-embed-text'],
    listMcps:false,   mcps:'specific',   mcpNames:['brave-search','github'],
    created:'Oct 22, 2025', lastUsed:'Yesterday', status:'active' },

  { id:'tok_3', name:'automation-pipeline', scope:'power',
    listModels:true,  models:'all',
    listMcps:true,    mcps:'all',
    created:'Sep 5, 2025', lastUsed:'3 weeks ago', status:'active' },

  { id:'tok_4', name:'dev-test',            scope:'user',
    listModels:false, models:'specific', modelNames:['llama3.2:3b'],
    listMcps:false,   mcps:'none',       mcpNames:[],
    created:'Aug 12, 2025', lastUsed:'Never', status:'inactive' },

  { id:'tok_5', name:null,                  scope:'user',
    listModels:false, models:'all',
    listMcps:false,   mcps:'specific',   mcpNames:['filesystem'],
    created:'Dec 1, 2025', lastUsed:'5 min ago', status:'active' },
];

function scopeLabel(scope) { return scope === 'power' ? 'scope_token_power_user' : 'scope_token_user'; }

/* ── Listing permission line — mirrors the New Token form's "List all…" toggle ── */
function ListingLine({ on, label, code }) {
  if (!on) return null;
  return (
    <div className="dp-perm-row on">
      <span className="dp-perm-k"><Ic name="list" size={13} /> {label} <code className="dp-perm-code">{code}</code></span>
    </div>
  );
}

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
        <span className={`token-name${!token.name ? ' unnamed' : ''}`}>{token.name || 'Unnamed token'}</span>
      </div>

      <div className="tk-role">
        <span className={token.scope === 'power' ? 'scope-power' : 'scope-user'}>{scopeLabel(token.scope)}</span>
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
        <span className={`status-chip ${active ? 'status-active' : 'status-revoked'}`}><i className="status-dot" />{active ? 'Active' : 'Inactive'}</span>
        <span className={token.scope === 'power' ? 'scope-power' : 'scope-user'}>{scopeLabel(token.scope)}</span>
      </div>

      <div className="dp-body">
        {/* MODELS — listing permission + inference grant (mirrors New Token form) */}
        <div className="dp-section">
          <div className="dp-sec-lbl">Models</div>
          <ListingLine on={token.listModels} label="List all models" code="/v1/models" />
          <div className="dp-perm-sub">Inference</div>
          {token.models === 'all' ?
            <div className="dp-resource"><Ic name="cpu" size={14} /> All models</div> :
            (token.modelNames || []).length ?
            <div className="dp-chips">{token.modelNames.map(m => <span key={m} className="dp-chip"><Ic name="cpu" size={12} /> {m}</span>)}</div> :
            <div className="dp-resource" style={{ opacity: .55 }}><Ic name="cpu" size={14} /> No inference access</div>
          }
        </div>

        {/* MCP SERVERS — listing permission + connect grant */}
        <div className="dp-section">
          <div className="dp-sec-lbl">MCP servers</div>
          <ListingLine on={token.listMcps} label="List all MCPs" code="/v1/mcps" />
          <div className="dp-perm-sub">Connect</div>
          {token.mcps === 'all' ?
            <div className="dp-resource"><Ic name="plug" size={14} /> All MCPs</div> :
            (token.mcpNames || []).length ?
            <div className="dp-chips">{token.mcpNames.map(m => <span key={m} className="dp-chip"><Ic name="plug" size={12} /> {m}</span>)}</div> :
            <div className="dp-resource" style={{ opacity: .55 }}><Ic name="plug" size={14} /> No connections</div>
          }
        </div>

        {/* DETAILS */}
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
        {confirmDelete ? (
          <button className="dp-btn dp-btn-danger is-confirm" onClick={() => onDelete(token.id)}><Ic name="trash-2" size={14} /> Confirm delete</button>
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
          <ListView className="tk-listview" head={
            <>
              <div className="tk-icon"></div>
              <div className="tk-id l-lh">Token</div>
              <div className="tk-role l-lh">Role</div>
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
        { label: 'Bodhi', href: 'Chat.html' },
        { label: 'Access Tokens', href: 'Tokens-API.html' },
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
