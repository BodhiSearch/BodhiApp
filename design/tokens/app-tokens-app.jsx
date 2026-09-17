/* ═══════════════════════════════════════════════════
   APP TOKENS — tokens issued to approved third-party apps (on AppShell)
   app-tokens-app.jsx   (load after bodhi-app-shell.jsx + bodhi-list.jsx)

   In sync with the App Access Request domain:
     • Listing (List all models / List all MCPs) and inference/connect are
       shown as SEPARATE permissions.
     • Inference grants can be per-category (slots) and/or a flat model list.
     • App tokens are immutable: upgrading mints a NEW token and invalidates
       the old one. Invalidated tokens show status "exchanged" — no toggle,
       with a link to the replacement (?selected=<id> selects that row).
     • The detail panel's at-a-glance summary is the full record of a
       token's granted permissions.
═══════════════════════════════════════════════════ */
const { useState, useEffect } = React;
const Ic = ShellIcon;

const SAMPLE_APP_TOKENS = [
{ id: 'app_1b', name: 'Research Copilot', initial: 'R', color: '#3E4AA8', verified: true, scope: 'power',
  listModels: true, models: 'specific',
  slots: [{ label: 'Main Reasoning Model', items: ['my-qwen-long', 'claude-sonnet-4-6'] },
  { label: 'Visual Language Model', items: ['my-vision-llm'] },
  { label: 'Embedding Model', items: ['my-embed'] }],
  listMcps: true, mcps: 'specific', mcpNames: ['Exa Search', 'Notion'],
  issued: 'Today', lastUsed: '2 min ago', status: 'active', supersedes: 'app_1' },

{ id: 'app_1', name: 'Research Copilot', initial: 'R', color: '#3E4AA8', verified: true, scope: 'user',
  listModels: false, models: 'specific',
  slots: [{ label: 'Main Reasoning Model', items: ['my-qwen-long'] },
  { label: 'Visual Language Model', items: ['my-vision-llm'] },
  { label: 'Embedding Model', items: ['my-embed'] }],
  listMcps: true, mcps: 'none', mcpNames: [],
  issued: 'Nov 18, 2025', lastUsed: '1 hour ago', status: 'exchanged', exchangedFor: 'app_1b' },

{ id: 'app_2', name: 'DataSync Pro', initial: 'D', color: '#5E6AD2', verified: false, scope: 'user',
  listModels: false, models: 'specific', slots: null, modelNames: ['gpt-4o', 'claude-haiku-4-5'],
  listMcps: false, mcps: 'specific', mcpNames: ['Postgres'],
  issued: 'Oct 25, 2025', lastUsed: 'Yesterday', status: 'active' },

{ id: 'app_3', name: 'AutoReporter', initial: 'A', color: '#2F7D1F', verified: true, scope: 'user',
  listModels: true, models: 'all', slots: null, modelNames: [],
  listMcps: false, mcps: 'none', mcpNames: [],
  issued: 'Sep 8, 2025', lastUsed: '3 weeks ago', status: 'active' },

{ id: 'app_4', name: 'DevBot', initial: 'DV', color: '#0F6F67', verified: true, scope: 'power',
  listModels: true, models: 'all', slots: null, modelNames: [],
  listMcps: true, mcps: 'all', mcpNames: [],
  issued: 'Aug 14, 2025', lastUsed: 'Never', status: 'revoked' }];


function scopeLabel(scope) {return scope === 'power' ? 'scope_user_power_user' : 'scope_user_user';}
function modelCount(t) {return t.slots ? t.slots.reduce((a, s) => a + s.items.length, 0) : (t.modelNames || []).length;}

/* ── Row summary pills ── */
function ModelsSummary({ t }) {
  if (t.models === 'all') return <span className="access-pill"><Ic name="cpu" size={11} /> All models</span>;
  const n = modelCount(t);
  return <span className="access-pill"><Ic name="cpu" size={11} /> {n} model{n !== 1 ? 's' : ''}{t.listModels ? ' · lists all' : ''}</span>;
}
function McpSummary({ t }) {
  if (t.mcps === 'all') return <span className="access-pill"><Ic name="plug" size={11} /> All MCPs</span>;
  if (t.mcps === 'none') return <span className="access-pill" style={{ opacity: t.listMcps ? 1 : .5 }}><Ic name={t.listMcps ? 'list' : 'plug'} size={11} /> {t.listMcps ? 'Lists all MCPs' : 'No MCPs'}</span>;
  const n = (t.mcpNames || []).length;
  return <span className="access-pill"><Ic name="plug" size={11} /> {n} MCP{n !== 1 ? 's' : ''}{t.listMcps ? ' · lists all' : ''}</span>;
}

