/* ═══════════════════════════════════════════════════
   NEW APP TOKEN — React App  (bodhi-form.css layout)
   Sections:
     1 · Token Identity
     2 · Model Access
     3 · MCP Access
     4 · Token Scope
   Generic shell + form primitives come from bodhi-form.css (.bf-*).
   Page-unique: selection box, role cards, token reveal (new-app-token.css).
═══════════════════════════════════════════════════ */

/* ── Sample data ── */
const SAMPLE_MODELS = [
  { id: 'llama3.2:3b',       name: 'Llama 3.2 · 3B',      type: 'local', ctx: '128k' },
  { id: 'llama3.2:1b',       name: 'Llama 3.2 · 1B',      type: 'local', ctx: '128k' },
  { id: 'llama3.1:8b',       name: 'Llama 3.1 · 8B',      type: 'local', ctx: '128k' },
  { id: 'llama3.1:70b',      name: 'Llama 3.1 · 70B',     type: 'local', ctx: '128k' },
  { id: 'mistral:7b',        name: 'Mistral · 7B',         type: 'local', ctx: '32k'  },
  { id: 'mixtral:8x7b',      name: 'Mixtral · 8×7B',      type: 'local', ctx: '32k'  },
  { id: 'phi3:mini',         name: 'Phi-3 · Mini',         type: 'local', ctx: '128k' },
  { id: 'phi3:medium',       name: 'Phi-3 · Medium',       type: 'local', ctx: '128k' },
  { id: 'gemma2:9b',         name: 'Gemma 2 · 9B',         type: 'local', ctx: '8k'   },
  { id: 'gemma2:27b',        name: 'Gemma 2 · 27B',        type: 'local', ctx: '8k'   },
  { id: 'qwen2.5:7b',        name: 'Qwen 2.5 · 7B',        type: 'local', ctx: '128k' },
  { id: 'deepseek-r1:8b',    name: 'DeepSeek-R1 · 8B',    type: 'local', ctx: '64k'  },
  { id: 'deepseek-r1:32b',   name: 'DeepSeek-R1 · 32B',   type: 'local', ctx: '64k'  },
  { id: 'codellama:13b',     name: 'CodeLlama · 13B',      type: 'local', ctx: '16k'  },
  { id: 'nomic-embed-text',  name: 'Nomic Embed Text',     type: 'local', ctx: '8k'   },
];

const SAMPLE_MCPS = [
  { id: 'filesystem',           label: 'filesystem',     meta: 'Read / write local files' },
  { id: 'brave-search',         label: 'brave-search',   meta: 'Web search via Brave API' },
  { id: 'github',               label: 'github',         meta: 'GitHub repos & issues' },
  { id: 'sqlite',               label: 'sqlite',         meta: 'Query SQLite databases' },
  { id: 'puppeteer',            label: 'puppeteer',      meta: 'Browser automation' },
  { id: 'postgres',             label: 'postgres',       meta: 'PostgreSQL queries' },
  { id: 'slack',                label: 'slack',          meta: 'Slack workspace access' },
  { id: 'memory',               label: 'memory',         meta: 'Persistent key-value store' },
  { id: 'sequential-thinking',  label: 'sequential-thinking', meta: 'Step-by-step reasoning' },
  { id: 'fetch',                label: 'fetch',          meta: 'HTTP fetch & scrape' },
];

/* ── Icon helper ── */
function Icon({ name, size = 13, style = {} }) {
  const ref = React.useRef(null);
  React.useEffect(() => {
    if (!ref.current) return;
    ref.current.innerHTML = '';
    const el = document.createElement('i');
    el.setAttribute('data-lucide', name);
    ref.current.appendChild(el);
    lucide.createIcons({ nodes: [el] });
  }, [name, size]);
  return (
    <span ref={ref} style={{
      display: 'inline-flex', width: size, height: size,
      alignItems: 'center', justifyContent: 'center',
      flexShrink: 0, ...style
    }} />
  );
}

