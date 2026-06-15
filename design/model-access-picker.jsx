/* ═══════════════════════════════════════════════════
   MODEL ACCESS PICKER — Shared React component
   Used in: New App Token · Bodhi Access Request

   Exports to window:
     ModelPickerPanel  — slide-in side panel (search + filter + grouped list)
     ModelAccessPicker — radio (All / Specific) + panel trigger + selected list
═══════════════════════════════════════════════════ */

(function () {
  const { useState, useEffect, useRef } = React;

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
     Props:
       title        string
       subtitle     string
       allModels    [{ id, name?, label?, type?, ctx?, cost?, caps? }]
       selectedIds  string[]
       onToggle     (id: string) => void
       onClose      () => void
       suggestedIds string[]   — shown first, marked ★ best
       requiredCaps string[]   — show "missing:" warning if model lacks them
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

          {/* Header */}
          <div className="panel-head">
            <div>
              <div className="panel-title">{title}</div>
              <div className="panel-subtitle">{subtitle}</div>
            </div>
            <button className="panel-close" onClick={onClose}>
              <i data-lucide="x"></i>
            </button>
          </div>

          {/* Filters */}
          <div className="panel-filters">
            <input
              className="panel-search"
              placeholder="Search models…"
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

          {/* List */}
          <div className="panel-body">
            {groups.length === 0 && (
              <div style={{ padding: '28px 16px', textAlign: 'center', fontSize: 13, color: 'hsl(var(--muted-foreground))' }}>
                No models match "{search}"
              </div>
            )}
            {groups.map(g => (
              <div key={g.label}>
                <div className="panel-group-label">{g.label}</div>
                {g.models.map(m => {
                  const isSelected  = selectedIds.includes(m.id);
                  const isSuggested = suggestedIds.includes(m.id);
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

          {/* Footer */}
          <div className="panel-foot">
            <span className="panel-foot-count">
              {selectedIds.length} model{selectedIds.length !== 1 ? 's' : ''} selected
            </span>
            <button className="btn-sm btn-sm-indigo" onClick={onClose}>Done</button>
          </div>

        </div>
      </div>
    );
  }

  /* ════════════════════════════════════════════════
     MODEL ACCESS PICKER — radio + summary + panel
     Props:
       mode           'all' | 'specific'
       onModeChange   (mode) => void
       allModels      same shape as ModelPickerPanel
       selectedIds    string[]  — ordered list of selected model IDs
       onToggle       (id: string) => void
       panelTitle     string (optional)
       panelSubtitle  string (optional)
       suggestedIds   string[] (optional)
       requiredCaps   string[] (optional)
       showRanks      bool    — show rank badge + drag handle (for slot priority)
       onReorder      (newIdOrder: string[]) => void (optional, enables drag)
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
  }) {
    const [panelOpen, setPanelOpen] = useState(false);
    const dragItem = useRef(null);
    const dragOver = useRef(null);

    useEffect(() => { if (window.lucide) window.lucide.createIcons(); });

    /* Derive ordered selected models (preserve selectedIds order, not allModels order) */
    const selectedModels = selectedIds
      .map(id => allModels.find(m => m.id === id))
      .filter(Boolean);

    const getDisplayName = m => m.name || m.label || m.id;

    /* Mode change: switching to specific opens panel immediately */
    const handleModeChange = newMode => {
      onModeChange(newMode);
      if (newMode === 'specific' && mode !== 'specific') {
        setPanelOpen(true);
      }
    };

    /* Drag-to-reorder (only when showRanks + onReorder provided) */
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

    return (
      <>
        {/* ── Radio: All Models ── */}
        <div className="map-radio-group">
          <div
            className={`map-radio-option${mode === 'all' ? ' selected' : ''}`}
            onClick={() => handleModeChange('all')}
          >
            <div className="map-radio-dot">
              <div className="map-radio-dot-inner"
                   style={{ transform: mode === 'all' ? 'scale(1)' : 'scale(0)' }}>
              </div>
            </div>
            <div className="map-radio-body">
              <span className="map-radio-text">
                All Models
                <span className="map-future-badge">+ future</span>
              </span>
              <span className="map-radio-desc">
                Grant access to all current and future models.
              </span>
            </div>
          </div>

          {/* ── Radio: Specific Models ── */}
          <div
            className={`map-radio-option${mode === 'specific' ? ' selected' : ''}`}
            onClick={() => handleModeChange('specific')}
          >
            <div className="map-radio-dot">
              <div className="map-radio-dot-inner"
                   style={{ transform: mode === 'specific' ? 'scale(1)' : 'scale(0)' }}>
              </div>
            </div>
            <div className="map-radio-body">
              <span className="map-radio-text">Specific Models</span>
              <span className="map-radio-desc">
                Choose exactly which models are accessible.
              </span>
            </div>
          </div>
        </div>

        {/* ── Specific models area ── */}
        {mode === 'specific' && (
          <div className="map-specific-area">
            {selectedModels.length > 0 ? (
              <div className="map-selected-list">
                {selectedModels.map((m, idx) => (
                  <div
                    key={m.id}
                    className="map-selected-row"
                    draggable={!!(showRanks && onReorder)}
                    onDragStart={() => { dragItem.current = idx; }}
                    onDragEnter={() => { dragOver.current = idx; }}
                    onDragEnd={handleDragEnd}
                    onDragOver={e => e.preventDefault()}
                  >
                    {showRanks && onReorder && (
                      <div className="map-drag-handle">
                        <i data-lucide="grip-vertical"></i>
                      </div>
                    )}
                    {showRanks && (
                      <div className={`map-rank-badge${idx === 0 ? ' r1' : ''}`}>{idx + 1}</div>
                    )}
                    <span className="map-selected-name">{getDisplayName(m)}</span>
                    {m.type === 'local' && <span className="model-type-local">local</span>}
                    {m.type === 'api'   && <span className="model-type-api">api</span>}
                    <button
                      className="map-remove-btn"
                      onClick={e => { e.stopPropagation(); onToggle(m.id); }}
                      title="Remove"
                    >
                      <i data-lucide="x"></i>
                    </button>
                  </div>
                ))}
              </div>
            ) : (
              <div className="map-empty-hint">No models selected — no access will be granted.</div>
            )}

            <button className="map-add-btn" onClick={() => setPanelOpen(true)}>
              <i data-lucide="plus"></i>
              {selectedModels.length > 0 ? 'Add more models' : 'Select models'}
            </button>
          </div>
        )}

        {/* ── Side panel ── */}
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
          />
        )}
      </>
    );
  }

  /* Export both to window so other Babel scripts can use them */
  Object.assign(window, { ModelPickerPanel, ModelAccessPicker });
})();