function TokenToggle({ token, onToggle, size = 'sm' }) {
  const on = token.status === 'active';
  return (
    <button
      className={`tk-switch tk-switch-${size}${on ? ' is-on' : ''}`}
      role="switch" aria-checked={on}
      title={on ? 'Set inactive' : 'Set active'}
      onClick={(e) => {e.stopPropagation();onToggle(token.id);}}>
      
      <span className="tk-switch-label">{on ? 'Active' : 'Inactive'}</span>
      <span className="tk-switch-track"><span className="tk-switch-thumb" /></span>
    </button>);

}

function TokenRow({ token, selected, onSelect, onToggle, onSelectId }) {
  const exchanged = token.status === 'exchanged';
  const inactive = token.status !== 'active';
  return (
    <ListRow className={`tk-row${inactive ? ' is-dim' : ''}`} active={selected}
    onSelect={() => onSelect(token.id)} label={`Open token ${token.name}`}>
      <div className="tk-icon">
        <div className="app-token-avatar" style={{ background: token.color }}>{token.initial}</div>
      </div>

      <div className="tk-id">
        <span className="app-token-name">{token.name}</span>
      </div>

      <div className="tk-role">
        <span className={token.scope === 'power' ? 'scope-power' : 'scope-user'}>{scopeLabel(token.scope)}</span>
      </div>

      <div className="tk-created"><span className="tk-date-lbl">Issued</span><span className="tk-date-val">{token.issued}</span></div>
      <div className="tk-used"><span className="tk-date-lbl">Last used</span><span className="tk-date-val">{token.lastUsed}</span></div>

      <div className="tk-act">
        {exchanged ?
        <button className="tk-replacement-link" title="Go to replacement token"
        onClick={(e) => {e.stopPropagation();onSelectId(token.exchangedFor);}}>
            Exchanged <Ic name="arrow-right" size={12} />
          </button> :

        <TokenToggle token={token} onToggle={onToggle} />
        }
      </div>
    </ListRow>);

}

/* ── Rail header ── */
function TokenDetailHeader({ token, onClose }) {
  const exchanged = token.status === 'exchanged';
  return (
    <div className="dp-head">
      <div className="dp-head-icon" style={{ background: token.color }}>{token.initial}</div>
      <div className="dp-head-body">
        <div className="dp-head-title">{token.name}{exchanged && <span className="status-chip status-exchanged" style={{ marginLeft: 8 }}><Ic name="repeat" size={10} /> Exchanged</span>}</div>
        <div className="dp-head-sub">3rd-party app</div>
      </div>
      <button className="dp-close" onClick={onClose} title="Close"><Ic name="x" size={15} /></button>
    </div>);

}

/* ── Permission line (listing) ── */
function ListingLine({ on, label, code }) {
  if (!on) return null;
  return (
    <div className="dp-perm-row on">
      <span className="dp-perm-k"><Ic name="list" size={13} /> {label} <code className="dp-perm-code">{code}</code></span>
    </div>);

}

