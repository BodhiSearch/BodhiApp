// Create API Model · v29
// Flat one-form layout with production-parity fields. See specs/api.md.
// Four variants: Standalone / Overlay / Medium / Mobile (parity with alias.jsx).
// Model Selection section is CONDITIONAL on forwarding mode — shown only when
// "Forward for selected models only" is active.

// Shared form body used by every variant. Takes a mode ('desktop' | 'mobile')
// for small layout nudges (padding/font), but the field set is identical.
function ApiFormBody({forwarding='selected', setForwarding,
                     selected, setSelected, search, setSearch,
                     compact=false}) {
  const selectedList = selected;
  const onDeselect = (m) => setSelected(selectedList.filter(x => x !== m));
  const onSelect   = (m) => !selectedList.includes(m) && setSelected([...selectedList, m]);
  const onClear    = () => setSelected([]);
  const onSelectAll = () => {
    const add = FIXTURE_OPENAI_MODELS
      .filter(m => !search || m.toLowerCase().includes(search.toLowerCase()))
      .filter(m => !selectedList.includes(m));
    setSelected([...selectedList, ...add]);
  };
  return (
    <>
      {/* ── Provider connection ── */}
      <div className="api-form-section-head">1 · Provider connection</div>
      <ApiFormatPicker value="openai-completions"/>
      <div style={{marginTop:6}}>
        <Field
          label={<span>Base URL <Chip tone="warn" style={{fontSize:9, marginLeft:4}}>required</Chip></span>}
          filled
          value="https://api.openai.com/v1"
        />
        <span className="sm" style={{fontSize:10, color:'var(--ink-3)'}}>Enter the complete API endpoint URL for your provider</span>
      </div>
      <div style={{marginTop:6}}>
        <ApiKeyField enabled={true} masked={true}/>
      </div>

      <hr className="api-form-divider"/>

      {/* ── Request routing ── */}
      <div className="api-form-section-head">2 · Request routing</div>
      <PrefixField enabled={true} value="openai/" example="openai/gpt-4"/>
      <div style={{marginTop:8}}>
        <div className="sm" style={{fontWeight:700, color:'var(--ink)', marginBottom:4}}>Request forwarding mode</div>
        <ForwardingModeRadio value={forwarding} onChange={setForwarding}/>
      </div>

      {/* ── Model selection (conditional) ── */}
      {forwarding === 'selected' && (
        <>
          <hr className="api-form-divider"/>
          <div className="api-form-section-head">3 · Model selection</div>
          <div className="sm" style={{fontSize:11, color:'var(--ink-3)', marginBottom:4}}>
            Select which OpenAI models you'd like to use. Only the selected set will be forwarded through the alias prefix.
          </div>
          <ModelMultiSelect
            selected={selectedList}
            available={FIXTURE_OPENAI_MODELS}
            search={search}
            onSearch={setSearch}
            onSelect={onSelect}
            onDeselect={onDeselect}
            onFetch={() => { /* wireframe: no-op */ }}
            onSelectAll={onSelectAll}
            onClear={onClear}
          />
        </>
      )}
    </>
  );
}

function ApiFormFooter({onCancel, primaryLabel='Create API Model'}) {
  return (
    <div className="api-form-footer">
      <Btn size="xs">🔌 Test connection</Btn>
      <div className="api-form-footer-actions">
        <Btn onClick={onCancel}>Cancel</Btn>
        <Btn variant="primary">{primaryLabel}</Btn>
      </div>
    </div>
  );
}

// ── 1. ApiStandalone ─────────────────────────────────────────────
function ApiStandalone() {
  const [forwarding, setForwarding] = React.useState('selected');
  const [selected, setSelected] = React.useState(['gpt-4-turbo','gpt-5-mini','gpt-5.3-codex']);
  const [search, setSearch] = React.useState('codex');
  return (
    <Browser url="bodhi.local/models/api/new">
      <Crumbs items={['Bodhi','Models','New API model']}/>
      <div style={{display:'flex', alignItems:'baseline', justifyContent:'space-between', gap:10, marginBottom:6}}>
        <div>
          <div className="h1" style={{fontSize:20, marginBottom:2}}>Create New API Model</div>
          <div className="sm">Configure a new external AI API model. Connect a provider, choose routing, pick which models forward through this alias.</div>
        </div>
      </div>
      <div style={{display:'grid', gridTemplateColumns:'170px 1fr', gap:12, alignItems:'start'}}>
        <ApiRail active={forwarding==='selected' ? 'models' : 'routing'}/>
        <div>
          <ApiFormBody
            forwarding={forwarding} setForwarding={setForwarding}
            selected={selected} setSelected={setSelected}
            search={search} setSearch={setSearch}
          />
          <ApiFormFooter/>
          <Callout style={{position:'static', display:'inline-block', marginTop:8}}>
            ★ Model selection only appears when "Forward for selected" is active · toggle radios to compare
          </Callout>
        </div>
      </div>
    </Browser>
  );
}

