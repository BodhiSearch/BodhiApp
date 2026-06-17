/* ═══════════════════════════════════════════════════
   CREATE FALLBACK ALIAS — React App
   Sections:
     1 · Identity (alias name)
     2 · Fallback Sequence (ordered alias steps)
   Right sidebar: live chain preview + how-it-works + tips.
═══════════════════════════════════════════════════ */

/* ── Available aliases ────────────────────────────
   In the real app this comes from the API; mocked here.
   For api-models the `fwdMode` field mirrors how the
   API-model alias was configured ("selected" or "all"),
   determining whether the model field below is a
   constrained dropdown or a free-text autocomplete.
─────────────────────────────────────────────────── */
const AVAILABLE_ALIASES = [
  { id:'afrideva/Llama-68M-Chat:Q8_0',     type:'local-file',  display:'afrideva/Llama-68M-Chat:Q8_0',     size:'0.07 GB' },
  { id:'Qwen/Qwen3-Coder-32B:Q4_K_M',      type:'local-file',  display:'Qwen/Qwen3-Coder-32B:Q4_K_M',      size:'18.5 GB' },
  { id:'meta-llama/Llama-3.3-70B:Q4_K_M',  type:'local-file',  display:'meta-llama/Llama-3.3-70B:Q4_K_M',  size:'35.0 GB' },
  { id:'my-qwen-coder',                    type:'model-alias', display:'my-qwen-coder', backing:'Qwen/Qwen3-Coder-32B:Q4_K_M' },
  { id:'01kp50czqbcgnhnwtnv7jq2s',         type:'api-model',   display:'01kp50czqbcgnhnwtnv7jq2s',  provider:'ANTHROPIC',       fwdMode:'selected', models:['claude-sonnet-4-5'] },
  { id:'01kp506g2crx8pgqtp4ts1jfh7',       type:'api-model',   display:'01kp506g2crx8pgqtp4ts1jfh7', provider:'ANTHROPIC_OAUTH', fwdMode:'selected', models:['claude-opus-4'] },
  { id:'openai-gpt-main',                  type:'api-model',   display:'openai-gpt-main',           provider:'OPENAI',          fwdMode:'selected', models:['gpt-5','gpt-4o','gpt-4o-mini'] },
  { id:'openrouter-all',                   type:'api-model',   display:'openrouter-all',            provider:'OPENROUTER',      fwdMode:'all',      models:[] },
];

/* Broader autocomplete pool for api-models with fwdMode:'all' */
const ALL_KNOWN_MODELS = [
  'gpt-4o','gpt-4o-mini','gpt-4-turbo','gpt-3.5-turbo','o1','o1-mini','o3','o3-mini',
  'claude-opus-4','claude-sonnet-4-5','claude-haiku-3-5','claude-3-5-sonnet','claude-3-opus',
  'gemini-2.0-flash','gemini-1.5-pro','gemini-1.5-flash',
  'llama-3.3-70b-versatile','mixtral-8x7b-32768','gemma2-9b-it',
  'deepseek-v3','deepseek-r1',
  'qwen-plus','qwen-turbo',
];

/* Type-badge style table */
const TYPE_CFG = {
  'local-file':  { bg:'var(--c-saffron-bg)', bd:'var(--c-saffron-bd)', text:'var(--c-saffron-text)', icon:'hard-drive', label:'Local File' },
  'model-alias': { bg:'var(--c-lotus-bg)',   bd:'var(--c-lotus-bd)',   text:'var(--c-lotus-text)',   icon:'tag',        label:'Model Alias' },
  'api-model':   { bg:'var(--c-indigo-bg)',  bd:'var(--c-indigo-bd)',  text:'var(--c-indigo-text)',  icon:'at-sign',    label:'API Model' },
};

/* ── Lucide icon helper ─────────────────── */
function Icon({ name, size = 13, style = {} }) {
  const ref = React.useRef(null);
  React.useEffect(() => {
    if (!ref.current) return;
    ref.current.innerHTML = '';
    const el = document.createElement('i');
    el.setAttribute('data-lucide', name);
    ref.current.appendChild(el);
    if (typeof lucide !== 'undefined') lucide.createIcons({ nodes: [el] });
  }, [name]);
  return (
    <span
      ref={ref}
      style={{
        display:'inline-flex', width:size, height:size,
        alignItems:'center', justifyContent:'center', flexShrink:0,
        ...style,
      }}
    />
  );
}