/* ── Selectable list (models or MCPs) ── */
function SelectableList({ items, selected, onToggle, onClearAll, onSelectAll, searchPlaceholder }) {
  const [query, setQuery] = React.useState('');

  const filtered = React.useMemo(() => {
    const q = query.trim().toLowerCase();
    if (!q) return items;
    return items.filter(it =>
      it.id.toLowerCase().includes(q) ||
      it.label.toLowerCase().includes(q) ||
      (it.meta && it.meta.toLowerCase().includes(q))
    );
  }, [items, query]);

  return (
    <div className="nat-sel-box">
      {/* Selected chips */}
      <div className="nat-sel-chips-area">
        <div className="nat-sel-chips-header">
          <span className="nat-sel-chips-label">Selected ({selected.length})</span>
          {selected.length > 0 && (
            <button className="nat-sel-clear" onClick={onClearAll}>Clear all</button>
          )}
        </div>
        <div className="nat-chips-row">
          {selected.length === 0
            ? <span className="nat-chip-empty">None selected — token has no access</span>
            : selected.map(id => {
                const item = items.find(x => x.id === id);
                return (
                  <span key={id} className="nat-chip">
                    {item ? item.label : id}
                    <button className="nat-chip-x" onClick={() => onToggle(id)}>×</button>
                  </span>
                );
              })
          }
        </div>
      </div>

      {/* Available list */}
      <div className="nat-sel-list-area">
        <div className="nat-sel-list-header">
          <span className="nat-sel-list-label">Available ({filtered.length})</span>
          <div className="nat-sel-actions">
            <button className="nat-link-btn" onClick={() => onSelectAll(filtered.map(x => x.id))}>
              Select all ({filtered.length})
            </button>
          </div>
        </div>

        {/* Search */}
        <div className="nat-sel-search-wrap">
          <span className="nat-sel-search-icon"><Icon name="search" size={12} /></span>
          <input
            className="nat-sel-search"
            value={query}
            onChange={e => setQuery(e.target.value)}
            placeholder={searchPlaceholder || 'Filter…'}
          />
          {query && (
            <button className="nat-sel-search-clear" onClick={() => setQuery('')}>
              <Icon name="x" size={11} />
            </button>
          )}
        </div>

        {/* List */}
        <div className="nat-sel-item-list">
          {filtered.length === 0 && (
            <div className="nat-sel-empty">No items match "{query}"</div>
          )}
          {filtered.map(item => {
            const checked = selected.includes(item.id);
            return (
              <div
                key={item.id}
                className={`nat-sel-item${checked ? ' checked' : ''}`}
                onClick={() => onToggle(item.id)}
              >
                <input type="checkbox" className="nat-sel-cb" checked={checked} readOnly />
                <span className="nat-sel-item-name">{item.label}</span>
                {item.meta && <span className="nat-sel-item-meta">{item.meta}</span>}
              </div>
            );
          })}
        </div>
      </div>
    </div>
  );
}

