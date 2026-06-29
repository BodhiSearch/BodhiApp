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
  { id: 'embedding-model', label: 'Embedding Model',       desc: 'Generates vector embeddings for semantic search across paper archives.', caps: ['embedding'], selection: 'single' },
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
  { id:'exa',    name:'Exa Search', url:'https://mcp.exa.ai/mcp',       desc:'Search the web',            icon:'E', iconBg:'#1A1A2E', state:'ready',          instances:[{id:'exa',name:'exa'},{id:'exa-work',name:'exa-work'}], selectedInst:'exa' },
  { id:'notion', name:'Notion',     url:'https://mcp.notion.com/mcp',   desc:'Your Notion workspace',     icon:'N', iconBg:'#000',    state:'reauth',         instances:[{id:'notion-personal',name:'notion-personal'}],         selectedInst:'notion-personal' },
  { id:'linear', name:'Linear',     url:'https://mcp.linear.app/mcp',   desc:'Issues & project tracking', icon:'L', iconBg:'#5E6AD2', state:'needs-instance', instances:[], selectedInst:null },
  { id:'gmail',  name:'Gmail',      url:'https://mcp.google.com/gmail', desc:'Your Gmail inbox',          icon:'G', iconBg:'#EA4335', state:IS_ADMIN?'not-configured':'pending-admin', instances:[], selectedInst:null },
];
/* Registered MCP servers the OWNER can additionally grant — the app did NOT
   request these. User-driven privilege: the approver proactively connects them.
   Kept distinct from the requested MCP_SERVERS above. */
const EXTRA_MCPS = [
  { id:'filesystem',   label:'filesystem',   meta:'Read / write local files' },
  { id:'github',       label:'github',       meta:'Repos, issues & pull requests' },
  { id:'postgres',     label:'postgres',     meta:'PostgreSQL queries' },
  { id:'slack',        label:'slack',        meta:'Workspace messages & channels' },
  { id:'brave-search', label:'brave-search', meta:'Web search via Brave API' },
  { id:'memory',       label:'memory',       meta:'Persistent key-value store' },
  { id:'fetch',        label:'fetch',        meta:'HTTP fetch & scrape' },
  { id:'sqlite',       label:'sqlite',       meta:'Query SQLite databases' },
];
const CAP_COLORS = { 'text2text':'indigo','tool-use':'teal','image2text':'saffron','embedding':'leaf' };
const CAP_LABELS = { 'text2text':'Text','tool-use':'Tools','image2text':'Images','embedding':'Search' };

/* ── Token-exchange scenario: what the SUBMITTED token already holds ──
   In upgrade mode the form initialises to the requested (elevated) state and
   we diff against this to derive "previously granted" (green) vs "new" (amber)
   at every layer. Additive only — the request never asks for LESS than this.
   The new-request flow is unchanged; upgrade mode READS this prior state so
   previously-granted slots load in their granted tier with the granted models
   pre-selected (never reset), then layers the new asks on top. */
const PREV_GRANT = {
  prevTokenId: 'bodhi_pat_b3f1c8a9-7d2e-4a55-9c01-2f6e8d4a1b90',
  listModels: false,            // model listing was NOT held → newly requested
  listMcps:   true,             // MCP listing already held
  role:       'user',           // previously a plain User
  slots: {                      // per-slot grant the prior token carried
    'main-model':      { granted: true,  mode: 'specific', ids: ['my-qwen-long'] },
    'visual-lm':       { granted: true,  mode: 'specific', ids: ['my-vision-llm'] },
    'embedding-model': { granted: false },                   // brand-new slot
  },
  mcps: { exa: true, notion: true, linear: false, gmail: false },
  extraMcps: ['filesystem', 'github'],   // user-driven MCP grants the prior token already carried
};

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

/* ── Placeholder dialog — stands in for forms that open in a separate window ──
   MCP servers / instances can't be configured from this consent flow (or a side
   panel); the real app launches a dedicated window. Here we just acknowledge that. */