/* ── Type badge ─────────────────────────── */
function TypeBadge({ type, small = false }) {
  const cfg = TYPE_CFG[type];
  if (!cfg) return null;
  const padding = small ? '1px 5px' : '2px 7px';
  const fontSize = small ? 9.5 : 10;
  const iconSize = small ? 8 : 9;
  return (
    <span style={{
      display:'inline-flex', alignItems:'center', gap:4,
      padding, borderRadius:99,
      fontSize, fontWeight:600,
      background:cfg.bg, border:`1px solid ${cfg.bd}`, color:cfg.text,
      whiteSpace:'nowrap',
    }}>
      <Icon name={cfg.icon} size={iconSize} />
      {cfg.label}
    </span>
  );
}

/* ── Provider badge ─────────────────────── */
function ProviderBadge({ provider }) {
  if (!provider) return null;
  return (
    <span style={{
      fontSize:10, fontWeight:700, padding:'2px 6px', borderRadius:4,
      background:'var(--c-leaf-bg)', color:'var(--c-leaf-text)',
      border:'1px solid var(--c-leaf-bd)', whiteSpace:'nowrap',
      letterSpacing:.02,
    }}>{provider}</span>
  );
}

/* ── Alias combobox ─────────────────────────────────
   Lists every configured alias. Each option shows the
   type badge + name + (optional) provider badge.
─────────────────────────────────────────────────── */
function AliasCombobox({ value, onChange, excludeIds = [] }) {
  const [open, setOpen]   = React.useState(false);
  const [query, setQuery] = React.useState('');
  const inputRef = React.useRef(null);
  const wrapRef  = React.useRef(null);

  const selected = AVAILABLE_ALIASES.find(a => a.id === value);
  const filtered = AVAILABLE_ALIASES.filter(a => {
    if (excludeIds.includes(a.id) && a.id !== value) return false;
    const q = query.toLowerCase();
    return !q
      || a.display.toLowerCase().includes(q)
      || a.type.includes(q)
      || (a.provider || '').toLowerCase().includes(q);
  });

  React.useEffect(() => {
    function onDocDown(e) {
      if (wrapRef.current && !wrapRef.current.contains(e.target)) {
        setOpen(false); setQuery('');
      }
    }
    document.addEventListener('mousedown', onDocDown);
    return () => document.removeEventListener('mousedown', onDocDown);
  }, []);

  const pick = a => { onChange(a.id); setOpen(false); setQuery(''); };

  return (
    <div className="cfm-combobox" ref={wrapRef}>
      <div
        className={'cfm-combobox-trigger' + (open ? ' open' : '')}
        onClick={() => {
          setOpen(v => !v);
          if (!open) setTimeout(() => inputRef.current?.focus(), 30);
        }}
      >
        {open ? (
          <input
            ref={inputRef}
            className="cfm-combobox-input"
            value={query}
            onChange={e => setQuery(e.target.value)}
            placeholder="Search aliases…"
            onClick={e => e.stopPropagation()}
          />
        ) : (
          <div className="cfm-combobox-value">
            {selected
              ? selected.display
              : <span className="cfm-combobox-placeholder">Select an alias…</span>
            }
          </div>
        )}
        <span className="cfm-combobox-caret">
          <Icon name="chevrons-up-down" size={13} />
        </span>
      </div>

      {open && (
        <div className="cfm-combobox-dropdown">
          {filtered.length === 0 && (
            <div className="cfm-combobox-empty">No aliases match "{query}"</div>
          )}
          {filtered.map(a => (
            <div
              key={a.id}
              className={'cfm-combobox-item' + (a.id === value ? ' selected' : '')}
              onClick={() => pick(a)}
            >
              <TypeBadge type={a.type} small />
              <span className="cfm-combobox-item-name">{a.display}</span>
              <ProviderBadge provider={a.provider} />
              {a.id === value && (
                <Icon name="check" size={12} style={{ color:'var(--c-teal-text)' }} />
              )}
            </div>
          ))}
        </div>
      )}
    </div>
  );
}

