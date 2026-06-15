/* New Model Alias — app component */

const TWEAK_DEFAULTS = /*EDITMODE-BEGIN*/{
  "theme": "light"
}/*EDITMODE-END*/;

const QUANTS = [
  { name:'Q4_K_M', size:'4.9 GB', bpw:'4.85 bpw', rec:true },
  { name:'Q3_K_S', size:'3.7 GB', bpw:'3.50 bpw', rec:false },
  { name:'Q5_K',   size:'5.7 GB', bpw:'5.69 bpw', rec:false },
  { name:'Q8_0',   size:'8.5 GB', bpw:'8.50 bpw', rec:false },
  { name:'F16',    size:'16.0 GB', bpw:'16.00 bpw', rec:false },
];

const PRESETS = [
  { id:'default', label:'Default' },
  { id:'coding', label:'Coding' },
  { id:'non-coder', label:'Non Coder' },
  { id:'smart', label:'Smart' },
  { id:'translate', label:'Translate' },
  { id:'rag', label:'RAG' },
  { id:'kwa-deep', label:'KWA Deep desc', active:true },
  { id:'show', label:'show' },
];

const RUNTIME_ARGS_LEFT = [
  { key:'--ctx-size', val:'47536' },
  { key:'--flash-attn', val:'auto' },
  { key:'--parallel', val:'10' },
  { key:'--cont-batch-n', val:'of 8' },
  { key:'--cache-type-k', val:'of 8' },
  { key:'--rope-scaling', val:'yarn' },
  { key:'--cache-prompt', val:'' },
  { key:'--grp-attn-n', val:'true' },
];

const RUNTIME_ARGS_RIGHT = [
  { key:'--n-predict', val:'by ctx_len,limit,conf' },
  { key:'--n-batch', val:'' },
  { key:'--batch-size', val:'' },
  { key:'--ubatch-size', val:'' },
  { key:'--keep', val:'' },
  { key:'--mmap-locked', val:'' },
  { key:'--split-mode', val:'' },
  { key:'--rope-scaling', val:'' },
];

const EXTRA_ARGS = [
  '--chat-template','--model-draft','--sampling-42',
  '--n-probs','--slot-save-path','--no-warmup',
  '--repeat-penalty','--min-keep','--penalty-freq',
  '--penalty-present','--mirostat','--mirostat-lr',
  '--mirostat-ent','--penalize-nl','--samplers',
  '--repeat-last-n','--association-intent',
];