/* ── Rail body ── */
function TokenDetailPanel({ token, onToggle, onDelete, onSelectId }) {
  const [confirmDelete, setConfirmDelete] = useState(false);
  useEffect(() => {setConfirmDelete(false);}, [token && token.id]);

  const active = token.status === 'active';
  const exchanged = token.status === 'exchanged';
  const prevTok = token.supersedes ? SAMPLE_APP_TOKENS.find((t) => t.id === token.supersedes) : null;
  const nextTok = token.exchangedFor ? SAMPLE_APP_TOKENS.find((t) => t.id === token.exchangedFor) : null;

  return (
    <div className="dp-panel">
      <div className="dp-status-row">
        <span className={`status-chip ${active ? 'status-active' : exchanged ? 'status-exchanged' : 'status-revoked'}`}>
          <i className="status-dot" />{active ? 'Active' : exchanged ? 'Exchanged' : 'Inactive'}
        </span>
        <span className={token.scope === 'power' ? 'scope-power' : 'scope-user'}>{scopeLabel(token.scope)}</span>
      </div>

      <div className="dp-body">
        {/* Exchange relationship — scrolls with the rail content */}
        {exchanged &&
        <div className="dp-exchange dp-exchange-old">
          <Ic name="repeat" size={14} />
          <span className="dp-exchange-line">
            Replaced by
            <button className="dp-exchange-link" onClick={() => onSelectId(token.exchangedFor)}>{nextTok ? nextTok.name : 'new token'} <Ic name="link" size={12} /></button>
          </span>
        </div>
        }
        {token.supersedes &&
        <div className="dp-exchange dp-exchange-new">
          <Ic name="check-circle-2" size={14} />
          <span className="dp-exchange-line">
            Exchanged for
            <button className="dp-exchange-link" onClick={() => onSelectId(token.supersedes)}>{prevTok ? prevTok.name : 'previous token'} <Ic name="link" size={12} /></button>
          </span>
        </div>
        }

        {/* MODELS */}
        <div className="dp-section">
          <div className="dp-sec-lbl">Models</div>
          <ListingLine on={token.listModels} label="List all models" code="/v1/models" />
          <div className="dp-perm-sub">Inference</div>
          {token.models === 'all' ?
          <div className="dp-resource"><Ic name="cpu" size={14} /> All models</div> :
          token.slots ?
          token.slots.map((s) =>
          <div key={s.label} className="dp-cat">
                <div className="dp-cat-lbl">{s.label}</div>
                <div className="dp-chips">{s.items.map((m) => <span key={m} className="dp-chip"><Ic name="cpu" size={12} /> {m}</span>)}</div>
              </div>
          ) :
          (token.modelNames || []).length ?
          <div className="dp-chips">{token.modelNames.map((m) => <span key={m} className="dp-chip"><Ic name="cpu" size={12} /> {m}</span>)}</div> :

          <div className="dp-resource" style={{ opacity: .55 }}><Ic name="cpu" size={14} /> No inference access</div>
          }
        </div>

        {/* MCP */}
        <div className="dp-section">
          <div className="dp-sec-lbl">MCP servers</div>
          <ListingLine on={token.listMcps} label="List all MCPs" code="/v1/mcps" />
          <div className="dp-perm-sub">Connect</div>
          {token.mcps === 'all' ?
          <div className="dp-resource"><Ic name="plug" size={14} /> All MCPs</div> :
          (token.mcpNames || []).length ?
          <div className="dp-chips">{token.mcpNames.map((m) => <span key={m} className="dp-chip"><Ic name="plug" size={12} /> {m}</span>)}</div> :

          <div className="dp-resource" style={{ opacity: .55 }}><Ic name="plug" size={14} /> No connections</div>
          }
        </div>

        {/* DETAILS */}
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
        {exchanged ? (
        /* immutable + already invalidated → no toggle, just remove from list */
        <>
            {confirmDelete ?
            <button className="dp-btn dp-btn-danger is-confirm" onClick={() => onDelete(token.id)}><Ic name="trash-2" size={14} /> Confirm remove</button> :
            <button className="dp-btn dp-btn-danger" onClick={() => setConfirmDelete(true)}><Ic name="trash-2" size={14} /> Remove</button>
            }
            {confirmDelete && <div className="dp-field-hint" style={{ textAlign: 'center' }}>Already invalidated. Removing only clears it from this list.</div>}
          </>) :

        <>
            {confirmDelete ?
            <button className="dp-btn dp-btn-danger is-confirm" onClick={() => onDelete(token.id)}><Ic name="trash-2" size={14} /> Confirm delete</button> :
            <button className="dp-btn dp-btn-danger" onClick={() => setConfirmDelete(true)}><Ic name="trash-2" size={14} /> Delete</button>
            }
            {confirmDelete && <div className="dp-field-hint" style={{ textAlign: 'center' }}>Deleting revokes the token permanently. Click again to confirm.</div>}
          </>
        }
      </div>
    </div>);

}

