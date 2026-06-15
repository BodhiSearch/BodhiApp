// My MCPs · v30
// User's MCP instances list. Preserves production-parity table (download (9).png):
// Name · URL · Status · action icons. Adds pending-approvals banner above and
// surfaces needs-reauth chips on affected rows. 3 variants: Desktop / Medium / Mobile.
// No overlay variant here — overlays only live on Discover.

function MyMcpsBody({compact=false}) {
  const pending = MCP_APPROVAL_FIXTURE.filter(a => a.requester === 'arjun@bodhi.ai');
  return (
    <>
      {pending.length > 0 && <McpInstancePendingBanner pending={pending}/>}
      <div style={{display:'flex', justifyContent:'space-between', alignItems:'baseline', marginBottom:8}}>
        <div className={compact ? 'h2' : 'h1'} style={{margin:0}}>My MCPs</div>
        <Btn variant="primary" size={compact ? 'xs' : ''}>+ New MCP</Btn>
      </div>
      <div className="sm" style={{color:'var(--ink-3)', marginBottom:6}}>Instances you have connected · click ▷ to open in Playground · ✎ to edit · 🗑 to remove.</div>
      <div style={{display:'grid', gridTemplateColumns:'24px 1fr 2fr 90px 110px', gap:10,
                   fontSize:10, textTransform:'uppercase', color:'var(--ink-3)',
                   padding:'4px 10px', letterSpacing:0.5}}>
        <span></span><span>Name</span><span>URL</span><span>Status</span><span style={{textAlign:'right'}}>Actions</span>
      </div>
      {MCP_INSTANCES_FIXTURE.map(inst => <McpInstanceListRow key={inst.slug} instance={inst}/>)}
    </>
  );
}

// ── 1. Desktop ────────────────────────────────────────────────
function MyMcpsDesktop() {
  return (
    <Browser url="bodhi.local/mcps">
      <Crumbs items={['Bodhi','MCP','My MCPs']}/>
      <MyMcpsBody/>
      <Callout style={{position:'static', display:'inline-block', marginTop:8}}>
        ★ 4 instances including one in `needs_reauth` state · pending-approval banner reflects user's submitted requests · go to Discover to add more
      </Callout>
    </Browser>
  );
}

// ── 2. Medium · tablet ────────────────────────────────────────
function MyMcpsMedium() {
  return (
    <TabletFrame label="Tablet · condensed table">
      <MobileHeader active="My MCPs"/>
      <div style={{padding:'8px 10px'}}>
        <MyMcpsBody compact/>
      </div>
    </TabletFrame>
  );
}

// ── 3. Mobile · phone ─────────────────────────────────────────
function MyMcpsMobile() {
  return (
    <div className="phone-deck">
      <PhoneFrame label="1 · Stacked cards">
        <MobileHeader active="My MCPs"/>
        <div style={{padding:'6px 8px'}}>
          <McpInstancePendingBanner pending={[MCP_APPROVAL_FIXTURE[0]]}/>
          <div className="h1" style={{fontSize:14, marginBottom:4}}>My MCPs</div>
          {MCP_INSTANCES_FIXTURE.slice(0,3).map(inst => (
            <div key={inst.slug} style={{padding:8, background:'var(--paper-2)',
                 border:'1.3px solid var(--ink)', borderRadius:8, marginBottom:6,
                 fontFamily:'var(--hand)', fontSize:12}}>
              <div style={{display:'flex', justifyContent:'space-between', alignItems:'center'}}>
                <strong>{inst.name}</strong>
                {inst.authState === 'needs_reauth'
                  ? <Chip tone="warn" style={{fontSize:9}}>⚠ reauth</Chip>
                  : <Chip tone="leaf" style={{fontSize:9}}>● active</Chip>}
              </div>
              <div className="sm" style={{fontSize:10, color:'var(--ink-3)'}}>{inst.url}</div>
              <div style={{display:'flex', gap:4, marginTop:4}}>
                <Btn size="xs">▷ Play</Btn>
                <Btn size="xs">✎ Edit</Btn>
                <Btn size="xs">🗑</Btn>
              </div>
            </div>
          ))}
          <Btn variant="primary" size="xs">+ New MCP (go to Discover)</Btn>
        </div>
      </PhoneFrame>
    </div>
  );
}

window.MyMcpsScreens = [
  {label:'A · Desktop', tag:'balanced',
    note:'Production-parity table with 4 instances (deepwiki / exa / notion / gmail-a). gmail-a shows needs_reauth chip. Pending-approvals banner above the table reflects user\'s 1 submitted request. + New MCP button routes to Discover. Click ▷ jumps to Playground with instance pre-selected.',
    novel:'needs-reauth chip · pending banner · jump-to-playground',
    component:MyMcpsDesktop},
  {label:'A · Medium · tablet', tag:'medium',
    note:'Condensed table at tablet width. Same columns; smaller type.',
    novel:'',
    component:MyMcpsMedium},
  {label:'A · Mobile', tag:'mobile',
    note:'Each instance becomes a stacked card with inline actions. Pending banner shown at top.',
    novel:'',
    component:MyMcpsMobile},
];