// ── 2. ApiOverlay ────────────────────────────────────────────────
function ApiOverlay() {
  const [forwarding, setForwarding] = React.useState('selected');
  const [selected, setSelected] = React.useState(['gpt-4-turbo','gpt-5-mini','gpt-5.3-codex']);
  const [search, setSearch] = React.useState('codex');
  const context = (
    <>
      <span className="sm" style={{color:'var(--ink)'}}>Adding</span>
      <Chip tone="indigo" style={{fontSize:10}}>api provider</Chip>
      <code>openai</code>
      <span className="sm" style={{marginLeft:'auto'}}>from + ▾ Add model</span>
    </>
  );
  const body = (
    <ApiFormBody
      forwarding={forwarding} setForwarding={setForwarding}
      selected={selected} setSelected={setSelected}
      search={search} setSearch={setSearch}
    />
  );
  const footer = (
    <>
      <Btn variant="ghost" size="xs">🔌 Test connection</Btn>
      <Btn variant="ghost" size="xs">Open full page ↗</Btn>
      <Btn>Cancel</Btn>
      <Btn variant="primary">Create API Model</Btn>
    </>
  );
  return <OverlayShell title="Connect API provider" context={context} body={body} footer={footer}/>;
}

// ── 3. ApiMedium (tablet) ────────────────────────────────────────
function ApiMedium() {
  const [forwarding, setForwarding] = React.useState('all'); // demo short form
  const [selected, setSelected] = React.useState([]);
  const [search, setSearch] = React.useState('');
  return (
    <TabletFrame label="Tablet · Forward all · short form">
      <MobileHeader active="Create API model"/>
      <ApiMediumAnchors active={forwarding==='selected' ? 'models' : 'routing'}/>
      <div style={{padding:'8px 10px'}}>
        <div className="h1" style={{fontSize:16, marginBottom:2}}>Create New API Model</div>
        <div className="sm" style={{fontSize:11, marginBottom:6}}>Demo: "Forward all with prefix" — Model selection section hidden.</div>
        <ApiFormBody
          forwarding={forwarding} setForwarding={setForwarding}
          selected={selected} setSelected={setSelected}
          search={search} setSearch={setSearch}
        />
        <ApiFormFooter/>
        <Callout style={{position:'static', fontSize:9, margin:'6px 0'}}>
          Tap "Forward for selected models only" above to reveal the model picker · no scroll-jump
        </Callout>
      </div>
    </TabletFrame>
  );
}

// ── 4. ApiMobile (phone) ─────────────────────────────────────────
function ApiMobile() {
  return (
    <div className="phone-deck">
      {/* 1. Default — Forward for selected (main demo) */}
      <PhoneFrame label="1 · Forward for selected">
        <ApiMobileBody initialForwarding="selected"
          initialSelected={['gpt-4-turbo','gpt-5-mini','gpt-5.3-codex']}
          initialSearch="codex"/>
      </PhoneFrame>
      {/* 2. Forward all — short form */}
      <PhoneFrame label="2 · Forward all (short form)">
        <ApiMobileBody initialForwarding="all" initialSelected={[]} initialSearch=""/>
      </PhoneFrame>
    </div>
  );
}

function ApiMobileBody({initialForwarding, initialSelected, initialSearch}) {
  const [forwarding, setForwarding] = React.useState(initialForwarding);
  const [selected, setSelected] = React.useState(initialSelected);
  const [search, setSearch] = React.useState(initialSearch);
  return (
    <>
      <MobileHeader active="Create API model"/>
      <div style={{padding:'6px 8px'}}>
        <div className="h1" style={{fontSize:14, marginBottom:2}}>Create API Model</div>
        <ApiFormBody
          forwarding={forwarding} setForwarding={setForwarding}
          selected={selected} setSelected={setSelected}
          search={search} setSearch={setSearch}
          compact
        />
        <ApiFormFooter/>
      </div>
    </>
  );
}

window.ApiScreens = [
  {label:'A · Standalone · full page', tag:'balanced',
    note:'Flat one-form layout · fields match production (Use API key toggle, Enable prefix toggle, Forwarding radio, full Model Selection). Standalone route `/models/api/new` with ApiRail sticky nav. Demo: Forward for selected with 3 preselected models + codex search filter.',
    novel:'conditional Model selection · toggle-aware primitives · production-parity fields',
    component:ApiStandalone},
  {label:'A · Overlay', tag:'balanced',
    note:'Reached from Models · `+ ▾ Add model → Add API provider`. OverlayShell chrome with context banner and footer actions. Same form body as Standalone.',
    novel:'in-context add from Models page',
    component:ApiOverlay},
  {label:'A · Medium · tablet', tag:'medium',
    note:'Tablet width with top anchor strip. Demos Forward all (short form) — Model selection section hidden. Flipping the radio reveals it.',
    novel:'anchor-strip nav · conditional section demo',
    component:ApiMedium},
  {label:'A · Mobile', tag:'mobile',
    note:'Two frames: (1) Forward for selected with search+chips, (2) Forward all short form. Same field order as Standalone, stacked.',
    novel:'conditional model selection preserved at phone width',
    component:ApiMobile},
];
