/* ═══════════════════════════════════════════════════
   API MODEL FORM — shared form component
   api-model-form.jsx

   ONE source of truth for the "Create API Model" form, reused by:
     • Create API Model.html      (inside AppShell)
     • setup-4-api-models.html    (inside the setup wizard)

   Renders the bf-card (.bf-* primitives + .cam-* model selection).
   Each host supplies its own page chrome / heading / wizard nav.

   Props:
     showCancel  — show the footer "Cancel" button (app page) vs hide (wizard)

   Exports ApiModelForm to window.
═══════════════════════════════════════════════════ */

/* ── API format catalogue (matches production) ──────
   value          → backend api_format enum
   label          → dropdown text
   baseUrl        → auto-filled default when this format is chosen
   extras         → shows Extra Headers / Extra Body textareas (anthropic_oauth only)
   liberty        → swaps Base URL/Key/Extras for the OAuth-credentials envelope */
const AMF_FORMATS = [
  { value: 'openai',            label: 'OpenAI - Completions', baseUrl: 'https://api.openai.com/v1' },
  { value: 'openai_responses',  label: 'OpenAI - Responses',   baseUrl: 'https://api.openai.com/v1' },
  { value: 'anthropic',         label: 'Anthropic',            baseUrl: 'https://api.anthropic.com/v1' },
  { value: 'anthropic_oauth',   label: 'Anthropic Setup Token', baseUrl: 'https://api.anthropic.com/v1', extras: true },
  { value: 'gemini',            label: 'Google Gemini',        baseUrl: 'https://generativelanguage.googleapis.com/v1beta' },
  { value: 'llm_liberty_oauth', label: 'LLM Liberty OAuth',    baseUrl: '', liberty: true },
];
const AMF_FORMAT_MAP = Object.fromEntries(AMF_FORMATS.map((f) => [f.value, f]));

/* Default JSON shown for the Anthropic Setup Token extras (indicative). */
const AMF_DEFAULT_EXTRA_HEADERS = `{
  "anthropic-version": "2023-06-01",
  "anthropic-beta": "claude-code-20250219,oauth-2025-04-20",
  "user-agent": "claude-cli/2.1.80 (external, cli)"
}`;
const AMF_DEFAULT_EXTRA_BODY = `{
  "max_tokens": 4096,
  "system": [
    {
      "type": "text",
      "text": "You are Claude Code, Anthropic's official CLI for Claude."
    }
  ]
}`;
const AMF_LIBERTY_PLACEHOLDER = `{
  "version": "1.0.0",
  "provider": "anthropic",
  "access_token": "...",
  "refresh_token": "...",
  "expires_at": 1234567890,
  ...
}`;

/* ── All available OpenAI models ─────── */
const AMF_ALL_MODELS = [
  'gpt-4o', 'gpt-4o-2024-11-20', 'gpt-4o-2024-08-06', 'gpt-4o-2024-05-13',
  'gpt-4o-mini', 'gpt-4o-mini-2024-07-18', 'gpt-4o-mini-realtime-preview-2024-12-17',
  'gpt-4o-realtime-preview-2024-12-17', 'gpt-4-turbo', 'gpt-4-turbo-2024-04-09',
  'gpt-4-turbo-preview', 'gpt-4', 'gpt-4-32k', 'gpt-3.5-turbo', 'gpt-3.5-turbo-0125',
  'gpt-3.5-turbo-instruct', 'gpt-5-mini', 'gpt-5.4-mini', 'gpt-5.1-codex-mini',
  'gpt-5.1-codex-max', 'gpt-5.2-codex', 'gpt-5.3-codex', 'codex-latest',
  'o1', 'o1-mini', 'o1-preview', 'o3', 'o3-mini', 'text-embedding-3-small',
];

/* ── Lucide icon helper (self-contained) ── */
function AmfIcon({ name, size = 13, style = {} }) {
  const ref = React.useRef(null);
  React.useEffect(() => {
    if (!ref.current || !window.lucide) return;
    ref.current.innerHTML = '';
    const el = document.createElement('i');
    el.setAttribute('data-lucide', name);
    ref.current.appendChild(el);
    window.lucide.createIcons({ nodes: [el] });
  }, [name, size]);
  return (
    <span ref={ref} style={{
      display: 'inline-flex', width: size, height: size,
      alignItems: 'center', justifyContent: 'center', flexShrink: 0, ...style,
    }} />
  );
}