/* ── Free model autocomplete input (fwdMode:'all') ── */
function FreeModelInput({ value, onChange, invalid }) {
  const [focused, setFocused] = React.useState(false);
  const wrapRef = React.useRef(null);

  const suggestions = ALL_KNOWN_MODELS
    .filter(m => !value || m.toLowerCase().includes(value.toLowerCase()))
    .slice(0, 8);

  React.useEffect(() => {
    function onDocDown(e) {
      if (wrapRef.current && !wrapRef.current.contains(e.target)) setFocused(false);
    }
    document.addEventListener('mousedown', onDocDown);
    return () => document.removeEventListener('mousedown', onDocDown);
  }, []);

  return (
    <div className="cfm-model-input-wrap" ref={wrapRef}>
      <input
        className={'cfm-model-input' + (invalid ? ' invalid' : '')}
        value={value || ''}
        onChange={e => onChange(e.target.value)}
        onFocus={() => setFocused(true)}
        placeholder="e.g. gpt-4o, anthropic/claude-sonnet-4-5…"
        autoComplete="off"
        spellCheck={false}
      />
      {focused && (
        <div className="cfm-autocomplete">
          <div className="cfm-autocomplete-hint">
            <Icon name="info" size={9} />
            free-text — any model name; suggestions are non-binding
          </div>
          {suggestions.length === 0
            ? <div className="cfm-combobox-empty" style={{ padding:'10px 12px' }}>Type any model name</div>
            : suggestions.map(m => (
                <div
                  key={m}
                  className="cfm-autocomplete-item"
                  onMouseDown={e => { e.preventDefault(); onChange(m); setFocused(false); }}
                >
                  <span>{m}</span>
                  {m === value && <Icon name="check" size={11} style={{ color:'var(--c-teal-text)' }} />}
                </div>
              ))
          }
        </div>
      )}
    </div>
  );
}

/* ── Model field — shown only for API-model aliases ── */
function ModelField({ alias, value, onChange }) {
  if (!alias || alias.type !== 'api-model') return null;

  if (alias.fwdMode === 'selected') {
    const invalid = !value || !alias.models.includes(value);
    return (
      <div className="cfm-model-field">
        <div className="cfm-sub-label">
          Route to model
          <span className="cfm-model-mode-pill selected-only">pre-configured only</span>
        </div>
        <select
          className={'cfm-model-select' + (invalid ? ' invalid' : '')}
          value={value || ''}
          onChange={e => onChange(e.target.value)}
        >
          <option value="">Select model…</option>
          {alias.models.map(m => <option key={m} value={m}>{m}</option>)}
        </select>
        {invalid && (
          <div className="cfm-err-text">
            Required — select one of the {alias.models.length} model{alias.models.length === 1 ? '' : 's'} this alias exposes.
          </div>
        )}
      </div>
    );
  }

  /* fwdMode: 'all' — free-text autocomplete */
  const invalid = !value || !value.trim();
  return (
    <div className="cfm-model-field">
      <div className="cfm-sub-label">
        Route to model
        <span className="cfm-model-mode-pill forward-all">any model · forward-all</span>
      </div>
      <FreeModelInput value={value} onChange={onChange} invalid={invalid} />
      {invalid && (
        <div className="cfm-err-text">Required — specify which model to forward to.</div>
      )}
    </div>
  );
}

