/* ═══════════════════════════════════════════════════
   MODEL ACCESS PICKER — Shared React component
   Used in: New App Token · App Access Request

   Exports to window:
     ModelPickerPanel  — slide-in side panel (search + filter + grouped list)
     ModelAccessPicker — radio (List / All / Specific) + panel trigger + selected list

   Access tiers (radio):
     'list'      — listing-only: app can enumerate the catalog, no invoke/connect
     'all'       — invoke/connect ANY resource (now + future)
     'specific'  — invoke/connect only the chosen resources
   Listing tier is opt-in via allowListing.

   Upgrade / token-exchange highlighting:
     grantedMode  — the tier the PREVIOUS (submitted) token already held
     grantedIds   — specific ids the previous token already held
   Previously-granted things get a green "granted" cue; anything the current
   selection adds beyond the grant gets an amber "new" cue. Fully editable so
   the approver can still downgrade.
═══════════════════════════════════════════════════ */

(function () {
  const { useState, useEffect, useRef } = React;

  const TIER_RANK = { list: 1, specific: 2, all: 3 };

  /* ── Internal checkmark SVG ── */
  function MAPCheck({ size = 9, color = '#12142B' }) {
    return (
      <svg viewBox="0 0 12 12" fill="none" stroke={color} strokeWidth="2.5"
           strokeLinecap="round" strokeLinejoin="round"
           style={{ width: size, height: size, display: 'block' }}>
        <polyline points="1.5,6 5,9.5 10.5,2.5" />
      </svg>
    );
  }

  /* ════════════════════════════════════════════════
     MODEL PICKER PANEL — slide-in side panel
  ════════════════════════════════════════════════ */
  function ModelPickerPanel({
    title = 'Select Models',
    subtitle = 'Choose which models to grant access',
    allModels,
    selectedIds,
    onToggle,
    onClose,
    suggestedIds = [],
    requiredCaps  = [],
    grantedIds = [],
    itemNoun = 'model',
  }) {
    const [search,     setSearch]     = useState('');
    const [typeFilter, setTypeFilter] = useState('all');

    const hasTypes = allModels.some(m => m.type);
    const getDisplayName = m => m.name || m.label || m.id;

    const filtered = allModels.filter(m => {
      if (hasTypes && typeFilter === 'local' && m.type !== 'local') return false;
      if (hasTypes && typeFilter === 'api'   && m.type !== 'api')   return false;
      const q = search.toLowerCase();
      if (q && !getDisplayName(m).toLowerCase().includes(q) && !m.id.toLowerCase().includes(q)) return false;
      return true;
    });

    const sorted = suggestedIds.length
      ? [...filtered].sort((a, b) => {
          const aS = suggestedIds.includes(a.id) ? 0 : 1;
          const bS = suggestedIds.includes(b.id) ? 0 : 1;
          return aS - bS;
        })
      : filtered;

    const locals  = sorted.filter(m => m.type === 'local');
    const apis    = sorted.filter(m => m.type === 'api');
    const untyped = sorted.filter(m => !m.type);

    const groups = [];
    if (untyped.length) groups.push({ label: 'Models',       models: untyped });
    if (locals.length)  groups.push({ label: 'Local Models', models: locals  });
    if (apis.length)    groups.push({ label: 'API Models',   models: apis    });

    useEffect(() => { if (window.lucide) window.lucide.createIcons(); });

    const handleBackdropClick = e => {
      if (e.target.classList.contains('panel-backdrop')) onClose();
    };

    return (
      <div className="panel-overlay" onClick={handleBackdropClick}>
        <div className="panel-backdrop"></div>
        <div className="panel-sheet">

          <div className="panel-head">
            <div>
              <div className="panel-title">{title}</div>
              <div className="panel-subtitle">{subtitle}</div>
            </div>
            <button className="panel-close" onClick={onClose}>
              <i data-lucide="x"></i>
            </button>
          </div>

          <div className="panel-filters">
            <input
              className="panel-search"
              placeholder={`Search ${itemNoun}s…`}
              value={search}
              onChange={e => setSearch(e.target.value)}
              autoFocus
            />
            {hasTypes && (
              <select className="panel-type-select" value={typeFilter} onChange={e => setTypeFilter(e.target.value)}>
                <option value="all">All</option>
                <option value="local">Local</option>
                <option value="api">API</option>
              </select>
            )}
          </div>

          <div className="panel-body">
            {groups.length === 0 && (
              <div style={{ padding: '28px 16px', textAlign: 'center', fontSize: 13, color: 'hsl(var(--muted-foreground))' }}>
                No {itemNoun}s match "{search}"
              </div>
            )}
            {groups.map(g => (
              <div key={g.label}>
                <div className="panel-group-label">{g.label}</div>
                {g.models.map(m => {
                  const isSelected  = selectedIds.includes(m.id);
                  const isSuggested = suggestedIds.includes(m.id);
                  const isGranted   = grantedIds.includes(m.id);
                  const missingCaps = (requiredCaps.length && m.caps)
                    ? requiredCaps.filter(c => !m.caps.includes(c))
                    : [];
                  return (
                    <div
                      key={m.id}
                      className={`panel-model-row${isSelected ? ' is-added' : ''}`}
                      onClick={() => onToggle(m.id)}
                    >
                      <div className="panel-row-check">
                        {isSelected && <MAPCheck size={9} color="#12142B" />}
                      </div>
                      <span className="panel-row-name">{getDisplayName(m)}</span>
                      <div className="panel-row-tags">
                        {m.meta && <span className="panel-row-meta">{m.meta}</span>}
                        {isGranted       && <span className="tag tag-leaf">✓ granted</span>}
                        {isSuggested     && <span className="tag tag-lotus">★ best</span>}
                        {missingCaps.length > 0 && (
                          <span className="tag tag-saffron">missing: {missingCaps.join(', ')}</span>
                        )}
                        {m.ctx  && <span className="tag tag-muted">{m.ctx}</span>}
                        {m.cost && <span className="tag tag-muted">{m.cost}</span>}
                        {m.type === 'local' && <span className="model-type-local">local</span>}
                        {m.type === 'api'   && <span className="model-type-api">api</span>}
                      </div>
                    </div>
                  );
                })}
              </div>
            ))}
          </div>

          <div className="panel-foot">
            <span className="panel-foot-count">
              {selectedIds.length} {itemNoun}{selectedIds.length !== 1 ? 's' : ''} selected
            </span>
            <button className="btn-sm btn-sm-indigo" onClick={onClose}>Done</button>
          </div>

        </div>
      </div>
    );
  }

  /* ── A single radio tier option ── */
  function TierOption({ mode, current, onPick, readOnly, label, desc, futureBadge, grantedMode, requestedMode }) {
    const selected = current === mode;
    const isGranted = grantedMode === mode;
    /* "new" = the selected tier sits ABOVE what was previously granted */
    const isNewLevel = grantedMode && selected && !isGranted
      && (TIER_RANK[current] || 0) > (TIER_RANK[grantedMode] || 0);
    return (
      <div
        className={`map-radio-option${selected ? ' selected' : ''}${readOnly ? ' is-readonly' : ''}${isGranted ? ' is-granted' : ''}`}
        onClick={() => !readOnly && onPick(mode)}
      >
        <div className="map-radio-dot">
          <div className="map-radio-dot-inner" style={{ transform: selected ? 'scale(1)' : 'scale(0)' }}></div>
        </div>
        <div className="map-radio-body">
          <span className="map-radio-text">
            {label}
            {futureBadge && <span className="map-future-badge">+ future</span>}
            {isGranted     && <span className="map-granted-pill">✓ previously granted</span>}
            {isNewLevel    && <span className="map-new-pill">new access</span>}
          </span>
          <span className="map-radio-desc">{desc}</span>
        </div>
      </div>
    );
  }

  /* ════════════════════════════════════════════════
     MODEL ACCESS PICKER — radio + summary + panel
  ════════════════════════════════════════════════ */
  function ModelAccessPicker({
    mode,
    onModeChange,
    allModels,
    selectedIds,
    onToggle,
    panelTitle,
    panelSubtitle,
    suggestedIds = [],
    requiredCaps  = [],
    showRanks     = false,
    onReorder,
    itemNoun = 'model',
    allLabel,
    allDesc,
    specificLabel,
    specificDesc,
    /* upgrade highlighting */
    grantedMode = null,
    grantedIds = [],
    /* read-only (view a token) */
    readOnly = false,
  }) {
    const [panelOpen, setPanelOpen] = useState(false);
    const dragItem = useRef(null);
    const dragOver = useRef(null);

    useEffect(() => { if (window.lucide) window.lucide.createIcons(); });

    const selectedModels = selectedIds
      .map(id => allModels.find(m => m.id === id))
      .filter(Boolean);

    const getDisplayName = m => m.name || m.label || m.id;

    const handleModeChange = newMode => {
      if (readOnly) return;
      onModeChange(newMode);
      if (newMode === 'specific' && mode !== 'specific') setPanelOpen(true);
    };

    const handleDragEnd = () => {
      if (!onReorder || dragItem.current === null || dragOver.current === null
          || dragItem.current === dragOver.current) return;
      const reordered = [...selectedIds];
      const [moved] = reordered.splice(dragItem.current, 1);
      reordered.splice(dragOver.current, 0, moved);
      onReorder(reordered);
      dragItem.current = null;
      dragOver.current = null;
    };

    const Up = { current: mode, grantedMode, readOnly, requestedMode: mode, onPick: handleModeChange };

    return (
      <>
        <div className={`map-radio-group${readOnly ? ' is-readonly' : ''}`}>
          <TierOption {...Up} mode="all"
            label={allLabel || 'All Models'} futureBadge
            desc={allDesc || 'Grant access to all current and future models.'}
          />
          <TierOption {...Up} mode="specific"
            label={specificLabel || 'Specific Models'}
            desc={specificDesc || 'Choose exactly which models are accessible.'}
          />
        </div>

        {mode === 'specific' && (
          <div className="map-specific-area">
            {selectedModels.length > 0 ? (
              <div className="map-selected-list">
                {selectedModels.map((m, idx) => {
                  const granted = grantedIds.includes(m.id);
                  const isNew = grantedMode && !granted;
                  return (
                  <div
                    key={m.id}
                    className={`map-selected-row${granted ? ' is-granted' : ''}${isNew ? ' is-new' : ''}`}
                    draggable={!!(showRanks && onReorder && !readOnly)}
                    onDragStart={() => { dragItem.current = idx; }}
                    onDragEnter={() => { dragOver.current = idx; }}
                    onDragEnd={handleDragEnd}
                    onDragOver={e => e.preventDefault()}
                  >
                    {showRanks && onReorder && !readOnly && (
                      <div className="map-drag-handle">
                        <i data-lucide="grip-vertical"></i>
                      </div>
                    )}
                    {showRanks && (
                      <div className={`map-rank-badge${idx === 0 ? ' r1' : ''}`}>{idx + 1}</div>
                    )}
                    <span className="map-selected-name">{getDisplayName(m)}</span>
                    {m.meta && <span className="map-selected-meta">{m.meta}</span>}
                    {granted && <span className="map-granted-pill">✓ granted</span>}
                    {isNew   && <span className="map-new-pill">new</span>}
                    {m.type === 'local' && <span className="model-type-local">local</span>}
                    {m.type === 'api'   && <span className="model-type-api">api</span>}
                    {!readOnly && (
                      <button
                        className="map-remove-btn"
                        onClick={e => { e.stopPropagation(); onToggle(m.id); }}
                        title="Remove"
                      >
                        <i data-lucide="x"></i>
                      </button>
                    )}
                  </div>
                  );
                })}
              </div>
            ) : (
              <div className="map-empty-hint">No {itemNoun}s selected — no access will be granted.</div>
            )}

            {!readOnly && (
              <button className="map-add-btn" onClick={() => setPanelOpen(true)}>
                <i data-lucide="plus"></i>
                {selectedModels.length > 0 ? `Add more ${itemNoun}s` : `Select ${itemNoun}s`}
              </button>
            )}
          </div>
        )}

        {panelOpen && (
          <ModelPickerPanel
            title={panelTitle}
            subtitle={panelSubtitle}
            allModels={allModels}
            selectedIds={selectedIds}
            onToggle={onToggle}
            onClose={() => setPanelOpen(false)}
            suggestedIds={suggestedIds}
            requiredCaps={requiredCaps}
            grantedIds={grantedIds}
            itemNoun={itemNoun}
          />
        )}
      </>
    );
  }

  /* ════════════════════════════════════════════════
     SINGLE MODEL COMBO — free-text autocomplete (ONE model)
     The other tier of model selection: instead of the All/Specific
     multi-picker + side panel, this grants exactly ONE model to a
     slot. Any value is accepted — pick an installed/API model from
     the dropdown OR type a plain-text name, because upstream
     providers may serve models we don't have in the catalog.
     Mirrors the model picker in Bodhi Chat.
     Props:
       value        string    — current model name / free text
       onChange     (str)=>void
       allModels    object[]  — suggestions (id, name|label, type, caps)
       suggestedIds string[]  — capability-matching ids floated up + ★ best
       requiredCaps string[]  — caps the slot wants (soft "missing" hint only)
       placeholder  string
       readOnly     bool
  ════════════════════════════════════════════════ */
  function SingleModelCombo({
    value,
    onChange,
    allModels = [],
    suggestedIds = [],
    requiredCaps = [],
    placeholder = 'Select a model or type a name…',
    readOnly = false,
  }) {
    const [open, setOpen] = useState(false);
    const ref = useRef(null);

    useEffect(() => {
      if (!open) return;
      const h = e => { if (ref.current && !ref.current.contains(e.target)) setOpen(false); };
      document.addEventListener('mousedown', h);
      return () => document.removeEventListener('mousedown', h);
    }, [open]);
    useEffect(() => { if (window.lucide) window.lucide.createIcons(); });

    // The popover is rendered position:fixed so it escapes the slot card's
    // overflow:hidden and the scroll container's clipping. Anchored to the
    // field, it flips above when there's no room below and follows scroll/resize.
    const [pos, setPos] = useState(null);
    const place = () => {
      if (!ref.current) return;
      const r = ref.current.getBoundingClientRect();
      const gap = 5, margin = 12, want = 272;
      const below = window.innerHeight - r.bottom;
      const above = r.top;
      const dropUp = below < Math.min(want, 200) && above > below;
      setPos({
        left: Math.round(r.left),
        width: Math.round(r.width),
        top:    dropUp ? 'auto' : Math.round(r.bottom + gap),
        bottom: dropUp ? Math.round(window.innerHeight - r.top + gap) : 'auto',
        maxHeight: Math.max(140, Math.round((dropUp ? above : below) - margin)),
      });
    };
    React.useLayoutEffect(() => {
      if (!open) { setPos(null); return; }
      place();
      const on = () => place();
      window.addEventListener('scroll', on, true);
      window.addEventListener('resize', on);
      return () => { window.removeEventListener('scroll', on, true); window.removeEventListener('resize', on); };
    }, [open]);

    const getName = m => m.name || m.label || m.id;
    const v = value || '';
    const q = v.toLowerCase().trim();
    const isMatch = m => getName(m).toLowerCase().includes(q) || m.id.toLowerCase().includes(q);
    const byName  = (a, b) => getName(a).localeCompare(getName(b));
    const rank    = m => (suggestedIds.includes(m.id) ? 0 : 1);
    const matching    = allModels.filter(isMatch).sort((a, b) => rank(a) - rank(b) || byName(a, b));
    const nonMatching = q ? allModels.filter(m => !isMatch(m)).sort(byName) : [];
    const exact      = allModels.some(m => getName(m) === v.trim());
    const showCustom = !!v.trim() && !exact;

    const pick = name => { onChange(name); setOpen(false); };

    const renderOpt = m => {
      const name = getName(m);
      const sel  = name === v;
      const suggested = suggestedIds.includes(m.id);
      const missing = (requiredCaps.length && m.caps) ? requiredCaps.filter(c => !m.caps.includes(c)) : [];
      return (
        <button key={m.id} type="button"
                className={`map-combo-opt${sel ? ' sel' : ''}`}
                onMouseDown={e => e.preventDefault()} onClick={() => pick(name)}>
          <span className="map-combo-opt-name">{name}</span>
          <span className="map-combo-opt-tags">
            {suggested         && <span className="tag tag-lotus">★ best</span>}
            {missing.length > 0 && <span className="tag tag-saffron">missing: {missing.join(', ')}</span>}
            {m.type === 'local' && <span className="model-type-local">local</span>}
            {m.type === 'api'   && <span className="model-type-api">api</span>}
          </span>
          {sel && <span className="map-combo-opt-check"><MAPCheck size={13} color="var(--c-lotus-text)" /></span>}
        </button>
      );
    };

    return (
      <div className={`map-combo${open ? ' open' : ''}${readOnly ? ' is-readonly' : ''}`} ref={ref}>
        <input
          className="map-combo-input"
          type="text"
          value={v}
          placeholder={placeholder}
          spellCheck={false}
          autoComplete="off"
          disabled={readOnly}
          onChange={e => { onChange(e.target.value); setOpen(true); }}
          onFocus={() => setOpen(true)}
          onKeyDown={e => {
            if (e.key === 'Escape') { setOpen(false); e.currentTarget.blur(); }
            if (e.key === 'Enter')  { e.preventDefault(); setOpen(false); }
          }}
        />
        <span className="map-combo-caret"><i data-lucide="chevron-down"></i></span>
        {open && !readOnly && pos && (
          <div className="map-combo-pop" style={{ position: 'fixed', left: pos.left, width: pos.width, right: 'auto', top: pos.top, bottom: pos.bottom, maxHeight: pos.maxHeight }}>
            {showCustom && (
              <button type="button" className="map-combo-opt is-custom"
                      onMouseDown={e => e.preventDefault()} onClick={() => setOpen(false)}>
                <i data-lucide="pencil-line"></i>
                <span className="map-combo-opt-name">Use “{v.trim()}”</span>
                <span className="map-combo-custom-hint">custom value</span>
              </button>
            )}
            {showCustom && (matching.length > 0 || nonMatching.length > 0) && <div className="map-combo-div" />}
            {matching.map(renderOpt)}
            {matching.length > 0 && nonMatching.length > 0 && <div className="map-combo-div" />}
            {nonMatching.map(renderOpt)}
            {!showCustom && matching.length === 0 && nonMatching.length === 0 && (
              <div className="map-combo-empty">Type a model name to use a custom value.</div>
            )}
          </div>
        )}
      </div>
    );
  }

  /* ════════════════════════════════════════════════
     LISTING TOGGLE — standalone permission (like S3 ListBucket)
     Fully INDEPENDENT of the inference/connect tier. "All" already
     exposes the whole catalog, but because listing is tracked as its
     own param we never auto-enable or lock it — it stays independent.
     Props:
       on          bool   — current listing grant
       onToggle    ()=>void
       readOnly    bool
       redundant   bool   — tier is "all" (listing already covered) → soft hint only
       label, desc string
       code        string — e.g. "/v1/models" endpoint hint
       granted     bool   — previous token already had it (green)
       isNew       bool   — newly requested (amber)
  ════════════════════════════════════════════════ */
  function ListingToggle({ on, onToggle, readOnly = false, redundant = false, label, desc, code, granted = false, isNew = false }) {
    return (
      <div
        className={`map-listing${on ? ' on' : ''}${readOnly ? ' is-locked' : ''}`}
        onClick={() => { if (!readOnly) onToggle(); }}
        role="checkbox" aria-checked={on}
      >
        <div className={`map-listing-check${on ? ' on' : ''}`}>
          {on && (
            <svg viewBox="0 0 12 12" fill="none" stroke="#fff" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round">
              <polyline points="1.5,6 5,9.5 10.5,2.5" />
            </svg>
          )}
        </div>
        <div className="map-listing-main">
          <div className="map-listing-title">
            {label}
            {code && <span className="map-listing-code">{code}</span>}
            {granted && <span className="map-granted-pill">✓ previously granted</span>}
            {isNew && !granted && <span className="map-new-pill">new access</span>}
          </div>
          <div className="map-listing-desc">
            {desc}
            {redundant && <span className="map-listing-implied"> · “All” already lists everything</span>}
          </div>
        </div>
      </div>
    );
  }

  Object.assign(window, { ModelPickerPanel, ModelAccessPicker, SingleModelCombo, ListingToggle });
})();