/* ── Model Selection ──────────────────── */
function AmfModelSelection({ selectedModels, onToggle, onClearAll, onSelectAll }) {
  const [query, setQuery] = React.useState('mini');
  const [fetching, setFetching] = React.useState(false);
  const [modelPool] = React.useState(AMF_ALL_MODELS);

  const filtered = React.useMemo(() => {
    const q = query.trim().toLowerCase();
    if (!q) return modelPool;
    return modelPool.filter((m) => m.toLowerCase().includes(q));
  }, [query, modelPool]);

  const handleFetch = () => {
    setFetching(true);
    setTimeout(() => setFetching(false), 1400);
  };

  return (
    <div className="cam-model-box">
      <div className="cam-selected-area">
        <div className="cam-selected-header">
          <span className="cam-selected-label">Selected Models ({selectedModels.length})</span>
          {selectedModels.length > 0 && (
            <button className="cam-clear-all" onClick={onClearAll}>Clear All</button>
          )}
        </div>
        <div className="cam-chips-row">
          {selectedModels.length === 0 && (
            <span style={{ fontSize: 11, color: 'hsl(var(--muted-foreground))' }}>No models selected</span>
          )}
          {selectedModels.map((m) => (
            <span key={m} className="cam-chip">
              {m}
              <button className="cam-chip-x" onClick={() => onToggle(m)} title={`Remove ${m}`}>×</button>
            </span>
          ))}
        </div>
      </div>

      <div className="cam-available-area">
        <div className="cam-available-header">
          <span className="cam-available-label">Available Models</span>
          <div className="cam-available-actions">
            <button className={`cam-link-btn${fetching ? ' loading' : ''}`} onClick={handleFetch} disabled={fetching}>
              {fetching ? <><span className="cam-fetch-spin"></span> Fetching…</> : 'Fetch Models'}
            </button>
            <button className="cam-link-btn" onClick={() => onSelectAll(filtered)}>Select All ({filtered.length})</button>
          </div>
        </div>

        <div className="cam-search-wrap">
          <span className="cam-search-icon"><AmfIcon name="search" size={13} /></span>
          <input className="cam-search-input" value={query} onChange={(e) => setQuery(e.target.value)} placeholder="Filter models…" />
          {query && <button className="cam-search-clear" onClick={() => setQuery('')}><AmfIcon name="x" size={12} /></button>}
        </div>

        <div className="cam-model-list">
          {filtered.length === 0 && <div className="cam-no-models">No models match "{query}"</div>}
          {filtered.map((m) => {
            const checked = selectedModels.includes(m);
            return (
              <div key={m} className={`cam-model-item${checked ? ' checked' : ''}`} onClick={() => onToggle(m)}>
                <input type="checkbox" className="cam-model-cb" checked={checked} readOnly />
                <span className="cam-model-name">{m}</span>
              </div>
            );
          })}
        </div>
      </div>
    </div>
  );
}

