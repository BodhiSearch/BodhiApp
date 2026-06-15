// Access Request Review · v31
// 3rd-party app review page with per-model allow-list, per-MCP grants, and
// inline prerequisite resolution (admin one-click Add server; user Request
// admin; OAuth in popup with focus-refetch hint). 3 responsive variants.

function AccessRequestBody({viewerRole='user', onRoleChange, compact=false, stage='all', initialInlineOpen=null}) {
  const req = ACCESS_REQUEST_FIXTURE;
  const [selectedModels, setSelectedModels] = React.useState(
    ACCESS_MODELS_FIXTURE
      .filter(m => {
        const s = accessModelState(m, req.caps, req.suggestedModels);
        return s === 'matches-envelope' || s === 'app-suggested';
      })
      .map(m => m.name)
  );
  const [mcpSelections, setMcpSelections] = React.useState(req.mcps.reduce((acc, m) => (acc[m.slug] = true, acc), {}));
  const [mcpStates, setMcpStates] = React.useState(req.mcps.reduce((acc, m) => (acc[m.slug] = m.rowState, acc), {}));
  const [inlineOpen, setInlineOpen] = React.useState(initialInlineOpen);

  const toggleModel = (name) => {
    setSelectedModels(s => s.includes(name) ? s.filter(x => x !== name) : [...s, name]);
  };

  const aliases   = ACCESS_MODELS_FIXTURE.filter(m => m.kind === 'alias');
  const apis      = ACCESS_MODELS_FIXTURE.filter(m => m.kind === 'api');
  const providers = ACCESS_MODELS_FIXTURE.filter(m => m.kind === 'provider');

  const mcpEntries = req.mcps.map(m => {
    const registryServer = MCP_SERVERS_FIXTURE.find(s => s.slug === m.slug) || {slug: m.slug, url: MCP_CATALOG_FIXTURE.find(e => e.slug === m.slug)?.defaultBaseUrl};
    const instance = MCP_INSTANCES_FIXTURE.find(i => i.slug === m.slug);
    return {...m, server: registryServer, instance};
  });

  const checkedMcps   = mcpEntries.filter(m => mcpSelections[m.slug]);
  const blockerCount  = checkedMcps.filter(m => {
    const st = mcpStates[m.slug];
    return st !== 'has-instance';
  }).length;

  const showSection = (name) => stage === 'all' || stage === name;

  return (
    <>
      {showSection('header') && (
        <AccessRequestHeader request={req} viewerRole={viewerRole} onRoleChange={onRoleChange}/>
      )}

      {showSection('header') && (
        <div className="access-section">
          <div className="access-section-head">
            1 · Required capabilities <span className="sm" style={{fontSize:10, color:'var(--ink-3)', fontWeight:400}}>declared by the app</span>
          </div>
          <AccessCapsEnvelope caps={req.caps}/>
          <div className="access-section-hint">We'll pre-check models that match. You can still grant others — the app gets what you allow.</div>
        </div>
      )}

      {showSection('models') && (
        <div className="access-section">
          <div className="access-section-head">
            2 · Model access <span className="sm" style={{fontSize:10, color:'var(--ink-3)', fontWeight:400}}>{selectedModels.length} of {ACCESS_MODELS_FIXTURE.length} selected</span>
          </div>
          <AccessModelGroup title="Aliases (local)"       models={aliases}   caps={req.caps} suggested={req.suggestedModels} selected={selectedModels} onToggle={toggleModel}/>
          <AccessModelGroup title="API models"            models={apis}      caps={req.caps} suggested={req.suggestedModels} selected={selectedModels} onToggle={toggleModel}/>
          <AccessModelGroup title="Provider models"       models={providers} caps={req.caps} suggested={req.suggestedModels} selected={selectedModels} onToggle={toggleModel}/>
          <div className="access-section-hint">★ {req.suggestedModels.length} suggested by the app · {ACCESS_MODELS_FIXTURE.filter(m => accessModelState(m, req.caps, req.suggestedModels) === 'below-envelope').length} below envelope (disabled)</div>
        </div>
      )}

      {showSection('mcps') && (
        <div className="access-section">
          <div className="access-section-head">
            3 · MCP access <span className="sm" style={{fontSize:10, color:'var(--ink-3)', fontWeight:400}}>{checkedMcps.length} of {mcpEntries.length} selected</span>
          </div>
          {mcpEntries.map(m => (
            <AccessMcpRow
              key={m.slug}
              server={m.server}
              instance={m.instance}
              rowState={mcpStates[m.slug]}
              role={viewerRole}
              checked={!!mcpSelections[m.slug]}
              inlineOpen={inlineOpen === m.slug}
              onToggleInline={() => setInlineOpen(inlineOpen === m.slug ? null : m.slug)}
            />
          ))}
          <div className="access-section-hint">
            OAuth opens in a popup window · the page refreshes on focus (TanStack Query) and new instances surface here automatically
          </div>
        </div>
      )}

      {showSection('mcps') && (
        <div className="access-section">
          <div className="access-section-head">4 · Approved role</div>
          <AccessRoleSelect value="User"/>
        </div>
      )}

      {showSection('mcps') && !compact && (
        <AccessActionBar
          checkedModels={selectedModels.length}
          totalModels={ACCESS_MODELS_FIXTURE.length}
          checkedMcps={checkedMcps.length}
          totalMcps={mcpEntries.length}
          blockers={blockerCount}
        />
      )}
    </>
  );
}

