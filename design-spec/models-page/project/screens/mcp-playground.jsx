// MCP Playground · v30
// Exercise tools on a connected instance. Left sidebar picks an instance + tool;
// main pane hosts parameter form + Execute + tabbed response. Matches production
// screenshot download (11).png. 3 variants: Desktop / Medium / Mobile.

// ── 1. Desktop ────────────────────────────────────────────────
function McpPlaygroundDesktop() {
  const tools = MCP_TOOLS_FIXTURE.notion;
  const selected = tools.find(t => t.name === 'notion-get-users');
  return (
    <Browser url="bodhi.local/mcps/playground">
      <Crumbs items={['Bodhi','MCP','Playground']}/>
      <div style={{display:'flex', alignItems:'baseline', justifyContent:'space-between', gap:10, marginBottom:6}}>
        <div>
          <div className="h1" style={{fontSize:20, marginBottom:2}}>
            MCP Playground <span style={{color:'var(--ink-3)', fontSize:14}}>· notion</span>
          </div>
          <div className="sm">Pick an instance · select a tool · run it with custom arguments · inspect request/response.</div>
        </div>
        <div style={{display:'flex', gap:6}}>
          <select style={{fontFamily:'var(--hand)', fontSize:12, padding:'4px 8px', border:'1.3px solid var(--ink)', borderRadius:6, background:'var(--paper-2)'}}>
            <option>notion</option><option>exa</option><option>deepwiki</option>
          </select>
        </div>
      </div>
      <div className="mcp-playground-layout">
        <McpToolSidebar tools={tools} selected="notion-get-users" search="" instanceName="notion"/>
        <McpToolExecutor tool={selected} activeResponseTab="success"/>
      </div>
      <Callout style={{position:'static', display:'inline-block', marginTop:8}}>
        ★ notion instance has 7 tools · notion-get-users selected · 4 params shown · Success response rendered as JSON
      </Callout>
    </Browser>
  );
}

// ── 2. Medium · tablet ───────────────────────────────────────
function McpPlaygroundMedium() {
  const tools = MCP_TOOLS_FIXTURE.notion;
  const selected = tools.find(t => t.name === 'notion-get-users');
  return (
    <TabletFrame label="Tablet · sidebar collapses to drawer">
      <MobileHeader active="MCP Playground"/>
      <div style={{padding:'8px 10px'}}>
        <div style={{display:'flex', gap:6, alignItems:'center', marginBottom:6}}>
          <Btn size="xs">◧ Tools</Btn>
          <span className="h2" style={{margin:0, fontSize:14}}>notion · notion-get-users</span>
        </div>
        <McpToolExecutor tool={selected}/>
      </div>
    </TabletFrame>
  );
}

// ── 3. Mobile · phone (3 stage wizard frames) ────────────────
function McpPlaygroundMobile() {
  const tools = MCP_TOOLS_FIXTURE.notion;
  const selected = tools.find(t => t.name === 'notion-get-users');
  return (
    <div className="phone-deck">
      <PhoneFrame label="1 · Pick instance + tool">
        <MobileHeader active="MCP Playground"/>
        <div style={{padding:'6px 8px'}}>
          <Field label="Instance" filled value="notion"/>
          <div style={{marginTop:6}}>
            <div className="sm" style={{fontWeight:700, marginBottom:4}}>Pick a tool</div>
            {tools.slice(0,5).map(t => (
              <div key={t.name} className={`mcp-tool-sidebar-item${t.name==='notion-get-users'?' active':''}`}>
                <div className="mcp-tool-name">{t.name}</div>
                <div className="mcp-tool-desc">{t.desc.slice(0,50)}…</div>
              </div>
            ))}
          </div>
        </div>
      </PhoneFrame>
      <PhoneFrame label="2 · Fill params · execute">
        <MobileHeader active="notion-get-users"/>
        <div style={{padding:'6px 8px'}}>
          <div className="h2" style={{margin:0, fontSize:14}}>notion-get-users</div>
          <div className="sm" style={{color:'var(--ink-3)', marginBottom:6}}>{selected.desc.slice(0,80)}…</div>
          {(selected.parameters || []).slice(0,3).map(p => (
            <div key={p.name} style={{marginBottom:4}}>
              <div className="sm" style={{fontWeight:700}}>{p.name} <span style={{color:'var(--ink-3)', fontWeight:400}}>({p.type})</span></div>
              <Field filled value=""/>
            </div>
          ))}
          <Btn variant="primary" size="xs">Execute</Btn>
        </div>
      </PhoneFrame>
      <PhoneFrame label="3 · Response">
        <MobileHeader active="Response"/>
        <div style={{padding:'6px 8px'}}>
          <div style={{display:'flex', gap:4, marginBottom:6}}>
            <Chip on tone="leaf" style={{fontSize:9}}>● Success</Chip>
            <Chip style={{fontSize:9}}>Response</Chip>
            <Chip style={{fontSize:9}}>Raw</Chip>
            <Chip style={{fontSize:9}}>Request</Chip>
          </div>
          <div className="mcp-response-body" style={{fontSize:9.5, maxHeight:220}}>
            {`[\n  {\n    "type": "text",\n    "text": "{\\"results\\":[{\\"type\\":\\"person\\",\\"id\\":\\"73b3…\\"}]}"\n  }\n]`}
          </div>
        </div>
      </PhoneFrame>
    </div>
  );
}

window.McpPlaygroundScreens = [
  {label:'A · Desktop', tag:'balanced',
    note:'Sidebar (instance picker + search + 7 Notion tools) + main pane (notion-get-users selected, Form/JSON tabs, 4 params, Execute, Success/Response/Raw JSON/Request tabs). Matches production download (11).png.',
    novel:'production-parity playground',
    component:McpPlaygroundDesktop},
  {label:'A · Medium · tablet', tag:'medium',
    note:'Sidebar collapses to a ◧ Tools drawer button; main pane occupies full width.',
    novel:'',
    component:McpPlaygroundMedium},
  {label:'A · Mobile', tag:'mobile',
    note:'Three frames: (1) pick-tool list, (2) params + Execute, (3) response tabs. Progressive disclosure for small screens.',
    novel:'three-step wizard layout at phone width',
    component:McpPlaygroundMobile},
];