function AppTokensMain({ tokens, filter, setFilter, search, setSearch, counts, selId, onSelect, onToggle, onSelectId }) {
  const { openRail } = useShell();
  useListKeyNav();
  const select = (id) => {onSelect(id);openRail();};

  const visible = tokens.filter((t) => {
    if (filter === 'active' && t.status !== 'active') return false;
    if (filter === 'revoked' && t.status === 'active') return false;
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
        { id: 'all', label: 'All', badge: counts.all },
        { id: 'active', label: 'Active', badge: counts.active },
        { id: 'revoked', label: 'Inactive', badge: counts.revoked }]
        }
        category={filter} onCategory={setFilter}
        search={search} onSearch={setSearch} searchPlaceholder="Search apps by name or id…" />

      <div className="l-scroll">
        {visible.length === 0 ?
        <div className="l-empty">
            <Ic name="layout-grid" size={32} />
            <div className="l-empty-t">{search ? 'No app tokens match your search' : 'No app tokens yet'}</div>
            <div className="l-empty-s">{search ? 'Try a different search term.' : 'App tokens are issued when an app is granted access.'}</div>
          </div> :

        <ListView className="tk-listview" head={
        <>
              <div className="tk-icon"></div>
              <div className="tk-id l-lh">Application</div>
              <div className="tk-role l-lh">Role</div>
              <div className="tk-created l-lh">Issued</div>
              <div className="tk-used l-lh">Last used</div>
              <div className="tk-act"></div>
            </>
        }>
            {visible.map((token) =>
          <TokenRow key={token.id} token={token} selected={token.id === selId} onSelect={select} onToggle={onToggle} onSelectId={onSelectId} />
          )}
          </ListView>
        }
      </div>
    </div>);

}

function AppTokensApp() {
  const [tokens, setTokens] = useState(SAMPLE_APP_TOKENS);
  const [filter, setFilter] = useState('all');
  const [search, setSearch] = useState('');
  const [selId, setSelId] = useState(null);

  /* ?selected=<id> preselects a row (used by exchange links) */
  useEffect(() => {
    const sel = new URLSearchParams(location.search).get('selected');
    if (sel && SAMPLE_APP_TOKENS.some((t) => t.id === sel)) {setSelId(sel);return;}
    if (!window.matchMedia('(max-width:767px)').matches) setSelId(SAMPLE_APP_TOKENS[0].id);
  }, []);

  const selectId = (id) => {
    setSelId(id);
    const u = new URL(location.href);u.searchParams.set('selected', id);history.replaceState(null, '', u);
  };
  const handleToggle = (id) => setTokens((p) => p.map((t) => t.id === id ? { ...t, status: t.status === 'active' ? 'revoked' : 'active' } : t));
  const handleDelete = (id) => {setTokens((p) => p.filter((t) => t.id !== id));setSelId((s) => s === id ? null : s);};

  const counts = {
    all: tokens.length,
    active: tokens.filter((t) => t.status === 'active').length,
    revoked: tokens.filter((t) => t.status !== 'active').length
  };

  const selected = tokens.find((t) => t.id === selId) || null;

  return (
    <AppShell
      section="api-keys" subPage="app-tokens" resizeKey="api-keys"
      contentClass="flush" mainScroll={false} railScroll={false}
      breadcrumb={[
      { label: 'Bodhi', href: 'Chat.html' },
      { label: 'Access Tokens', href: 'Tokens-API.html' },
      { label: 'App Tokens', current: true }]
      }
      rail={selected ? <TokenDetailPanel token={selected} onToggle={handleToggle} onDelete={handleDelete} onSelectId={selectId} /> : null}
      railHeader={selected ? <TokenDetailHeader token={selected} onClose={() => setSelId(null)} /> : undefined}>
      
      <AppTokensMain tokens={tokens} filter={filter} setFilter={setFilter} search={search} setSearch={setSearch}
      counts={counts} selId={selId} onSelect={setSelId} onToggle={handleToggle} onSelectId={selectId} />
    </AppShell>);

}

ReactDOM.createRoot(document.getElementById('root')).render(<AppTokensApp />);