function PlaceholderDialog({ icon, title, subtitle, message, onClose }) {
  useEffect(() => {
    const onKey = e => { if (e.key === 'Escape') onClose(); };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, [onClose]);
  return (
    <div className="ar-dialog-overlay" onClick={e => e.target.classList.contains('ar-dialog-overlay') && onClose()}>
      <div className="ar-dialog" role="dialog" aria-modal="true" aria-label={title}>
        <div className="ar-dialog-head">
          <div className="ar-dialog-icon"><Icon name={icon} size={18} /></div>
          <div style={{flex:1,minWidth:0}}>
            <div className="ar-dialog-title">{title}</div>
            <div className="ar-dialog-sub">{subtitle}</div>
          </div>
          <button className="panel-close" onClick={onClose}><Icon name="x" size={13} /></button>
        </div>
        <div className="ar-dialog-stub"><Icon name="external-link" size={15} /><span>{message}</span></div>
        <div className="ar-dialog-note">Placeholder — in the live app this opens the full form in a separate window. Nothing is created here.</div>
        <div className="ar-dialog-foot">
          <button className="btn-sm btn-sm-indigo" style={{height:34,fontSize:13,padding:'0 16px'}} onClick={onClose}>Got it</button>
        </div>
      </div>
    </div>
  );
}

/* ── Model Slot ── */
function ModelSlot({ slot, enabled, onToggle, slotMode, onSlotModeChange, selectedIds, onToggleModel, onReorder, granted, isNew, grantedMode, grantedIds, singleValue, onSingleChange }) {
  const suggested = ALL_MODELS.filter(m => m.suggested.includes(slot.id)).map(m => m.id);
  const isSingle = slot.selection === 'single';
  return (
    <div className={`slot-card${granted?' is-granted':''}${isNew?' is-new':''}`}>
      <div className="slot-head" onClick={() => onToggle(slot.id)} role="checkbox" aria-checked={enabled}>
        <div className={`slot-check${enabled?' on':''}`}>
          {enabled && <svg viewBox="0 0 12 12" fill="none" stroke="#fff" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round" style={{width:9,height:9}}><polyline points="1.5,6 5,9.5 10.5,2.5"/></svg>}
        </div>
        <div className="slot-head-body">
          <div style={{display:'flex',alignItems:'center',gap:8,flexWrap:'wrap'}}>
            <span className="slot-name">{slot.label}</span>
            {granted && <span className="map-granted-pill">✓ previously granted</span>}
            {isNew && <span className="map-new-pill">new access</span>}
          </div>
          <div className="slot-desc">{slot.desc}</div>
          <div className="slot-caps">{slot.caps.map(c => <span key={c} className={`tag tag-${CAP_COLORS[c]||'muted'}`}>{CAP_LABELS[c]||c}</span>)}</div>
        </div>
      </div>
      {enabled
        ? <div className="slot-body">
            {isSingle
              ? <div className="slot-single">
                  <div className="slot-single-help"><Icon name="info" size={12} /><span>Choose one model — pick one of yours or type the name of any model your provider offers.</span></div>
                  <SingleModelCombo value={singleValue} onChange={onSingleChange} allModels={ALL_MODELS} suggestedIds={suggested} requiredCaps={slot.caps} placeholder="Select a model or type a name…" plainLanguage={true} />
                  {!singleValue && <div className="slot-single-empty">No model selected — this won't be granted.</div>}
                </div>
              : <ModelAccessPicker mode={slotMode} onModeChange={onSlotModeChange} allModels={ALL_MODELS} selectedIds={selectedIds} onToggle={onToggleModel} panelTitle={`Models — ${slot.label}`} panelSubtitle="Select models to grant for this slot" suggestedIds={suggested} requiredCaps={slot.caps} showRanks={true} onReorder={onReorder} grantedMode={grantedMode} grantedIds={grantedIds} plainLanguage={true} />}
          </div>
        : <div className="slot-denied">This model slot will be denied for the app.</div>
      }
    </div>
  );
}

/* ── MCP Row ── */
function McpRow({ server, enabled, onToggle, onStateChange, onInstanceSelect, isAdmin, onShowPlaceholder, granted, isNew }) {
  return (
    <div className={`mcp-row${granted?' is-granted':''}${isNew?' is-new':''}`}>
      <div className={`mcp-row-head${!enabled?' denied':''}`} onClick={() => onToggle(server.id)} role="checkbox" aria-checked={enabled}>
        <div className={`mcp-check${enabled?' on':''}`}>
          {enabled && <svg viewBox="0 0 12 12" fill="none" stroke="#fff" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round" style={{width:9,height:9}}><polyline points="1.5,6 5,9.5 10.5,2.5"/></svg>}
        </div>
        <div className="mcp-icon" style={{background:server.iconBg}}>{server.icon}</div>
        <div style={{flex:1,minWidth:0}}>
          <div style={{display:'flex',alignItems:'center',gap:7,flexWrap:'wrap'}}>
            <span className="mcp-name">{server.name}</span>
            {granted && <span className="map-granted-pill">✓ previously granted</span>}
            {isNew && <span className="map-new-pill">new connection</span>}
          </div>
          <div className="mcp-desc">{server.desc}</div>
        </div>
      </div>
      {enabled && (
        <div className="mcp-row-body">
          {(server.state === 'ready' || server.state === 'reauth') && <select className="inst-select" value={server.selectedInst||''} onChange={e=>onInstanceSelect(server.id,e.target.value)}><option value="">Select a connection…</option>{server.instances.map(i=><option key={i.id} value={i.id}>{i.name}</option>)}</select>}
          {server.state === 'needs-instance' && (<><div style={{fontSize:12.5,color:'hsl(var(--muted-foreground))',marginBottom:10}}>No connection yet. Set one up to use this tool.</div><button className="btn-sm btn-sm-indigo" onClick={()=>onShowPlaceholder({icon:'plus-circle',title:`Set up ${server.name}`,subtitle:'Add a new connection',message:'Opens the setup form in a new window'})}><Icon name="plus-circle" size={12}/> Set up connection</button></>)}
          {server.state === 'not-configured' && isAdmin  && (<><div style={{fontSize:12.5,color:'hsl(var(--muted-foreground))',marginBottom:10}}>This tool hasn't been set up yet.</div><button className="btn-sm btn-sm-saffron" onClick={()=>onShowPlaceholder({icon:'settings-2',title:`Set up ${server.name}`,subtitle:'Set up this tool',message:'Opens the setup form in a new window'})}><Icon name="settings-2" size={12}/> Set up tool</button></>)}
          {server.state === 'not-configured' && !isAdmin && (<><div style={{fontSize:12.5,color:'hsl(var(--muted-foreground))',marginBottom:10}}>This tool hasn't been set up by your admin.</div><button className="btn-sm btn-sm-saffron" onClick={()=>onStateChange(server.id,'pending-admin')}><Icon name="send" size={12}/> Ask your admin to set it up</button></>)}
          {server.state === 'pending-admin' && <div className="info-banner info-pending"><Icon name="clock" size={14}/><span>Request sent to your admin. You'll be notified once it's ready.</span></div>}
        </div>
      )}
    </div>
  );
}

/* ── Form ── */
function AccessRequestForm({ scenario }) {
  const isUpgrade = scenario === 'upgrade';
  // In UPGRADE mode the form reads its initial state from the submitted token
  // (PREV_GRANT): previously-granted slots load in their granted tier with the
  // granted models pre-selected — those are never reset. New-request mode keeps
  // the original defaults (every slot at "All", nothing pre-selected).
  const slotInit = id => {
    const g = isUpgrade ? PREV_GRANT.slots[id] : null;
    if (g && g.granted) return { mode: g.mode, models: g.mode === 'specific' ? [...(g.ids || [])] : [] };
    return { mode: 'all', models: [] };
  };
  const [slotEnabled, setSlotEnabled] = useState(Object.fromEntries(MODEL_SLOTS.map(s=>[s.id,true])));
  const [slotMode,    setSlotMode]    = useState(Object.fromEntries(MODEL_SLOTS.map(s=>[s.id, slotInit(s.id).mode])));
  const [slotModels,  setSlotModels]  = useState(Object.fromEntries(MODEL_SLOTS.map(s=>[s.id, slotInit(s.id).models])));
  // single-selection slots hold ONE value (a model name or free text)
  const singleInit = id => { const g = isUpgrade ? PREV_GRANT.slots[id] : null; return (g && g.granted && g.value) ? g.value : ''; };
  const [slotSingle,  setSlotSingle]  = useState(Object.fromEntries(MODEL_SLOTS.filter(s=>s.selection==='single').map(s=>[s.id, singleInit(s.id)])));
  const [placeholder, setPlaceholder] = useState(null);
  const [mcpServers,  setMcpServers]  = useState(MCP_SERVERS);
  const [mcpEnabled,  setMcpEnabled]  = useState(Object.fromEntries(MCP_SERVERS.map(s=>[s.id,true])));
  const [role,        setRole]        = useState(isUpgrade ? 'power-user' : 'user');
  const [listModels,  setListModels]  = useState(isUpgrade);
  const [listMcps,    setListMcps]    = useState(isUpgrade);
  const [extraMcpMode, setExtraMcpMode] = useState('specific'); // user-driven extra MCP grant
  const [extraMcps,    setExtraMcps]    = useState(isUpgrade ? [...PREV_GRANT.extraMcps] : []);
  useEffect(() => { if (window.lucide) window.lucide.createIcons(); });

  const toggleSlot    = id => setSlotEnabled(p=>({...p,[id]:!p[id]}));
  const toggleModel   = (slotId,mid) => setSlotModels(p=>{const c=p[slotId]||[];return{...p,[slotId]:c.includes(mid)?c.filter(x=>x!==mid):[...c,mid]};});
  const reorderModels = (slotId,arr) => setSlotModels(p=>({...p,[slotId]:arr}));
  const setSingle     = (id,v) => setSlotSingle(p=>({...p,[id]:v}));
  const toggleMcp     = id => setMcpEnabled(p=>({...p,[id]:!p[id]}));
  const toggleExtraMcp = id => setExtraMcps(p => p.includes(id) ? p.filter(x=>x!==id) : [...p, id]);
  const updateState   = (id,s) => setMcpServers(p=>p.map(x=>x.id===id?{...x,state:s}:x));
  const updateInst    = (id,v) => setMcpServers(p=>p.map(x=>x.id===id?{...x,selectedInst:v}:x));

  const enabledMcps  = mcpServers.filter(s=>mcpEnabled[s.id]);
  const blockedMcps  = enabledMcps.filter(s=>['needs-instance','not-configured','pending-admin'].includes(s.state));
  const readyMcps    = enabledMcps.filter(s=>s.state==='ready'||s.state==='reauth');
  const enabledSlots = MODEL_SLOTS.filter(s=>slotEnabled[s.id]).length;
  const extraGrantCount = extraMcpMode === 'all' ? 1 : extraMcps.length;
  const approveCount = enabledSlots + readyMcps.filter(s=>s.selectedInst).length + extraGrantCount + 1;
  const isBlocked    = blockedMcps.length > 0;

  /* ── granted-vs-new derivation (upgrade / token-exchange mode) ── */
  const slotGranted     = id => isUpgrade && !!PREV_GRANT.slots[id]?.granted;
  const slotIsNew       = id => isUpgrade && slotEnabled[id] && !PREV_GRANT.slots[id]?.granted;
  const slotGrantedMode = id => slotGranted(id) ? PREV_GRANT.slots[id].mode : null;
  const slotGrantedIds  = id => slotGranted(id) ? (PREV_GRANT.slots[id].ids || []) : [];
  const mcpGranted      = id => isUpgrade && !!PREV_GRANT.mcps[id];
  const mcpIsNew        = id => isUpgrade && mcpEnabled[id] && !PREV_GRANT.mcps[id];
  const roleIsNew       = isUpgrade && role !== PREV_GRANT.role;

  /* Plain-language summary of what the submitted token already holds — shown
     in the exchange banner instead of its raw id. */
  const prevRoleLabel  = PREV_GRANT.role === 'user' ? 'User' : 'Power User';
  const prevModelCount = Object.values(PREV_GRANT.slots).filter(s => s.granted).length;
  const prevToolCount  = Object.values(PREV_GRANT.mcps).filter(Boolean).length + PREV_GRANT.extraMcps.length;

  return (
    <div className="ar-main">
    <div className="ar-scroll">
    <div className="ar-inner">
      <div className="page-header" style={{display:'block',marginBottom:24}}>
        <div className="page-title">{isUpgrade ? 'Review Permission Upgrade' : 'Review Access Request'}</div>
        <div className="page-subtitle">{isUpgrade
          ? 'This app already has access and is asking for more.'
          : 'Decide which of your resources this app can use.'}</div>
      </div>
    <div className="form-card">
      {isUpgrade && (
        <div className="ar-exchange-banner">
          <div className="ar-xchg-icon"><Icon name="repeat" size={16} /></div>
          <div className="ar-xchg-body">
            <div className="ar-xchg-title">Existing app requesting more access</div>
            <div className="ar-xchg-text">
              <b>{REQUEST.appName}</b> already has access to your account and is asking for <b>more permissions</b>. An app's access can't be changed once granted, so approving <b>creates new access and cancels the old one</b>. You can approve everything, switch off any <span className="ar-xchg-amber">new</span> item you don't want, or reduce <span className="ar-xchg-green">access it already has</span>.
            </div>
            <div className="ar-xchg-token">
              <span className="ar-xchg-token-label">Currently has</span>
              <span className="ar-xchg-token-note">{prevRoleLabel} access · {prevModelCount} {prevModelCount===1?'model':'models'} · {prevToolCount} {prevToolCount===1?'tool':'tools'}</span>
            </div>
          </div>
        </div>
      )}
      <div className="app-identity">
        <div className="app-icon" style={{background:REQUEST.appColor}}>{REQUEST.appInitial}</div>
        <div className="app-identity-body">
          <div className="app-name-row">
            <span className="app-name">{REQUEST.appName}</span>
            {REQUEST.verified && <span className="tag tag-leaf">✓ verified</span>}
            <span className="tag tag-muted">3rd-party app</span>
          </div>
          <div className="app-subtitle">is requesting access to your resources.</div>
          <div className="app-desc">{REQUEST.appDesc}</div>
        </div>
        <span className="tag tag-muted">Role: {IS_ADMIN?'Admin':'User'}</span>
      </div>

      <div className="form-divider"></div>

      <div className="section-label">
        <span>AI models</span>
        <span className="section-label-count">{MODEL_SLOTS.filter(s=>slotEnabled[s.id]).length} of {MODEL_SLOTS.length} allowed</span>
      </div>
      <ListingToggle
        on={listModels}
        onToggle={() => setListModels(v => !v)}
        granted={isUpgrade && PREV_GRANT.listModels}
        isNew={isUpgrade && listModels && !PREV_GRANT.listModels}
        label="Let the app see your full model list"
        desc="The app can see the names of all your models. It still can't use a model unless you allow it below."
      />
      {MODEL_SLOTS.map(slot => (
        <ModelSlot key={slot.id} slot={slot} enabled={slotEnabled[slot.id]} onToggle={toggleSlot}
          slotMode={slotMode[slot.id]} onSlotModeChange={mode=>setSlotMode(p=>({...p,[slot.id]:mode}))}
          selectedIds={slotModels[slot.id]||[]} onToggleModel={mid=>toggleModel(slot.id,mid)}
          onReorder={arr=>reorderModels(slot.id,arr)}
          singleValue={slotSingle[slot.id]||''} onSingleChange={v=>setSingle(slot.id,v)}
          granted={slotGranted(slot.id)} isNew={slotIsNew(slot.id)} grantedMode={slotGrantedMode(slot.id)} grantedIds={slotGrantedIds(slot.id)} />
      ))}

      <div className="form-divider"></div>

      <div className="section-label">
        <span>Connected tools</span>
        <span className="section-label-count">{enabledMcps.length} of {mcpServers.length} allowed · {readyMcps.filter(s=>s.selectedInst).length} ready</span>
      </div>
      <ListingToggle
        on={listMcps}
        onToggle={() => setListMcps(v => !v)}
        granted={isUpgrade && PREV_GRANT.listMcps}
        isNew={isUpgrade && listMcps && !PREV_GRANT.listMcps}
        label="Let the app see your full list of tools"
        desc="The app can see the names of all your connected tools. It still can't use a tool unless you allow it below."
      />

      <div className="mcp-extra">
        <div className="mcp-extra-head">
          <div className="mcp-extra-heading">
            <Icon name="plus-circle" size={15} />
            <span>Give the app extra tools</span>
          </div>
          <div className="mcp-extra-sub">Tools the app didn't ask for, but you can add. Grant only what you're comfortable sharing.</div>
        </div>
        <ModelAccessPicker
          mode={extraMcpMode}
          onModeChange={setExtraMcpMode}
          allModels={EXTRA_MCPS}
          selectedIds={extraMcps}
          onToggle={toggleExtraMcp}
          panelTitle="Add tools"
          panelSubtitle="Choose which tools to give this app"
          itemNoun="tool"
          allLabel="All tools"
          allDesc="Give access to every connected tool, including ones added later."
          specificLabel="Specific tools"
          specificDesc="Choose exactly which tools to add."
          grantedMode={isUpgrade ? 'specific' : null}
          grantedIds={isUpgrade ? PREV_GRANT.extraMcps : []}
          plainLanguage={true}
        />
      </div>

      {mcpServers.map(server => (
        <McpRow key={server.id} server={server} enabled={mcpEnabled[server.id]}
          onToggle={toggleMcp} onStateChange={updateState} onInstanceSelect={updateInst}
          isAdmin={IS_ADMIN} onShowPlaceholder={setPlaceholder}
          granted={mcpGranted(server.id)} isNew={mcpIsNew(server.id)} />
      ))}

      <div className="form-divider"></div>

      <div className="section-label" style={{marginBottom:12}}>
        <span>Approved role</span>
        {isUpgrade && (roleIsNew
          ? <span className="map-new-pill">new · was {PREV_GRANT.role==='user'?'User':'Power User'}</span>
          : <span className="map-granted-pill">✓ previously granted</span>)}
      </div>
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
          ? <div className="action-warn"><Icon name="alert-triangle" size={14}/>{blockedMcps.length} {blockedMcps.length===1?'tool needs':'tools need'} setup before approving</div>
          : isUpgrade
            ? <div className="action-ok"><Icon name="key-round" size={14}/>Approving replaces the app's current access with a new one</div>
            : <div className="action-ok"><Icon name="check-circle-2" size={14}/>Everything's ready to approve</div>
        }
      </div>
      <div className="action-btns">
        <button className="btn-deny">{isUpgrade ? 'Deny upgrade' : 'Deny & return to app'}</button>
        <button className="btn-approve" disabled={isBlocked}>
          <Icon name={isUpgrade ? 'key-round' : 'check'} size={15}/>
          {isUpgrade ? 'Approve & issue new access' : `Approve ${approveCount} item${approveCount!==1?'s':''}`}
        </button>
      </div>
    </div>
    </div>{/* ar-inner */}
    </div>{/* ar-scroll */}

    {placeholder && <PlaceholderDialog {...placeholder} onClose={()=>setPlaceholder(null)} />}
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

/* ── Demo scenario switch (review aid; sits OUTSIDE the consent card) ── */
function ScenarioSwitch({ scenario, onChange }) {
  return (
    <div className="ar-scenario-switch" role="tablist" aria-label="Demo scenario">
      <span className="ar-scenario-label">Demo</span>
      <button className={`ar-scenario-btn${scenario==='new'?' active':''}`} role="tab" aria-selected={scenario==='new'} onClick={() => onChange('new')}>New request</button>
      <button className={`ar-scenario-btn${scenario==='upgrade'?' active':''}`} role="tab" aria-selected={scenario==='upgrade'} onClick={() => onChange('upgrade')}>Upgrade</button>
    </div>
  );
}

/* ── Full page app (standalone — no shell, no breadcrumb) ── */
function AccessRequestApp() {
  const [scenario, setScenario] = useState(() => {
    const p = new URLSearchParams(window.location.search).get('mode');
    return p === 'upgrade' ? 'upgrade' : 'new';
  });
  const changeScenario = next => {
    setScenario(next);
    const url = new URL(window.location.href);
    if (next === 'upgrade') url.searchParams.set('mode', 'upgrade'); else url.searchParams.delete('mode');
    window.history.replaceState(null, '', url);
  };
  useEffect(() => { if (window.lucide) window.lucide.createIcons(); });
  return (
    <div className="std-page">
      <div className="std-topbar">
        <a className="std-brand" href="Chat.html">
          <span className="std-brand-mark"></span>
          <span className="std-brand-text">
            <span className="std-brand-word">Bodhi</span>
            <span className="std-brand-sub">AI Operating System</span>
          </span>
        </a>
        <div className="std-topbar-right">
          <ScenarioSwitch scenario={scenario} onChange={changeScenario} />
          <StdThemeToggle />
        </div>
      </div>
      <div className="std-main is-fill">
        <AccessRequestForm key={scenario} scenario={scenario} />
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