/* ── Step card ─────────────────────────── */
function StepCard({ step, index, total, onRemove, onChange, onMove, onToggleEnabled, otherAliasIds }) {
  const alias    = AVAILABLE_ALIASES.find(a => a.id === step.aliasId);
  const isFirst  = index === 0;
  const isLast   = index === total - 1;
  const enabled  = step.enabled !== false;  // default true if undefined

  const IcoBtn = ({ onClick, icon, disabled, title, danger }) => (
    <button
      className={'cfm-step-ico-btn' + (danger ? ' del' : '')}
      onClick={onClick} disabled={disabled} title={title}
    >
      <Icon name={icon} size={12} />
    </button>
  );

  return (
    <div className={'cfm-step-card' + (enabled ? '' : ' disabled')}>
      {/* Header */}
      <div className="cfm-step-header">
        <span className="cfm-drag-handle" title="Drag to reorder">
          <Icon name="grip-vertical" size={14} />
        </span>
        <span className="cfm-step-num">{index + 1}</span>
        <span className="cfm-step-label">
          Step {index + 1}
          {enabled && isFirst && total > 1 && <span className="cfm-step-note">primary</span>}
          {enabled && isLast && !isFirst && <span className="cfm-step-note">final fallback</span>}
          {!enabled && <span className="cfm-step-note warn">disabled</span>}
        </span>
        <span className="cfm-step-spacer" />
        {/* Enable / disable switch */}
        <label
          className="cfm-enable-toggle"
          title={enabled ? 'Disable this step temporarily (kept in sequence, skipped at runtime)' : 'Re-enable this step'}
        >
          <input
            type="checkbox"
            checked={enabled}
            onChange={e => onToggleEnabled(step.id, e.target.checked)}
          />
          <span className="cfm-enable-track"><span className="cfm-enable-thumb" /></span>
        </label>
        <IcoBtn onClick={() => onMove(index, -1)} icon="chevron-up"   disabled={isFirst}     title="Move up" />
        <IcoBtn onClick={() => onMove(index, +1)} icon="chevron-down" disabled={isLast}      title="Move down" />
        <IcoBtn onClick={() => onRemove(step.id)} icon="x"            disabled={total <= 1}  title="Remove step" danger />
      </div>

      {/* Alias selector */}
      <div>
        <div className="cfm-sub-label">Model alias</div>
        <AliasCombobox
          value={step.aliasId}
          onChange={id => onChange(step.id, { aliasId:id, model:null })}
          excludeIds={otherAliasIds}
        />
        {alias && (
          <div className="cfm-alias-meta">
            <TypeBadge type={alias.type} small />
            <ProviderBadge provider={alias.provider} />
            {alias.backing && (
              <span className="cfm-alias-meta-text">→ {alias.backing}</span>
            )}
            {alias.size && (
              <span className="cfm-alias-meta-text">· {alias.size}</span>
            )}
            {alias.type === 'api-model' && alias.fwdMode === 'all' && (
              <span className="cfm-alias-meta-text" style={{ color:'var(--c-leaf-text)' }}>
                · forwards any model
              </span>
            )}
          </div>
        )}
        {!step.aliasId && (
          <div className="cfm-err-text" style={{ marginTop:5 }}>Select an alias to continue.</div>
        )}
      </div>

      {/* Conditional model field for API models */}
      <ModelField
        alias={alias}
        value={step.model}
        onChange={m => onChange(step.id, { model: m })}
      />
    </div>
  );
}

/* ── Step connector (vertical "on error" pill) ── */
function StepConnector() {
  return (
    <div className="cfm-connector">
      <div className="cfm-connector-line" />
      <div className="cfm-connector-badge">
        <Icon name="arrow-down" size={9} /> on error
      </div>
      <div className="cfm-connector-line" />
    </div>
  );
}

/* ── Chain preview (sidebar) ─────────── */
function ChainPreview({ steps }) {
  if (steps.length === 0) {
    return <div className="cfm-chain-empty">No steps yet — add one to start.</div>;
  }
  return (
    <div className="cfm-chain">
      {steps.map((step, i) => {
        const alias = AVAILABLE_ALIASES.find(a => a.id === step.aliasId);
        const enabled = step.enabled !== false;
        const needsModel = enabled && alias?.type === 'api-model';
        const missingModel = needsModel && !step.model;
        return (
          <React.Fragment key={step.id}>
            <div className={'cfm-chain-item' + (enabled ? '' : ' disabled')}>
              <span className="cfm-chain-num">{i + 1}</span>
              <div className="cfm-chain-body">
                <div className="cfm-chain-name">
                  {alias
                    ? alias.display
                    : <span style={{ color:'hsl(var(--muted-foreground))', fontStyle:'italic', fontFamily:'var(--font-sans)' }}>(not selected)</span>}
                  {!enabled && <span className="cfm-chain-skip-tag">skipped</span>}
                </div>
                {step.model && <div className="cfm-chain-model">→ {step.model}</div>}
                {missingModel && (
                  <div className="cfm-chain-model err">→ model required</div>
                )}
              </div>
              {alias && <TypeBadge type={alias.type} small />}
            </div>
            {i < steps.length - 1 && (
              <div className={'cfm-chain-arrow' + (enabled ? '' : ' dim')}>↓ on error</div>
            )}
          </React.Fragment>
        );
      })}
    </div>
  );
}

