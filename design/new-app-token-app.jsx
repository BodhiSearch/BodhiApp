/* ═══════════════════════════════════════════════════
   NEW APP TOKEN — React App
   Sections:
     1 · Token Identity
     2 · Model Access
     3 · MCP Access
     4 · User Role
═══════════════════════════════════════════════════ */

const NAT_TWEAK_DEFAULTS = /*EDITMODE-BEGIN*/{
  "theme": "light"
}/*EDITMODE-END*/;

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
  const [tweaks, setTweak] = useTweaks(NAT_TWEAK_DEFAULTS);

  /* Section 1 — Token Identity */
  const [tokenName, setTokenName] = React.useState('');

  /* Section 2 — Model Access */
  const [modelMode,      setModelMode]      = React.useState('all'); // 'all' | 'specific'
  const [selectedModels, setSelectedModels] = React.useState([]);

  /* Section 3 — MCP Access */
  const [mcpMode,        setMcpMode]        = React.useState('all'); // 'all' | 'specific'
  const [selectedMcps,   setSelectedMcps]   = React.useState([]);

  /* Section 4 — User Role */
  const [role, setRole] = React.useState('user');

  /* Success state */
  const [generated,   setGenerated]   = React.useState(false);
  const [tokenValue,  setTokenValue]  = React.useState('');
  const [copied,      setCopied]      = React.useState(false);

  /* Active section in sidebar — unused after section nav removal */
  const [activeSection, setActiveSection] = React.useState('identity');

  /* Sync theme */
  React.useEffect(() => {
    document.documentElement.setAttribute('data-theme', tweaks.theme);
  }, [tweaks.theme]);



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
    <div className="nat-app">

      {/* ══ SIDEBAR ══ */}
      <BodhiSidebar section="api-keys" subPage="new-token" />

      {/* ══ MAIN ══ */}
      <main className="nat-main">

        {/* Topbar */}
        <div className="nat-topbar">
          <div className="nat-breadcrumb">
            <span>Bodhi</span>
            <i data-lucide="chevron-right" className="nat-bc-sep"></i>
            <span>API Keys</span>
            <i data-lucide="chevron-right" className="nat-bc-sep"></i>
            <span className="nat-bc-curr">New App Token</span>
          </div>
          <div className="nat-topbar-actions">
            <button className="nat-btn nat-btn-cancel" onClick={() => setGenerated(false) || setTokenName('') || setSelectedModels([]) || setSelectedMcps([]) || setRole('user')}>
              Cancel
            </button>
            {!generated && (
              <button className="nat-btn nat-btn-generate" onClick={handleGenerate}>
                <Icon name="shield-plus" size={13} />
                Generate Token
              </button>
            )}
          </div>
        </div>

        {/* Scroll area */}
        <div className="nat-scroll" id="natScroll">
          <div className="nat-form-card">

            <h1 className="nat-page-title">New App Token</h1>
            <p className="nat-page-sub">
              Generate a scoped token for programmatic access to the Bodhi API.
              Configure which models, MCPs, and capabilities this token can access.
            </p>

            {/* ── GENERATED TOKEN REVEAL ── */}
            {generated && (
              <div style={{ marginTop: 20 }}>
                <div className="nat-token-reveal">
                  <div className="nat-token-reveal-header">
                    <Icon name="check-circle-2" size={14} style={{ color: 'var(--c-leaf-text)' }} />
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
              </div>
            )}

            {/* ══ SECTION 1: TOKEN IDENTITY ══ */}
            <hr className="nat-divider" id="nat-sec-identity" />

            <div className="nat-field">
              <label className="nat-label">
                Token Name
                <span className="nat-opt-badge">Optional</span>
              </label>
              <input
                className="nat-input"
                type="text"
                value={tokenName}
                onChange={e => setTokenName(e.target.value)}
                placeholder="e.g. my-app-token"
              />
              <div className="nat-hint">A human-readable label to identify this token in the token list.</div>
            </div>

            {/* ══ SECTION 2: MODEL ACCESS ══ */}
            <hr className="nat-divider" id="nat-sec-models" />

            <div className="nat-field">
              <label className="nat-label">Model Access</label>
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

            {/* ══ SECTION 3: MCP ACCESS ══ */}
            <hr className="nat-divider" id="nat-sec-mcps" />

            <div className="nat-field">
              <label className="nat-label">MCP Access</label>
              <div className="nat-radio-group">
                <div
                  className={`nat-radio-option${mcpMode === 'all' ? ' selected' : ''}`}
                  onClick={() => setMcpMode('all')}
                >
                  <div className="nat-radio-dot">
                    <div className="nat-radio-dot-inner" style={{ transform: mcpMode === 'all' ? 'scale(1)' : 'scale(0)' }}></div>
                  </div>
                  <div className="nat-radio-body">
                    <span className="nat-radio-text">
                      All MCPs
                      <span className="nat-future-badge" style={{ marginLeft: 6 }}>+ future</span>
                    </span>
                    <span className="nat-radio-desc">Access all currently registered MCP servers and any added in the future.</span>
                  </div>
                </div>
                <div
                  className={`nat-radio-option${mcpMode === 'specific' ? ' selected' : ''}`}
                  onClick={() => setMcpMode('specific')}
                >
                  <div className="nat-radio-dot">
                    <div className="nat-radio-dot-inner" style={{ transform: mcpMode === 'specific' ? 'scale(1)' : 'scale(0)' }}></div>
                  </div>
                  <div className="nat-radio-body">
                    <span className="nat-radio-text">Select specific MCPs</span>
                    <span className="nat-radio-desc">Choose exactly which MCP servers this token can invoke.</span>
                  </div>
                </div>
              </div>
              {mcpMode === 'specific' && (
                <div style={{ marginTop: 10 }}>
                  <SelectableList
                    items={SAMPLE_MCPS}
                    selected={selectedMcps}
                    onToggle={toggleMcp}
                    onClearAll={() => setSelectedMcps([])}
                    onSelectAll={selectAllMcps}
                    searchPlaceholder="Filter MCPs…"
                  />
                </div>
              )}
            </div>

            {/* ══ SECTION 4: USER ROLE ══ */}
            <hr className="nat-divider" id="nat-sec-role" />

            <div className="nat-field">
              <label className="nat-label" style={{ marginBottom: 8 }}>Token Scope</label>
              <div className="nat-role-grid">

                {/* User */}
                <div
                  className={`nat-role-card${role === 'user' ? ' selected' : ''}`}
                  onClick={() => setRole('user')}
                >
                  <div className="nat-role-card-header">
                    <span className="nat-role-card-name">User</span>
                    <div className="nat-radio-dot">
                      <div className="nat-radio-dot-inner" style={{ transform: role === 'user' ? 'scale(1)' : 'scale(0)' }}></div>
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
                    <div className="nat-radio-dot">
                      <div className="nat-radio-dot-inner" style={{ transform: role === 'power' ? 'scale(1)' : 'scale(0)' }}></div>
                    </div>
                  </div>
                  <div className="nat-role-card-desc">
                    Elevated access. Can manage models, configure MCP servers, and perform admin-level API operations.
                  </div>
                  <span className="nat-role-badge power">scope_token_power_user</span>
                </div>

              </div>
            </div>

            {/* ── BOTTOM ACTION BAR ── */}
            <div className="nat-action-bar">
              <div className="nat-action-spacer"></div>
              <button className="nat-btn nat-btn-cancel">Cancel</button>
              <button className="nat-btn nat-btn-generate" onClick={handleGenerate}>
                <Icon name="shield-plus" size={13} />
                Generate Token
              </button>
            </div>

          </div>
        </div>
      </main>

      {/* ══ TWEAKS ══ */}
      <TweaksPanel>
        <TweakSection title="Theme">
          <TweakRadio
            value={tweaks.theme}
            options={[{ label: 'Light', value: 'light' }, { label: 'Dark', value: 'dark' }]}
            onChange={v => setTweak('theme', v)}
          />
        </TweakSection>
      </TweaksPanel>

    </div>
  );
}

const natRoot = ReactDOM.createRoot(document.getElementById('root'));
natRoot.render(<NewAppTokenApp />);
