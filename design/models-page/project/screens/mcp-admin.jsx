// MCP Admin · v30
// Admin/Manager registry page: registered servers + approval inbox. Two sub-sections
// connected by an anchor rail on Desktop and anchor strip on Medium. 3 variants.

function McpAdminBody({compact=false}) {
  return (
    <>
      <div style={{display:'flex', justifyContent:'space-between', alignItems:'baseline', marginBottom:8}}>
        <div className={compact ? 'h2' : 'h1'} style={{margin:0}}>
          MCP Admin <Chip tone="indigo" style={{fontSize:10, marginLeft:4}}>admin</Chip>
        </div>
        <div style={{display:'flex', gap:6}}>
          <Btn size={compact?'xs':''}>+ From catalog ▾</Btn>
          <Btn variant="primary" size={compact?'xs':''}>+ Register MCP Server</Btn>
        </div>
      </div>
      <div className="sm" style={{color:'var(--ink-3)', marginBottom:6}}>Approve which MCP servers users in this app instance can connect to · review submitted requests in the inbox.</div>

      <div className="mcp-admin-section" id="servers">
        <h3>Registered servers <Chip style={{fontSize:10}}>{MCP_SERVERS_FIXTURE.length}</Chip></h3>
        <div style={{display:'grid', gridTemplateColumns:'24px 1fr 2fr 80px 60px 110px', gap:10,
                     fontSize:10, textTransform:'uppercase', color:'var(--ink-3)',
                     padding:'4px 10px', letterSpacing:0.5}}>
          <span></span><span>Name</span><span>URL</span><span>Status</span><span>Uses</span><span style={{textAlign:'right'}}>Actions</span>
        </div>
        {MCP_SERVERS_FIXTURE.map(s => <McpServerListRow key={s.slug} server={s}/>)}
      </div>

      <div className="mcp-admin-section" id="approvals" style={{background:'var(--warn-soft)'}}>
        <h3>
          Pending approvals
          <span className="mcp-admin-inbox-badge">{MCP_APPROVAL_FIXTURE.length}</span>
        </h3>
        <div className="sm" style={{color:'var(--ink-3)', marginBottom:6}}>User-submitted requests to add a catalog server to this app instance · approve to register.</div>
        {MCP_APPROVAL_FIXTURE.map(req => <McpApprovalRow key={req.id} request={req}/>)}
      </div>
    </>
  );
}

// ── 1. Desktop ───────────────────────────────────────────────
function McpAdminDesktop() {
  return (
    <Browser url="bodhi.local/admin/mcp">
      <Crumbs items={['Bodhi','Admin','MCP']}/>
      <div style={{display:'grid', gridTemplateColumns:'160px 1fr', gap:12, alignItems:'start'}}>
        <McpRail active="servers"/>
        <div>
          <McpAdminBody/>
          <Callout style={{position:'static', display:'inline-block', marginTop:8}}>
            ★ + From catalog opens the Discover flow with admin-pre-fill handler · + Register MCP Server opens blank McpServerForm · context7 is disabled
          </Callout>
        </div>
      </div>
    </Browser>
  );
}

// ── 2. Medium · tablet ───────────────────────────────────────
function McpAdminMedium() {
  return (
    <TabletFrame label="Tablet · anchor strip · sections stack">
      <MobileHeader active="MCP Admin"/>
      <McpMediumAnchors active="servers"/>
      <div style={{padding:'8px 10px'}}>
        <McpAdminBody compact/>
      </div>
    </TabletFrame>
  );
}

// ── 3. Mobile · phone ────────────────────────────────────────
function McpAdminMobile() {
  return (
    <div className="phone-deck">
      <PhoneFrame label="1 · Servers tab">
        <MobileHeader active="MCP Admin"/>
        <div style={{padding:'6px 8px'}}>
          <div style={{display:'flex', gap:4, marginBottom:6}}>
            <Chip on tone="indigo">Servers</Chip>
            <Chip tone="warn">Approvals · 2</Chip>
          </div>
          <div className="h2" style={{margin:0, marginBottom:6, fontSize:14}}>Registered</div>
          {MCP_SERVERS_FIXTURE.slice(0,4).map(s => (
            <div key={s.slug} style={{padding:8, background:'var(--paper-2)',
                 border:'1.3px solid var(--ink)', borderRadius:8, marginBottom:6,
                 fontFamily:'var(--hand)', fontSize:12}}>
              <div style={{display:'flex', justifyContent:'space-between'}}>
                <strong>{s.name}</strong>
                <Chip tone={s.enabled?'leaf':''} style={{fontSize:9}}>{s.enabled?'enabled':'disabled'}</Chip>
              </div>
              <div className="sm" style={{fontSize:10, color:'var(--ink-3)'}}>{s.url}</div>
              <div className="sm" style={{fontSize:10, color:'var(--ink-3)'}}>{s.authType} · {s.instances} inst</div>
              <div style={{display:'flex', gap:4, marginTop:4}}>
                <Btn size="xs">✎</Btn>
                <Btn size="xs">{s.enabled?'⏸':'▶'}</Btn>
                <Btn size="xs">🗑</Btn>
              </div>
            </div>
          ))}
          <Btn variant="primary" size="xs">+ Register MCP Server</Btn>
        </div>
      </PhoneFrame>
      <PhoneFrame label="2 · Approvals tab">
        <MobileHeader active="MCP Admin"/>
        <div style={{padding:'6px 8px'}}>
          <div style={{display:'flex', gap:4, marginBottom:6}}>
            <Chip>Servers</Chip>
            <Chip on tone="warn">Approvals · 2</Chip>
          </div>
          {MCP_APPROVAL_FIXTURE.map(req => (
            <div key={req.id} style={{padding:8, background:'var(--warn-soft)',
                 border:'1.3px dashed var(--warn)', borderRadius:8, marginBottom:6,
                 fontFamily:'var(--hand)', fontSize:12}}>
              <strong>{req.name}</strong>
              <div className="sm" style={{fontSize:10, color:'var(--ink-3)'}}>by {req.requester} · {req.requestedAt}</div>
              <div className="sm" style={{color:'var(--ink-2)', fontSize:11, marginTop:3}}>{req.reason}</div>
              <div style={{display:'flex', gap:4, marginTop:4}}>
                <Btn size="xs">✗ Reject</Btn>
                <Btn variant="primary" size="xs">✓ Approve</Btn>
              </div>
            </div>
          ))}
        </div>
      </PhoneFrame>
    </div>
  );
}

window.McpAdminScreens = [
  {label:'A · Desktop', tag:'balanced',
    note:'Two sections linked by rail: Registered servers (6 rows, with 1 disabled context7) + Pending approvals (2 inbox rows). Top-right actions: + From catalog (opens Discover w/ admin-prefill) and + Register MCP Server (blank form). Badge shows inbox count.',
    novel:'admin inbox · from-catalog admin pre-fill · disabled servers shown',
    component:McpAdminDesktop},
  {label:'A · Medium · tablet', tag:'medium',
    note:'Anchor strip replaces rail. Sections stack vertically.',
    novel:'',
    component:McpAdminMedium},
  {label:'A · Mobile', tag:'mobile',
    note:'Two frames: Servers tab + Approvals tab — pill-tab navigation. Servers collapse to stacked cards; approvals highlighted in warn tone.',
    novel:'pill-tabs replace rail at phone width',
    component:McpAdminMobile},
];
