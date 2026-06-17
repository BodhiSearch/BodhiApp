/* ═══════════════════════════════════════════════════
   ACCESS REQUEST REVIEW — Full page React app
   Depends on: BodhiSidebar, ModelAccessPicker
═══════════════════════════════════════════════════ */
const { useState, useEffect, useRef } = React;

const IS_ADMIN = true;
const REQUEST = {
  appId: 'bodhi-app-f181a4d1-d7af-43f4-965a-0a8efd453d86',
  appName: 'Research Copilot',
  appDesc: 'An agent that helps you summarise research papers, organise Notion pages, and pull market data from web search.',
  appInitial: 'R', appColor: '#3E4AA8', verified: true,
};
const MODEL_SLOTS = [
  { id: 'main-model',      label: 'Main Reasoning Model',  desc: 'Used for complex reasoning, summarisation, and research synthesis.',   caps: ['text2text','tool-use'] },
  { id: 'visual-lm',       label: 'Visual Language Model', desc: 'Analyses images and charts from research papers alongside textual context.', caps: ['text2text','image2text'] },
  { id: 'embedding-model', label: 'Embedding Model',       desc: 'Generates vector embeddings for semantic search across paper archives.', caps: ['embedding'] },
];
const ALL_MODELS = [
  { id: 'my-qwen-long',   name: 'my-qwen-long',                  type:'local', ctx:'128k', caps:['text2text','tool-use'],               suggested:['main-model'], failFor:{} },
  { id: 'my-gemma',       name: 'my-gemma',                       type:'local', ctx:'32k',  caps:['text2text'],                          suggested:[],             failFor:{'main-model':'Missing: tool-use'} },
  { id: 'my-vision-llm',  name: 'my-vision-llm',                  type:'local', ctx:'16k',  caps:['text2text','image2text'],             suggested:['visual-lm'],  failFor:{} },
  { id: 'my-embed',       name: 'my-embed',                       type:'local', ctx:'8k',   caps:['embedding'],                         suggested:['embedding-model'], failFor:{} },
  { id: 'gpt-4o-mini',    name: 'openai/gpt-4o-mini',             type:'api',   ctx:'128k', caps:['text2text','tool-use','image2text'],  suggested:['main-model','visual-lm'], failFor:{}, cost:'$0.15/M' },
  { id: 'claude-sonnet',  name: 'anthropic/claude-sonnet-4-6',    type:'api',   ctx:'200k', caps:['text2text','tool-use','image2text'],  suggested:['main-model','visual-lm'], failFor:{}, cost:'$3/M' },
  { id: 'claude-haiku',   name: 'anthropic/claude-haiku-4-5',     type:'api',   ctx:'200k', caps:['text2text','tool-use'],               suggested:['main-model'], failFor:{}, cost:'$0.25/M' },
  { id: 'gemini-pro',     name: 'google/gemini-2.0-pro',          type:'api',   ctx:'1M',   caps:['text2text','tool-use','image2text'],  suggested:['main-model','visual-lm'], failFor:{}, cost:'$2/M' },
  { id: 'text-embed-3',   name: 'openai/text-embedding-3-small',  type:'api',   ctx:'8k',   caps:['embedding'],                         suggested:['embedding-model'], failFor:{}, cost:'$0.02/M' },
];
const MCP_SERVERS = [
  { id:'exa',    name:'Exa Search', url:'https://mcp.exa.ai/mcp',       icon:'E', iconBg:'#1A1A2E', state:'ready',          instances:[{id:'exa',name:'exa'},{id:'exa-work',name:'exa-work'}], selectedInst:'exa' },
  { id:'notion', name:'Notion',     url:'https://mcp.notion.com/mcp',   icon:'N', iconBg:'#000',    state:'reauth',         instances:[{id:'notion-personal',name:'notion-personal'}],         selectedInst:'notion-personal' },
  { id:'linear', name:'Linear',     url:'https://mcp.linear.app/mcp',   icon:'L', iconBg:'#5E6AD2', state:'needs-instance', instances:[], selectedInst:null },
  { id:'gmail',  name:'Gmail',      url:'https://mcp.google.com/gmail', icon:'G', iconBg:'#EA4335', state:IS_ADMIN?'not-configured':'pending-admin', instances:[], selectedInst:null },
];
const CAP_COLORS = { 'text2text':'indigo','tool-use':'teal','image2text':'saffron','embedding':'leaf' };

