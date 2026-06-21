/* ═══════════════════════════════════════════════════
   Create Model Router — page root
   models/create-fallback-model-app.jsx   (load LAST of the cfm modules)

   Owns the form state (identity, resilience knobs, ordered steps),
   validation, and the live rail (chain preview + how-it-works + tips),
   then assembles it all inside the shell.

   Module load order (set in Create Fallback Model.html):
     create-fallback-data · create-fallback-fields ·
     create-fallback-steps · create-fallback-model-app
═══════════════════════════════════════════════════ */
const {
  AVAILABLE_ALIASES, Icon, AliasCombobox, StepCard, StepConnector,
  ChainPreview, CfmRailHeader,
} = window.CFM;

function CreateFallbackModelApp() {
  const [aliasName, setAliasName] = React.useState('smart-fallback');
  const [strategy, setStrategy] = React.useState('fallback');
  const [cooldown, setCooldown] = React.useState('30');
  const [maxAttempts, setMaxAttempts] = React.useState('0');
  const [honorRetry, setHonorRetry] = React.useState(true);
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

  /* ── Right-rail content: live chain preview + help, surfaced as the
     shell's standard detail panel (open by default) so we don't invent
     a new in-page side column pattern. ── */
  const railContent = (
    <div className="cfm-rail-pad">
      {validationMsg && (
        <div className="cfm-validation-badge warn">
          <Icon name="alert-circle" size={13} />
          <div>{validationMsg}</div>
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
            <>Client sends a request with <code style={{fontFamily:'var(--font-mono)', fontSize:10, background:'hsl(var(--muted))', padding:'0 3px', borderRadius:2}}>model={aliasName || '<router>'}</code>.</>,
            'Step 1 is tried first. If it returns a successful response, we stop and return it.',
            'If Step 1 errors (timeout, 5xx, rate limit, model down…), Step 2 is tried — and so on.',
            honorRetry
              ? 'A failed target is put on cooldown; an upstream Retry-After is honored when present.'
              : 'A failed target is put on cooldown before it is retried again.',
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
            'Put fastest / cheapest targets first to minimize cost and latency.',
            'End with a local model alias for a self-hosted last resort.',
            'Cooldown keeps a failing target out of rotation so requests are not wasted retrying it.',
            'Set max attempts to cap how many targets a single request will try before giving up.',
            'Toggle a step off to skip it temporarily — the sequence and config are preserved.',
          ].map((tip, i) => (
            <div key={i} className="cfm-tip-row">
              <span className="cfm-tip-bullet">›</span>
              <span>{tip}</span>
            </div>
          ))}
        </div>
      </div>
    </div>
  );

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
        { label: 'New Model Router', current: true },
      ]}
      rail={railContent}
      railHeader={<CfmRailHeader />}
      railDefaultOpen={true}
      railWidth={320} railMin={280} railMax={460}
      contentClass="flush" mainScroll={false}
    >
        {/* ── Scroll · single form column (help lives in the right rail) ── */}
        <div className="bf-scroll">
          <div className="cfm-form-wrap">

              <div className="bf-card">
                <div className="bf-card-head">
                  <h1 className="bf-card-title">New Model Router</h1>
                  <p className="bf-card-sub">
                    Chain targets into a priority order and route requests through them.
                  </p>
                </div>
                <div className="bf-card-body">

                  {/* ═══ Section 1 · Identity ═══ */}
                  <div className="bf-section">
                    <div className="bf-section-title">Identity</div>
                    <div className="bf-field">
                      <label className="bf-label">
                        <span className="bf-label-text">Name</span>
                        <span className="bf-req">*</span>
                      </label>
                      <input
                        className="bf-input bf-input-mono"
                        value={aliasName}
                        onChange={e => setAliasName(e.target.value.replace(/\s/g, ''))}
                        placeholder="e.g. smart-fallback"
                      />
                      <div className="bf-hint">
                        Becomes the <code className="cfm-code">model</code> value clients send to your API. No spaces.
                      </div>
                    </div>
                    <div className="bf-field">
                      <label className="bf-label">
                        <span className="bf-label-text">Strategy</span>
                      </label>
                      <select className="bf-select" value="fallback" disabled aria-readonly="true">
                        <option value="fallback">Fallback</option>
                      </select>
                    </div>
                  </div>

                  <div className="bf-divider"></div>

                  {/* ═══ Section 2 · Resilience ═══ */}
                  <div className="bf-section">
                    <div className="bf-section-title">Resilience</div>
                    <div className="bf-field-row">
                      <div className="bf-field">
                        <label className="bf-label">
                          <span className="bf-label-text">Cooldown (seconds)</span>
                        </label>
                        <input
                          type="number" min="0"
                          className="bf-input"
                          value={cooldown}
                          onChange={e => setCooldown(e.target.value)}
                          placeholder="30"
                        />
                        <div className="bf-hint">How long a failed target is skipped before it is retried.</div>
                      </div>
                      <div className="bf-field">
                        <label className="bf-label">
                          <span className="bf-label-text">Max attempts per request</span>
                        </label>
                        <input
                          type="number" min="0"
                          className="bf-input"
                          value={maxAttempts}
                          onChange={e => setMaxAttempts(e.target.value)}
                          placeholder="0"
                        />
                        <div className="bf-hint">0 = try all enabled targets.</div>
                      </div>
                    </div>
                    <div className="bf-toggle-row" onClick={() => setHonorRetry(v => !v)} style={{ cursor:'pointer' }}>
                      <div className="bf-toggle-body">
                        <div className="bf-toggle-label">Honor upstream Retry-After</div>
                        <div className="bf-toggle-desc">
                          When a target returns a <code className="cfm-code">Retry-After</code> header, use it instead of the fixed cooldown above.
                        </div>
                      </div>
                      <div className={'bf-switch' + (honorRetry ? ' on' : '')} role="switch" aria-checked={honorRetry} />
                    </div>
                  </div>

                  <div className="bf-divider"></div>

                  {/* ═══ Section 3 · Targets ═══ */}
                  <div className="bf-section">
                    <div className="bf-section-title">Targets (in priority order)</div>

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
                  <div className="bf-footer-spacer" />
                  <button className="bf-btn bf-btn-ghost">Cancel</button>
                  <button className="bf-btn bf-btn-primary" disabled={!isValid}>
                    <Icon name="route" size={13} />
                    Create Model Router
                  </button>
                </div>
              </div>{/* end card */}
          </div>{/* end form-wrap */}
        </div>{/* end bf-scroll */}
    </AppShell>
    </>
  );
}

const cfmRoot = ReactDOM.createRoot(document.getElementById('root'));
cfmRoot.render(<CreateFallbackModelApp />);