/* ═══════════════════════════════════════════════════
   MAIN APP
═══════════════════════════════════════════════════ */
function CreateFallbackModelApp() {
  const [aliasName, setAliasName] = React.useState('smart-fallback');
  const [nextStepId, setNextStepId] = React.useState(10);
  const [steps, setSteps] = React.useState([
    { id:1, aliasId:'openai-gpt-main',          model:'gpt-4o',            enabled:true },
    { id:2, aliasId:'01kp50czqbcgnhnwtnv7jq2s', model:'claude-sonnet-4-5', enabled:true },
    { id:3, aliasId:'my-qwen-coder',            model:null,                enabled:true },
  ]);

  const addStep = () => {
    setSteps(prev => [...prev, { id:nextStepId, aliasId:null, model:null, enabled:true }]);
    setNextStepId(n => n + 1);
  };
  const removeStep = id => setSteps(prev => prev.filter(s => s.id !== id));
  const updateStep = (id, patch) =>
    setSteps(prev => prev.map(s => s.id === id ? { ...s, ...patch } : s));
  const toggleEnabled = (id, enabled) =>
    setSteps(prev => prev.map(s => s.id === id ? { ...s, enabled } : s));
  const moveStep = (index, dir) => {
    setSteps(prev => {
      const next = [...prev];
      const target = index + dir;
      if (target < 0 || target >= next.length) return prev;
      [next[index], next[target]] = [next[target], next[index]];
      return next;
    });
  };

  /* Validation */
  const validationMsg = React.useMemo(() => {
    const name = aliasName.trim();
    if (!name) return 'Alias name is required.';
    if (!/^[a-z0-9-]+$/.test(name)) return 'Alias name: lowercase, digits and dashes only.';
    const enabledSteps = steps.filter(s => s.enabled !== false);
    if (enabledSteps.length < 2) {
      if (steps.length < 2) return 'Add at least 2 steps for a useful chain.';
      return 'At least 2 steps must be enabled for the chain to fall back.';
    }
    for (let i = 0; i < steps.length; i++) {
      const s = steps[i];
      if (s.enabled === false) continue; // skip disabled steps
      if (!s.aliasId) return `Step ${i+1}: select an alias.`;
      const alias = AVAILABLE_ALIASES.find(a => a.id === s.aliasId);
      if (alias?.type === 'api-model' && !s.model) {
        return `Step ${i+1}: choose a model to route to.`;
      }
      if (alias?.type === 'api-model' && alias.fwdMode === 'selected' && !alias.models.includes(s.model)) {
        return `Step ${i+1}: "${s.model}" isn't in the alias's allowed list.`;
      }
    }
    return null;
  }, [aliasName, steps]);
  const isValid = !validationMsg;
  const enabledCount = steps.filter(s => s.enabled !== false).length;

  /* Aliases already used in earlier steps — we still allow re-use,
     but in practice you usually don't repeat. We won't enforce
     uniqueness, just expose the ids list in case we want to. */
  const _otherAliasIds = []; // empty => no exclusion

  return (
    <>
    <AppShell
      section="models" subPage="new-fallback-model" resizeKey="createmodel"
      breadcrumb={[
        { label: 'Bodhi', href: 'Bodhi Chat.html' },
        { label: 'Models', href: 'Bodhi Models.html' },
        { label: 'New Fallback Alias', current: true },
      ]}
      contentClass="flush" mainScroll={false}
    >
        {/* ── Scroll · form column + info rail ── */}
        <div className="bf-scroll">
          <div className="cfm-layout">

            {/* FORM COLUMN */}
            <div className="cfm-form-col">
              <div className="bf-page-head">
                <h1 className="bf-page-title">New Fallback Alias</h1>
                <p className="bf-page-sub">
                  Chain multiple model aliases into a prioritized sequence. When a request arrives with this alias name as its <code className="cfm-code">model</code>, each step is tried in order — on error, the next step takes over.
                </p>
              </div>

              <div className="bf-card">
                <div className="bf-card-body">

                  {/* ═══ Section 1 · Identity ═══ */}
                  <div className="bf-section">
                    <div className="bf-section-title">Identity</div>
                    <div className="bf-field">
                      <label className="bf-label">
                        <span className="bf-label-text">Alias name</span>
                        <span className="bf-req">*</span>
                      </label>
                      <input
                        className="bf-input bf-input-mono"
                        value={aliasName}
                        onChange={e => setAliasName(e.target.value.toLowerCase().replace(/[^a-z0-9-]/g, ''))}
                        placeholder="e.g. smart-fallback"
                      />
                      <div className="bf-hint">
                        Lowercase letters, digits, and dashes only. This becomes the <code className="cfm-code">model</code> value clients send to your API.
                      </div>
                    </div>
                  </div>

                  <div className="bf-divider"></div>

                  {/* ═══ Section 2 · Fallback Sequence ═══ */}
                  <div className="bf-section">
                    <div className="bf-section-title">Fallback Sequence</div>
                    <p className="bf-section-desc">
                      Tried top-to-bottom. The first step that responds without error wins.
                      Pick any alias type — local files, model aliases, or API models.
                    </p>

                    <div className="cfm-steps">
                      {steps.map((step, i) => (
                        <React.Fragment key={step.id}>
                          <StepCard
                            step={step}
                            index={i}
                            total={steps.length}
                            onRemove={removeStep}
                            onChange={updateStep}
                            onMove={moveStep}
                            onToggleEnabled={toggleEnabled}
                            otherAliasIds={_otherAliasIds}
                          />
                          {i < steps.length - 1 && <StepConnector />}
                        </React.Fragment>
                      ))}
                    </div>

                    <button className="cfm-add-step" onClick={addStep}>
                      <Icon name="plus-circle" size={14} />
                      Add step
                    </button>
                  </div>
                </div>{/* end card-body */}

                {/* ═══ FOOTER — the ONLY place actions live ═══ */}
                <div className="bf-footer">
                  {validationMsg ? (
                    <div className="cfm-foot-msg warn">
                      <Icon name="info" size={12} />
                      {validationMsg}
                    </div>
                  ) : (
                    <div className="cfm-foot-msg ok">
                      <Icon name="check-circle" size={12} />
                      Ready to create.
                    </div>
                  )}
                  <div className="bf-footer-spacer" />
                  <button className="bf-btn bf-btn-ghost">Cancel</button>
                  <button className="bf-btn bf-btn-primary" disabled={!isValid}>
                    <Icon name="route" size={13} />
                    Create Fallback Alias
                  </button>
                </div>
              </div>{/* end card */}
            </div>{/* end form-col */}

            {/* INFO RAIL */}
            <aside className="cfm-side-col">
            {/* Validation badge */}
            {validationMsg ? (
              <div className="cfm-validation-badge warn">
                <Icon name="alert-circle" size={13} />
                <div>{validationMsg}</div>
              </div>
            ) : (
              <div className="cfm-validation-badge ok">
                <Icon name="check-circle" size={13} />
                <div>
                  Ready to create. Will route <strong style={{fontFamily:'var(--font-mono)'}}>{aliasName}</strong> through {enabledCount} of {steps.length} step{steps.length === 1 ? '' : 's'}.
                </div>
              </div>
            )}

            {/* Chain preview */}
            <div className="cfm-side-card">
              <div className="cfm-side-card-head">
                <Icon name="route" size={11} />
                Routing chain
              </div>
              <div className="cfm-side-card-body">
                <ChainPreview steps={steps} />
              </div>
            </div>

            {/* How it works */}
            <div className="cfm-side-card">
              <div className="cfm-side-card-head">
                <Icon name="help-circle" size={11} />
                How it works
              </div>
              <div className="cfm-side-card-body">
                {[
                  <>Client sends a request with <code style={{fontFamily:'var(--font-mono)', fontSize:10, background:'hsl(var(--muted))', padding:'0 3px', borderRadius:2}}>model={aliasName || '<alias>'}</code>.</>,
                  'Step 1 is tried first. If it returns a successful response, we stop and return it.',
                  'If Step 1 errors (timeout, 5xx, rate limit, model down…), Step 2 is tried — and so on.',
                  'A final error is only surfaced when every step has failed.',
                ].map((text, i) => (
                  <div key={i} className="cfm-how-item">
                    <span className="cfm-how-num">{i + 1}</span>
                    <span className="cfm-how-text">{text}</span>
                  </div>
                ))}
              </div>
            </div>

            {/* Tips */}
            <div className="cfm-side-card">
              <div className="cfm-side-card-head">
                <Icon name="lightbulb" size={11} />
                Tips
              </div>
              <div className="cfm-side-card-body">
                {[
                  'Put fastest / cheapest aliases first to minimize cost and latency.',
                  'End with a local model alias for a self-hosted last resort.',
                  'Toggle a step off to skip it temporarily — the sequence and config are preserved.',
                  'For API-model steps, the model field is constrained when the alias was created in "selected" mode — free-text in "forward-all" mode.',
                ].map((tip, i) => (
                  <div key={i} className="cfm-tip-row">
                    <span className="cfm-tip-bullet">›</span>
                    <span>{tip}</span>
                  </div>
                ))}
              </div>
            </div>
          </aside>{/* end info rail */}
          </div>{/* end cfm-layout */}
        </div>{/* end bf-scroll */}
    </AppShell>
    </>
  );
}

const cfmRoot = ReactDOM.createRoot(document.getElementById('root'));
cfmRoot.render(<CreateFallbackModelApp />);