/* ── Icon helper ── */
function Icon({ name, size = 14 }) {
  const ref = useRef(null);
  useEffect(() => {
    if (!ref.current) return;
    ref.current.innerHTML = '';
    const el = document.createElement('i');
    el.setAttribute('data-lucide', name);
    ref.current.appendChild(el);
    if (window.lucide) window.lucide.createIcons({ nodes: [el] });
  }, [name]);
  return <span ref={ref} style={{display:'inline-flex',width:size,height:size,alignItems:'center',justifyContent:'center',flexShrink:0}} />;
}

/* ── Admin Config Panel ── */
function AdminConfigPanel({ server, onSave, onClose }) {
  const [authType, setAuthType] = useState('oauth2');
  const [regType,  setRegType]  = useState('dynamic');
  useEffect(() => { if (window.lucide) window.lucide.createIcons(); });
  const PF = { gmail: { url:'https://mcp.google.com/gmail', authEp:'https://accounts.google.com/o/oauth2/auth', tokenEp:'https://oauth2.googleapis.com/token' } };
  const pf = PF[server.id] || {};
  return (
    <div className="panel-overlay" onClick={e => e.target.classList.contains('panel-backdrop') && onClose()}>
      <div className="panel-backdrop"></div>
      <div className="cfg-sheet">
        <div className="panel-head" style={{flexShrink:0}}>
          <div style={{display:'flex',alignItems:'center',gap:10}}>
            <div style={{width:32,height:32,borderRadius:8,background:server.iconBg,display:'flex',alignItems:'center',justifyContent:'center',fontSize:13,fontWeight:800,color:'#fff',flexShrink:0}}>{server.icon}</div>
            <div><div className="panel-title">Configure — {server.name}</div><div className="panel-subtitle">Set up auth so users can create instances</div></div>
          </div>
          <button className="panel-close" onClick={onClose}><Icon name="x" size={13} /></button>
        </div>
        <div className="cfg-body">
          <div className="cfg-section-title">1 · Server Connection</div>
          <div className="cfg-field"><div className="cfg-label">URL <span className="req">*</span></div><input className="cfg-input mono" defaultValue={pf.url||''} placeholder="https://mcp.example.com/mcp" /></div>
          <div className="cfg-field"><div className="cfg-label">Name</div><input className="cfg-input" defaultValue={server.name.toLowerCase()} /></div>
          <div className="cfg-divider"></div>
          <div className="cfg-section-title">2 · Authentication</div>
          <div style={{marginBottom:16}}>
            <div className="cfg-label" style={{marginBottom:8}}>Auth type</div>
            <div className="cfg-auth-types">
              {['oauth2','header','query','none'].map(t => <button key={t} className={`cfg-type-btn${authType===t?' active':''}`} onClick={()=>setAuthType(t)}>{t}</button>)}
            </div>
          </div>
          {authType === 'oauth2' && (<>
            <div className="cfg-field"><div className="cfg-label">Registration type</div><select className="cfg-select" value={regType} onChange={e=>setRegType(e.target.value)}><option value="dynamic">Dynamic Registration</option><option value="pre-registered">Pre-registered</option></select></div>
            <div className="cfg-field"><div className="cfg-label">Authorization Endpoint</div><input className="cfg-input mono" defaultValue={pf.authEp||''} placeholder="https://example.com/authorize" /></div>
            <div className="cfg-field"><div className="cfg-label">Token Endpoint</div><input className="cfg-input mono" defaultValue={pf.tokenEp||''} placeholder="https://example.com/token" /></div>
          </>)}
          <div style={{display:'flex',gap:8,marginTop:20}}>
            <button className="btn-sm btn-sm-indigo" style={{height:36,fontSize:13,padding:'0 16px'}} onClick={onSave}><Icon name="check" size={13}/> Save &amp; continue</button>
            <button className="btn-sm btn-sm-ghost"  style={{height:36,fontSize:13,padding:'0 14px'}} onClick={onClose}>Cancel</button>
          </div>
        </div>
      </div>
    </div>
  );
}