/* ── Main App ── */
function NewAppTokenApp() {
  /* Section 1 — Token Identity */
  const [tokenName, setTokenName] = React.useState('');

  /* Section 2 — Model Access */
  const [modelMode,      setModelMode]      = React.useState('all'); // 'all' | 'specific'
  const [selectedModels, setSelectedModels] = React.useState([]);
  const [listAllModels,  setListAllModels]  = React.useState(false);

  /* Section 3 — MCP Access */
  const [mcpMode,        setMcpMode]        = React.useState('all'); // 'all' | 'specific'
  const [selectedMcps,   setSelectedMcps]   = React.useState([]);
  const [listAllMcps,    setListAllMcps]    = React.useState(false);

  /* Section 4 — Token Scope */
  const [role, setRole] = React.useState('user');

  /* Success state */
  const [generated,   setGenerated]   = React.useState(false);
  const [tokenValue,  setTokenValue]  = React.useState('');
  const [copied,      setCopied]      = React.useState(false);

  /* Lucide icons after render */
  React.useEffect(() => { lucide.createIcons(); });

  /* Helpers */
  const toggleModel = id => setSelectedModels(prev =>
    prev.includes(id) ? prev.filter(x => x !== id) : [...prev, id]
  );
  const toggleMcp = id => setSelectedMcps(prev =>
    prev.includes(id) ? prev.filter(x => x !== id) : [...prev, id]
  );
  const selectAllMcps = ids => setSelectedMcps(prev => {
    const next = [...prev];
    ids.forEach(id => { if (!next.includes(id)) next.push(id); });
    return next;
  });

  const resetForm = () => {
    setGenerated(false); setTokenName('');
    setSelectedModels([]); setSelectedMcps([]);
    setModelMode('all'); setMcpMode('all'); setRole('user');
    setListAllModels(false); setListAllMcps(false);
  };

  const handleGenerate = () => {
    const fake = 'bdt_' + Array.from({ length: 48 }, () =>
      'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789'[Math.floor(Math.random() * 62)]
    ).join('');
    setTokenValue(fake);
    setGenerated(true);
    setTimeout(() => {
      const scroll = document.getElementById('natScroll');
      if (scroll) scroll.scrollTo({ top: 0, behavior: 'smooth' });
    }, 100);
  };

  const handleCopy = () => {
    navigator.clipboard.writeText(tokenValue).catch(() => {});
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <>
    <AppShell
      section="api-keys" subPage="new-token" resizeKey="api-keys"
      breadcrumb={[
        { label: 'Bodhi', href: 'Bodhi Chat.html' },
        { label: 'Access Tokens', href: 'API Tokens.html' },
        { label: 'New App Token', current: true },
      ]}
      contentClass="flush" mainScroll={false}
    >
        {/* Scroll · centered container */}
        <div className="bf-scroll" id="natScroll">
          <div className="bf-container">

            <div className="bf-card">
              <div className="bf-card-head">
                <h1 className="bf-card-title">New App Token</h1>
                <p className="bf-card-sub">
                  Generate a scoped token for API access — pick the models, MCPs, and capabilities it can use.
                </p>
              </div>
              <div className="bf-card-body">

                {/* ── GENERATED TOKEN REVEAL ── */}
                {generated && (
                  <div className="nat-token-reveal" style={{ marginBottom: 22 }}>
                    <div className="nat-token-reveal-header">
                      <Icon name="check-circle-2" size={14} />
                      <span className="nat-token-reveal-title">Token generated — copy it now</span>
                    </div>
                    <div className="nat-token-reveal-body">
                      <span className="nat-token-value">{tokenValue}</span>
                      <button className="nat-copy-btn" onClick={handleCopy}>
                        <Icon name={copied ? 'check' : 'copy'} size={11} />
                        {copied ? 'Copied!' : 'Copy'}
                      </button>
                    </div>
                    <div className="nat-token-warn">
                      This token will not be shown again. Store it securely.
                    </div>
                  </div>
                )}

                {/* ══ SECTION 1: TOKEN IDENTITY ══ */}
                <div className="bf-section">
                  <div className="bf-section-title">Token Identity</div>
                  <div className="bf-field">
                    <label className="bf-label">
                      <span className="bf-label-text">Token Name</span>
                      <span className="bf-optional">Optional</span>
                    </label>
                    <input
                      className="bf-input bf-input-mono"
                      type="text"
                      value={tokenName}
                      onChange={e => setTokenName(e.target.value)}
                      placeholder="e.g. my-app-token"
                    />
                    <div className="bf-hint">A human-readable label to identify this token in the token list.</div>
                  </div>
                </div>

                <div className="bf-divider"></div>

                {/* ══ SECTION 2: MODEL ACCESS ══ */}
                <div className="bf-section">
                  <div className="bf-section-title">Model Access</div>
                  <div className="bf-field">
                    <ListingToggle
                      on={listAllModels}
                      onToggle={() => setListAllModels(v => !v)}
                      redundant={modelMode === 'all'}
                      label="List all models"
                      code="/v1/models"
                      desc="Let the app enumerate every model via the catalog. Off → it only sees the models you grant for inference below. (Listing is separate from running inference.)"
                    />
                    <ModelAccessPicker
                      mode={modelMode}
                      onModeChange={setModelMode}
                      allModels={SAMPLE_MODELS}
                      selectedIds={selectedModels}
                      onToggle={toggleModel}
                      panelTitle="Select Models"
                      panelSubtitle="Choose which models this token can access"
                    />
                  </div>
                </div>

                <div className="bf-divider"></div>

                {/* ══ SECTION 3: MCP ACCESS ══ */}
                <div className="bf-section">
                  <div className="bf-section-title">MCP Access</div>
                  <div className="bf-field">
                    <ListingToggle
                      on={listAllMcps}
                      onToggle={() => setListAllMcps(v => !v)}
                      redundant={mcpMode === 'all'}
                      label="List all MCPs"
                      code="/v1/mcps"
                      desc="Let the app discover every MCP server. Off → it only sees the servers you grant a connection to below. (Listing is separate from connecting.)"
                    />
                    <ModelAccessPicker
                      mode={mcpMode}
                      onModeChange={setMcpMode}
                      allModels={SAMPLE_MCPS}
                      selectedIds={selectedMcps}
                      onToggle={toggleMcp}
                      panelTitle="Select MCPs"
                      panelSubtitle="Choose which MCP servers this token can invoke"
                      itemNoun="MCP"
                      allLabel="All MCPs"
                      allDesc="Access all currently registered MCP servers and any added in the future."
                      specificLabel="Specific MCPs"
                      specificDesc="Choose exactly which MCP servers this token can invoke."
                    />
                  </div>
                </div>

                <div className="bf-divider"></div>

                {/* ══ SECTION 4: TOKEN SCOPE ══ */}
                <div className="bf-section">
                  <div className="bf-section-title">Token Scope</div>
                  <div className="bf-field">
                    <div className="nat-role-grid">

                      {/* User */}
                      <div
                        className={`nat-role-card${role === 'user' ? ' selected' : ''}`}
                        onClick={() => setRole('user')}
                      >
                        <div className="nat-role-card-header">
                          <span className="nat-role-card-name">User</span>
                          <div className="bf-radio-dot">
                            <div className="bf-radio-dot-inner" style={{ transform: role === 'user' ? 'scale(1)' : 'scale(0)' }}></div>
                          </div>
                        </div>
                        <div className="nat-role-card-desc">
                          Standard access. Can make inference requests, list models and MCPs permitted by this token.
                        </div>
                        <span className="nat-role-badge user">scope_token_user</span>
                      </div>

                      {/* Power User */}
                      <div
                        className={`nat-role-card${role === 'power' ? ' selected' : ''}`}
                        onClick={() => setRole('power')}
                      >
                        <div className="nat-role-card-header">
                          <span className="nat-role-card-name">Power User</span>
                          <div className="bf-radio-dot">
                            <div className="bf-radio-dot-inner" style={{ transform: role === 'power' ? 'scale(1)' : 'scale(0)' }}></div>
                          </div>
                        </div>
                        <div className="nat-role-card-desc">
                          Elevated access. Can manage models, configure MCP servers, and perform admin-level API operations.
                        </div>
                        <span className="nat-role-badge power">scope_token_power_user</span>
                      </div>

                    </div>
                  </div>
                </div>

              </div>{/* end card-body */}

              {/* ══ FOOTER — the ONLY place actions live ══ */}
              <div className="bf-footer">
                <div className="bf-footer-spacer"></div>
                <button className="bf-btn bf-btn-ghost" onClick={resetForm}>Cancel</button>
                <button className="bf-btn bf-btn-primary" onClick={handleGenerate}>
                  <Icon name="shield-plus" size={13} />
                  Generate Token
                </button>
              </div>
            </div>{/* end card */}
          </div>
        </div>
    </AppShell>
    </>
  );
}

const natRoot = ReactDOM.createRoot(document.getElementById('root'));
natRoot.render(<NewAppTokenApp />);
