/* ═══════════════════════════════════════════
   CREATE NEW API MODEL — React App
   Sections:
     1 · Provider Connection
     2 · Request Routing
     3 · Model Selection
   Light / dark theme via Tweaks panel.
═══════════════════════════════════════════ */

const CAM_TWEAK_DEFAULTS = /*EDITMODE-BEGIN*/{
  "theme": "light"
}/*EDITMODE-END*/;

/* ── All available OpenAI models ─────── */
const ALL_MODELS = [
  'gpt-4o',
  'gpt-4o-2024-11-20',
  'gpt-4o-2024-08-06',
  'gpt-4o-2024-05-13',
  'gpt-4o-mini',
  'gpt-4o-mini-2024-07-18',
  'gpt-4o-mini-realtime-preview-2024-12-17',
  'gpt-4o-realtime-preview-2024-12-17',
  'gpt-4-turbo',
  'gpt-4-turbo-2024-04-09',
  'gpt-4-turbo-preview',
  'gpt-4',
  'gpt-4-32k',
  'gpt-3.5-turbo',
  'gpt-3.5-turbo-0125',
  'gpt-3.5-turbo-instruct',
  'gpt-5-mini',
  'gpt-5.4-mini',
  'gpt-5.1-codex-mini',
  'gpt-5.1-codex-max',
  'gpt-5.2-codex',
  'gpt-5.3-codex',
  'codex-latest',
  'o1',
  'o1-mini',
  'o1-preview',
  'o3',
  'o3-mini',
  'text-embedding-3-small',
];

/* ── Lucide icon helper ─────────────── */
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
    <span
      ref={ref}
      style={{
        display: 'inline-flex', width: size, height: size,
        alignItems: 'center', justifyContent: 'center',
        flexShrink: 0, ...style
      }}
    />
  );
}

/* SectionHeader removed — clean flowing form */

/* ── Model Selection component ──────── */
function ModelSelection({ selectedModels, onToggle, onClearAll, onSelectAll }) {
  const [query, setQuery]       = React.useState('mini');
  const [fetching, setFetching] = React.useState(false);
  const [modelPool, setModelPool] = React.useState(ALL_MODELS);

  const filtered = React.useMemo(() => {
    const q = query.trim().toLowerCase();
    if (!q) return modelPool;
    return modelPool.filter(m => m.toLowerCase().includes(q));
  }, [query, modelPool]);

  const handleFetch = () => {
    setFetching(true);
    setTimeout(() => {
      setFetching(false);
    }, 1400);
  };

  const handleSelectAll = () => {
    onSelectAll(filtered);
  };

  return (
    <div className="cam-model-box">
      {/* Selected chips */}
      <div className="cam-selected-area">
        <div className="cam-selected-header">
          <span className="cam-selected-label">Selected Models ({selectedModels.length})</span>
          {selectedModels.length > 0 && (
            <button className="cam-clear-all" onClick={onClearAll}>Clear All</button>
          )}
        </div>
        <div className="cam-chips-row">
          {selectedModels.length === 0 && (
            <span style={{ fontSize: 11, color: 'hsl(var(--muted-foreground))' }}>
              No models selected
            </span>
          )}
          {selectedModels.map(m => (
            <span key={m} className="cam-chip">
              {m}
              <button className="cam-chip-x" onClick={() => onToggle(m)} title={`Remove ${m}`}>
                ×
              </button>
            </span>
          ))}
        </div>
      </div>

      {/* Available models */}
      <div className="cam-available-area">
        <div className="cam-available-header">
          <span className="cam-available-label">Available Models</span>
          <div className="cam-available-actions">
            <button
              className={`cam-link-btn${fetching ? ' loading' : ''}`}
              onClick={handleFetch}
              disabled={fetching}
            >
              {fetching
                ? <><span className="cam-fetch-spin"></span> Fetching…</>
                : 'Fetch Models'
              }
            </button>
            <button className="cam-link-btn" onClick={handleSelectAll}>
              Select All ({filtered.length})
            </button>
          </div>
        </div>

        {/* Search */}
        <div className="cam-search-wrap">
          <span className="cam-search-icon">
            <Icon name="search" size={13} />
          </span>
          <input
            className="cam-search-input"
            value={query}
            onChange={e => setQuery(e.target.value)}
            placeholder="Filter models…"
          />
          {query && (
            <button className="cam-search-clear" onClick={() => setQuery('')}>
              <Icon name="x" size={12} />
            </button>
          )}
        </div>

        {/* List */}
        <div className="cam-model-list">
          {filtered.length === 0 && (
            <div className="cam-no-models">No models match "{query}"</div>
          )}
          {filtered.map(m => {
            const checked = selectedModels.includes(m);
            return (
              <div
                key={m}
                className={`cam-model-item${checked ? ' checked' : ''}`}
                onClick={() => onToggle(m)}
              >
                <input
                  type="checkbox"
                  className="cam-model-cb"
                  checked={checked}
                  readOnly
                />
                <span className="cam-model-name">{m}</span>
              </div>
            );
          })}
        </div>
      </div>
    </div>
  );
}

