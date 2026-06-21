/* ═══════════════════════════════════════════════════
   Create Model Router — step cards + chain preview
   models/create-fallback-steps.jsx   (load after create-fallback-fields.jsx)

   The editable target rows (StepCard), the "on error" connector between
   them, the read-only chain preview shown in the rail (ChainPreview),
   and the rail header. Adds StepCard, StepConnector, ChainPreview,
   CfmRailHeader to window.CFM.
═══════════════════════════════════════════════════ */
const CFM = window.CFM;
const { AVAILABLE_ALIASES, Icon, TypeBadge, ProviderBadge, AliasCombobox, ModelField } = CFM;

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

/* ── Rail header (railHeader slot) — with a close affordance like
   every other right detail panel in the app ── */
function CfmRailHeader() {
  const { collapseRail } = useShell();
  return (
    <div className="cfm-rail-head">
      <Icon name="info" size={13} />
      <span className="cfm-rail-head-title">Routing &amp; help</span>
      <button className="cfm-rail-close" title="Close panel"
              onClick={() => collapseRail && collapseRail()}>
        <Icon name="x" size={14} />
      </button>
    </div>
  );
}

Object.assign(CFM, { StepCard, StepConnector, ChainPreview, CfmRailHeader });