const newModelStyles = {
  /* top bar */
  topBar: {
    display:'flex', alignItems:'center', gap:10,
    padding:'0 20px', height:48, flexShrink:0,
    borderBottom:'1px solid hsl(var(--border))',
    background:'hsl(var(--card))',
    position:'sticky', top:0, zIndex:50,
  },
  logoArea: { display:'flex', alignItems:'center', gap:8 },
  logoImg: { width:24, height:24 },
  breadcrumb: { display:'flex', alignItems:'center', gap:4, fontSize:12.5, color:'hsl(var(--muted-foreground))', marginLeft:8 },
  bcSep: { opacity:.4, margin:'0 2px' },
  bcCurrent: { fontWeight:700, color:'var(--c-lotus-text)' },
  spacer: { flex:1 },
  /* action buttons */
  btnCancel: {
    height:32, padding:'0 14px', borderRadius:7,
    border:'1px solid hsl(var(--border))', background:'transparent',
    fontSize:12.5, fontWeight:500, color:'hsl(var(--foreground))',
    display:'inline-flex', alignItems:'center', gap:5,
  },
  btnSave: {
    height:32, padding:'0 14px', borderRadius:7,
    border:'1px solid hsl(var(--border))', background:'hsl(var(--card))',
    fontSize:12.5, fontWeight:600, color:'hsl(var(--foreground))',
    display:'inline-flex', alignItems:'center', gap:5,
  },
  btnCreate: {
    height:32, padding:'0 16px', borderRadius:7,
    border:'none', background:'hsl(var(--primary))', color:'hsl(var(--primary-foreground))',
    fontSize:12.5, fontWeight:700,
    display:'inline-flex', alignItems:'center', gap:5,
  },
  /* page layout */
  pageWrap: { display:'flex', gap:0, minHeight:'calc(100vh - 48px)' },
  mainCol: { flex:1, minWidth:0, padding:'24px 28px 60px', maxWidth:720, overflow:'auto' },
  sideCol: {
    width:300, flexShrink:0, borderLeft:'1px solid hsl(var(--border))',
    background:'hsl(var(--card))', padding:'20px 16px', overflow:'auto',
    position:'sticky', top:48, height:'calc(100vh - 48px)',
  },
  /* page title */
  pageTitle: { fontSize:22, fontWeight:700, letterSpacing:'-.02em', marginBottom:4 },
  pageSub: { fontSize:13, color:'hsl(var(--muted-foreground))', marginBottom:20, lineHeight:'1.45' },
  /* sections nav */
  sectionsNav: {
    display:'flex', alignItems:'center', gap:4, marginBottom:24,
    fontSize:10, fontWeight:700, textTransform:'uppercase', letterSpacing:'.08em',
    color:'hsl(var(--muted-foreground))',
  },
  secPill: {
    display:'inline-flex', alignItems:'center', justifyContent:'center',
    width:22, height:22, borderRadius:'50%', fontSize:11, fontWeight:700,
    border:'1px solid hsl(var(--border))', background:'transparent',
    color:'hsl(var(--muted-foreground))', cursor:'pointer',
  },
  secPillActive: {
    background:'var(--c-lotus-bg)', borderColor:'var(--c-lotus-bd)',
    color:'var(--c-lotus-text)',
  },
  /* section */
  section: { marginBottom:28 },
  secHeader: {
    display:'flex', alignItems:'center', gap:8,
    paddingBottom:10, marginBottom:14,
    borderBottom:'1px solid hsl(var(--border))',
  },
  secNum: {
    width:22, height:22, borderRadius:'50%',
    display:'flex', alignItems:'center', justifyContent:'center',
    fontSize:11, fontWeight:700,
    background:'var(--c-lotus-bg)', color:'var(--c-lotus-text)', flexShrink:0,
  },
  secTitle: { fontSize:14, fontWeight:600 },
  secDesc: { fontSize:11.5, color:'hsl(var(--muted-foreground))', marginTop:1 },
  /* fields */
  fieldGroup: { marginBottom:14 },
  fieldLabel: { display:'flex', alignItems:'center', gap:4, fontSize:12.5, fontWeight:500, marginBottom:4 },
  req: { color:'hsl(var(--destructive))', fontSize:11 },
  fieldHint: { fontSize:11, color:'hsl(var(--muted-foreground))', marginTop:3 },
  input: {
    width:'100%', height:34, padding:'0 10px',
    border:'1px solid hsl(var(--border))', borderRadius:6,
    background:'hsl(var(--background))', color:'hsl(var(--foreground))',
    fontSize:13, outline:'none', fontFamily:'var(--font-mono)',
  },
  select: {
    width:'100%', height:34, padding:'0 10px',
    border:'1px solid hsl(var(--border))', borderRadius:6,
    background:'hsl(var(--background))', color:'hsl(var(--foreground))',
    fontSize:13, outline:'none', appearance:'none', cursor:'pointer',
    paddingRight:28,
    backgroundImage:"url(\"data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='12' height='12' viewBox='0 0 24 24' fill='none' stroke='%2371717F' stroke-width='2' stroke-linecap='round' stroke-linejoin='round'%3E%3Cpath d='m6 9 6 6 6-6'/%3E%3C/svg%3E\")",
    backgroundRepeat:'no-repeat', backgroundPosition:'right 8px center',
  },
  fieldRow: { display:'grid', gridTemplateColumns:'1fr 1fr', gap:14 },
  /* tag chips */
  tagChip: {
    display:'inline-flex', alignItems:'center', gap:3,
    padding:'2px 8px', borderRadius:5,
    fontSize:11, fontWeight:500, border:'1px solid hsl(var(--border))',
    background:'hsl(var(--muted))', color:'hsl(var(--muted-foreground))',
    cursor:'pointer',
  },
  tagChipActive: {
    background:'var(--c-lotus-bg)', borderColor:'var(--c-lotus-bd)',
    color:'var(--c-lotus-text)', fontWeight:600,
  },
  /* quant table */
  tableWrap: { border:'1px solid hsl(var(--border))', borderRadius:6, overflow:'hidden' },
  table: { width:'100%', borderCollapse:'collapse', fontSize:12.5 },
  th: {
    textAlign:'left', padding:'6px 10px',
    fontSize:10, fontWeight:700, textTransform:'uppercase',
    letterSpacing:'.07em', color:'hsl(var(--muted-foreground))',
    background:'hsl(var(--surface-2))', borderBottom:'1px solid hsl(var(--border))',
  },
  td: { padding:'6px 10px', borderBottom:'1px solid hsl(var(--border))', verticalAlign:'middle' },
  trSelected: { background:'var(--c-lotus-bg)' },
  qradio: {
    width:14, height:14, borderRadius:'50%',
    border:'2px solid hsl(var(--border))', background:'transparent',
    display:'flex', alignItems:'center', justifyContent:'center',
  },
  qradioOn: { borderColor:'var(--c-lotus-text)' },
  qradioDot: { width:6, height:6, borderRadius:'50%', background:'var(--c-lotus-text)' },
  recBadge: {
    fontSize:9, fontWeight:700, padding:'1px 5px', borderRadius:4,
    background:'var(--c-leaf-bg)', color:'var(--c-leaf-text)',
    border:'1px solid var(--c-leaf-bd)',
  },
  mono: { fontFamily:'var(--font-mono)' },
  /* file status banner */
  fileBanner: {
    display:'flex', alignItems:'center', gap:8,
    padding:'8px 12px', borderRadius:6, marginTop:10,
    background:'var(--c-leaf-bg)', border:'1px solid var(--c-leaf-bd)',
    fontSize:12, color:'var(--c-leaf-text)',
  },
  pullBtn: {
    marginLeft:'auto', height:26, padding:'0 10px', borderRadius:5,
    border:'1px solid var(--c-leaf-bd)', background:'transparent',
    fontSize:11, fontWeight:600, color:'var(--c-leaf-text)', flexShrink:0,
  },
  /* preset pills */
  presetRow: { display:'flex', flexWrap:'wrap', gap:5, marginBottom:10 },
  presetPill: {
    display:'inline-flex', alignItems:'center', gap:4,
    padding:'4px 10px', borderRadius:99, fontSize:11.5, fontWeight:500,
    border:'1px solid hsl(var(--border))', background:'transparent',
    color:'hsl(var(--muted-foreground))', cursor:'pointer',
    transition:'all 100ms',
  },
  presetPillActive: {
    background:'var(--c-saffron-bg)', borderColor:'var(--c-saffron-bd)',
    color:'var(--c-saffron-text)', fontWeight:600,
  },
  /* capability tags */
  capRow: { display:'flex', flexWrap:'wrap', gap:4, marginBottom:10 },
  capTag: {
    padding:'2px 7px', borderRadius:5, fontSize:10.5, fontWeight:500,
    border:'1px solid hsl(var(--border))', background:'transparent',
    color:'hsl(var(--muted-foreground))', cursor:'pointer',
  },
  /* runtime args code area */
  runtimeBar: {
    display:'flex', alignItems:'center', gap:6, marginBottom:6,
    fontSize:11, color:'hsl(var(--muted-foreground))',
  },
  runtimeBarBtn: {
    height:24, padding:'0 8px', borderRadius:5,
    border:'1px solid hsl(var(--border))', background:'hsl(var(--card))',
    fontSize:10.5, fontWeight:600, color:'hsl(var(--foreground))',
    display:'inline-flex', alignItems:'center', gap:3, cursor:'pointer',
  },
  presetLabel: {
    marginLeft:'auto', fontSize:10.5, fontWeight:600,
    padding:'2px 8px', borderRadius:5,
    background:'var(--c-saffron-bg)', border:'1px solid var(--c-saffron-bd)',
    color:'var(--c-saffron-text)',
  },
  codeArea: {
    display:'grid', gridTemplateColumns:'1fr 1fr', gap:0,
    border:'1px solid hsl(var(--border))', borderRadius:6,
    fontFamily:'var(--font-mono)', fontSize:11.5, lineHeight:'1.7',
    overflow:'hidden',
  },
  codeCol: {
    padding:'10px 12px',
  },
  codeColRight: {
    padding:'10px 12px',
    borderLeft:'1px solid hsl(var(--border))',
  },
  codeLine: { display:'flex', gap:6 },
  codeKey: { color:'hsl(var(--muted-foreground))', whiteSpace:'nowrap' },
  codeVal: { fontWeight:500 },
  codeHighlight: {
    background:'var(--c-saffron-bg)', margin:'0 -12px', padding:'0 12px',
    borderRadius:0,
  },
  /* runtime stats */
  runtimeStats: {
    display:'flex', gap:16, padding:'8px 0', marginTop:6,
    fontSize:11, color:'hsl(var(--muted-foreground))',
  },
  statItem: { display:'flex', flexDirection:'column', gap:1 },
  statLabel: { fontSize:9, fontWeight:700, textTransform:'uppercase', letterSpacing:'.06em' },
  statVal: { fontFamily:'var(--font-mono)', fontSize:12, fontWeight:600, color:'hsl(var(--foreground))' },
  /* green tip banner */
  tipBanner: {
    display:'flex', alignItems:'flex-start', gap:6,
    padding:'8px 12px', borderRadius:6, marginTop:8,
    background:'var(--c-leaf-bg)', border:'1px solid var(--c-leaf-bd)',
    fontSize:11.5, color:'var(--c-leaf-text)',
  },
  /* request params grid */
  paramsGrid: { display:'grid', gridTemplateColumns:'1fr 1fr', gap:10 },
  paramField: {},
  paramLabel: { fontSize:11, fontWeight:500, color:'hsl(var(--muted-foreground))', marginBottom:3 },
  paramInput: {
    width:'100%', height:32, padding:'0 8px',
    border:'1px solid hsl(var(--border))', borderRadius:5,
    background:'hsl(var(--background))', color:'hsl(var(--foreground))',
    fontFamily:'var(--font-mono)', fontSize:12, outline:'none',
  },
  paramTextarea: {
    width:'100%', padding:'8px', minHeight:60, resize:'vertical',
    border:'1px solid hsl(var(--border))', borderRadius:5,
    background:'hsl(var(--background))', color:'hsl(var(--foreground))',
    fontFamily:'var(--font-mono)', fontSize:12, outline:'none', lineHeight:'1.5',
  },
  /* sidebar cards */
  sideCard: {
    border:'1px solid hsl(var(--border))', borderRadius:8,
    overflow:'hidden', marginBottom:16,
  },
  sideCardHead: {
    padding:'8px 12px', background:'hsl(var(--surface-2))',
    borderBottom:'1px solid hsl(var(--border))',
    fontSize:10, fontWeight:700, textTransform:'uppercase',
    letterSpacing:'.07em', color:'hsl(var(--muted-foreground))',
    display:'flex', alignItems:'center', gap:5,
  },
  sideCardBody: { padding:'10px 12px' },
  sideStatRow: {
    display:'flex', justifyContent:'space-between', padding:'4px 0',
    borderBottom:'1px solid hsl(var(--border))', fontSize:11.5,
  },
  sideStatK: { color:'hsl(var(--muted-foreground))' },
  sideStatV: { fontWeight:600, fontFamily:'var(--font-mono)', fontSize:11.5 },
  /* live preview */
  livePreview: {
    fontFamily:'var(--font-mono)', fontSize:10.5, lineHeight:'1.6',
    padding:'10px 12px', background:'hsl(var(--surface-3))',
    borderRadius:6, border:'1px solid hsl(var(--border))',
    maxHeight:260, overflow:'auto', whiteSpace:'pre-wrap', wordBreak:'break-all',
  },
  /* tips */
  tipItem: {
    fontSize:11.5, color:'hsl(var(--muted-foreground))',
    padding:'4px 0', lineHeight:'1.45',
  },
  tipBullet: { color:'var(--c-lotus-text)', marginRight:4 },
};