/* ── Model Slot ── */
function ModelSlot({ slot, enabled, onToggle, slotMode, onSlotModeChange, selectedIds, onToggleModel, onReorder }) {
  const suggested = ALL_MODELS.filter(m => m.suggested.includes(slot.id)).map(m => m.id);
  return (
    <div className="slot-card">
      <div className="slot-head">
        <div className={`slot-check${enabled?' on':''}`} onClick={() => onToggle(slot.id)}>
          {enabled && <svg viewBox="0 0 12 12" fill="none" stroke="#fff" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round" style={{width:9,height:9}}><polyline points="1.5,6 5,9.5 10.5,2.5"/></svg>}
        </div>
        <div className="slot-head-body">
          <div style={{display:'flex',alignItems:'center',gap:8,flexWrap:'wrap'}}>
            <span className="slot-name">{slot.label}</span>
            <span className="slot-id">{slot.id}</span>
          </div>
          <div className="slot-desc">{slot.desc}</div>
          <div className="slot-caps">{slot.caps.map(c => <span key={c} className={`tag tag-${CAP_COLORS[c]||'muted'}`}>{c}</span>)}</div>
        </div>
      </div>
      {enabled
        ? <div className="slot-body"><ModelAccessPicker mode={slotMode} onModeChange={onSlotModeChange} allModels={ALL_MODELS} selectedIds={selectedIds} onToggle={onToggleModel} panelTitle={`Models — ${slot.label}`} panelSubtitle="Select models to grant for this slot" suggestedIds={suggested} requiredCaps={slot.caps} showRanks={true} onReorder={onReorder} /></div>
        : <div className="slot-denied">This model slot will be denied for the app.</div>
      }
    </div>
  );
}

/* ── MCP Row ── */
function McpRow({ server, enabled, onToggle, onStateChange, onInstanceSelect, isAdmin, onOpenConfig }) {
  return (
    <div className="mcp-row">
      <div className={`mcp-row-head${!enabled?' denied':''}`}>
        <div className={`mcp-check${enabled?' on':''}`} onClick={() => onToggle(server.id)}>
          {enabled && <svg viewBox="0 0 12 12" fill="none" stroke="#fff" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round" style={{width:9,height:9}}><polyline points="1.5,6 5,9.5 10.5,2.5"/></svg>}
        </div>
        <div className="mcp-icon" style={{background:server.iconBg}}>{server.icon}</div>
        <div style={{flex:1,minWidth:0}}><div className="mcp-name">{server.name}</div><div className="mcp-url">{server.url}</div></div>
      </div>
      {enabled && (
        <div className="mcp-row-body">
          {server.state === 'ready' && <select className="inst-select" value={server.selectedInst||''} onChange={e=>onInstanceSelect(server.id,e.target.value)}><option value="">Select an instance…</option>{server.instances.map(i=><option key={i.id} value={i.id}>{i.name}</option>)}</select>}
          {server.state === 'reauth' && (<><div className="info-banner info-saffron"><Icon name="info" size={14}/><span>Token expired — instance will need to reconnect when first used. You can still select it now.</span></div><select className="inst-select" value={server.selectedInst||''} onChange={e=>onInstanceSelect(server.id,e.target.value)}><option value="">Select an instance…</option>{server.instances.map(i=><option key={i.id} value={i.id}>{i.name}</option>)}</select></>)}
          {server.state === 'needs-instance' && (<><div style={{fontSize:12.5,color:'hsl(var(--muted-foreground))',marginBottom:10}}>No instances yet. Create one to connect this server.</div><button className="btn-sm btn-sm-indigo" onClick={()=>window.open('Bodhi MCP New Instance.html','_blank')}><Icon name="plus-circle" size={12}/> Connect instance</button></>)}
          {server.state === 'not-configured' && isAdmin  && (<><div style={{fontSize:12.5,color:'hsl(var(--muted-foreground))',marginBottom:10}}>This server hasn't been configured yet.</div><button className="btn-sm btn-sm-saffron" onClick={()=>onOpenConfig(server.id)}><Icon name="settings-2" size={12}/> Configure MCP server</button></>)}
          {server.state === 'not-configured' && !isAdmin && (<><div style={{fontSize:12.5,color:'hsl(var(--muted-foreground))',marginBottom:10}}>This server hasn't been configured by your admin.</div><button className="btn-sm btn-sm-saffron" onClick={()=>onStateChange(server.id,'pending-admin')}><Icon name="send" size={12}/> Request admin to configure</button></>)}
          {server.state === 'pending-admin' && <div className="info-banner info-pending"><Icon name="clock" size={14}/><span>Request sent to your admin. You'll be notified once configured.</span></div>}
        </div>
      )}
    </div>
  );
}

/* ── Form ── */
function AccessRequestForm() {
  const [slotEnabled, setSlotEnabled] = useState(Object.fromEntries(MODEL_SLOTS.map(s=>[s.id,true])));
  const [slotMode,    setSlotMode]    = useState(Object.fromEntries(MODEL_SLOTS.map(s=>[s.id,'all'])));
  const [slotModels,  setSlotModels]  = useState(Object.fromEntries(MODEL_SLOTS.map(s=>[s.id,[]])));
  const [openConfig,  setOpenConfig]  = useState(null);
  const [mcpServers,  setMcpServers]  = useState(MCP_SERVERS);
  const [mcpEnabled,  setMcpEnabled]  = useState(Object.fromEntries(MCP_SERVERS.map(s=>[s.id,true])));
  const [role,        setRole]        = useState('user');
  useEffect(() => { if (window.lucide) window.lucide.createIcons(); });

  const toggleSlot    = id => setSlotEnabled(p=>({...p,[id]:!p[id]}));
  const toggleModel   = (slotId,mid) => setSlotModels(p=>{const c=p[slotId]||[];return{...p,[slotId]:c.includes(mid)?c.filter(x=>x!==mid):[...c,mid]};});
  const reorderModels = (slotId,arr) => setSlotModels(p=>({...p,[slotId]:arr}));
  const toggleMcp     = id => setMcpEnabled(p=>({...p,[id]:!p[id]}));
  const updateState   = (id,s) => setMcpServers(p=>p.map(x=>x.id===id?{...x,state:s}:x));
  const updateInst    = (id,v) => setMcpServers(p=>p.map(x=>x.id===id?{...x,selectedInst:v}:x));
  const saveConfig    = id  => { setMcpServers(p=>p.map(x=>x.id===id?{...x,state:'needs-instance'}:x)); setOpenConfig(null); };

  const enabledMcps  = mcpServers.filter(s=>mcpEnabled[s.id]);
  const blockedMcps  = enabledMcps.filter(s=>['needs-instance','not-configured','pending-admin'].includes(s.state));
  const readyMcps    = enabledMcps.filter(s=>s.state==='ready'||s.state==='reauth');
  const enabledSlots = MODEL_SLOTS.filter(s=>slotEnabled[s.id]).length;
  const approveCount = enabledSlots + readyMcps.filter(s=>s.selectedInst).length + 1;
  const isBlocked    = blockedMcps.length > 0;

  return (
    <div className="ar-main">
    <div className="ar-scroll">
    <div className="ar-inner">
      <div className="page-header" style={{display:'block',marginBottom:24}}>
        <div className="page-title">Review Access Request</div>
        <div className="page-subtitle">Decide which of your resources this 3rd-party app can use.</div>
      </div>
    <div className="form-card">
      <div className="app-identity">
        <div className="app-icon" style={{background:REQUEST.appColor}}>{REQUEST.appInitial}</div>
        <div className="app-identity-body">
          <div className="app-name-row">
            <span className="app-name">{REQUEST.appName}</span>
            {REQUEST.verified && <span className="tag tag-leaf">✓ verified</span>}
            <span className="tag tag-muted">3rd-party app</span>
          </div>
          <div className="app-subtitle">is requesting access to your resources.</div>
          <div className="app-id">{REQUEST.appId}</div>
          <div className="app-desc">{REQUEST.appDesc}</div>
        </div>
        <span className="tag tag-muted">Role: {IS_ADMIN?'Admin':'User'}</span>
      </div>

      <div className="form-divider"></div>

      <div className="section-label">
        <span>Model access</span>
        <span className="section-label-count">{MODEL_SLOTS.filter(s=>slotEnabled[s.id]).length} of {MODEL_SLOTS.length} slots enabled</span>
      </div>
      {MODEL_SLOTS.map(slot => (
        <ModelSlot key={slot.id} slot={slot} enabled={slotEnabled[slot.id]} onToggle={toggleSlot}
          slotMode={slotMode[slot.id]} onSlotModeChange={mode=>setSlotMode(p=>({...p,[slot.id]:mode}))}
          selectedIds={slotModels[slot.id]||[]} onToggleModel={mid=>toggleModel(slot.id,mid)}
          onReorder={arr=>reorderModels(slot.id,arr)} />
      ))}

      <div className="form-divider"></div>

      <div className="section-label">
        <span>MCP access</span>
        <span className="section-label-count">{enabledMcps.length} of {mcpServers.length} selected · {readyMcps.filter(s=>s.selectedInst).length} ready</span>
      </div>
      {mcpServers.map(server => (
        <McpRow key={server.id} server={server} enabled={mcpEnabled[server.id]}
          onToggle={toggleMcp} onStateChange={updateState} onInstanceSelect={updateInst}
          isAdmin={IS_ADMIN} onOpenConfig={setOpenConfig} />
      ))}

      <div className="form-divider"></div>

      <div className="section-label" style={{marginBottom:12}}><span>Approved role</span></div>
      <div style={{display:'flex',alignItems:'center',gap:12}}>
        <select className="role-select" value={role} onChange={e=>setRole(e.target.value)}>
          <option value="user">User</option>
          <option value="power-user">Power User</option>
        </select>
        <span style={{fontSize:12.5,color:'hsl(var(--muted-foreground))'}}>Drives what this app can see in your resources</span>
      </div>
    </div>

    <div className="action-bar">
      <div>
        {isBlocked
          ? <div className="action-warn"><Icon name="alert-triangle" size={14}/>{blockedMcps.length} MCP {blockedMcps.length===1?'server needs':'servers need'} setup before approving</div>
          : <div className="action-ok"><Icon name="check-circle-2" size={14}/>All resources ready to approve</div>
        }
      </div>
      <div className="action-btns">
        <button className="btn-deny">Deny &amp; return to app</button>
        <button className="btn-approve" disabled={isBlocked}>
          <Icon name="check" size={15}/>
          {`Approve ${approveCount} resource${approveCount!==1?'s':''}`}
        </button>
      </div>
    </div>
    </div>{/* ar-inner */}
    </div>{/* ar-scroll */}

    {openConfig && <AdminConfigPanel server={mcpServers.find(s=>s.id===openConfig)} onSave={()=>saveConfig(openConfig)} onClose={()=>setOpenConfig(null)} />}
    </div>
  );
}

/* ── Theme toggle (top bar) ── */
function StdThemeToggle() {
  const [dark, setDark] = useState(() => window.bodhiTheme && window.bodhiTheme.resolved === 'dark');
  useEffect(() => {
    if (!window.bodhiTheme) return;
    return window.bodhiTheme.subscribe((m, r) => setDark(r === 'dark'));
  }, []);
  return (
    <button className="std-theme-toggle" onClick={() => window.bodhiTheme && window.bodhiTheme.toggle()}>
      <Icon name={dark ? 'sun' : 'moon'} size={15} />
      <span className="std-tt-label">{dark ? 'Light' : 'Dark'}</span>
    </button>
  );
}

/* ── Full page app (standalone — no shell, no breadcrumb) ── */
function AccessRequestApp() {
  useEffect(() => { if (window.lucide) window.lucide.createIcons(); });
  return (
    <div className="std-page">
      <div className="std-topbar">
        <a className="std-brand" href="Bodhi Chat.html">
          <span className="std-brand-mark"></span>
          <span className="std-brand-text">
            <span className="std-brand-word">Bodhi</span>
            <span className="std-brand-sub">AI Gateway</span>
          </span>
        </a>
        <StdThemeToggle />
      </div>
      <div className="std-main is-fill">
        <AccessRequestForm />
      </div>
    </div>
  );
}

let __rootEl = document.getElementById('root');
if (!__rootEl) {
  __rootEl = document.createElement('div');
  __rootEl.id = 'root';
  document.body.appendChild(__rootEl);
}
ReactDOM.createRoot(__rootEl).render(<AccessRequestApp />);