/* ── Main App ────────────────────────── */
function CreateApiModelApp() {
  const [tweaks, setTweak] = useTweaks(CAM_TWEAK_DEFAULTS);

  /* Section 1 — Provider Connection */
  const [apiFormat,      setApiFormat]      = React.useState('openai-completions');
  const [baseUrl,        setBaseUrl]        = React.useState('https://api.openai.com/v1');
  const [useApiKey,      setUseApiKey]      = React.useState(true);
  const [apiKey,         setApiKey]         = React.useState('sk-proj-••••••••••••••••••••••••••••••••••••••••••••••••••••');
  const [showKey,        setShowKey]        = React.useState(false);

  /* Section 2 — Request Routing */
  const [enablePrefix,   setEnablePrefix]   = React.useState(true);
  const [prefix,         setPrefix]         = React.useState('openai/');
  const [fwdMode,        setFwdMode]        = React.useState('selected'); // 'all' | 'selected'

  /* Section 3 — Model Selection */
  const [selectedModels, setSelectedModels] = React.useState([
    'gpt-5-mini', 'gpt-5.4-mini', 'gpt-5.1-codex-mini'
  ]);

  /* sync theme */
  React.useEffect(() => {
    document.documentElement.setAttribute('data-theme', tweaks.theme);
  }, [tweaks.theme]);

  const toggleModel = (m) => {
    setSelectedModels(prev =>
      prev.includes(m) ? prev.filter(x => x !== m) : [...prev, m]
    );
  };

  const selectAll = (models) => {
    setSelectedModels(prev => {
      const next = [...prev];
      models.forEach(m => { if (!next.includes(m)) next.push(m); });
      return next;
    });
  };

  /* Format base URL hint based on selection */
  const urlHint = apiFormat === 'openai-completions'
    ? 'Enter the complete API endpoint URL for your provider'
    : apiFormat === 'anthropic'
    ? 'e.g. https://api.anthropic.com/v1'
    : 'Enter the complete API endpoint URL for your provider';

  return (
    <div className="cam-app-shell">
      <BodhiSidebar section="models" subPage="new-api-model" />
      <div className="cam-content">
      {/* ═══ TOP BAR ═══ */}
      <header className="cam-top-bar">
        <nav className="cam-breadcrumb">
          <a href="Bodhi Models.html">Bodhi</a>
          <span className="cam-bc-sep">/</span>
          <a href="Bodhi Models.html">Model Aliases</a>
          <span className="cam-bc-sep">/</span>
          <span className="cam-bc-curr">New API Model</span>
        </nav>
        <div className="cam-spacer"></div>
        <div className="cam-top-actions">
          <button className="cam-btn cam-btn-test">
            <Icon name="plug-zap" size={12} />
            Test Connection
          </button>
          <button className="cam-btn cam-btn-cancel">Cancel</button>
          <button className="cam-btn cam-btn-create">Create API Model</button>
        </div>
      </header>

      {/* ═══ PAGE ═══ */}
      <div className="cam-page-wrap">
        <main className="cam-main-col">
          <h1 className="cam-page-title">Create New API Model</h1>
          <p className="cam-page-sub">Configure a new external AI API model. Connect a provider, choose routing, pick which models forward through this alias.</p>

          {/* ══ PROVIDER CONNECTION ══ */}
          <div className="cam-section">

            {/* API Format */}
            <div className="cam-field">
              <label className="cam-label">
                <span className="cam-label-text">API Format</span>
                <span className="cam-req-badge">Required</span>
              </label>
              <select
                className="cam-select"
                value={apiFormat}
                onChange={e => setApiFormat(e.target.value)}
              >
                <option value="openai-completions">OpenAI — Completions</option>
                <option value="openai-chat">OpenAI — Chat Completions</option>
                <option value="anthropic">Anthropic Messages</option>
                <option value="cohere">Cohere Generate</option>
                <option value="ollama">Ollama</option>
              </select>
            </div>

            {/* Base URL */}
            <div className="cam-field">
              <label className="cam-label">
                <span className="cam-label-text">Base URL</span>
                <span className="cam-req-badge">Required</span>
              </label>
              <input
                className="cam-input"
                type="url"
                value={baseUrl}
                onChange={e => setBaseUrl(e.target.value)}
                placeholder="https://api.openai.com/v1"
              />
              <div className="cam-hint">{urlHint}</div>
            </div>

            {/* API Key */}
            <div className="cam-field">
              <label className="cam-label"><span className="cam-label-text">API Key</span></label>
              <div className="cam-check-row">
                <input
                  type="checkbox"
                  id="useApiKey"
                  className="cam-checkbox"
                  checked={useApiKey}
                  onChange={e => setUseApiKey(e.target.checked)}
                />
                <label htmlFor="useApiKey" className="cam-check-label">Use API key</label>
              </div>
              {useApiKey && (
                <div className="cam-indent">
                  <div className="cam-pw-wrap">
                    <input
                      className={`cam-input cam-input-password`}
                      type={showKey ? 'text' : 'password'}
                      value={apiKey}
                      onChange={e => setApiKey(e.target.value)}
                      placeholder="sk-…"
                      autoComplete="new-password"
                    />
                    <button
                      className="cam-pw-toggle"
                      type="button"
                      onClick={() => setShowKey(v => !v)}
                      title={showKey ? 'Hide key' : 'Show key'}
                    >
                      <Icon name={showKey ? 'eye-off' : 'eye'} size={14} />
                    </button>
                  </div>
                  <div className="cam-hint">Your API key is stored securely</div>
                </div>
              )}
            </div>
          </div>

          {/* ══ REQUEST ROUTING ══ */}
          <div className="cam-section">

            {/* Model Prefix */}
            <div className="cam-field">
              <label className="cam-label"><span className="cam-label-text">Model Prefix</span></label>
              <div className="cam-check-row">
                <input
                  type="checkbox"
                  id="enablePrefix"
                  className="cam-checkbox"
                  checked={enablePrefix}
                  onChange={e => setEnablePrefix(e.target.checked)}
                />
                <label htmlFor="enablePrefix" className="cam-check-label">Enable prefix</label>
              </div>
              {enablePrefix && (
                <div className="cam-indent">
                  <input
                    className="cam-input"
                    value={prefix}
                    onChange={e => setPrefix(e.target.value)}
                    placeholder="e.g. openai/"
                  />
                  <div className="cam-hint">Add a prefix to all model names (useful for organization or API routing).</div>
                  <div className="cam-hint-example">Example: {prefix || 'openai/'}gpt-4</div>
                </div>
              )}
            </div>

            {/* Request Forwarding Mode */}
            <div className="cam-field">
              <label className="cam-label"><span className="cam-label-text">Request Forwarding Mode</span></label>
              <div className="cam-radio-group">
                <div
                  className={`cam-radio-option${fwdMode === 'all' ? ' selected' : ''}`}
                  onClick={() => setFwdMode('all')}
                >
                  <div className="cam-radio-dot">
                    <div className="cam-radio-dot-inner" style={{ transform: fwdMode === 'all' ? 'scale(1)' : 'scale(0)' }}></div>
                  </div>
                  <span className="cam-radio-text">Forward all requests with prefix</span>
                </div>
                <div
                  className={`cam-radio-option${fwdMode === 'selected' ? ' selected' : ''}`}
                  onClick={() => setFwdMode('selected')}
                >
                  <div className="cam-radio-dot">
                    <div className="cam-radio-dot-inner" style={{ transform: fwdMode === 'selected' ? 'scale(1)' : 'scale(0)' }}></div>
                  </div>
                  <span className="cam-radio-text">Forward for selected models only</span>
                </div>
              </div>
            </div>
          </div>

          {/* ══ MODEL SELECTION ══ */}
          {fwdMode === 'selected' && (
            <div className="cam-section">
              <p className="cam-section-desc">
                Select which OpenAI models you'd like to use. Only the selected set will be forwarded through the alias prefix.
              </p>
              <ModelSelection
                selectedModels={selectedModels}
                onToggle={toggleModel}
                onClearAll={() => setSelectedModels([])}
                onSelectAll={selectAll}
              />
            </div>
          )}

          {/* ══ BOTTOM ACTION BAR (desktop/tablet) ══ */}
          <div className="cam-action-bar">
            <button className="cam-btn cam-btn-test">
              <Icon name="plug-zap" size={12} />
              Test Connection
            </button>
            <div className="cam-action-spacer"></div>
            <button className="cam-btn cam-btn-cancel">Cancel</button>
            <button className="cam-btn cam-btn-create">Create API Model</button>
          </div>
        </main>
      </div>

      {/* ═══ MOBILE BOTTOM BAR ═══ */}
      <div className="cam-mobile-bar">
        <button className="cam-btn cam-btn-test">
          <Icon name="plug-zap" size={12} />
          Test
        </button>
        <button className="cam-btn cam-btn-cancel">Cancel</button>
        <button className="cam-btn cam-btn-create">Create API Model</button>
      </div>
      </div>{/* end cam-content */}

      {/* ═══ TWEAKS ═══ */}
      <TweaksPanel>
        <TweakSection title="Theme">
          <TweakRadio
            value={tweaks.theme}
            options={[{label:'Light',value:'light'},{label:'Dark',value:'dark'}]}
            onChange={v => setTweak('theme', v)}
          />
        </TweakSection>
      </TweaksPanel>
    </div>
  );
}

const camRoot = ReactDOM.createRoot(document.getElementById('root'));
camRoot.render(<CreateApiModelApp />);