function Icon({name, size=14, style={}}) {
  const ref = React.useRef(null);
  React.useEffect(() => {
    if (ref.current) {
      ref.current.innerHTML = '';
      const el = document.createElement('i');
      el.setAttribute('data-lucide', name);
      ref.current.appendChild(el);
      lucide.createIcons({nodes: [el]});
    }
  }, [name]);
  return <span ref={ref} style={{display:'inline-flex',width:size,height:size,alignItems:'center',justifyContent:'center',...style}}></span>;
}

function App() {
  const [tweaks, setTweak] = useTweaks(TWEAK_DEFAULTS);
  const s = newModelStyles;

  // Form state
  const [aliasName, setAliasName] = React.useState('qwen-chn');
  const [tags, setTags] = React.useState(['text','inst ver','I18n']);
  const [repo, setRepo] = React.useState('Qwen/Qwen3-8B-GGUF');
  const [snapshot, setSnapshot] = React.useState('(GGUF_PREVIEW)');
  const [selectedQuant, setSelectedQuant] = React.useState(0); // Q4_K_M
  const [activePreset, setActivePreset] = React.useState('kwa-deep');
  const [customChecked, setCustomChecked] = React.useState(false);

  // Request params
  const [reqParams, setReqParams] = React.useState({
    temperature:'0.7', top_p:'disti',
    max_tokens:'', seed:'',
    frequency_penalty:'', presence_penalty:'',
    n:'1', stop:'0',
    top:'', sort:'',
    response_format:'', tool_choice_default:'',
    suffix:'auto', code:'code',
  });
  const [systemPrompt, setSystemPrompt] = React.useState('You are a helpful assistant.');

  React.useEffect(() => { document.documentElement.setAttribute('data-theme', tweaks.theme); }, [tweaks.theme]);

  const updateParam = (key, val) => setReqParams(prev => ({...prev, [key]:val}));

  const selectedFile = QUANTS[selectedQuant];
  const filePath = `Qwen/Qwen3-8B-GGUF/${selectedFile.name}/Qwen3-8B-${selectedFile.name}.gguf`;

  // Config JSON for preview
  const configJson = {
    alias: aliasName,
    repo: repo,
    snapshot: snapshot,
    file: `Qwen3-8B-${selectedFile.name}.gguf`,
    preset: activePreset,
    ctx_size: 47536,
    flash_attn: 'auto',
    parallel: 10,
    ...Object.fromEntries(Object.entries(reqParams).filter(([_,v]) => v)),
    system_prompt: systemPrompt || undefined,
  };

  return (
    <div>
      {/* Top bar */}
      <header style={s.topBar}>
        <div style={s.logoArea}>
          <img src="assets/bodhi-logo-60.svg" alt="Bodhi" style={s.logoImg} />
        </div>
        <nav style={s.breadcrumb}>
          <a href="Bodhi Models.html" style={{cursor:'pointer'}}>Bodhi</a>
          <span style={s.bcSep}>&gt;</span>
          <a href="Bodhi Models.html" style={{cursor:'pointer'}}>Models</a>
          <span style={s.bcSep}>&gt;</span>
          <span style={s.bcCurrent}>New alias</span>
        </nav>
        <div style={s.spacer}></div>
        <div style={{display:'flex', gap:6}}>
          <button style={s.btnCancel}>Cancel</button>
          <button style={s.btnSave}>Save &amp; test</button>
          <button style={s.btnCreate}>Create alias</button>
        </div>
      </header>

      <div style={s.pageWrap}>
        {/* ════ MAIN COLUMN ════ */}
        <div style={s.mainCol}>
          <h1 style={s.pageTitle}>New model alias</h1>
          <p style={s.pageSub}>
            Runtime arg flags are read defaults with llama.cpp release. Openai request defaults are a final form at safe alias.
          </p>

          {/* Sections nav */}
          <div style={s.sectionsNav}>
            <span style={{marginRight:6}}>SECTIONS</span>
            {[1,2,3,4,5].map(n => (
              <span key={n} style={{...s.secPill, ...(n <= 4 ? s.secPillActive : {})}}>{n}</span>
            ))}
          </div>

          {/* ═══ 1. Identity ═══ */}
          <div style={s.section}>
            <div style={s.secHeader}>
              <div style={s.secNum}>1</div>
              <div>
                <div style={s.secTitle}>Identity</div>
              </div>
            </div>
            <div style={s.fieldGroup}>
              <label style={s.fieldLabel}>Alias name <span style={s.req}>*</span></label>
              <input style={s.input} value={aliasName} onChange={e => setAliasName(e.target.value)} />
              <div style={{display:'flex', gap:4, marginTop:6}}>
                {tags.map((t, i) => (
                  <span key={i} style={{...s.tagChip, ...s.tagChipActive}}>{t}</span>
                ))}
              </div>
              <div style={s.fieldHint}>Alias names can only contain lowercase, digits, and dashes.</div>
            </div>
          </div>

          {/* ═══ 2. Model file ═══ */}
          <div style={s.section}>
            <div style={s.secHeader}>
              <div style={s.secNum}>2</div>
              <div><div style={s.secTitle}>Model file</div></div>
            </div>
            <div style={{...s.fieldRow, marginBottom:14}}>
              <div style={{...s.fieldGroup, marginBottom:0}}>
                <label style={s.fieldLabel}>Repo</label>
                <select style={s.select} value={repo} onChange={e => setRepo(e.target.value)}>
                  <option>Qwen/Qwen3-8B-GGUF</option>
                </select>
              </div>
              <div style={{...s.fieldGroup, marginBottom:0}}>
                <label style={s.fieldLabel}>Snapshot</label>
                <select style={s.select} value={snapshot} onChange={e => setSnapshot(e.target.value)}>
                  <option>(GGUF_PREVIEW)</option>
                  <option>main</option>
                </select>
              </div>
            </div>

            <label style={{...s.fieldLabel, marginBottom:6, textTransform:'uppercase', fontSize:10, fontWeight:700, letterSpacing:'.07em', color:'hsl(var(--muted-foreground))'}}>
              Quant — picks the file
            </label>
            <div style={s.tableWrap}>
              <table style={s.table}>
                <thead>
                  <tr>
                    <th style={{...s.th, width:30}}></th>
                    <th style={s.th}>Quant</th>
                    <th style={s.th}>Size</th>
                    <th style={s.th}>BPW</th>
                    <th style={{...s.th, width:36}}></th>
                  </tr>
                </thead>
                <tbody>
                  {QUANTS.map((q, i) => (
                    <tr key={i} style={{...(selectedQuant === i ? s.trSelected : {}), cursor:'pointer', transition:'background 80ms'}}
                        onClick={() => setSelectedQuant(i)}
                        onMouseEnter={e => { if(selectedQuant !== i) e.currentTarget.style.background = 'hsl(var(--muted))'; }}
                        onMouseLeave={e => { if(selectedQuant !== i) e.currentTarget.style.background = ''; }}>
                      <td style={s.td}>
                        <div style={{...s.qradio, ...(selectedQuant===i ? s.qradioOn : {})}}>
                          {selectedQuant===i && <div style={s.qradioDot}></div>}
                        </div>
                      </td>
                      <td style={{...s.td, ...s.mono, fontWeight:selectedQuant===i?600:400}}>{q.name}</td>
                      <td style={{...s.td, ...s.mono}}>{q.size}</td>
                      <td style={{...s.td, ...s.mono, color:'hsl(var(--muted-foreground))'}}>{q.bpw}</td>
                      <td style={s.td}>{q.rec && <span style={s.recBadge}>recommended</span>}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
            <div style={s.fieldHint}>
              Automatic quant sets the filename — e.g. <span style={s.mono}>Q4_K_M</span> → <span style={s.mono}>Qwen3-8B-Q4_K_M.gguf</span>. The symantic filename mapping.
            </div>

            {/* File status banner */}
            <div style={s.fileBanner}>
              <Icon name="check-circle" size={14} />
              <div>
                <strong>File set local — will download on save</strong>
                <div style={{...s.mono, fontSize:11, marginTop:2, opacity:.8}}>{filePath} · {selectedFile.size} · gguf</div>
              </div>
              <button style={s.pullBtn}>Pull now</button>
            </div>
          </div>

          {/* ═══ 3. Preset & Runtime args ═══ */}
          <div style={s.section}>
            <div style={s.secHeader}>
              <div style={s.secNum}>3</div>
              <div>
                <div style={s.secTitle}>Preset &amp; Runtime args</div>
                <div style={s.secDesc}>Pick a preset to fill in the runtime arg below. Expand the editor to hand-set flags. Also other tags too to attach within default known releases.</div>
              </div>
            </div>

            {/* Preset pills */}
            <div style={s.presetRow}>
              {PRESETS.map(p => (
                <button key={p.id}
                  style={{...s.presetPill, ...(activePreset===p.id ? s.presetPillActive : {})}}
                  onClick={() => setActivePreset(p.id)}>
                  {p.id === activePreset && <Icon name="check" size={11} />}
                  {p.label}
                </button>
              ))}
            </div>

            {/* Capability tags */}
            <div style={s.capRow}>
              {['Top Performance (listed)','Non Context','Family','Modern','Is Ready','May','Is Firmware ver','Machine','Hardware ultra','Easy-go','Search & Set'].map((c,i) => (
                <span key={i} style={s.capTag}>{c}</span>
              ))}
            </div>

            {/* Custom checkbox */}
            <label style={{display:'flex', alignItems:'center', gap:6, fontSize:12, marginBottom:12, cursor:'pointer'}}>
              <input type="checkbox" checked={customChecked} onChange={e => setCustomChecked(e.target.checked)} style={{accentColor:'var(--c-lotus-text)'}} />
              <span>custom</span>
            </label>

            {/* Runtime info line */}
            <div style={{fontSize:11.5, color:'hsl(var(--muted-foreground))', marginBottom:8}}>
              Runtime: <span style={s.mono}>llama server args</span> — 7 days from RAK (long desc)
            </div>

            {/* Action bar */}
            <div style={s.runtimeBar}>
              <button style={s.runtimeBarBtn}><Icon name="clipboard-paste" size={10} /> Paste command</button>
              <button style={s.runtimeBarBtn}><Icon name="copy" size={10} /> Copy</button>
              <button style={s.runtimeBarBtn}><Icon name="file-text" size={10} /> Raw yaml</button>
              <span style={s.presetLabel}>preset: KWA Deep desc</span>
            </div>

            {/* Two-column runtime args */}
            <div style={s.codeArea}>
              <div style={s.codeCol}>
                {RUNTIME_ARGS_LEFT.map((a, i) => (
                  <div key={i} style={{...s.codeLine, ...(i===2 ? s.codeHighlight : {})}}>
                    <span style={s.codeKey}>{a.key}</span>
                    <span style={s.codeVal}>{a.val}</span>
                  </div>
                ))}
                <div style={{height:8}}></div>
                {EXTRA_ARGS.map((a, i) => (
                  <div key={i} style={s.codeLine}>
                    <span style={s.codeKey}>{a}</span>
                  </div>
                ))}
              </div>
              <div style={s.codeColRight}>
                {RUNTIME_ARGS_RIGHT.map((a, i) => (
                  <div key={i} style={s.codeLine}>
                    <span style={s.codeKey}>{a.key}</span>
                    {a.val && <span style={s.codeVal}>{a.val}</span>}
                  </div>
                ))}
                <div style={{height:8}}></div>
                <div style={s.codeLine}><span style={s.codeKey}>SAMPLING_42</span></div>
              </div>
            </div>

            {/* Runtime stats */}
            <div style={s.runtimeStats}>
              <div style={s.statItem}><span style={s.statLabel}>runs</span><span style={s.statVal}>--parallel (cur) 10</span></div>
              <div style={s.statItem}><span style={s.statLabel}>disc</span><span style={s.statVal}>number of active file</span></div>
            </div>
            <div style={{...s.runtimeStats, marginTop:0}}>
              <div style={s.statItem}><span style={s.statLabel}>disk</span><span style={s.statVal}>4.9 GB</span></div>
              <div style={s.statItem}><span style={s.statLabel}>VRAM</span><span style={s.statVal}>~11 GB @fp16 ctx</span></div>
            </div>

            <div style={s.tipBanner}>
              <Icon name="target" size={13} />
              <span>Target: is a hardware flag CLI — device model capacity (T / L unwinded).</span>
            </div>
          </div>

          {/* ═══ 4. Request Defaults — OpenAI compat ═══ */}
          <div style={s.section}>
            <div style={s.secHeader}>
              <div style={s.secNum}>4</div>
              <div>
                <div style={s.secTitle}>Request Defaults — OpenAI compat</div>
                <div style={s.secDesc}>Applied as defaults to every chat request — overridable per call. Stable OpenAI schema, as a final form at safe alias.</div>
              </div>
            </div>

            <div style={s.paramsGrid}>
              {[
                ['temperature', reqParams.temperature],
                ['top_p', reqParams.top_p],
                ['max_tokens', reqParams.max_tokens],
                ['seed', reqParams.seed],
                ['frequency_penalty', reqParams.frequency_penalty],
                ['presence_penalty', reqParams.presence_penalty],
                ['n', reqParams.n],
                ['stop', reqParams.stop],
                ['top', reqParams.top],
                ['sort', reqParams.sort],
                ['response_format', reqParams.response_format],
                ['tool_choice_default', reqParams.tool_choice_default],
                ['suffix', reqParams.suffix],
                ['code', reqParams.code],
              ].map(([key, val]) => (
                <div key={key} style={s.paramField}>
                  <div style={s.paramLabel}>{key}</div>
                  <input style={s.paramInput} value={val} onChange={e => updateParam(key, e.target.value)} />
                </div>
              ))}
            </div>

            <div style={{marginTop:14}}>
              <div style={s.paramLabel}>system_prompt</div>
              <textarea style={s.paramTextarea} value={systemPrompt} onChange={e => setSystemPrompt(e.target.value)}></textarea>
            </div>
          </div>
        </div>

        {/* ════ RIGHT SIDEBAR ════ */}
        <div style={s.sideCol}>
          {/* File info card */}
          <div style={s.sideCard}>
            <div style={s.sideCardHead}>
              <Icon name="check-circle" size={12} style={{color:'var(--c-leaf-text)'}} />
              File with {selectedFile.size} Bandwidth
            </div>
            <div style={s.sideCardBody}>
              <div style={s.sideStatRow}><span style={s.sideStatK}>RAM</span><span style={s.sideStatV}>5.6 / 32 GB · 2.12 Pres</span></div>
              <div style={s.sideStatRow}><span style={s.sideStatK}>est ctx</span><span style={s.sideStatV}>~19K (FULL_CTX of ctx)</span></div>
              <div style={s.sideStatRow}><span style={s.sideStatK}>gpu offload</span><span style={s.sideStatV}>✓ 100% w/llama.cpp</span></div>
              <div style={s.sideStatRow}><span style={s.sideStatK}>downloads</span><span style={s.sideStatV}>~obsolete via 'Save & test' in session</span></div>
              <div style={{...s.sideStatRow, borderBottom:'none'}}><span style={s.sideStatK}>first</span><span style={s.sideStatV}>~67 tk to gen eq · Right IO 70cc</span></div>
            </div>
          </div>

          {/* Live config preview */}
          <div style={{marginBottom:16}}>
            <div style={{fontSize:10, fontWeight:700, textTransform:'uppercase', letterSpacing:'.07em', color:'hsl(var(--muted-foreground))', marginBottom:6}}>
              Live Config Preview
            </div>
            <div style={s.livePreview}>
              {JSON.stringify(configJson, null, 2)}
            </div>
          </div>

          {/* Tips */}
          <div style={s.sideCard}>
            <div style={s.sideCardHead}>Tips</div>
            <div style={s.sideCardBody}>
              <div style={s.tipItem}><span style={s.tipBullet}>›</span> Preset args and Runtime args — you can break any for a new</div>
              <div style={s.tipItem}><span style={s.tipBullet}>›</span> Value is bold via form <code style={{fontSize:10, padding:'1px 3px', background:'hsl(var(--muted))', borderRadius:3}}>llama-server</code> — it's like, to</div>
              <div style={s.tipItem}><span style={s.tipBullet}>›</span> Esp. sleep or for empty</div>
              <div style={s.tipItem}><span style={s.tipBullet}>›</span> —model instead flag —directly block save</div>
              <div style={s.tipItem}><span style={s.tipBullet}>›</span> Paste command accepts a full llama-server run</div>
            </div>
          </div>
        </div>
      </div>

      <TweaksPanel>
        <TweakSection title="Theme">
          <TweakRadio value={tweaks.theme} options={[{label:'Light',value:'light'},{label:'Dark',value:'dark'}]} onChange={v => setTweak('theme', v)} />
        </TweakSection>
      </TweaksPanel>
    </div>
  );
}

const root = ReactDOM.createRoot(document.getElementById('root'));
root.render(<App />);