/* ── The form card (sections + footer) ── */
function ApiModelForm({ showCancel = true, title, subtitle }) {
  /* Provider Connection */
  const [name, setName] = React.useState('');
  const [apiFormat, setApiFormat] = React.useState('openai');
  const [baseUrl, setBaseUrl] = React.useState('https://api.openai.com/v1');
  const [useApiKey, setUseApiKey] = React.useState(true);
  const [apiKey, setApiKey] = React.useState('sk-proj-••••••••••••••••••••••••••••••••••••••••••••••••••••');
  const [showKey, setShowKey] = React.useState(false);

  /* Anthropic Setup Token extras */
  const [extraHeaders, setExtraHeaders] = React.useState(AMF_DEFAULT_EXTRA_HEADERS);
  const [extraBody, setExtraBody] = React.useState(AMF_DEFAULT_EXTRA_BODY);

  /* LLM Liberty OAuth envelope */
  const [libertyCreds, setLibertyCreds] = React.useState('');
  const [copiedCmd, setCopiedCmd] = React.useState(false);

  /* Selecting a format auto-fills its default Base URL. */
  const onFormatChange = (value) => {
    setApiFormat(value);
    const next = AMF_FORMAT_MAP[value];
    if (next && next.baseUrl) setBaseUrl(next.baseUrl);
  };

  const fmt = AMF_FORMAT_MAP[apiFormat] || {};
  const isLiberty = !!fmt.liberty;
  const showExtras = !!fmt.extras;

  const copyCmd = () => {
    const cmd = 'npx @bodhiapp/llm-liberty@latest login';
    if (navigator.clipboard) navigator.clipboard.writeText(cmd).catch(() => {});
    setCopiedCmd(true);
    setTimeout(() => setCopiedCmd(false), 1400);
  };

  /* Request Routing */
  const [enablePrefix, setEnablePrefix] = React.useState(true);
  const [prefix, setPrefix] = React.useState('openai/');
  const [fwdMode, setFwdMode] = React.useState('selected');

  /* Model Selection */
  const [selectedModels, setSelectedModels] = React.useState(['gpt-5-mini', 'gpt-5.4-mini', 'gpt-5.1-codex-mini']);

  const toggleModel = (m) => setSelectedModels((prev) => prev.includes(m) ? prev.filter((x) => x !== m) : [...prev, m]);
  const selectAll = (models) => setSelectedModels((prev) => {
    const next = [...prev];
    models.forEach((m) => { if (!next.includes(m)) next.push(m); });
    return next;
  });

  const urlHint = 'Enter the complete API endpoint URL for your provider';

  return (
    <div className="bf-card">
      {title && (
        <div className="bf-card-head">
          <h1 className="bf-card-title">{title}</h1>
          {subtitle && <p className="bf-card-sub">{subtitle}</p>}
        </div>
      )}
      <div className="bf-card-body">

        {/* ══ PROVIDER CONNECTION ══ */}
        <div className="bf-section">
          <div className="bf-section-title">Provider Connection</div>

          <div className="bf-field">
            <label className="bf-label"><span className="bf-label-text">Name</span><span className="bf-req">*</span></label>
            <input className="bf-input" value={name} onChange={(e) => setName(e.target.value)} placeholder="e.g. openai-prod" />
            <div className="bf-hint">A unique name to identify this API model configuration.</div>
          </div>

          <div className="bf-field">
            <label className="bf-label"><span className="bf-label-text">API Format</span><span className="bf-req">*</span></label>
            <select className="bf-select" value={apiFormat} onChange={(e) => onFormatChange(e.target.value)}>
              {AMF_FORMATS.map((f) => (
                <option key={f.value} value={f.value}>{f.label}</option>
              ))}
            </select>
          </div>

          {isLiberty ? (
            /* ── LLM Liberty OAuth envelope ── */
            <div className="bf-field">
              <label className="bf-label"><span className="bf-label-text">LLM Liberty OAuth Credentials</span></label>
              <div className="bf-hint" style={{ marginTop: 0, marginBottom: 8 }}>Run the login command to get credentials, then paste the JSON output below.</div>
              <div className="amf-cmd">
                <code className="amf-cmd-text">npx @bodhiapp/llm-liberty@latest login</code>
                <button className="amf-cmd-copy" type="button" onClick={copyCmd} title="Copy command">
                  <AmfIcon name={copiedCmd ? 'check' : 'copy'} size={14} />
                </button>
              </div>
              <textarea
                className="bf-textarea bf-input-mono"
                style={{ minHeight: 168 }}
                value={libertyCreds}
                onChange={(e) => setLibertyCreds(e.target.value)}
                placeholder={AMF_LIBERTY_PLACEHOLDER}
                spellCheck={false}
              />
            </div>
          ) : (
            <>
              <div className="bf-field">
                <label className="bf-label"><span className="bf-label-text">Base URL</span><span className="bf-req">*</span></label>
                <input className="bf-input" type="url" value={baseUrl} onChange={(e) => setBaseUrl(e.target.value)} placeholder="https://api.openai.com/v1" />
                <div className="bf-hint">{urlHint}</div>
              </div>

              {/* API Key — input ALWAYS visible, disabled until "Use API key" is checked */}
              <div className="bf-field">
                <label className="bf-label"><span className="bf-label-text">API Key</span></label>
                <div className="bf-check-row">
                  <input type="checkbox" id="amf-useApiKey" className="bf-checkbox" checked={useApiKey} onChange={(e) => setUseApiKey(e.target.checked)} />
                  <label htmlFor="amf-useApiKey" className="bf-check-label">Use API key</label>
                </div>
                <div className={`bf-indent${useApiKey ? '' : ' is-locked'}`}>
                  <div className="bf-pw-wrap">
                    <input
                      className="bf-input bf-input-mono"
                      type={showKey ? 'text' : 'password'}
                      value={apiKey}
                      onChange={(e) => setApiKey(e.target.value)}
                      placeholder="sk-…"
                      autoComplete="new-password"
                      disabled={!useApiKey}
                    />
                    <button className="bf-pw-toggle" type="button" onClick={() => setShowKey((v) => !v)} disabled={!useApiKey} title={showKey ? 'Hide key' : 'Show key'}>
                      <AmfIcon name={showKey ? 'eye-off' : 'eye'} size={14} />
                    </button>
                  </div>
                  <div className="bf-hint">Your API key is stored securely.</div>
                </div>
              </div>

              {/* Extras — Anthropic Setup Token only */}
              {showExtras && (
                <>
                  <div className="bf-field">
                    <label className="bf-label"><span className="bf-label-text">Extra Headers</span><span className="bf-optional">Optional</span></label>
                    <textarea
                      className="bf-textarea bf-input-mono"
                      style={{ minHeight: 120 }}
                      value={extraHeaders}
                      onChange={(e) => setExtraHeaders(e.target.value)}
                      spellCheck={false}
                    />
                    <div className="bf-hint">JSON object of headers added to every request.</div>
                  </div>
                  <div className="bf-field">
                    <label className="bf-label"><span className="bf-label-text">Extra Body</span><span className="bf-optional">Optional</span></label>
                    <textarea
                      className="bf-textarea bf-input-mono"
                      style={{ minHeight: 140 }}
                      value={extraBody}
                      onChange={(e) => setExtraBody(e.target.value)}
                      spellCheck={false}
                    />
                    <div className="bf-hint">JSON merged into every request body.</div>
                  </div>
                </>
              )}
            </>
          )}
        </div>

        <div className="bf-divider"></div>

        {/* ══ REQUEST ROUTING ══ */}
        <div className="bf-section">
          <div className="bf-section-title">Request Routing</div>

          {/* Model Prefix — input ALWAYS visible, disabled until "Enable prefix" is checked */}
          <div className="bf-field">
            <label className="bf-label"><span className="bf-label-text">Model Prefix</span></label>
            <div className="bf-check-row">
              <input type="checkbox" id="amf-enablePrefix" className="bf-checkbox" checked={enablePrefix} onChange={(e) => setEnablePrefix(e.target.checked)} />
              <label htmlFor="amf-enablePrefix" className="bf-check-label">Enable prefix</label>
            </div>
            <div className={`bf-indent${enablePrefix ? '' : ' is-locked'}`}>
              <input className="bf-input" value={prefix} onChange={(e) => setPrefix(e.target.value)} placeholder="e.g. openai/" disabled={!enablePrefix} />
              <div className="bf-hint">Add a prefix to all model names (useful for organization or API routing).</div>
              <div className="bf-hint-example">Example: {prefix || 'openai/'}gpt-4</div>
            </div>
          </div>

          {/* Request Forwarding Mode */}
          <div className="bf-field">
            <label className="bf-label"><span className="bf-label-text">Request Forwarding Mode</span></label>
            <div className="bf-radio-group">
              <div className={`bf-radio-option${fwdMode === 'all' ? ' selected' : ''}`} onClick={() => setFwdMode('all')}>
                <div className="bf-radio-dot"><div className="bf-radio-dot-inner" style={{ transform: fwdMode === 'all' ? 'scale(1)' : 'scale(0)' }}></div></div>
                <span className="bf-radio-text">Forward all requests with prefix</span>
              </div>
              <div className={`bf-radio-option${fwdMode === 'selected' ? ' selected' : ''}`} onClick={() => setFwdMode('selected')}>
                <div className="bf-radio-dot"><div className="bf-radio-dot-inner" style={{ transform: fwdMode === 'selected' ? 'scale(1)' : 'scale(0)' }}></div></div>
                <span className="bf-radio-text">Forward for selected models only</span>
              </div>
            </div>
          </div>
        </div>

        {/* ══ MODEL SELECTION ══ */}
        {fwdMode === 'selected' && (
          <>
            <div className="bf-divider"></div>
            <div className="bf-section">
              <div className="bf-section-title">Model Selection</div>
              <p className="bf-section-desc">Select which models you'd like to use. Only the selected set will be forwarded through the alias prefix.</p>
              <AmfModelSelection
                selectedModels={selectedModels}
                onToggle={toggleModel}
                onClearAll={() => setSelectedModels([])}
                onSelectAll={selectAll}
              />
            </div>
          </>
        )}
      </div>

      {/* ══ FOOTER — actions ══ */}
      <div className="bf-footer">
        <button className="bf-btn bf-btn-secondary" disabled={isLiberty} title={isLiberty ? 'Not available for LLM Liberty OAuth' : undefined}><AmfIcon name="plug-zap" size={13} /> Test Connection</button>
        <div className="bf-footer-spacer"></div>
        {showCancel && <button className="bf-btn bf-btn-ghost">Cancel</button>}
        <button className="bf-btn bf-btn-primary">Create API Model</button>
      </div>
    </div>
  );
}

Object.assign(window, { ApiModelForm, AmfIcon, AMF_ALL_MODELS });