// ── 1. Desktop ────────────────────────────────────────────────
function AccessRequestDesktop() {
  const [viewerRole, setViewerRole] = React.useState('admin');
  return (
    <Browser url="bodhi.local/apps/access-requests/review?id=01kpmrw…">
      <Crumbs items={['Bodhi','Apps','Access request']}/>
      <div style={{display:'flex', alignItems:'baseline', justifyContent:'space-between', gap:10, marginBottom:6}}>
        <div>
          <div className="h1" style={{fontSize:20, marginBottom:2}}>Review Access Request</div>
          <div className="sm">Decide which of your resources this 3rd-party app can use · resolve missing MCP prerequisites inline</div>
        </div>
      </div>
      <div className="access-card">
        <AccessRequestBody viewerRole={viewerRole} onRoleChange={setViewerRole} initialInlineOpen="gmail"/>
      </div>
      <Callout style={{position:'static', display:'inline-block', marginTop:8}}>
        ★ Toggle User/Admin (top-right of the card) to flip CTA · needs-server row shows the inline mini-overlay when admin clicks One-click Add
      </Callout>
    </Browser>
  );
}

// ── 2. Medium · tablet ──────────────────────────────────────
function AccessRequestMedium() {
  const [viewerRole, setViewerRole] = React.useState('user');
  return (
    <TabletFrame label="Tablet · User view (non-admin)">
      <MobileHeader active="Access Request"/>
      <div style={{padding:'8px 10px'}}>
        <div className="h2" style={{margin:'0 0 6px', fontSize:15}}>Review Access Request</div>
        <div className="access-card fill compact">
          <AccessRequestBody viewerRole={viewerRole} onRoleChange={setViewerRole} compact/>
          <div style={{marginTop:10, display:'flex', justifyContent:'space-between'}}>
            <Btn size="xs">Deny</Btn>
            <Btn variant="primary" size="xs">Approve 7 of 12 resources</Btn>
          </div>
        </div>
        <Callout style={{position:'static', display:'inline-block', marginTop:6, fontSize:9}}>
          User view: needs-server row shows `Request admin to add` instead of One-click Add
        </Callout>
      </div>
    </TabletFrame>
  );
}

// ── 3. Mobile · phone (3-stage wizard) ──────────────────────
function AccessRequestMobile() {
  return (
    <div className="phone-deck">
      <PhoneFrame label="1 · App + capabilities">
        <MobileHeader active="Access Request"/>
        <div style={{padding:'6px 8px'}}>
          <div className="access-card fill compact" style={{boxShadow:'none'}}>
            <AccessRequestBody viewerRole="admin" compact stage="header"/>
            <div style={{display:'flex', justifyContent:'space-between', marginTop:8}}>
              <Btn size="xs">Deny</Btn>
              <Btn variant="primary" size="xs">Continue →</Btn>
            </div>
          </div>
        </div>
      </PhoneFrame>
      <PhoneFrame label="2 · Choose models">
        <MobileHeader active="2 / 3 · Models"/>
        <div style={{padding:'6px 8px'}}>
          <div className="access-card fill compact" style={{boxShadow:'none'}}>
            <AccessRequestBody viewerRole="admin" compact stage="models"/>
            <div style={{display:'flex', justifyContent:'space-between', marginTop:8}}>
              <Btn size="xs">← Back</Btn>
              <Btn variant="primary" size="xs">Continue →</Btn>
            </div>
          </div>
        </div>
      </PhoneFrame>
      <PhoneFrame label="3 · MCPs + Approve">
        <MobileHeader active="3 / 3 · MCPs"/>
        <div style={{padding:'6px 8px'}}>
          <div className="access-card fill compact" style={{boxShadow:'none'}}>
            <AccessRequestBody viewerRole="admin" compact stage="mcps"/>
            <div style={{display:'flex', justifyContent:'space-between', marginTop:8}}>
              <Btn size="xs">Deny</Btn>
              <Btn variant="primary" size="xs">Approve</Btn>
            </div>
          </div>
        </div>
      </PhoneFrame>
    </div>
  );
}

window.AccessRequestScreens = [
  {label:'A · Desktop · Admin view', tag:'balanced',
    note:'Full access-request page with 4 sections: header (with role toggle) · capability envelope · 8 models across 3 groups (pre-checked by envelope match) · 4 MCP rows covering has-instance / needs-reauth / needs-instance / needs-server. Admin sees One-click Add on needs-server; inline mini-overlay expands on click.',
    novel:'inline-resolve prerequisites · role-adaptive CTAs · envelope-aware model pre-check',
    component:AccessRequestDesktop},
  {label:'A · Medium · tablet · User view', tag:'medium',
    note:'User (non-admin) view · needs-server row shows `Request admin to add` instead of One-click Add. Everything else unchanged.',
    novel:'role difference visible at tablet width',
    component:AccessRequestMedium},
  {label:'A · Mobile', tag:'mobile',
    note:'Three frames: (1) app + capabilities, (2) model selection, (3) MCP grants + approve. Deny button persistent across all 3 stages.',
    novel:'progressive disclosure for small screens',
    component:AccessRequestMobile},
];
