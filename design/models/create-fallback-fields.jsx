/* ═══════════════════════════════════════════════════
   Create Model Router — alias/model input fields
   models/create-fallback-fields.jsx   (load after create-fallback-data.jsx)

   The alias combobox, the free-text model autocomplete (for api-models
   configured to forward ANY model), and the conditional ModelField that
   switches between a constrained <select> and the free input based on
   the alias's fwdMode. Adds AliasCombobox, FreeModelInput, ModelField
   to window.CFM.
═══════════════════════════════════════════════════ */
const CFM = window.CFM;
const { AVAILABLE_ALIASES, ALL_KNOWN_MODELS, Icon, TypeBadge, ProviderBadge } = CFM;

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

Object.assign(CFM, { AliasCombobox, FreeModelInput, ModelField });
