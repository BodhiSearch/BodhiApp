// Shared wireframe primitives — exposed as window globals for other babel scripts.

const Ph = ({w='70%', h=7, style}) => (
  <div className="ph line" style={{width:w, height:h, ...style}} />
);

const Lines = ({rows=[60,80,40]}) => (
  <div style={{display:'flex', flexDirection:'column', gap:5}}>
    {rows.map((w,i) => <div key={i} className="ph line" style={{width: w+'%'}}/>) }
  </div>
);

const Chip = ({on, tone, children, style, onClick}) => (
  <span className={`chip${on?' on':''}${tone?' '+tone:''}`} style={{...(onClick?{cursor:'pointer'}:{}), ...style}} onClick={onClick}>{children}</span>
);

const Btn = ({variant='', size='', children, style, title, onClick}) => (
  <button className={`btn ${variant} ${size}`} style={style} title={title} onClick={onClick}>{children}</button>
);

const Field = ({label, value, hint, filled, ta, style, right}) => (
  <div style={style}>
    {label && <div className="lbl" style={{marginBottom:3}}>{label}</div>}
    <div className={`field ${filled?'filled':''} ${ta?'ta':''}`}>
      <span>{value || <span style={{color:'var(--ink-4)'}}>{hint || 'placeholder'}</span>}</span>
      {right}
    </div>
  </div>
);

const TL = ({tone='green', children}) => <span className={`tl ${tone}`}>{children}</span>;

const Stars = ({n=4.5}) => {
  const full = Math.floor(n), half = n - full >= 0.5;
  return <span className="stars">{'★'.repeat(full)}{half?'½':''}{'☆'.repeat(5-full-(half?1:0))}</span>;
};

const Bar = ({pct=60, tone='leaf'}) => (
  <div className="bar"><span style={{width: pct+'%', background: `var(--${tone}-soft, var(--leaf-soft))`}}/></div>
);

const Crumbs = ({items=['Bodhi','Models']}) => (
  <div className="crumbs">
    {items.map((it,i)=>(
      <React.Fragment key={i}>
        <span className={i===0?'c-home':''}>{it}</span>
        {i<items.length-1 && <span>›</span>}
      </React.Fragment>
    ))}
  </div>
);

const Browser = ({url='bodhi.local/models', children, style}) => (
  <div className="wf-browser" style={style}>
    <div className="wf-browser-bar">
      <span className="wf-dot r"/><span className="wf-dot y"/><span className="wf-dot g"/>
      <span style={{marginLeft:8}}>{url}</span>
    </div>
    <div className="wf-body">{children}</div>
  </div>
);

const Variant = ({label, tag, note, novel, className='', children}) => (
  <section className={`variant ${className}`}>
    <div className="variant-head">
      <span className="variant-label">{label}</span>
      {tag && <span className="variant-tag">{tag}</span>}
    </div>
    {note && <p className="variant-note">{note}</p>}
    {novel && <div style={{margin:'0 0 10px'}}><span className="novel">★ novel: {novel}</span></div>}
    {children}
  </section>
);

const Callout = ({style, children}) => (
  <div className="callout" style={style}>{children}</div>
);

const SectionHead = ({n, title, concept}) => (
  <div style={{display:'flex', alignItems:'baseline', gap:14, marginBottom:6, flexWrap:'wrap'}}>
    <span style={{fontFamily:'var(--hand)', fontSize:18, fontWeight:700, color:'var(--lotus)', border:'1.5px solid var(--ink)', borderRadius:'50%', width:36, height:36, display:'inline-flex', alignItems:'center', justifyContent:'center', background:'var(--lotus-soft)'}}>{n}</span>
    <h2 className="page-title">{title}</h2>
    {concept && <span className="concept">— {concept}</span>}
  </div>
);

// Small model row used across multiple screens
const ModelRow = ({name='Qwen3.5-9B', org='Qwen', task='Text→Text', size='9B', dl='443k', likes='415', fit='green', fitLabel='~38 tok/s', children, highlight}) => (
  <div className="card row" style={{background: highlight?'var(--lotus-soft)':'#fff'}}>
    <div className="ph thumb" style={{background:'linear-gradient(135deg,#ffd6e0,#d4d8f4)'}} />
    <div style={{flex:1, minWidth:0}}>
      <div style={{display:'flex', gap:6, alignItems:'center', flexWrap:'wrap'}}>
        <span className="h2" style={{margin:0}}>{org}/{name}</span>
        <Chip tone="leaf">{size}</Chip>
        <Chip>{task}</Chip>
      </div>
      <div style={{display:'flex', gap:10, alignItems:'center', flexWrap:'wrap', marginTop:3}}>
        <span className="sm">↓ {dl}</span>
        <span className="sm">♥ {likes}</span>
        <TL tone={fit}>{fitLabel}</TL>
        {children}
      </div>
    </div>
  </div>
);

// Shared Downloads panel — used as the right-pane detail view on Hub + Discover
// when the user clicks the "↓ Downloads" menu entry.
const DownloadsPanel = () => (
  <>
    <div className="right-collapsed-rail">downloads · 1 active</div>
    <div className="right-topbar">
      <div style={{display:'flex', alignItems:'center', gap:6, flexWrap:'wrap'}}>
        <Chip tone="saff" style={{fontSize:10}}>downloads</Chip>
        <span className="h2" style={{margin:0}}>Downloads</span>
        <Chip tone="saff">● 1 active</Chip>
      </div>
      <Btn variant="ghost" size="xs" title="collapse">→</Btn>
    </div>
    <div className="sm">8 files · 94 GB on disk · last pull 12m ago</div>

    <div style={{display:'flex', gap:4, marginTop:8, flexWrap:'wrap'}}>
      <Chip on>All</Chip><Chip>Downloading (1)</Chip><Chip>Ready (7)</Chip><Chip>Failed</Chip>
    </div>

    <div className="h3">In progress</div>
    <div className="card" style={{padding:'8px 10px'}}>
      <div style={{display:'flex', alignItems:'center', gap:6, flexWrap:'wrap', marginBottom:4}}>
        <Chip tone="saff" style={{fontSize:10}}>downloading</Chip>
        <code style={{flex:1}}>unsloth/gemma-4-e4b-it:Q4_K_M</code>
        <span className="sm">2.40 / 3.99 GB</span>
      </div>
      <Bar pct={60}/>
      <div style={{display:'flex', justifyContent:'space-between', alignItems:'center', marginTop:4}}>
        <span className="sm">12 MB/s · eta 2m · started 1m ago</span>
        <div style={{display:'flex', gap:4}}>
          <Btn size="xs" title="pause">⏸</Btn>
          <Btn size="xs" title="cancel">✕</Btn>
        </div>
      </div>
    </div>

    <div className="h3">Recently completed</div>
    <div style={{display:'flex', flexDirection:'column', gap:4}}>
      <div className="card row" style={{padding:'5px 7px'}}>
        <Chip tone="leaf" style={{fontSize:10}}>ready</Chip>
        <code style={{flex:1}}>google/gemma-2-9b:Q4_K_M</code>
        <span className="sm">5.4 GB · 12m ago</span>
        <Btn variant="ghost" size="xs">open →</Btn>
      </div>
      <div className="card row" style={{padding:'5px 7px'}}>
        <Chip tone="leaf" style={{fontSize:10}}>ready</Chip>
        <code style={{flex:1}}>qwen/qwen3-14b:Q5_K_M</code>
        <span className="sm">10.1 GB · 2h ago</span>
        <Btn variant="ghost" size="xs">open →</Btn>
      </div>
      <div className="card row" style={{padding:'5px 7px'}}>
        <Chip tone="leaf" style={{fontSize:10}}>ready</Chip>
        <code style={{flex:1}}>LiquidAI/LFM2.5-1.2B:Q8_0</code>
        <span className="sm">1.3 GB · yesterday</span>
        <Btn variant="ghost" size="xs">open →</Btn>
      </div>
      <div className="card row" style={{padding:'5px 7px'}}>
        <Chip tone="warn" style={{fontSize:10}}>failed</Chip>
        <code style={{flex:1}}>meta/Llama-3.3-70B:Q4_K_M</code>
        <span className="sm">disk full · 2d ago</span>
        <Btn size="xs">retry</Btn>
      </div>
    </div>

    <div className="h3">Disk usage</div>
    <div style={{display:'grid', gridTemplateColumns:'auto 1fr', columnGap:8, rowGap:3}}>
      <span className="sm">models dir</span><span className="sm"><code>~/.bodhi/models</code></span>
      <span className="sm">on disk</span><span className="sm">94 GB / 1 TB <Bar pct={9}/></span>
      <span className="sm">cached</span><span className="sm">8 files · 2 broken cleanup candidates</span>
    </div>

    <div style={{display:'flex', gap:4, marginTop:10, flexWrap:'wrap'}}>
      <Btn variant="primary" size="xs">+ Pull from URL…</Btn>
      <Btn variant="ghost" size="xs">Open models folder ↗</Btn>
      <Btn variant="ghost" size="xs">Clear history</Btn>
    </div>
  </>
);

// Tiny sidebar row used on Hub + Discover to open the DownloadsPanel.
// Shows a live badge when something is in progress.
const DownloadsMenu = ({active, count=1, onClick}) => (
  <div className={`downloads-menu${active?' active':''}${count>0?' live':''}`} onClick={onClick}>
    <span className="downloads-menu-icon">↓</span>
    <span style={{flex:1}}>Downloads</span>
    {count > 0 && <span className="downloads-menu-badge">{count} ↓</span>}
  </div>
);

// Generic list-view row used by the unified Models page in list mode.
// v25: accepts optional duality props for the unified stream:
//   · localBadge (tag alias/file/api-model as 'local' when mixed in All mode)
//   · backlink (↗ catalog link on file-first rows)
//   · catalogAliases ({count, onClick}) for hf-repo rows with local aliases
//   · directoryAttribution (show 'from api.getbodhi.app' on unconnected providers)
const ModelListRow = ({kind='file', title, subtitle, caps=[], meta, cost, status, fitLabel, fit, selected, onClick,
                        localBadge, backlink, catalogAliases, directoryAttribution}) => {
  const kindTone =
    kind==='alias' ? 'saff' :
    kind==='file' ? 'leaf' :
    kind==='api-model' ? 'indigo' :
    kind==='provider' ? 'indigo' :
    kind==='provider-off' ? '' :
    kind==='hf-repo' ? 'leaf' : 'leaf';
  const statusTone =
    status==='live' || status==='ready' || status==='fits' || status==='connected' ? 'leaf' :
    status==='oauth' ? 'saff' :
    status==='rate-limited' || status==='tight' ? 'warn' : '';
  const fitTone = fit==='green' ? 'leaf' : fit==='yellow' ? 'warn' : fit==='red' ? 'warn' : '';
  const extra = kind==='provider-off' ? ' dashed' : '';
  const kindLabel = kind==='provider-off' ? 'provider' : kind;
  return (
    <div className={`model-list-row${selected?' selected':''}${extra}`} onClick={onClick}>
      <Chip tone={kindTone} style={{fontSize:10}}>{kindLabel}</Chip>
      <div className="mlr-title-cell">
        <div className="model-card-title" style={{fontSize:13, margin:0}}>
          {title}
          {localBadge && <span className="row-local-badge">local</span>}
        </div>
        {subtitle && <div className="sm">{subtitle}</div>}
        {backlink && <div><span className="row-backlink">↗ {backlink}</span></div>}
        {directoryAttribution && (
          <div className="row-directory-attribution">from Bodhi directory · <code>api.getbodhi.app</code></div>
        )}
      </div>
      <div className="mlr-caps-cell">
        {caps.map((c,i)=>(<Chip key={i}>{c}</Chip>))}
      </div>
      <div className="mlr-meta-cell sm">
        {cost && <div className="mlr-cost">{cost}</div>}
        {meta && <div>{meta}</div>}
        {catalogAliases && catalogAliases.count>0 && (
          <div><span className="row-catalog-aliases-badge">✓ {catalogAliases.count} local aliases ↗</span></div>
        )}
      </div>
      <div className="mlr-status-cell">
        {status && <Chip tone={statusTone}>● {status}</Chip>}
        {fitLabel && <Chip tone={fitTone}>● {fitLabel}</Chip>}
      </div>
    </div>
  );
};

// Breadcrumb-style header for mobile + medium (replaces the old ☰-brand topbar).
// The whole path is tappable and opens MobileMenu.
const MobileHeader = ({active='My Models', dlCount=1, rightSlot}) => (
  <div className="m-bc-header">
    <div className="m-bc-path">
      <span>Bodhi</span>
      <span className="m-bc-sep">›</span>
      <span>Models</span>
      <span className="m-bc-sep">›</span>
      <span className="m-bc-active">{active}</span>
      <span className="m-bc-caret">▾</span>
    </div>
    {rightSlot || (
      <span className={`m-ico m-ico-dl${dlCount>0?' live':''}`} title="downloads">
        ↓{dlCount>0 && <span className="m-dl-badge">{dlCount}</span>}
      </span>
    )}
  </div>
);

// Nested app menu that drops from tapping the breadcrumb.
// v25: Models is a single leaf — no more My Models / Discover sub-tree since
// the unified Models page handles that split via its mode toggle.
const MobileMenu = ({active='Models', withDownloads=false, dlCount=1}) => {
  // Backwards-compat: treat legacy "My Models" / "Discover" as Models active.
  const modelsActive = active==='Models' || active==='My Models' || active==='Discover';
  return (
    <div className="m-menu-overlay">
      <div className="m-menu">
        <div className="m-menu-item">Chat</div>
        <div className={`m-menu-item${modelsActive?' active':''}`}>Models</div>
        {withDownloads && (
          <div className="m-menu-item">
            <span>↓ Downloads</span>
            {dlCount>0 && <span className="m-menu-badge">{dlCount} ↓</span>}
          </div>
        )}
        <div className="m-menu-item">Agents</div>
        <div className="m-menu-item">Logs</div>
        <div className="m-menu-item">Settings</div>
      </div>
    </div>
  );
};

// Tablet-shaped frame used for the medium-width wireframes.
const TabletFrame = ({label, children}) => (
  <div className="tablet-frame">
    <div className="tablet-label">{label}</div>
    <div className="tablet-screen">
      <div className="tablet-content">{children}</div>
    </div>
  </div>
);

// Phone-shaped frame used for mobile variants.
const PhoneFrame = ({label, children}) => (
  <div className="phone-frame">
    <div className="phone-label">{label}</div>
    <div className="phone-screen">{children}</div>
  </div>
);

// ─────────────────────────────────────────────────────────────
// Alias form primitives · v20
// ─────────────────────────────────────────────────────────────

// Progressive-disclosure section. Static (no state) — summary shown when
// `open=false`, body shown when `open=true`. The wireframe shows both
// states across different variants rather than toggling live.
const ParamSection = ({n, title, summary, open=false, children}) => (
  <div className={`param-section ${open?'open':'collapsed'}`}>
    <div className="param-section-head">
      <div className="param-section-title">
        {n!=null && <span className="param-section-num">{n}</span>}
        <span>{title}</span>
      </div>
      <div className="param-section-summary">
        {!open && summary && <span>{summary}</span>}
        <span className="param-section-caret">{open?'▾':'▸'}</span>
      </div>
    </div>
    {open && <div className="param-section-body">{children}</div>}
  </div>
);

// Preset chip row — chat / coding / long-ctx / agent / reasoning / custom.
const PRESETS = [
  {k:'chat',     icon:'💬', label:'Chat',     hint:'balanced'},
  {k:'coding',   icon:'🧑‍💻', label:'Coding',   hint:'temp 0.2'},
  {k:'agent',    icon:'🛠️', label:'Agent',    hint:'tool-use'},
  {k:'reason',   icon:'🧠', label:'Reasoning', hint:'long cot'},
  {k:'longctx',  icon:'📄', label:'Long ctx', hint:'32K+'},
  {k:'custom',   icon:'⚙️', label:'Custom',   hint:'empty'},
];
const PresetChipRow = ({selected='chat'}) => (
  <div className="preset-row">
    {PRESETS.map(p => (
      <span key={p.k} className={`preset-chip${selected===p.k?' selected':''}`}>
        <span className="preset-chip-icon">{p.icon}</span>
        <span>{p.label}</span>
        <span className="sm">· {p.hint}</span>
      </span>
    ))}
  </div>
);

// Quant rows — fit dot, size, speed estimate, recommended badge.
const QuantPicker = ({quants, selected}) => (
  <div style={{display:'flex', flexDirection:'column', gap:3}}>
    {quants.map(q => (
      <div key={q.q} className={`quant-row${selected===q.q?' selected':''}`}>
        <span className={`fit-dot ${q.fit}`} />
        <code>{q.q}</code>
        <span className="sm">{q.size}</span>
        <span className="sm">{q.tok}</span>
        {q.rec ? <Chip tone="saff" style={{fontSize:9.5}}>recommended</Chip> : <span/>}
        {q.local
          ? <Chip tone="leaf" style={{fontSize:9.5}}>✓ local</Chip>
          : <Chip style={{fontSize:9.5}}>pull</Chip>}
      </div>
    ))}
  </div>
);

const DEFAULT_QUANTS = [
  {q:':Q4_K_M', size:'5.6 GB', tok:'~38 t/s', fit:'green', rec:true,  local:false},
  {q:':Q5_K_M', size:'6.8 GB', tok:'~30 t/s', fit:'green', rec:false, local:true},
  {q:':Q6_K',   size:'7.9 GB', tok:'~24 t/s', fit:'green', rec:false, local:false},
  {q:':Q8_0',   size:'9.6 GB', tok:'~16 t/s', fit:'yellow',rec:false, local:false},
  {q:':F16',    size:'18 GB',  tok:'OOM',     fit:'red',   rec:false, local:false},
];

// Fit check summary card.
const FitCheckCard = ({tone='good', title='✓ Fits with 21 GB headroom', sub='Est. ~38 tok/s on your rig · Apple M3 Max'}) => (
  <div className={`fit-check-card ${tone==='warn'?'warn':tone==='fail'?'fail':''}`}>
    <div className="fit-check-title">{title}</div>
    <div className="fit-check-grid">
      <span>VRAM</span><span>5.6 / 36 GB · 21 GB free</span>
      <span>est. t/s</span><span>~38 (Q4_K_M, 8K ctx)</span>
      <span>cpu fallback</span><span>off · 100% GPU offload</span>
      <span>disclaimer</span><span>estimates; run "Save & test" to measure</span>
    </div>
    <div className="sm" style={{marginTop:4}}>{sub}</div>
  </div>
);

// Live JSON config preview (static snapshot per wireframe).
const LiveConfigJson = ({config}) => {
  const json = JSON.stringify(config, null, 2);
  return <div className="live-json">{json}</div>;
};

const DEFAULT_ALIAS_CONFIG = {
  alias: 'qwen-chat',
  repo:  'Qwen/Qwen3.5-9B-GGUF',
  file:  'Qwen3.5-9B-Q4_K_M.gguf',
  preset: 'chat',
  context: { ctx_size: 8192, parallel: 1, n_gpu_layers: 'auto', flash_attn: true },
  sampling: { temperature: 0.7, top_p: 0.95, max_tokens: null },
  request:  { response_format: 'auto', tool_choice: 'auto' }
};

// Inline download strip — file not local yet.
const DownloadProgressStrip = ({repo='Qwen/Qwen3.5-9B-GGUF', quant=':Q4_K_M', state='queued', size='5.6 GB', pct=0}) => (
  <div className="dl-strip">
    <span className="dl-strip-icon">↓</span>
    <div style={{flex:1, minWidth:0}}>
      <div className="sm" style={{color:'var(--ink)', fontWeight:700}}>
        File not local — will download on save
      </div>
      <div className="sm">
        <code>{repo}{quant}</code> · {size} · {state}
      </div>
      {state==='downloading' && <Bar pct={pct} tone="saff" />}
    </div>
    <Btn size="xs">Pull now</Btn>
  </div>
);

// Slider with labeled tick marks.
const SliderWithMarks = ({label, marks=[], value, thumbPct=50, right}) => (
  <div>
    <div style={{display:'flex', justifyContent:'space-between', alignItems:'center'}}>
      <div className="lbl">{label}</div>
      {right && <span className="sm" style={{color:'var(--lotus)'}}>{right}</span>}
    </div>
    <div className="slider"><div className="thumb" style={{left: thumbPct+'%'}}/></div>
    <div style={{display:'flex', justifyContent:'space-between'}} className="sm">
      {marks.map((m,i)=>(<span key={i}>{m}</span>))}
    </div>
  </div>
);

// Task category data — 10 categories with illustrative benchmarks + sample refs.
const TASK_CATEGORIES = [
  {k:'chat',      icon:'💬', title:'Chat · general',        desc:'Everyday conversation, summarization, QA.',            bench:['MMLU','MT-Bench','Arena Elo'], refs:['Llama-3.3','Qwen3-chat','Gemma-2']},
  {k:'coding',    icon:'🧑‍💻', title:'Coding',               desc:'Code completion, refactor, fix, explain.',             bench:['HumanEval','MBPP','SWE-Bench'],     refs:['Qwen2.5-Coder','DeepSeek-Coder','Codestral']},
  {k:'agent',     icon:'🛠️', title:'Agentic · tool-use',   desc:'Function calling, multi-turn tool loops.',             bench:['BFCL','ToolBench','τ-Bench'],       refs:['Hermes-3','xLAM','Qwen2.5-Instruct']},
  {k:'reason',    icon:'🧠', title:'Reasoning',            desc:'Math, logic, multi-step CoT problems.',                bench:['GSM8K','MATH','GPQA','AIME'],       refs:['DeepSeek-R1-distill','QwQ','Phi-4']},
  {k:'longctx',   icon:'📄', title:'Long context',         desc:'Whole-repo, long docs, 32K+ contexts.',                bench:['RULER','LongBench','NIAH'],         refs:['Qwen3-long','Llama-3.1-128k','Yi-1.5']},
  {k:'multiling', icon:'🌐', title:'Multilingual',         desc:'Non-English quality: Hindi, Arabic, CJK, etc.',        bench:['XNLI','FLORES','mMMLU'],            refs:['Aya-23','Qwen3','Mistral-Small']},
  {k:'vision',    icon:'👁️', title:'Vision + text',        desc:'Image understanding, OCR, charts, docs.',              bench:['MMMU','DocVQA','ChartQA'],          refs:['Qwen2-VL','Llama-3.2-Vision','InternVL']},
  {k:'embed',     icon:'🧬', title:'Text embedding',       desc:'Vector search, retrieval, clustering.',                bench:['MTEB'],                              refs:['bge-large','nomic-embed','gte-Qwen2']},
  {k:'memb',      icon:'🖼️', title:'Multimodal embedding', desc:'Text ↔ image retrieval, visual search.',              bench:['CLIP-bench','MMEB'],                refs:['SigLIP','Jina-CLIP','nomic-embed-vision']},
  {k:'small',     icon:'⚡', title:'Small & fast · edge',   desc:'On-device, low-VRAM, mobile, raspi.',                  bench:['Open LLM LB','size tier'],          refs:['LFM2','Phi-3.5-mini','Gemma-2-2B']},
];
const TaskCategoryCard = ({cat}) => (
  <div className="task-cat-card">
    <div className="task-cat-head">
      <span className="task-cat-icon">{cat.icon}</span>
      <span className="task-cat-title">{cat.title}</span>
    </div>
    <div className="task-cat-desc">{cat.desc}</div>
    <div className="task-cat-bench">
      {cat.bench.map(b => <Chip key={b}>{b}</Chip>)}
    </div>
    <div className="task-cat-refs">
      <b>Top:</b> {cat.refs.join(' · ')}
    </div>
    <div className="task-cat-browse">Browse {cat.refs.length * 9 + 3} models →</div>
  </div>
);
const TaskCategoryGrid = () => (
  <div className="task-cat-grid">
    {TASK_CATEGORIES.map(c => <TaskCategoryCard key={c.k} cat={c} />)}
  </div>
);

// Browse-by selector — Task / Capability / Family segmented control.
const BrowseBySelector = ({active='family'}) => (
  <div className="browse-by-row">
    <span className="browse-by-label">Browse by</span>
    <div className="browse-by-seg">
      {[['task','Task'],['capability','Capability'],['family','Family']].map(([k,l]) => (
        <span key={k} className={`browse-by-seg-item${active===k?' active':''}`}>{l}</span>
      ))}
    </div>
    <span className="sm" style={{marginLeft:'auto'}}>
      {active==='task' ? 'Pick a goal first — recommendations follow.' :
       active==='capability' ? 'Filter by caps: tool-use, vision, embedding, …' :
       'Classic HF-style browse by org/family.'}
    </span>
  </div>
);

// Overlay shell for desktop AliasOverlay variant.
const OverlayShell = ({title, context, body, footer}) => (
  <div className="alias-overlay-frame">
    {/* dimmed underlay represents the Discover page the overlay launched from */}
    <div className="alias-overlay-backdrop">
      <div className="h3" style={{marginTop:0}}>Discover · Qwen/Qwen3.5-9B</div>
      <div className="sm">HuggingFace · Apache-2 · 9B · tool-use · long-ctx</div>
      <div className="sm" style={{marginTop:8}}>[ ↓ Add to Bodhi ] clicked — overlay opened with repo + quant prefilled.</div>
    </div>
    <div className="alias-overlay-shell">
      <div className="alias-overlay-head">
        <div className="h2" style={{margin:0}}>{title}</div>
        <Btn variant="ghost" size="xs" title="close">✕</Btn>
      </div>
      {context && <div className="alias-overlay-context">{context}</div>}
      <div className="alias-overlay-body">{body}</div>
      <div className="alias-overlay-footer">{footer}</div>
    </div>
  </div>
);

// Left rail for alias standalone — section nav.
const ALIAS_SECTIONS = [
  {k:'identity', n:1, label:'Identity'},
  {k:'model',    n:2, label:'Model file'},
  {k:'preset',   n:3, label:'Preset'},
  {k:'context',  n:4, label:'Context & runtime'},
  {k:'sampling', n:5, label:'Sampling'},
  {k:'request',  n:6, label:'Request shape'},
  {k:'advanced', n:7, label:'Advanced'},
];
const AliasRail = ({active='identity'}) => (
  <div className="alias-rail">
    <div className="alias-rail-title">Sections</div>
    {ALIAS_SECTIONS.map(s => (
      <div key={s.k} className={`alias-rail-item${active===s.k?' active':''}`}>
        <span className="alias-rail-item-num">{s.n}</span>
        <span>{s.label}</span>
      </div>
    ))}
  </div>
);

const AliasMediumAnchors = ({active='identity'}) => (
  <div className="alias-medium-anchor-row">
    {ALIAS_SECTIONS.map(s => (
      <span key={s.k} className={`alias-medium-anchor${active===s.k?' active':''}`}>
        {s.n} · {s.label}
      </span>
    ))}
  </div>
);

// ─────────────────────────────────────────────────────────────
// ArgsEditor · v21
// Raw llama-server cmdline args in a code-like editor, with helpers:
//   · preset chips (named arg bundles)
//   · right-side palette populated from parsed --help (static sample here)
//   · inline help footer for the focused line
//   · enum-value chips (for flags like --flash-attn, --rope-scaling, --chat-template)
//   · validation strip (arg count, warnings, llama-server version match)
//   · paste-command import bar
// Textarea/lines remain the source of truth; everything else is assistance.
// ─────────────────────────────────────────────────────────────

// Parsed-help sample (in a real app this is built at runtime from
// `llama-server --help`). Grouped by section. Includes default, enum values,
// env var name where applicable.
const ARGS_HELP = {
  'Common': [
    {flag:'--ctx-size',     alias:'-c',    value:'N',      desc:'size of the prompt context',                 def:'0 = from model', env:'LLAMA_ARG_CTX_SIZE'},
    {flag:'--threads',      alias:'-t',    value:'N',      desc:'CPU threads during generation',              def:'-1',            env:'LLAMA_ARG_THREADS'},
    {flag:'--batch-size',   alias:'-b',    value:'N',      desc:'logical maximum batch size',                 def:'2048',          env:'LLAMA_ARG_BATCH'},
    {flag:'--ubatch-size',  alias:'-ub',   value:'N',      desc:'physical maximum batch size',                def:'512',           env:'LLAMA_ARG_UBATCH'},
    {flag:'--flash-attn',   alias:'-fa',   value:'on|off|auto', enum:['on','off','auto'], desc:'Flash Attention', def:'auto',     env:'LLAMA_ARG_FLASH_ATTN'},
    {flag:'--n-gpu-layers', alias:'-ngl',  value:'N|auto|all', desc:'layers to store in VRAM',                def:'auto',          env:'LLAMA_ARG_N_GPU_LAYERS'},
    {flag:'--split-mode',   alias:'-sm',   value:'none|layer|row', enum:['none','layer','row'], desc:'multi-GPU split', def:'layer', env:'LLAMA_ARG_SPLIT_MODE'},
    {flag:'--rope-scaling', value:'none|linear|yarn', enum:['none','linear','yarn'], desc:'RoPE frequency scaling', def:'linear', env:'LLAMA_ARG_ROPE_SCALING_TYPE'},
    {flag:'--rope-scale',   value:'N',      desc:'RoPE context scaling factor',                               env:'LLAMA_ARG_ROPE_SCALE'},
    {flag:'--cache-type-k', alias:'-ctk',  value:'TYPE',   enum:['f32','f16','bf16','q8_0','q4_0','q4_1','iq4_nl','q5_0','q5_1'], desc:'KV cache dtype for K', def:'f16', env:'LLAMA_ARG_CACHE_TYPE_K'},
    {flag:'--cache-type-v', alias:'-ctv',  value:'TYPE',   enum:['f32','f16','bf16','q8_0','q4_0','q4_1','iq4_nl','q5_0','q5_1'], desc:'KV cache dtype for V', def:'f16', env:'LLAMA_ARG_CACHE_TYPE_V'},
    {flag:'--mlock',                        desc:'keep model in RAM, never swap',                             env:'LLAMA_ARG_MLOCK'},
    {flag:'--mmap',                         desc:'memory-map model (default: on)',                            def:'on',            env:'LLAMA_ARG_MMAP'},
  ],
  'Sampling': [
    {flag:'--temp',         alias:'--temperature', value:'N', desc:'sampling temperature',                    def:'0.80'},
    {flag:'--top-k',                       value:'N',      desc:'top-k sampling',                             def:'40',            env:'LLAMA_ARG_TOP_K'},
    {flag:'--top-p',                       value:'N',      desc:'top-p sampling',                             def:'0.95'},
    {flag:'--min-p',                       value:'N',      desc:'min-p sampling',                             def:'0.05'},
    {flag:'--repeat-penalty', value:'N',   desc:'penalize repeat sequence of tokens',                         def:'1.00'},
    {flag:'--mirostat',                    value:'0|1|2',  enum:['0','1','2'], desc:'Mirostat sampling',      def:'0'},
    {flag:'--mirostat-lr',                 value:'N',      desc:'Mirostat η (learning rate)',                 def:'0.10'},
    {flag:'--mirostat-ent',                value:'N',      desc:'Mirostat τ (target entropy)',                def:'5.00'},
    {flag:'--samplers',                    value:'SEQUENCE', desc:'sampler order, ";"-separated',             def:'penalties;dry;top_n_sigma;top_k;typ_p;top_p;min_p;xtc;temperature'},
    {flag:'--seed',         alias:'-s',    value:'N',      desc:'RNG seed (-1 = random)',                     def:'-1'},
    {flag:'--grammar',                     value:'GRAMMAR', desc:'BNF-like grammar to constrain generation'},
    {flag:'--json-schema',  alias:'-j',    value:'SCHEMA', desc:'JSON schema to constrain generation'},
  ],
  'Server': [
    {flag:'--parallel',     alias:'-np',   value:'N',      desc:'number of server slots',                     def:'-1 = auto',     env:'LLAMA_ARG_N_PARALLEL'},
    {flag:'--cont-batching', alias:'-cb',                  desc:'continuous (dynamic) batching',              def:'on',            env:'LLAMA_ARG_CONT_BATCHING'},
    {flag:'--cache-prompt',                desc:'enable prompt caching (default: on)',                        def:'on',            env:'LLAMA_ARG_CACHE_PROMPT'},
    {flag:'--cache-reuse',                 value:'N',      desc:'min chunk size to reuse via KV shifting',    def:'0',             env:'LLAMA_ARG_CACHE_REUSE'},
    {flag:'--jinja',                       desc:'jinja template engine for chat (default: on)',               def:'on',            env:'LLAMA_ARG_JINJA'},
    {flag:'--chat-template', value:'JINJA_TEMPLATE',
      enum:['chatml','llama2','llama3','llama4','gemma','phi3','phi4','mistral-v3','mistral-v7','deepseek','deepseek3','granite','command-r','qwen','gpt-oss','openchat','vicuna','zephyr'],
      desc:'custom jinja chat template · built-ins enumerated',                                               env:'LLAMA_ARG_CHAT_TEMPLATE'},
    {flag:'--reasoning',    alias:'-rea',  value:'on|off|auto', enum:['on','off','auto'], desc:'reasoning/thinking in chat', def:'auto', env:'LLAMA_ARG_REASONING'},
    {flag:'--reasoning-budget', value:'N', desc:'token budget for thinking (-1 = unlimited)',                 def:'-1',            env:'LLAMA_ARG_THINK_BUDGET'},
    {flag:'--embedding',                   desc:'restrict to embedding use case',                             env:'LLAMA_ARG_EMBEDDINGS'},
    {flag:'--rerank',                      desc:'enable reranking endpoint',                                  env:'LLAMA_ARG_RERANKING'},
    {flag:'--pooling',                     value:'none|mean|cls|last|rank', enum:['none','mean','cls','last','rank'], desc:'pooling type for embeddings', env:'LLAMA_ARG_POOLING'},
  ],
  'Speculative': [
    {flag:'--model-draft',  alias:'-md',   value:'FNAME',  desc:'draft model for speculative decoding',       env:'LLAMA_ARG_MODEL_DRAFT'},
    {flag:'--draft-max',    alias:'--draft', value:'N',    desc:'draft tokens for speculative decoding',      def:'16',            env:'LLAMA_ARG_DRAFT_MAX'},
    {flag:'--draft-min',                   value:'N',      desc:'minimum draft tokens',                       def:'0',             env:'LLAMA_ARG_DRAFT_MIN'},
    {flag:'--draft-p-min',                 value:'P',      desc:'minimum speculative probability',            def:'0.75',          env:'LLAMA_ARG_DRAFT_P_MIN'},
    {flag:'--spec-type',                   value:'none|ngram-cache|ngram-simple|ngram-map-k|ngram-map-k4v|ngram-mod', enum:['none','ngram-cache','ngram-simple','ngram-map-k','ngram-map-k4v','ngram-mod'], desc:'speculative decoding type', def:'none', env:'LLAMA_ARG_SPEC_TYPE'},
  ],
};

// Preset arg bundles — applying a preset replaces current lines.
// Grouped into "usage" (what you want to do) and "tradeoff" (resource dial)
// plus a Default (minimal sensible set) and Custom (empty).
const ARGS_PRESETS = {
  // Default / Custom
  default:  ['--flash-attn auto', '--jinja'],
  custom:   [],

  // Usage presets
  chat:     ['--ctx-size 8192',  '--flash-attn auto', '--parallel 4', '--jinja'],
  coding:   ['--ctx-size 16384', '--flash-attn auto', '--parallel 2', '--jinja', '--cache-type-k q8_0', '--cache-type-v q8_0'],
  agent:    ['--ctx-size 16384', '--flash-attn auto', '--parallel 4', '--jinja', '--reasoning auto'],
  reason:   ['--ctx-size 32768', '--flash-attn auto', '--parallel 1', '--reasoning on', '--reasoning-budget 4096'],
  longctx:  ['--ctx-size 32768', '--flash-attn auto', '--parallel 1', '--cache-type-k q8_0', '--rope-scaling yarn'],
  embed:    ['--embedding',      '--pooling mean',    '--ctx-size 2048', '--parallel 8'],
  vision:   ['--ctx-size 8192',  '--flash-attn auto', '--parallel 2', '--mmproj-auto'],
  ragShort: ['--ctx-size 4096',  '--flash-attn auto', '--parallel 4', '--cache-type-k q8_0', '--cache-prompt', '--cache-reuse 256'],
  ragLong:  ['--ctx-size 65536', '--flash-attn auto', '--parallel 1', '--cache-type-k q8_0', '--cache-type-v q8_0', '--rope-scaling yarn', '--cache-prompt'],

  // Tradeoff presets
  maxPerf:  ['--ctx-size 4096',   '--flash-attn on',   '--parallel 1', '--n-gpu-layers all', '--cache-type-k q4_0', '--cache-type-v q4_0', '--ubatch-size 1024'],
  maxCtx:   ['--ctx-size 131072', '--flash-attn auto', '--parallel 1', '--rope-scaling yarn', '--rope-scale 4', '--cache-type-k q4_0', '--cache-type-v q4_0'],
  parMed:   ['--ctx-size 4096',   '--parallel 4',      '--cont-batching', '--cache-prompt'],
  parMax:   ['--ctx-size 2048',   '--parallel 16',     '--cont-batching', '--cache-prompt', '--ubatch-size 1024'],
  hwMed:    ['--threads 4',       '--n-gpu-layers 24', '--mlock'],
  hwMax:    ['--threads -1',      '--n-gpu-layers all','--mlock', '--mmap', '--flash-attn on'],
  small:    ['--ctx-size 4096',   '--flash-attn auto', '--parallel 8', '--cache-type-k q4_0'],
};

// Preset catalogue — label, icon, group. Used by the chip grid in the
// merged "Preset & Runtime args" section.
const PRESET_CATALOGUE = [
  // base
  {k:'default',  icon:'●',   label:'Default',             group:'base'},
  // usage
  {k:'chat',     icon:'💬',  label:'Chat',                group:'usage'},
  {k:'coding',   icon:'🧑‍💻', label:'Coding',              group:'usage'},
  {k:'agent',    icon:'🛠️',  label:'Agent',               group:'usage'},
  {k:'reason',   icon:'🧠',  label:'Reasoning',           group:'usage'},
  {k:'ragShort', icon:'📑',  label:'RAG (short docs)',    group:'usage'},
  {k:'ragLong',  icon:'📚',  label:'RAG (long docs)',     group:'usage'},
  {k:'vision',   icon:'👁️',  label:'Vision',              group:'usage'},
  {k:'embed',    icon:'🧬',  label:'Embed',               group:'usage'},
  // tradeoffs
  {k:'maxPerf',  icon:'⚡',  label:'Max Performance (tok/s)', group:'tradeoff'},
  {k:'maxCtx',   icon:'📏',  label:'Max Context',         group:'tradeoff'},
  {k:'parMed',   icon:'∥',   label:'Parallel · Medium',   group:'tradeoff'},
  {k:'parMax',   icon:'∥∥',  label:'Parallel · Max',      group:'tradeoff'},
  {k:'hwMed',    icon:'▤',   label:'Hardware Use · Medium', group:'tradeoff'},
  {k:'hwMax',    icon:'▦',   label:'Hardware Use · Max',  group:'tradeoff'},
  {k:'longctx',  icon:'📄',  label:'Long-ctx',            group:'tradeoff'},
  {k:'small',    icon:'·',   label:'Small & fast',        group:'tradeoff'},
  // free-form
  {k:'custom',   icon:'⚙️',  label:'Custom',              group:'custom'},
];

const DEFAULT_ARG_LINES = [
  {flag:'--ctx-size',    value:'8192',     default:false, focused:false},
  {flag:'--flash-attn',  value:'auto',     default:true,  focused:false},
  {flag:'--parallel',    value:'4',        default:false, focused:true},
  {flag:'--cache-type-k',value:'q8_0',     default:false, focused:false},
  {flag:'--jinja',       value:null,       default:true,  focused:false},
  {flag:'--chat-template', value:'chatml', default:false, focused:false},
  {flag:'--reasoning',   value:'auto',     default:true,  focused:false},
];

// Plain-text render: each line is text with hover tooltip on the flag.
// Warnings render with a wavy underline (native `title` holds the message).
const ArgLine = ({line, spec}) => {
  const tooltip = spec
    ? `${spec.flag}${spec.alias?` (${spec.alias})`:''}${spec.value?` ${spec.value}`:''}\n${spec.desc}${spec.def?`\ndefault: ${spec.def}`:''}${spec.enum?`\nallowed: ${spec.enum.join(', ')}`:''}${spec.env?`\nenv: ${spec.env}`:''}`
    : `unknown flag · not in parsed --help`;
  const flagCls = line.warn ? 'args-line-warn' : 'args-line-flag';
  return (
    <div className={`args-line${line.focused?' focused':''}${line.default?' default':''}`}>
      <span className={flagCls} title={tooltip}>{line.flag}</span>
      {line.value != null && <><span className="args-line-value"> {line.value}</span></>}
      {line.focused && <span className="args-caret"/>}
    </div>
  );
};

// Inline help footer for currently-focused line.
const ArgsHelpPop = ({help}) => (
  <div className="args-help-pop">
    <span>name</span>
    <span><code>{help.flag}{help.alias?` (${help.alias})`:''}{help.value?` ${help.value}`:''}</code></span>
    <span>desc</span>
    <span>{help.desc}</span>
    {help.def && (<><span>default</span><span>{help.def}</span></>)}
    {help.enum && (
      <><span>allowed</span>
        <span>{help.enum.map(v => <span key={v} className="enum-val">{v}</span>)}</span>
      </>
    )}
    {help.env && (<><span>env</span><span className="env">{help.env}</span></>)}
  </div>
);

// Tiny searchable palette — grouped list + filter.
const ArgsPalette = ({query='ctx'}) => (
  <div className="args-palette">
    <input className="args-palette-search" defaultValue={query} placeholder="filter args (e.g. ctx, flash, chat)…" />
    {Object.entries(ARGS_HELP).map(([group, args]) => {
      const filtered = args.filter(a => !query || a.flag.includes(query) || (a.alias||'').includes(query) || (a.desc||'').toLowerCase().includes(query));
      if (!filtered.length) return null;
      return (
        <div key={group} className="args-palette-group">
          <div className="args-palette-group-head">{group} · {filtered.length}</div>
          {filtered.slice(0,8).map(a => (
            <div key={a.flag} className="args-palette-item" title={a.desc}>
              <span>{a.flag}</span>
              {a.enum ? <span className="tag">enum</span> :
               a.value ? <span className="tag">{a.value}</span> :
               <span className="tag">flag</span>}
            </div>
          ))}
          {filtered.length>8 && <div className="args-palette-item" style={{color:'var(--ink-4)'}}>+ {filtered.length-8} more…</div>}
        </div>
      );
    })}
  </div>
);

// Full args editor — used by AliasStandalone / Medium / Overlay.
// `compact=true` stacks palette below for narrow shells.
const ArgsEditor = ({
  selectedPreset='chat',
  lines=DEFAULT_ARG_LINES,
  focusedHelp,
  showPaste=false,
  showEnv=false,
  paletteQuery='',
  compact=false,
  count='7 args',
  warn='1 unknown flag (⚠)',
  version='matches llama-server v0.52'
}) => {
  const lookup = (flag) => {
    for (const arr of Object.values(ARGS_HELP)) {
      const m = arr.find(a => a.flag===flag);
      if (m) return m;
    }
    return null;
  };
  const help = focusedHelp || lookup((lines.find(l=>l.focused)||{}).flag || '--parallel');
  return (
    <div className={`args-editor${compact?' compact':''}`}>
      <div className="args-editor-toolbar">
        <Btn variant="ghost" size="xs">Paste command ↓</Btn>
        <Btn variant="ghost" size="xs">Copy</Btn>
        <Btn variant="ghost" size="xs">{showEnv?'Lines':'Env vars'}</Btn>
        <span className="sep">·</span>
        <span className="sm">preset: <b>{PRESET_CATALOGUE.find(p=>p.k===selectedPreset)?.label || selectedPreset}</b></span>
      </div>

      {showPaste && (
        <div className="args-paste-row">
          <span>Pasted:</span>
          <code>llama-server -c 8192 -fa auto -np 4 -ctk q8_0 --jinja --chat-template chatml</code>
          <Btn size="xs">Import ({7} lines)</Btn>
        </div>
      )}

      <div className="args-editor-body">
        <div className="args-lines" contentEditable={false} suppressContentEditableWarning={true}>
          {lines.map((l, i) => {
            const spec = lookup(l.flag);
            return <ArgLine key={i} line={l} spec={spec}/>;
          })}
          {/* a deliberate unknown/warning line showing wavy underline + tooltip */}
          <ArgLine line={{flag:'--unknown-flag', value:'foo', warn:true}} spec={null}/>
          <div className="args-line args-line-add">hover a flag for help · click palette → to append · edit as plain text</div>
        </div>
        <ArgsPalette query={paletteQuery}/>
      </div>

      {help && <ArgsHelpPop help={help}/>}

      <div className="args-validation">
        <span className="ok">✓ {count}</span>
        <span className="warn">⚠ {warn}</span>
        <span>·</span>
        <span className="sm">preset match: {selectedPreset} (± 2 overrides)</span>
        <span className="ver">{version}</span>
      </div>
    </div>
  );
};

// Grid of all presets grouped by row (Base · Usage · Tradeoff · Custom).
const PresetGrid = ({selected='chat'}) => {
  const groups = [['base','Base'],['usage','Usage'],['tradeoff','Tradeoff'],['custom','']];
  return (
    <div className="preset-grid">
      {groups.map(([g,_], gi) => (
        <React.Fragment key={g}>
          {PRESET_CATALOGUE.filter(p=>p.group===g).map(p => (
            <span key={p.k}
                  className={`args-preset-chip${selected===p.k?' on':''}`}
                  title={`Applies ${(ARGS_PRESETS[p.k]||[]).length} args`}>
              <span>{p.icon}</span>
              <span>{p.label}</span>
            </span>
          ))}
          {gi < groups.length-1 && <span className="preset-grid-sep"/>}
        </React.Fragment>
      ))}
    </div>
  );
};

// Merged "Preset & Runtime args" block — preset chips always visible,
// editor collapsed by default. When collapsed, shows a one-liner summary
// with the first few cmdline args so users still see the preset's effect.
// Renders as a ParamSection with custom body.
const PresetAndArgsSection = ({n=3, selected='chat', open=false, compact=false}) => {
  const presetMeta = PRESET_CATALOGUE.find(p=>p.k===selected) || {label:selected, icon:'·'};
  const lines = ARGS_PRESETS[selected] || [];
  const previewLines = lines.slice(0, 4);
  const remaining = Math.max(0, lines.length - previewLines.length);
  const argLines = lines.map((t, i) => {
    const [flag, ...rest] = t.split(' ');
    return {flag, value: rest.join(' ') || null, focused: i===2};
  });
  return (
    <div className={`param-section ${open?'open':'collapsed'}`}>
      <div className="param-section-head">
        <div className="param-section-title">
          <span className="param-section-num">{n}</span>
          <span>Preset &amp; Runtime args</span>
        </div>
        <div className="param-section-summary">
          {!open && <span>{lines.length} lines · preset: <b>{presetMeta.label}</b></span>}
          <span className="param-section-caret">{open?'▾':'▸'}</span>
        </div>
      </div>
      <div className="param-section-body" style={open?undefined:{marginTop:6}}>
        <div className="sm">
          Pick a preset — it seeds the runtime args below. Expand the editor to tweak any line.
          Args editor stays raw so it never drifts with llama.cpp releases.
        </div>
        <PresetGrid selected={selected}/>
        {!open && (
          <div className="preset-summary-row">
            <span>→</span>
            <span><b>{lines.length}</b> cmdline args applied:</span>
            {previewLines.map((l,i) => <code key={i}>{l}</code>)}
            {remaining > 0 && <span className="sm">+ {remaining} more</span>}
            <span className="expand">▾ expand editor</span>
          </div>
        )}
        {open && (
          <>
            <div className="sm" style={{margin:'4px 0 4px', fontWeight:700}}>
              Runtime · llama-server args — {lines.length} lines from <b>{presetMeta.label}</b>
            </div>
            <ArgsEditor
              selectedPreset={selected}
              lines={argLines}
              count={`${lines.length} args`}
              compact={compact}/>
          </>
        )}
      </div>
    </div>
  );
};

// ─────────────────────────────────────────────────────────────
// Unified Models page primitives · v25
// ─────────────────────────────────────────────────────────────

// Mode radio — "[o] My Models · [ ] All Models" row 1 of the main toolbar.
// Shows a counts readout on the right so users see what each bucket contains.
const ModeToggle = ({mode='my', onChange, localCount=14, catalogCount='3.1M', directoryCount=23}) => (
  <div className="mode-toggle">
    <div className={`mode-toggle-option${mode==='my'?' active':''}`}
         onClick={() => onChange && onChange('my')}>
      <span className="dot"/>
      <span>My Models</span>
      <span className="sm" style={{color:'inherit', opacity:0.7}}>({localCount})</span>
    </div>
    <div className={`mode-toggle-option${mode==='all'?' active':''}`}
         onClick={() => onChange && onChange('all')}>
      <span className="dot"/>
      <span>All Models</span>
      <span className="sm" style={{color:'inherit', opacity:0.7}}>({catalogCount} + {directoryCount} directory)</span>
    </div>
  </div>
);

// Caption under the toggle — a one-liner explaining what each mode draws from.
const ModeToggleCaption = ({mode='my'}) => (
  <div className="mode-toggle-caption">
    {mode==='my'
      ? 'My Models: locally-hosted aliases, downloaded files, configured API models, and connected providers.'
      : 'All Models: HuggingFace catalog + connected + directory providers (api.getbodhi.app) + your local entities (tagged).'}
  </div>
);

// Toolbar row 2 — kind chips to narrow within the current mode.
const KindChipRow = ({active=['all'], mode='my'}) => {
  const kinds = mode==='my'
    ? [['all','All'],['alias','Aliases'],['file','Files'],['api-model','API models'],['provider','Providers']]
    : [['all','All'],['alias','Aliases'],['file','Files'],['api-model','API models'],['provider','Providers'],['hf-repo','HF repos']];
  return (
    <div className="kind-chip-row">
      <span className="kind-chip-row-label">kind</span>
      {kinds.map(([k,l]) => (
        <Chip key={k} on={active.includes(k)}>{l}</Chip>
      ))}
    </div>
  );
};

// Grouped Add + Browse dropdown shown below the `+ ▾` button in the page header.
// Desktop variant: two labelled groups; mobile/medium pass `compact` to tighten.
const ModelsAddBrowseMenu = ({style}) => (
  <div className="add-browse-menu" style={style}>
    <div className="add-browse-group-head">Add model</div>
    <div className="add-browse-item">Add by HF repo<span className="add-browse-item-badge">overlay</span></div>
    <div className="add-browse-item">Paste URL<span className="add-browse-item-badge">.gguf / hf://</span></div>
    <div className="add-browse-item">Add API provider<span className="add-browse-item-badge">connect</span></div>
    <div className="add-browse-item">Add API model<span className="add-browse-item-badge">from connected</span></div>
    <div className="add-browse-group-head">Browse</div>
    <div className="add-browse-item">↑ Trending</div>
    <div className="add-browse-item">★ New launches</div>
    <div className="add-browse-item">🏆 Leaderboards ›</div>
  </div>
);

// ─────────────────────────────────────────────────────────────
// Ranked display mode · v27
// Model-level rows that kick in when a benchmark sort is active.
// See specs/models.md §8 for the full spec. Key rules:
//   · local file downloaded → collapse HF entry; stack aliases
//   · API model → stack all api-aliases + show provider identity
//   · no local backing → show HF / provider / directory entry as-is
//   · rank numbers are ABSOLUTE across the leaderboard;
//     filtering narrows visible rows but never renumbers.
// ─────────────────────────────────────────────────────────────

// Banner shown below the toolbar when ranked mode is active. Dismissible.
const RankedModeCaption = ({benchmark='HumanEval', specLabel='Coding', onDismiss}) => (
  <div className="rank-caption">
    <span>
      Ranked by <b>{benchmark}</b> ({specLabel}). Local downloads shown as your
      aliases; API models stack all configurations. Filtering hides rows but
      keeps rank numbers.
    </span>
    <span className="rank-caption-dismiss" onClick={onDismiss} title="dismiss">×</span>
  </div>
);

// Static fixture used by groupIntoRankedRows. Each entry represents the model
// at a given global rank; `isLocal` controls which entries are visible in
// `My Models` scope. The shape mirrors what a production aggregator would
// produce after applying dedup + stack rules to raw entity data.
const RANKED_FIXTURE_CODING = [
  {
    rank: 1, score: 82.1, benchmark: 'HumanEval',
    primaries: [
      {label: 'claude-sonnet-4.5', chip: 'api-alias', tone: 'indigo', code: 'anthropic/claude-sonnet-4.5'},
    ],
    identity: {label: 'anthropic (provider — connected)', tag: 'provider', code: 'api.anthropic.com'},
    actions: [{label: 'use →'}],
    dispatchKind: 'provider-connected',
    isLocal: true, isDirectory: false,
  },
  {
    rank: 2, score: 79.3, benchmark: 'HumanEval',
    primaries: [
      {label: 'cc/opus-4-6',       chip: 'api-alias', tone: 'indigo', meta: 'api-key'},
      {label: 'cc-oauth/opus-4-6', chip: 'api-alias', tone: 'saff',   meta: 'oauth'},
    ],
    identity: {label: 'anthropic (provider — connected)', tag: 'provider', code: 'claude-opus-4.6'},
    actions: [{label: 'use →'}, {label: '+ new alias'}],
    dispatchKind: 'api-model',
    isLocal: true, isDirectory: false,
  },
  {
    rank: 3, score: 77.0, benchmark: 'HumanEval',
    primaries: [
      {label: 'gpt-5', chip: 'api-alias', tone: 'indigo', meta: 'api-key'},
    ],
    identity: {label: 'openai (provider — connected)', tag: 'provider', code: 'openai/gpt-5'},
    actions: [{label: 'use →'}],
    dispatchKind: 'api-model',
    isLocal: true, isDirectory: false,
  },
  {
    rank: 4, score: 75.8, benchmark: 'HumanEval',
    primaries: [
      {label: 'my-qwen-coder', chip: 'alias', tone: 'saff', meta: 'preset: coding'},
      {label: 'my-qwen-long',  chip: 'alias', tone: 'saff', meta: 'preset: long-ctx'},
    ],
    identity: {label: 'Qwen/Qwen3-Coder-32B:Q4_K_M', tag: 'modelfile', code: null, size: '20.3 GB'},
    actions: [{label: 'use →'}],
    dispatchKind: 'alias',
    isLocal: true, isDirectory: false,
  },
  {
    rank: 5, score: 74.2, benchmark: 'HumanEval',
    primaries: [
      {label: 'Qwen3-Coder-32B:Q8_0', chip: 'hf-repo', tone: 'leaf', code: 'Qwen/Qwen3-Coder-32B'},
    ],
    identity: {label: 'HuggingFace catalog · 34.0 GB · 5 quants', tag: 'huggingface', code: null},
    actions: [{label: 'pull →'}],
    dispatchKind: 'hf-repo',
    isLocal: false, isDirectory: false,
  },
  {
    rank: 6, score: 67.2, benchmark: 'HumanEval',
    primaries: [
      {label: 'my-gemma', chip: 'alias', tone: 'saff', meta: 'preset: chat'},
    ],
    identity: {label: 'google/gemma-3-9b:Q8_0', tag: 'modelfile', code: null, size: '9.1 GB'},
    actions: [{label: 'use →'}],
    dispatchKind: 'alias',
    isLocal: true, isDirectory: false,
  },
  {
    rank: 7, score: 62.7, benchmark: 'HumanEval',
    primaries: [
      {label: 'DeepSeek-V3:Q4_K_M', chip: 'file', tone: 'leaf', code: 'deepseek-ai/DeepSeek-V3', meta: 'orphan · no alias'},
    ],
    identity: {label: 'on disk · 404 GB · downloaded 2026-03-02', tag: 'modelfile', code: null},
    actions: [{label: '+ Create alias'}],
    dispatchKind: 'file',
    isLocal: true, isDirectory: false,
  },
  {
    rank: 8, score: 61.4, benchmark: 'HumanEval',
    primaries: [
      {label: 'Llama-3.3-70B:Q4_K_M', chip: 'hf-repo', tone: 'leaf', code: 'meta-llama/Llama-3.3-70B-Instruct'},
    ],
    identity: {label: 'HuggingFace catalog · 42.5 GB · 4 quants', tag: 'huggingface', code: null},
    actions: [{label: 'pull →'}],
    dispatchKind: 'hf-repo',
    isLocal: false, isDirectory: false,
  },
  {
    rank: 9, score: 58.4, benchmark: 'HumanEval',
    primaries: [
      {label: 'mixtral-8x7b-instruct', chip: 'provider', tone: '', code: 'groq/mixtral-8x7b-instruct'},
    ],
    identity: {label: 'groq (from Bodhi directory)', tag: 'from-directory', code: 'api.getbodhi.app'},
    actions: [{label: 'Connect to use →'}],
    dispatchKind: 'provider-unconnected',
    isLocal: false, isDirectory: true,
  },
];

// Pure function: takes the benchmark key + mode, returns ordered RankedRow props.
// For the wireframe the population is the static fixture above; in production
// this is where the backend-data shape would be formalized. The mode filter
// narrows the SET but never renumbers ranks — an item at global #6 still shows
// as #6 in My Models even if it's the 3rd surviving row.
const groupIntoRankedRows = (benchmark='HumanEval', mode='all') => {
  const all = RANKED_FIXTURE_CODING; // only Coding/HumanEval wired for the demo
  return mode === 'my' ? all.filter(r => r.isLocal) : all;
};

// One ranked entry — rank number · stacked primaries · identity line · meta.
// Click dispatches to the same detail panel as the equivalent non-ranked row.
const RankedRow = ({entry, selected, onClick}) => {
  if (!entry) return null;
  const {rank, primaries=[], identity, score, benchmark, actions=[],
         isLocal, isDirectory} = entry;
  const cls = [
    'rank-row',
    isLocal ? 'rank-local' : '',
    isDirectory ? 'rank-directory' : '',
    selected ? 'selected' : '',
  ].filter(Boolean).join(' ');
  return (
    <div className={cls} onClick={onClick}>
      <div className="rank-number">
        <span className="rank-number-prefix">#</span>{rank}
      </div>
      <div>
        <div className="rank-primaries">
          {primaries.map((p, i) => (
            <div key={i} className="rank-primary-line">
              <Chip tone={p.tone||''} style={{fontSize:9.5}}>{p.chip}</Chip>
              <span style={{flex:'0 0 auto'}}>{p.label}</span>
              {p.code && <code>{p.code}</code>}
              {p.meta && <span className="sm" style={{fontStyle:'italic'}}>· {p.meta}</span>}
            </div>
          ))}
        </div>
        {identity && (
          <div className="rank-identity">
            <span className="rank-identity-tag">{identity.tag}</span>
            <span>{identity.label}</span>
            {identity.code && <code>{identity.code}</code>}
            {identity.size && <span>· {identity.size}</span>}
          </div>
        )}
      </div>
      <div className="rank-meta">
        <div>
          <span className="rank-meta-score">{score.toFixed(1)}</span>{' '}
          <span className="rank-meta-score-label">{benchmark}</span>
        </div>
        <div className="rank-meta-actions">
          {actions.map((a, i) => (
            <Btn key={i} variant={i===0?'primary':'ghost'} size="xs">{a.label}</Btn>
          ))}
        </div>
      </div>
    </div>
  );
};

// ─────────────────────────────────────────────────────────────
// Create API Model primitives · v29
// Flat one-form layout with production-parity fields:
//   · ApiFormatPicker     — format dropdown (openai/anthropic/google/…)
//   · ApiKeyField         — "Use API key" toggle + masked input + eye
//   · PrefixField         — "Enable prefix" toggle + text input
//   · ForwardingModeRadio — "Forward all" / "Forward for selected models only"
//   · ModelMultiSelect    — selected chips + search + available list + actions
//   · ApiRail / ApiMediumAnchors — sticky/top section nav
// Model selection section is CONDITIONAL on forwarding mode — shown only when
// "Forward for selected" is active. See specs/api.md §4.
// ─────────────────────────────────────────────────────────────

const API_FORMATS = [
  {code: 'openai-responses',     label: 'OpenAI — Responses',       defaultBaseUrl: 'https://api.openai.com/v1'},
  {code: 'openai-completions',   label: 'OpenAI — Completions',     defaultBaseUrl: 'https://api.openai.com/v1'},
  {code: 'anthropic-messages',   label: 'Anthropic — Messages',     defaultBaseUrl: 'https://api.anthropic.com/v1'},
  {code: 'anthropic-oauth',      label: 'Anthropic — OAuth',        defaultBaseUrl: 'https://api.anthropic.com/v1'},
  {code: 'google-gemini',        label: 'Google — Gemini',          defaultBaseUrl: 'https://generativelanguage.googleapis.com/v1beta'},
  {code: 'openrouter',           label: 'OpenRouter',                defaultBaseUrl: 'https://openrouter.ai/api/v1'},
  {code: 'hf-inference',         label: 'HuggingFace — Inference',   defaultBaseUrl: 'https://api-inference.huggingface.co'},
  {code: 'nvidia-nim',           label: 'NVIDIA — NIM',              defaultBaseUrl: 'https://integrate.api.nvidia.com/v1'},
  {code: 'groq',                 label: 'Groq — OpenAI-compatible',  defaultBaseUrl: 'https://api.groq.com/openai/v1'},
  {code: 'together',             label: 'Together AI',               defaultBaseUrl: 'https://api.together.xyz/v1'},
];

const FIXTURE_OPENAI_MODELS = [
  'gpt-5', 'gpt-5-mini', 'gpt-5-nano',
  'o4-mini',
  'codex-latest',
  'gpt-5.1-codex-mini', 'gpt-5.1-codex-max', 'gpt-5.2-codex',
  'gpt-4.1', 'gpt-4-turbo',
  'gpt-5.3-codex',
  'text-embedding-3-large',
];

// Dropdown picker for API format. Wireframe: Field-styled closed state only.
const ApiFormatPicker = ({value='openai-completions', onChange}) => {
  const selected = API_FORMATS.find(f => f.code === value) || API_FORMATS[0];
  return (
    <Field
      label={<span>API format <Chip tone="warn" style={{fontSize:9, marginLeft:4}}>required</Chip></span>}
      filled
      value={selected.label}
      right={<span className="sm">▾</span>}
    />
  );
};

// "Use API key" toggle + masked input with eye toggle. Disabled until toggled on.
const ApiKeyField = ({enabled=true, value='••••••••••••••••••••••', masked=true, onEnabledChange, onValueChange, onToggleMask}) => (
  <div className="api-toggle-field">
    <input type="checkbox" checked={enabled} onChange={e => onEnabledChange && onEnabledChange(e.target.checked)} />
    <div className={`api-toggle-field-main${enabled?'':' disabled'}`}>
      <div className="sm" style={{fontWeight:700, color:'var(--ink)'}}>Use API key</div>
      <Field
        filled
        value={masked ? value : 'sk-proj-a71e-••••-••••-a71e2f09d4b6c83a5'}
        hint={enabled ? 'sk-…' : 'Toggle on to enter a key'}
        right={<span className="sm" style={{cursor:'pointer'}} onClick={onToggleMask}>👁</span>}
      />
      <span className="sm" style={{fontSize:10, color:'var(--ink-3)'}}>Your API key is stored securely</span>
    </div>
  </div>
);

// "Enable prefix" toggle + text input. Example helper shown under the input.
const PrefixField = ({enabled=true, value='openai/', onEnabledChange, onValueChange, example='openai/gpt-4'}) => (
  <div className="api-toggle-field">
    <input type="checkbox" checked={enabled} onChange={e => onEnabledChange && onEnabledChange(e.target.checked)} />
    <div className={`api-toggle-field-main${enabled?'':' disabled'}`}>
      <div className="sm" style={{fontWeight:700, color:'var(--ink)'}}>Enable prefix</div>
      <Field
        filled
        value={value}
      />
      <span className="sm" style={{fontSize:10, color:'var(--ink-3)'}}>
        Add a prefix to model names (useful for organization or API routing). Example: <code style={{fontSize:10}}>{example}</code>
      </span>
    </div>
  </div>
);

// Two-option radio: forward-all vs forward-for-selected. Drives the
// conditional rendering of the Model selection section in api.jsx.
const ForwardingModeRadio = ({value='selected', onChange}) => {
  const opts = [
    {k: 'all',      label: 'Forward all requests with prefix'},
    {k: 'selected', label: 'Forward for selected models only'},
  ];
  return (
    <div className="api-forward-radio">
      {opts.map(o => (
        <div key={o.k}
             className={`api-forward-radio-option${value===o.k?' active':''}`}
             onClick={() => onChange && onChange(o.k)}>
          <span className="api-forward-radio-dot"/>
          <span>{o.label}</span>
        </div>
      ))}
    </div>
  );
};

// Full model picker: selected-chips strip + search + available list + actions.
// `onSelect`/`onDeselect` toggle individual models; `onSelectAll`/`onClear`
// bulk ops; `onFetch` mimics a re-fetch of the provider's model list.
const ModelMultiSelect = ({
  selected=['gpt-4-turbo','gpt-5-mini','gpt-5.3-codex'],
  available=FIXTURE_OPENAI_MODELS,
  search='codex',
  onSearch, onSelect, onDeselect, onFetch, onSelectAll, onClear,
}) => {
  const filtered = available.filter(m => !search || m.toLowerCase().includes(search.toLowerCase()));
  const unselectedFiltered = filtered.filter(m => !selected.includes(m));
  const selectedSet = new Set(selected);
  return (
    <div className="api-models-card">
      <div className="api-models-selected-label">
        <span>Selected Models ({selected.length})</span>
        {selected.length>0 && <span className="clear-link" onClick={onClear}>Clear All</span>}
      </div>
      <div className="api-models-selected">
        {selected.length === 0
          ? <span className="sm" style={{fontStyle:'italic', color:'var(--ink-3)'}}>No models selected · pick from the list below</span>
          : selected.map(m => (
              <span key={m} className="api-models-chip">
                {m}
                <span className="x" onClick={() => onDeselect && onDeselect(m)} title="remove">×</span>
              </span>
            ))
        }
      </div>

      <div className="api-models-selected-label">
        <span>Available Models</span>
        <span style={{display:'flex', gap:6}}>
          <span className="clear-link" style={{color:'var(--indigo)'}} onClick={onFetch}>Fetch Models</span>
          <span className="clear-link" style={{color:'var(--indigo)'}} onClick={onSelectAll}>Select All ({unselectedFiltered.length})</span>
        </span>
      </div>
      <div className="api-models-search">
        <input type="text" value={search} placeholder="Search available models…"
               onChange={e => onSearch && onSearch(e.target.value)}/>
        {search && <span className="clear-x" onClick={() => onSearch && onSearch('')}>×</span>}
      </div>
      <div className="api-models-available">
        {filtered.length === 0 && (
          <div className="sm" style={{fontStyle:'italic', color:'var(--ink-3)', padding:'6px 8px'}}>
            No matches — try Fetch Models or a different search.
          </div>
        )}
        {filtered.map(m => (
          <label key={m}
                 className={`api-models-available-item${selectedSet.has(m)?' selected':''}`}>
            <input type="checkbox"
                   checked={selectedSet.has(m)}
                   onChange={e => {
                     if (e.target.checked) onSelect && onSelect(m);
                     else onDeselect && onDeselect(m);
                   }}/>
            <span>{m}</span>
          </label>
        ))}
      </div>
    </div>
  );
};

// Sticky section nav for ApiStandalone — mirrors AliasRail.
const ApiRail = ({active='provider'}) => {
  const items = [
    {k:'provider', label:'Provider'},
    {k:'routing',  label:'Routing'},
    {k:'models',   label:'Models'},
  ];
  return (
    <aside className="api-rail">
      <div className="api-rail-head">sections</div>
      {items.map(i => (
        <div key={i.k} className={`api-rail-item${active===i.k?' active':''}`}>{i.label}</div>
      ))}
    </aside>
  );
};

// Top-of-page jump chips for ApiMedium — mirrors AliasMediumAnchors.
const ApiMediumAnchors = ({active='provider'}) => {
  const items = [
    {k:'provider', label:'Provider'},
    {k:'routing',  label:'Routing'},
    {k:'models',   label:'Models'},
  ];
  return (
    <div className="api-medium-anchors">
      {items.map(i => (
        <span key={i.k} className={`api-medium-anchor${active===i.k?' active':''}`}>{i.label}</span>
      ))}
    </div>
  );
};

// ═════════════════ MCP primitives (v30) ═════════════════

// 9 curated categories drawn from research across Smithery / mcp.so / mcpmarket / Docker Hub MCP.
const MCP_CATEGORIES = [
  {code:'all',         label:'All',                icon:'◎'},
  {code:'productivity',label:'Productivity',       icon:'▤'},
  {code:'search',      label:'Search & Web',       icon:'⌕'},
  {code:'browser',     label:'Browser',            icon:'◰'},
  {code:'dev',         label:'Dev Tools',          icon:'⌁'},
  {code:'data',        label:'Data',               icon:'⊞'},
  {code:'ai',          label:'AI & Content',       icon:'✦'},
  {code:'memory',      label:'Memory',             icon:'◧'},
  {code:'comms',       label:'Comms',              icon:'✉'},
  {code:'finance',     label:'Finance',            icon:'◉'},
];

// Catalog fixture — 12 entries that span the 5 card-states, loosely modelled on the production
// Register-MCP-Server screenshots (download (23).png header-auth and download (24).png oauth).
// `state` is the DERIVED state a viewer would see, not a DB column — computed from the join.
const MCP_CATALOG_FIXTURE = [
  {slug:'notion', name:'Notion', publisher:'Notion Labs', verified:true, category:'productivity',
   logo:'N', tags:['oauth','featured','official'],
   short:'Search, read and write pages & databases across your Notion workspace.',
   defaultBaseUrl:'https://mcp.notion.com/mcp', transport:'streamable-http', authType:'oauth2',
   authConfig:{registrationType:'dynamic', authorizationEndpoint:'https://mcp.notion.com/authorize', tokenEndpoint:'https://mcp.notion.com/token', registrationEndpoint:'https://mcp.notion.com/register', scopes:['notion:read','notion:write']},
   stats:{installCount:'7.4k', weeklyCalls:'212k', uptimePct:98.1, latencyP50Ms:420},
   links:{homepage:'https://notion.com', repository:'notion/mcp', license:'MIT'},
   state:'connected'},
  {slug:'linear', name:'Linear', publisher:'Linear', verified:true, category:'productivity',
   logo:'L', tags:['oauth','featured'],
   short:'Manage issues, projects, cycles, and comments across your Linear workspace.',
   defaultBaseUrl:'https://mcp.linear.app/mcp', transport:'streamable-http', authType:'oauth2',
   authConfig:{registrationType:'dynamic', authorizationEndpoint:'https://mcp.linear.app/authorize', tokenEndpoint:'https://mcp.linear.app/token', registrationEndpoint:'https://mcp.linear.app/register', scopes:['read','write']},
   stats:{installCount:'3.1k', weeklyCalls:'88k', uptimePct:99.2, latencyP50Ms:310},
   links:{homepage:'https://linear.app', repository:'linear/mcp', license:'MIT'},
   state:'approved'},
  {slug:'gmail', name:'Gmail', publisher:'Google', verified:true, category:'comms',
   logo:'G', tags:['oauth','official'],
   short:'Send, draft, reply, forward, and bulk-modify messages and threads in Gmail.',
   defaultBaseUrl:'https://mcp.google.com/gmail', transport:'streamable-http', authType:'oauth2',
   authConfig:{registrationType:'preregistered', authorizationEndpoint:'https://accounts.google.com/o/oauth2/auth', tokenEndpoint:'https://oauth2.googleapis.com/token', registrationEndpoint:'', scopes:['https://mail.google.com/']},
   stats:{installCount:'46k', weeklyCalls:'1.2M', uptimePct:99.8, latencyP50Ms:280},
   links:{homepage:'https://mail.google.com', repository:'google/mcp-gmail', license:'Apache-2.0'},
   state:'catalog-only'},
  {slug:'slack', name:'Slack', publisher:'Slack', verified:true, category:'comms',
   logo:'S', tags:['oauth'],
   short:'Channel-based messaging: post, search, react across your Slack workspace.',
   defaultBaseUrl:'https://mcp.slack.com/mcp', transport:'streamable-http', authType:'oauth2',
   authConfig:{registrationType:'preregistered', authorizationEndpoint:'https://slack.com/oauth/v2/authorize', tokenEndpoint:'https://slack.com/api/oauth.v2.access', registrationEndpoint:'', scopes:['chat:write','channels:read']},
   stats:{installCount:'14k', weeklyCalls:'410k', uptimePct:99.4, latencyP50Ms:340},
   links:{homepage:'https://slack.com', repository:'slack/mcp', license:'MIT'},
   state:'pending-approval'},
  {slug:'exa', name:'Exa Search', publisher:'Exa Labs', verified:true, category:'search',
   logo:'E', tags:['header','featured'],
   short:'Fast, intelligent web search and crawling — + Exa-code context tool for coding.',
   defaultBaseUrl:'https://mcp.exa.ai/mcp', transport:'streamable-http', authType:'header',
   authConfig:{keyDefinitions:[{placement:'header', name:'Authorization', hint:'Bearer <EXA_API_KEY>'}]},
   stats:{installCount:'60k', weeklyCalls:'27k', uptimePct:58.6, latencyP50Ms:2300},
   links:{homepage:'https://exa.ai', repository:'exa-labs/exa-mcp-server', license:'MIT'},
   state:'connected'},
  {slug:'github', name:'GitHub', publisher:'GitHub', verified:true, category:'dev',
   logo:'◉', tags:['oauth','official','featured'],
   short:'Manage repos, issues, PRs, workflows, and Actions — official MCP server.',
   defaultBaseUrl:'https://api.github.com/mcp', transport:'streamable-http', authType:'oauth2',
   authConfig:{registrationType:'preregistered', authorizationEndpoint:'https://github.com/login/oauth/authorize', tokenEndpoint:'https://github.com/login/oauth/access_token', registrationEndpoint:'', scopes:['repo','workflow']},
   stats:{installCount:'5.2k', weeklyCalls:'180k', uptimePct:99.9, latencyP50Ms:210},
   links:{homepage:'https://github.com', repository:'github/github-mcp-server', license:'MIT'},
   state:'approved'},
  {slug:'playwright', name:'Playwright', publisher:'Microsoft', verified:true, category:'browser',
   logo:'▷', tags:['header'],
   short:'Browser automation: click, fill, screenshot, assert across Chrome/FF/Safari.',
   defaultBaseUrl:'https://mcp.playwright.dev/mcp', transport:'streamable-http', authType:'header',
   authConfig:{keyDefinitions:[{placement:'header', name:'X-API-Key', hint:'Your Playwright key'}]},
   stats:{installCount:'3.9k', weeklyCalls:'45k', uptimePct:97.2, latencyP50Ms:640},
   links:{homepage:'https://playwright.dev', repository:'microsoft/playwright-mcp', license:'Apache-2.0'},
   state:'catalog-only'},
  {slug:'sheets', name:'Google Sheets', publisher:'Google', verified:true, category:'data',
   logo:'⊞', tags:['oauth','official'],
   short:'Read, write and format spreadsheet data; manage sheets and collaborate.',
   defaultBaseUrl:'https://mcp.google.com/sheets', transport:'streamable-http', authType:'oauth2',
   authConfig:{registrationType:'preregistered', authorizationEndpoint:'https://accounts.google.com/o/oauth2/auth', tokenEndpoint:'https://oauth2.googleapis.com/token', registrationEndpoint:'', scopes:['https://www.googleapis.com/auth/spreadsheets']},
   stats:{installCount:'55k', weeklyCalls:'980k', uptimePct:99.7, latencyP50Ms:360},
   links:{homepage:'https://sheets.google.com', repository:'google/mcp-sheets', license:'Apache-2.0'},
   state:'catalog-only'},
  {slug:'supabase', name:'Supabase', publisher:'Supabase', verified:true, category:'data',
   logo:'▲', tags:['header'],
   short:'Search Supabase docs, troubleshoot errors, and manage projects & schema.',
   defaultBaseUrl:'https://mcp.supabase.com/mcp', transport:'streamable-http', authType:'header',
   authConfig:{keyDefinitions:[{placement:'header', name:'Authorization', hint:'Bearer <SUPABASE_ACCESS_TOKEN>'}]},
   stats:{installCount:'6.6k', weeklyCalls:'72k', uptimePct:98.8, latencyP50Ms:480},
   links:{homepage:'https://supabase.com', repository:'supabase/mcp-supabase', license:'Apache-2.0'},
   state:'approved'},
  {slug:'jina', name:'Jina AI', publisher:'Jina AI', verified:true, category:'ai',
   logo:'J', tags:['header'],
   short:'AI-powered search + retrieval: read pages, extract structured data, re-rank.',
   defaultBaseUrl:'https://mcp.jina.ai/mcp', transport:'streamable-http', authType:'header',
   authConfig:{keyDefinitions:[{placement:'header', name:'Authorization', hint:'Bearer <JINA_KEY>'}]},
   stats:{installCount:'3.3k', weeklyCalls:'58k', uptimePct:97.9, latencyP50Ms:520},
   links:{homepage:'https://jina.ai', repository:'jina-ai/jina-mcp', license:'Apache-2.0'},
   state:'catalog-only'},
  {slug:'context7', name:'Context7', publisher:'Upstash', verified:true, category:'memory',
   logo:'⊙', tags:['header','featured'],
   short:'Fetch up-to-date, version-specific docs and code examples into your prompts.',
   defaultBaseUrl:'https://mcp.context7.com/mcp', transport:'streamable-http', authType:'none',
   authConfig:{},
   stats:{installCount:'12.8k', weeklyCalls:'320k', uptimePct:99.1, latencyP50Ms:240},
   links:{homepage:'https://context7.com', repository:'upstash/context7-mcp', license:'MIT'},
   state:'disabled'},
  {slug:'firecrawl', name:'Firecrawl', publisher:'Mendable', verified:false, category:'search',
   logo:'🜂', tags:['header'],
   short:'Scrape and crawl any site into LLM-friendly markdown or structured data.',
   defaultBaseUrl:'https://mcp.firecrawl.dev/v2/mcp', transport:'streamable-http', authType:'header',
   authConfig:{keyDefinitions:[{placement:'header', name:'Authorization', hint:'Bearer <FIRECRAWL_KEY>'}]},
   stats:{installCount:'2.8k', weeklyCalls:'41k', uptimePct:96.4, latencyP50Ms:720},
   links:{homepage:'https://firecrawl.dev', repository:'mendableai/firecrawl-mcp-server', license:'MIT'},
   state:'catalog-only'},
];

// Admin Registered-servers fixture.
const MCP_SERVERS_FIXTURE = [
  {slug:'notion',    name:'notion',    url:'https://mcp.notion.com/mcp',    authType:'oauth2', enabled:true,  instances:4, approvedBy:'admin', approvedAt:'2026-04-02'},
  {slug:'linear',    name:'linear',    url:'https://mcp.linear.app/mcp',    authType:'oauth2', enabled:true,  instances:2, approvedBy:'admin', approvedAt:'2026-04-05'},
  {slug:'exa',       name:'exa',       url:'https://mcp.exa.ai/mcp',        authType:'header', enabled:true,  instances:6, approvedBy:'admin', approvedAt:'2026-03-30'},
  {slug:'github',    name:'github',    url:'https://api.github.com/mcp',    authType:'oauth2', enabled:true,  instances:1, approvedBy:'admin', approvedAt:'2026-04-10'},
  {slug:'supabase',  name:'supabase',  url:'https://mcp.supabase.com/mcp',  authType:'header', enabled:true,  instances:0, approvedBy:'admin', approvedAt:'2026-04-14'},
  {slug:'context7',  name:'context7',  url:'https://mcp.context7.com/mcp',  authType:'none',   enabled:false, instances:3, approvedBy:'admin', approvedAt:'2026-02-20'},
];

// User's MCP instances.
const MCP_INSTANCES_FIXTURE = [
  {slug:'deepwiki', name:'deepwiki', url:'https://mcp.deepwiki.com/mcp', authState:'connected',     serverSlug:'deepwiki', lastUsedAt:'12m ago'},
  {slug:'exa',      name:'exa',      url:'https://mcp.exa.ai/mcp',       authState:'connected',     serverSlug:'exa',      lastUsedAt:'3h ago'},
  {slug:'notion',   name:'notion',   url:'https://mcp.notion.com/mcp',   authState:'connected',     serverSlug:'notion',   lastUsedAt:'yesterday'},
  {slug:'gmail-a',  name:'gmail-a',  url:'https://mcp.google.com/gmail', authState:'needs_reauth',  serverSlug:'gmail',    lastUsedAt:'2 days ago'},
];

// Pending approvals in Admin inbox.
const MCP_APPROVAL_FIXTURE = [
  {id:'req-1', slug:'slack',   name:'Slack',   requester:'arjun@bodhi.ai',  requestedAt:'2026-04-18', reason:'Channel automation for sprint syncs'},
  {id:'req-2', slug:'firecrawl', name:'Firecrawl', requester:'priya@bodhi.ai', requestedAt:'2026-04-19', reason:'Need to pull docs into RAG pipeline'},
];

// Per-server tool fixture — drives Playground + Drawer Capabilities tab.
const MCP_TOOLS_FIXTURE = {
  notion: [
    {name:'notion-search', desc:'Perform a search across Notion — "internal" Search api.',
     parameters:[{name:'query',type:'string',hint:'Search query'},{name:'filter',type:'object',hint:'Optional property filter'}]},
    {name:'notion-fetch', desc:'Retrieves details about a Notion entity (page, database, block).',
     parameters:[{name:'id',type:'string',hint:'Page or database UUID'}]},
    {name:'notion-create-pages', desc:'Overview: creates one or more Notion pages with given properties.',
     parameters:[{name:'pages',type:'array',hint:'List of page property objects'}]},
    {name:'notion-update-page', desc:'Overview: update a Notion page properties.',
     parameters:[{name:'id',type:'string',hint:'Page UUID'},{name:'properties',type:'object',hint:'Patch object'}]},
    {name:'notion-get-users', desc:'Retrieves a list of users in the current workspace.',
     parameters:[{name:'query',type:'string',hint:'Optional name filter'},{name:'page_size',type:'number',hint:'Max 100'},{name:'user_id',type:'string',hint:'Specific user; "self" for current'}]},
    {name:'notion-move-pages', desc:'Move one or more Notion pages or databases to a new parent.',
     parameters:[{name:'ids',type:'array'},{name:'parentId',type:'string'}]},
    {name:'notion-create-database', desc:'Create a new Notion database.',
     parameters:[{name:'title',type:'string'},{name:'properties',type:'object'}]},
  ],
  linear: [
    {name:'linear-list-issues', desc:'List issues in the current workspace with filters.',
     parameters:[{name:'teamId',type:'string'},{name:'state',type:'string',hint:'e.g. open, done'}]},
    {name:'linear-create-issue', desc:'Create a new Linear issue.',
     parameters:[{name:'teamId',type:'string'},{name:'title',type:'string'},{name:'description',type:'string'}]},
    {name:'linear-update-issue', desc:'Patch an existing Linear issue.',
     parameters:[{name:'id',type:'string'},{name:'patch',type:'object'}]},
  ],
  exa: [
    {name:'web_search_exa', desc:'Fast semantic web search with neural ranking.',
     parameters:[{name:'query',type:'string'},{name:'numResults',type:'number',hint:'default 10'}]},
    {name:'get_code_context_exa', desc:'Pulls code-relevant context for a coding agent.',
     parameters:[{name:'task',type:'string'}]},
    {name:'crawling_exa', desc:'Crawl a URL and return structured contents.',
     parameters:[{name:'url',type:'string'},{name:'maxCharacters',type:'number'}]},
  ],
};

// Filter chip row (category).
const McpCategoryChipRow = ({active='all', onChange}) => (
  <div className="mcp-category-chips">
    {MCP_CATEGORIES.map(c => (
      <Chip key={c.code} on={c.code===active} tone="indigo" onClick={() => onChange && onChange(c.code)}>
        <span style={{opacity:0.6, marginRight:3}}>{c.icon}</span>{c.label}
      </Chip>
    ))}
  </div>
);

// Status pill group — filters cards by derived state.
const McpStatusFilter = ({active='all', onChange}) => {
  const items = [
    {k:'all',          label:'All'},
    {k:'approved',     label:'Approved'},
    {k:'connected',    label:'Connected'},
    {k:'catalog-only', label:'Not connected'},
    {k:'pending-approval', label:'Pending'},
  ];
  return (
    <div className="mcp-status-filter">
      {items.map(i => (
        <span key={i.k} className={`mcp-status-pill${active===i.k?' active':''}`}
              onClick={() => onChange && onChange(i.k)}>{i.label}</span>
      ))}
    </div>
  );
};

// Compute the visible CTA on a card given the derived state and the viewer's role.
const mcpCardCta = ({state, role='user'}) => {
  if (state === 'catalog-only')    return role === 'admin' ? {label:'One-click Add to app', tone:'indigo'} : {label:'Submit for Approval', tone:'indigo'};
  if (state === 'pending-approval') return role === 'admin' ? {label:'Review request', tone:'saff'} : {label:'Request pending…', tone:'warn', disabled:true};
  if (state === 'approved')        return {label:'+ Add MCP Server', tone:'indigo'};
  if (state === 'connected')       return {label:'View instance ↗', tone:'leaf'};
  if (state === 'disabled')        return role === 'admin' ? {label:'Re-enable', tone:''} : {label:'Unavailable', tone:'', disabled:true};
  return {label:'…', tone:''};
};

// Discover grid card.
const McpCatalogCard = ({entry, role='user', instance, onCta, onOpen}) => {
  const cta = mcpCardCta({state:entry.state, role});
  const stateClass = `state-${entry.state}`;
  return (
    <div className={`mcp-card ${stateClass}`} onClick={onOpen}>
      <div className="mcp-card-head">
        <div className="mcp-card-logo">{entry.logo}</div>
        <div style={{flex:1, minWidth:0}}>
          <div className="mcp-card-title">{entry.name}</div>
          <div className="mcp-card-publisher">{entry.publisher}{entry.verified && <span title="verified">✓</span>}</div>
        </div>
        <Chip tone="indigo" style={{fontSize:9}}>{MCP_CATEGORIES.find(c=>c.code===entry.category)?.label || entry.category}</Chip>
      </div>
      <div className="mcp-card-short">{entry.short}</div>
      <div className="mcp-card-metrics">
        <span className="mcp-card-metric">⚒ {(MCP_TOOLS_FIXTURE[entry.slug]?.length || entry.stats?.installCount) ? `${MCP_TOOLS_FIXTURE[entry.slug]?.length || '?'} tools` : '—'}</span>
        <span className="mcp-card-metric">↓ {entry.stats.installCount}</span>
        <span className="mcp-card-metric">{entry.authType === 'oauth2' ? '🔐 OAuth' : entry.authType === 'header' ? '🔑 Key' : entry.authType === 'none' ? '· No auth' : ''}</span>
      </div>
      {entry.state === 'connected' && instance && (
        <div className="mcp-card-inline-instance">
          <span>◉ <strong>{instance.name}</strong> · {instance.authState === 'needs_reauth' ? '⚠ needs reauth' : 'connected'}</span>
          <span style={{color:'var(--ink-3)'}}>{instance.lastUsedAt}</span>
        </div>
      )}
      <div className="mcp-card-cta">
        <span style={{fontSize:10, color:'var(--ink-3)'}}>
          {entry.state === 'pending-approval' ? 'submitted 1d ago' :
           entry.state === 'disabled' ? 'Admin disabled' :
           entry.state === 'approved' ? '✓ admin-approved' :
           entry.state === 'connected' ? `instance · ${instance?.lastUsedAt || ''}` :
           'not yet in this app'}
        </span>
        <Btn variant={cta.tone === 'indigo' ? 'primary' : cta.tone} size="xs"
             onClick={(e)=>{e.stopPropagation(); onCta && onCta(entry, cta);}}
             title={cta.disabled ? 'Disabled' : undefined}>
          {cta.label}
        </Btn>
      </div>
    </div>
  );
};

// Detail drawer — About / Capabilities / Connection / Metadata / Performance.
const McpCatalogDrawer = ({entry, activeTab='capabilities'}) => {
  if (!entry) return null;
  const tools = MCP_TOOLS_FIXTURE[entry.slug] || [];
  return (
    <aside className="mcp-drawer">
      <div className="mcp-drawer-head">
        <div className="mcp-card-logo" style={{width:24, height:24, fontSize:13}}>{entry.logo}</div>
        <div style={{flex:1, minWidth:0}}>
          <div className="mcp-card-title">{entry.name}</div>
          <div className="mcp-card-publisher">{entry.publisher}{entry.verified && ' ✓'}</div>
        </div>
      </div>
      <div className="mcp-drawer-tabs">
        {['About','Capabilities','Connection','Metadata','Performance'].map(t => (
          <span key={t} className={`mcp-drawer-tab${(activeTab||'').toLowerCase()===t.toLowerCase()?' active':''}`}>{t}</span>
        ))}
      </div>
      <div className="mcp-drawer-section">
        <h5>Tools ({tools.length})</h5>
        {tools.slice(0,4).map(t => (
          <div key={t.name} className="mcp-tool-list-item">
            <strong>{t.name}</strong>
            <div style={{color:'var(--ink-3)'}}>{t.desc}</div>
          </div>
        ))}
        {tools.length > 4 && <div style={{fontSize:10, color:'var(--ink-3)'}}>+ {tools.length - 4} more…</div>}
      </div>
      <div className="mcp-drawer-section">
        <h5>Connection</h5>
        <div><code style={{fontSize:9.5}}>{entry.defaultBaseUrl}</code></div>
        <div>transport: <code>{entry.transport}</code></div>
        <div>auth: <code>{entry.authType}</code></div>
      </div>
      <div className="mcp-drawer-section">
        <h5>Stats (30d)</h5>
        <div>installs · {entry.stats.installCount}</div>
        <div>calls · {entry.stats.weeklyCalls}/wk</div>
        <div>uptime · {entry.stats.uptimePct}%</div>
        <div>p50 · {entry.stats.latencyP50Ms}ms</div>
      </div>
      <div className="mcp-drawer-section">
        <h5>Metadata</h5>
        <div>license · {entry.links.license}</div>
        <div>repo · <code style={{fontSize:9.5}}>{entry.links.repository}</code></div>
      </div>
    </aside>
  );
};

// Header/Query auth key definitions block.
const McpAuthHeaderConfig = ({value={keyDefinitions:[]}, onChange}) => {
  const rows = value.keyDefinitions || [{placement:'header', name:'Authorization', hint:''}];
  return (
    <div className="mcp-auth-block">
      <div className="sm" style={{fontWeight:700, marginBottom:4}}>Key definitions</div>
      {rows.map((r, i) => (
        <div key={i} className="mcp-auth-key-row">
          <select defaultValue={r.placement} style={{fontFamily:'var(--hand)', fontSize:11, padding:'3px 4px', border:'1.3px solid var(--ink)', borderRadius:4}}>
            <option value="header">Header</option>
            <option value="query">Query</option>
          </select>
          <Field filled value={r.name} style={{margin:0}}/>
          <button title="Remove" style={{background:'none', border:'none', color:'var(--destructive)', cursor:'pointer'}}>🗑</button>
        </div>
      ))}
      <Btn size="xs">+ Add Key</Btn>
    </div>
  );
};

// OAuth2 auth config block.
const McpAuthOAuthConfig = ({value={}, onChange}) => {
  const cfg = {
    registrationType: value.registrationType || 'dynamic',
    authorizationEndpoint: value.authorizationEndpoint || '',
    tokenEndpoint: value.tokenEndpoint || '',
    registrationEndpoint: value.registrationEndpoint || '',
    scopes: (value.scopes || []).join(' '),
  };
  return (
    <div className="mcp-auth-block">
      <div className="sm" style={{fontWeight:700, marginBottom:4}}>Registration type</div>
      <select defaultValue={cfg.registrationType}
              style={{fontFamily:'var(--hand)', fontSize:12, padding:'4px 6px', width:'100%', border:'1.3px solid var(--ink)', borderRadius:6}}>
        <option value="dynamic">Dynamic Registration</option>
        <option value="metadata">Metadata Document</option>
        <option value="preregistered">Pre-registered</option>
      </select>
      <div className="mcp-auth-endpoint" style={{marginTop:6}}>
        <span className="sm" style={{fontWeight:700}}>Authorization Endpoint</span>
        <Field filled value={cfg.authorizationEndpoint}/>
      </div>
      <div className="mcp-auth-endpoint">
        <span className="sm" style={{fontWeight:700}}>Token Endpoint</span>
        <Field filled value={cfg.tokenEndpoint}/>
      </div>
      <div className="mcp-auth-endpoint">
        <span className="sm" style={{fontWeight:700}}>Registration Endpoint</span>
        <Field filled value={cfg.registrationEndpoint}/>
      </div>
      <div className="mcp-auth-endpoint">
        <span className="sm" style={{fontWeight:700}}>Scopes (Optional)</span>
        <Field filled value={cfg.scopes} hint="Space-separated, e.g. mcp:tools mcp:read"/>
      </div>
    </div>
  );
};

// Server registry form — usable standalone (Admin) or inside OverlayShell (Discover).
const McpServerForm = ({mode='blank', initial={}, compact=false}) => {
  const [authType, setAuthType] = React.useState(initial.authType || 'oauth2');
  const isPrefill = mode === 'prefilled';
  return (
    <div className="mcp-server-form" style={{fontSize: compact ? 11 : 12}}>
      <div className="mcp-form-section-head">1 · Server connection</div>
      <div style={{marginTop:4}}>
        <Field label={<span>URL <Chip tone="warn" style={{fontSize:9, marginLeft:4}}>required</Chip></span>}
               filled value={initial.url || 'https://mcp.linear.app/mcp'}/>
      </div>
      <div style={{marginTop:6}}>
        <Field label="Name" filled value={initial.name || 'linear'}/>
      </div>
      <div style={{marginTop:6}}>
        <Field label="Description" filled value={initial.description || 'Manage issues, projects, cycles.'} ta/>
      </div>
      <div className="mcp-toggle-field" style={{marginTop:6}}>
        <span className="mcp-toggle-check on">✓</span>
        <div>
          <div className="sm" style={{fontWeight:700}}>Enabled</div>
          <div className="sm" style={{fontSize:10, color:'var(--ink-3)'}}>Users can create instances of this server</div>
        </div>
      </div>

      <div className="mcp-form-section-head">2 · Authentication</div>
      <div className="sm" style={{fontWeight:700, marginBottom:4}}>Auth type</div>
      <div style={{display:'flex', gap:4, marginBottom:6}}>
        {['oauth2','header','query','none'].map(t => (
          <Chip key={t} on={authType===t} tone="indigo" onClick={() => setAuthType(t)}>{t}</Chip>
        ))}
      </div>
      {authType === 'oauth2' && <McpAuthOAuthConfig value={initial.authConfig || {}}/>}
      {(authType === 'header' || authType === 'query') && <McpAuthHeaderConfig value={initial.authConfig || {keyDefinitions:[{placement:authType, name:'Authorization'}]}}/>}
      {authType === 'none' && <Callout style={{position:'static', display:'inline-block', marginTop:6}}>No credentials needed — public server.</Callout>}

      {isPrefill && (
        <Callout style={{position:'static', marginTop:8, background:'var(--indigo-soft)'}}>
          ★ Pre-filled from catalog entry · admin can review and Save in one click
        </Callout>
      )}
    </div>
  );
};

// Instance form — for the user to create / edit an instance of an approved server.
const McpInstanceForm = ({initial={}, compact=false, serverOptions}) => {
  const servers = serverOptions || MCP_SERVERS_FIXTURE.filter(s => s.enabled);
  const selected = initial.serverSlug || 'linear';
  const server = servers.find(s => s.slug === selected) || servers[0];
  return (
    <div className="mcp-instance-form" style={{fontSize: compact ? 11 : 12}}>
      <div className="mcp-form-section-head">1 · Pick a registered server</div>
      <select defaultValue={selected} style={{fontFamily:'var(--hand)', fontSize:12, padding:'5px 6px', width:'100%', border:'1.3px solid var(--ink)', borderRadius:6, background:'var(--paper)'}}>
        {servers.map(s => (
          <option key={s.slug} value={s.slug}>{s.name} — {s.url}</option>
        ))}
      </select>
      <div className="sm" style={{fontSize:10, color:'var(--ink-3)', marginTop:3}}>{server?.url}</div>

      <div className="mcp-form-section-head">2 · Name this instance</div>
      <div style={{marginTop:4}}>
        <Field label="Name" filled value={initial.name || server.slug}/>
      </div>
      <div style={{marginTop:6}}>
        <Field label={<span>Slug <Chip tone="warn" style={{fontSize:9, marginLeft:4}}>unique</Chip></span>} filled value={initial.slug || server.slug}/>
        <span className="sm" style={{fontSize:10, color:'var(--ink-3)'}}>Used in tool names: <code>mcp_{initial.slug || server.slug}__tool</code></span>
      </div>
      <div style={{marginTop:6}}>
        <Field label="Description (optional)" filled value={initial.description || ''} ta/>
      </div>

      <div className="mcp-toggle-field" style={{marginTop:6}}>
        <span className="mcp-toggle-check on">✓</span>
        <div className="sm" style={{fontWeight:700}}>Enable MCP</div>
      </div>

      <div className="mcp-form-section-head">3 · Authentication</div>
      {server.authType === 'oauth2' ? (
        <div className="mcp-auth-block">
          <div className="sm" style={{fontWeight:700}}>OAuth ({server.authType === 'oauth2' ? 'OAuth' : 'header'})</div>
          <div className="sm" style={{fontSize:11, color:'var(--ink-3)', marginBottom:6}}>OAuth authentication is required. Click Connect to authorize.</div>
          {initial.authState === 'connected' ? (
            <div style={{padding:6, background:'var(--leaf-soft)', border:'1.3px solid var(--leaf)', borderRadius:6, display:'flex', justifyContent:'space-between', alignItems:'center'}}>
              <div>
                <div><Chip tone="leaf">● Connected</Chip></div>
                <div className="sm" style={{fontSize:10, color:'var(--ink-3)'}}>Client ID: nThkxKhhePfB92ss</div>
                <div className="sm" style={{fontSize:10, color:'var(--ink-3)'}}>Auth Server: {server.url.replace('/mcp','')}</div>
              </div>
              <Btn size="xs">🔌 Disconnect</Btn>
            </div>
          ) : (
            <Btn variant="primary" size="xs">Connect via OAuth ↗</Btn>
          )}
        </div>
      ) : server.authType === 'header' ? (
        <div className="mcp-auth-block">
          <div className="sm" style={{fontWeight:700, marginBottom:4}}>Provide credentials</div>
          <Field label="Authorization" filled value="••••••••••••••••••••••" right="👁"/>
        </div>
      ) : (
        <Callout style={{position:'static', display:'inline-block', marginTop:6}}>No credentials needed — public server.</Callout>
      )}
    </div>
  );
};

// Table row — admin registry list.
const McpServerListRow = ({server}) => (
  <div className="mcp-row mcp-row-server">
    <span>{server.authType === 'oauth2' ? '🔐' : server.authType === 'header' ? '🔑' : '·'}</span>
    <div>
      <div style={{fontWeight:700}}>{server.name}</div>
      <div className="sm" style={{fontSize:10, color:'var(--ink-3)'}}>approved {server.approvedAt} · {server.authType}</div>
    </div>
    <code style={{fontSize:10.5}}>{server.url}</code>
    <Chip tone={server.enabled ? 'leaf' : ''}>{server.enabled ? 'enabled' : 'disabled'}</Chip>
    <span className="sm">{server.instances} inst</span>
    <div className="mcp-actions">
      <Btn size="xs">✎</Btn>
      <Btn size="xs">{server.enabled ? '⏸' : '▶'}</Btn>
      <Btn size="xs" variant="">🗑</Btn>
    </div>
  </div>
);

// Approval inbox row.
const McpApprovalRow = ({request}) => (
  <div className="mcp-row mcp-row-approval" style={{background:'var(--warn-soft)'}}>
    <span>✉</span>
    <div>
      <div style={{fontWeight:700}}>{request.name}</div>
      <div className="sm" style={{fontSize:10, color:'var(--ink-3)'}}>by {request.requester} · {request.requestedAt}</div>
    </div>
    <div className="sm" style={{color:'var(--ink-2)'}}>{request.reason}</div>
    <div className="mcp-actions">
      <Btn size="xs">✗ Reject</Btn>
      <Btn variant="primary" size="xs">✓ Approve</Btn>
    </div>
  </div>
);

// Instance list row — My MCPs.
const McpInstanceListRow = ({instance}) => (
  <div className="mcp-row mcp-row-instance">
    <span>🔌</span>
    <div>
      <div style={{fontWeight:700}}>{instance.name}</div>
      <div className="sm" style={{fontSize:10, color:'var(--ink-3)'}}>last used · {instance.lastUsedAt}</div>
    </div>
    <code style={{fontSize:10.5}}>{instance.url}</code>
    {instance.authState === 'needs_reauth'
      ? <Chip tone="warn">⚠ reauth</Chip>
      : <Chip tone="leaf">● active</Chip>}
    <div className="mcp-actions">
      <Btn size="xs">▷</Btn>
      <Btn size="xs">✎</Btn>
      <Btn size="xs" variant="">🗑</Btn>
    </div>
  </div>
);

// Pending banner on My MCPs.
const McpInstancePendingBanner = ({pending=[]}) => (
  <div className="mcp-pending-banner">
    <span>⏳ {pending.length} pending approval{pending.length !== 1 ? 's' : ''}: {pending.map(p => p.name).join(', ')}</span>
    <Btn size="xs">View</Btn>
  </div>
);

// Playground — tool sidebar.
const McpToolSidebar = ({tools=[], selected, search='', onSearch, onSelect, instanceName='notion'}) => (
  <aside className="mcp-tool-sidebar">
    <div className="mcp-tool-sidebar-head">
      <Chip tone="leaf" style={{fontSize:9}}>● connected</Chip> {instanceName}
    </div>
    <Field filled value={search} hint="🔍 Search tools"/>
    {tools.map(t => (
      <div key={t.name} className={`mcp-tool-sidebar-item${selected===t.name?' active':''}`}
           onClick={() => onSelect && onSelect(t.name)}>
        <div className="mcp-tool-name">{t.name}</div>
        <div className="mcp-tool-desc">{t.desc.slice(0, 60)}{t.desc.length > 60 ? '…' : ''}</div>
      </div>
    ))}
  </aside>
);

// Playground — main executor (form + Execute + response tabs).
const McpToolExecutor = ({tool, activeResponseTab='success'}) => {
  if (!tool) return <div className="mcp-tool-main"><em>Select a tool from the sidebar.</em></div>;
  return (
    <div className="mcp-tool-main">
      <div className="h3" style={{margin:0, fontSize:14}}>{tool.name}</div>
      <div className="sm" style={{color:'var(--ink-3)', marginBottom:6}}>{tool.desc}</div>
      <div className="api-format-picker" style={{display:'flex', gap:4, marginBottom:6}}>
        <Chip on tone="indigo">Form</Chip>
        <Chip>JSON</Chip>
      </div>
      {(tool.parameters || []).map(p => (
        <div key={p.name} style={{marginBottom:6}}>
          <Field label={<span><strong>{p.name}</strong> <span style={{color:'var(--ink-3)', fontWeight:400}}>({p.type})</span> <span className="sm" style={{fontSize:10, color:'var(--ink-3)'}}>{p.hint || ''}</span></span>} filled value=""/>
        </div>
      ))}
      <Btn variant="primary">Execute</Btn>
      <div className="mcp-response-tabs">
        {['Success','Response','Raw JSON','Request'].map(t => (
          <span key={t} className={`mcp-response-tab${activeResponseTab.toLowerCase().replace(' ','')===t.toLowerCase().replace(' ','')?' active':''}`}>
            {t === 'Success' && <Chip tone="leaf" style={{fontSize:9, marginRight:3}}>●</Chip>}{t}
          </span>
        ))}
      </div>
      <div className="mcp-response-body">{`[\n  {\n    "type": "text",\n    "text": "{\\"results\\":[{\\"type\\":\\"person\\",\\"id\\":\\"73b3a1b1-…\\"}\n  }\n]`}</div>
    </div>
  );
};

// Sticky rail nav (Admin / Playground Desktop).
const McpRail = ({active='servers', items}) => {
  const defaultItems = [
    {k:'servers',   label:'Registered'},
    {k:'approvals', label:'Approvals'},
  ];
  const use = items || defaultItems;
  return (
    <aside className="mcp-rail">
      <div className="mcp-rail-head">sections</div>
      {use.map(i => (
        <div key={i.k} className={`mcp-rail-item${active===i.k?' active':''}`}>{i.label}</div>
      ))}
    </aside>
  );
};

// Top-of-page anchor strip (Medium variants).
const McpMediumAnchors = ({active='servers', items}) => {
  const defaultItems = [
    {k:'servers',   label:'Registered'},
    {k:'approvals', label:'Approvals'},
  ];
  const use = items || defaultItems;
  return (
    <div className="mcp-medium-anchors">
      {use.map(i => (
        <span key={i.k} className={`mcp-medium-anchor${active===i.k?' active':''}`}>{i.label}</span>
      ))}
    </div>
  );
};

// ═════════════════ Access-request primitives (v31) ═════════════════

// One representative 3rd-party app request. Capability envelope + suggested models
// + 4 requested MCPs covering the row-state matrix.
const ACCESS_REQUEST_FIXTURE = {
  app: {
    clientId: 'bodhi-app-f181a4d1-d7af-43f4-965a-0a8efd453d86',
    name: 'Research Copilot',
    publisher: 'Pebble Labs',
    verified: true,
    logo: 'R',
    description: 'An agent that helps you summarise research papers, organise Notion pages, and pull market data from web search.',
    homepage: 'https://research-copilot.pebble.ai',
  },
  caps: {
    required: ['tool-use', 'text-to-text'],
    preferred: ['vision'],
    minContext: 128_000,
    maxCostUsdPerMTok: 2.5,
  },
  suggestedModels: ['openai/gpt-5-mini', 'my-qwen-long', 'anthropic/claude-sonnet-4-6'],
  mcps: [
    {slug:'exa',     rowState:'has-instance',   reason:'User already has an exa instance · instance=exa · active'},
    {slug:'notion',  rowState:'needs-reauth',   reason:'Instance exists but token expired · user must reconnect'},
    {slug:'linear',  rowState:'needs-instance', reason:'Admin has registered Linear · user needs to run OAuth'},
    {slug:'gmail',   rowState:'needs-server',   reason:'No registry entry yet · admin can one-click Add'},
  ],
};

// User's configured models — drives the Model access section. Shape deliberately mirrors
// the Models page row-shape: kind (alias/api-model/provider) + caps + cost + ctx + origin.
const ACCESS_MODELS_FIXTURE = [
  {kind:'alias',  name:'my-qwen-long',  title:'my-qwen-long',   caps:['text2text','tool-use','long-ctx'], ctx:128_000, cost:null,     origin:'local'},
  {kind:'alias',  name:'my-gemma',      title:'my-gemma',       caps:['text2text'],                        ctx:32_768,  cost:null,     origin:'local'},
  {kind:'alias',  name:'my-qwen-coder', title:'my-qwen-coder',  caps:['text2text','tool-use','coding'],    ctx:32_768,  cost:null,     origin:'local'},
  {kind:'api',    name:'openai/gpt-5-mini', title:'openai/gpt-5-mini', caps:['text2text','tool-use','vision'], ctx:400_000, cost:{in:0.25, out:2.0, unit:'M'}, origin:'openai'},
  {kind:'api',    name:'openai/gpt-4-turbo', title:'openai/gpt-4-turbo', caps:['text2text','tool-use'],    ctx:128_000, cost:{in:10, out:30, unit:'M'}, origin:'openai'},
  {kind:'api',    name:'anthropic/claude-sonnet-4-6', title:'anthropic/claude-sonnet-4-6', caps:['text2text','tool-use','vision'], ctx:200_000, cost:{in:3, out:15, unit:'M'}, origin:'anthropic'},
  {kind:'provider', name:'anthropic/claude-opus-4-6', title:'anthropic/claude-opus-4-6', caps:['text2text','tool-use','vision','reasoning'], ctx:200_000, cost:{in:15, out:75, unit:'M'}, origin:'anthropic (provider)'},
  {kind:'alias',  name:'my-embed',      title:'my-embed',       caps:['embed'],                            ctx:8_192,   cost:null,     origin:'local'},
];

// Compute per-model access-row state given the envelope.
const accessModelState = (model, caps, suggested=[]) => {
  const matchesTool = !caps.required.includes('tool-use') || model.caps.includes('tool-use');
  const matchesCtx  = !caps.minContext || model.ctx >= caps.minContext;
  const matchesText = !caps.required.includes('text-to-text') || model.caps.includes('text2text');
  const fitsCost    = !caps.maxCostUsdPerMTok || !model.cost || model.cost.out <= caps.maxCostUsdPerMTok * 20;
  const isSuggested = suggested.includes(model.name);
  if (!matchesTool || !matchesText) return 'below-envelope';
  if (!matchesCtx) return 'below-envelope';
  if (isSuggested) return 'app-suggested';
  if (matchesTool && matchesCtx && fitsCost) return 'matches-envelope';
  return 'user-config';
};

// ── primitives ──────────────────────────────────────────────────
const AccessRequestHeader = ({request, viewerRole='user', onRoleChange}) => {
  const {app} = request;
  return (
    <div className="access-header">
      <div className="access-logo">{app.logo}</div>
      <div style={{flex:1, minWidth:0}}>
        <div className="access-app-title">
          {app.name}
          {app.verified && <Chip tone="leaf" style={{fontSize:9}}>✓ verified</Chip>}
          <Chip tone="indigo" style={{fontSize:9}}>3rd-party app</Chip>
        </div>
        <div className="access-app-sub">is requesting access to your resources.</div>
        <div className="access-client-id" title="client id">{app.clientId}</div>
        <div className="sm" style={{fontSize:11, color:'var(--ink-2)', marginTop:5}}>{app.description}</div>
      </div>
      <div className="access-role-toggle">
        <span className={viewerRole==='user'?'active':''} onClick={() => onRoleChange && onRoleChange('user')}>User</span>
        <span className={viewerRole==='admin'?'active':''} onClick={() => onRoleChange && onRoleChange('admin')}>Admin</span>
      </div>
    </div>
  );
};

const AccessCapsEnvelope = ({caps}) => (
  <div className="access-caps-row">
    {caps.required.map(c => <Chip key={c} on tone="indigo" style={{fontSize:10}}>⚒ {c}</Chip>)}
    {caps.preferred.map(c => <Chip key={c} style={{fontSize:10}}>preferred · {c}</Chip>)}
    {caps.minContext && <Chip style={{fontSize:10}}>min-ctx · {(caps.minContext/1000).toFixed(0)}k</Chip>}
    {caps.maxCostUsdPerMTok && <Chip style={{fontSize:10}}>max-cost · ${caps.maxCostUsdPerMTok}/MTok</Chip>}
  </div>
);

const AccessModelRow = ({model, state, checked, onChange}) => {
  const reasonChip = state === 'app-suggested' ? <Chip tone="leaf" style={{fontSize:9}}>★ app-suggested</Chip> :
                     state === 'matches-envelope' ? <Chip tone="indigo" style={{fontSize:9}}>matches</Chip> :
                     state === 'below-envelope' ? <Chip tone="warn" style={{fontSize:9}}>below envelope</Chip> :
                     null;
  const rowCls = `access-model-row ${state === 'app-suggested' ? 'suggested' : state === 'below-envelope' ? 'below-envelope disabled' : ''}`;
  return (
    <div className={rowCls}>
      <span className="mcp-toggle-check" style={{background: checked ? 'var(--indigo-soft)' : 'var(--paper)'}}>{checked ? '✓' : ''}</span>
      <div className="access-model-body">
        <span className="access-model-title">{model.title}</span>
        <div className="access-model-chips">
          {model.caps.slice(0,3).map(c => <Chip key={c} style={{fontSize:9}}>{c}</Chip>)}
        </div>
        {reasonChip}
      </div>
      <div className="access-model-meta">
        {(model.ctx/1000).toFixed(0)}k · {model.cost ? `$${model.cost.in}/${model.cost.out} per M` : model.origin}
      </div>
    </div>
  );
};

const AccessModelGroup = ({title, models, caps, suggested, selected, onToggle}) => (
  <>
    <div className="access-model-group-head">
      <span>{title} · {models.length}</span>
      <span style={{color:'var(--ink-3)', fontSize:10, cursor:'pointer'}}>select all</span>
    </div>
    {models.map(m => {
      const state = accessModelState(m, caps, suggested);
      const checked = selected.includes(m.name);
      return <AccessModelRow key={m.name} model={m} state={state} checked={checked} onChange={() => onToggle(m.name)}/>;
    })}
  </>
);

const AccessMcpRow = ({server, instance, rowState, role='user', checked=true, inlineOpen=false, onToggleInline}) => {
  const entry = MCP_CATALOG_FIXTURE.find(e => e.slug === server.slug);
  const body = (() => {
    if (rowState === 'has-instance') {
      return (
        <select defaultValue={instance?.slug || server.slug}
                style={{fontFamily:'var(--hand)', fontSize:11, padding:'3px 6px', border:'1.3px solid var(--ink)', borderRadius:6, background:'var(--paper)'}}>
          <option>{instance ? `${instance.name} · ● active` : `${server.slug} · ● active`}</option>
        </select>
      );
    }
    if (rowState === 'needs-reauth') {
      return <><Chip tone="warn" style={{fontSize:10}}>⚠ Reconnect required</Chip><Btn size="xs" variant="primary" style={{marginLeft:4}}>Reconnect via OAuth ↗</Btn></>;
    }
    if (rowState === 'needs-instance') {
      return <><Btn variant="primary" size="xs">+ Connect instance (OAuth ↗)</Btn><span className="access-tool-hint" style={{display:'block', marginTop:3}}>opens OAuth in a popup · page auto-refreshes when done</span></>;
    }
    if (rowState === 'oauth-in-progress') {
      return <span className="access-oauth-hint"><span className="dot"/>Waiting for OAuth confirmation in popup…</span>;
    }
    if (rowState === 'needs-server') {
      return role === 'admin'
        ? (<><Btn variant="primary" size="xs" onClick={onToggleInline}>⚡ One-click Add MCP Server</Btn><span className="access-tool-hint" style={{display:'block', marginTop:3}}>pre-filled from Bodhi catalog · review and save without leaving this page</span></>)
        : (<><Btn size="xs">✉ Request admin to add</Btn><span className="access-tool-hint" style={{display:'block', marginTop:3}}>your admin gets a notification · reviewer can approve from Admin Inbox</span></>);
    }
    if (rowState === 'pending-admin') {
      return <Chip style={{fontSize:10}}>⏳ Admin notified · 2m ago</Chip>;
    }
  })();

  const toolPreview = (MCP_TOOLS_FIXTURE[server.slug] || []).slice(0,3).map(t => t.name).join(' · ');

  return (
    <div className={`access-mcp-row state-${rowState}`}>
      <div className="access-mcp-row-head">
        <span className="access-mcp-check mcp-toggle-check" style={{background: checked ? 'var(--indigo-soft)' : 'var(--paper)'}}>{checked ? '✓' : ''}</span>
        <span className="access-mcp-title">{entry?.name || server.slug}</span>
        <code className="access-mcp-url">{entry?.defaultBaseUrl || server.url}</code>
        <span style={{marginLeft:'auto', fontSize:10, color:'var(--ink-3)'}}>
          {rowState === 'has-instance' ? 'ready' :
           rowState === 'needs-reauth' ? 'reauth' :
           rowState === 'needs-instance' ? 'needs instance' :
           rowState === 'needs-server' ? 'not in app' :
           rowState === 'pending-admin' ? 'pending' :
           rowState === 'oauth-in-progress' ? 'authorising…' : ''}
        </span>
      </div>
      <div className="access-mcp-row-body">
        {body}
        {toolPreview && rowState !== 'has-instance' && <div className="access-tool-hint">Will use: {toolPreview}…</div>}
      </div>
      {inlineOpen && entry && role === 'admin' && rowState === 'needs-server' && (
        <AccessMcpInlineAddServer entry={entry} onCancel={onToggleInline}/>
      )}
    </div>
  );
};

const AccessMcpInlineAddServer = ({entry, onSave, onCancel}) => (
  <div className="access-mcp-inline-add">
    <div className="access-mcp-inline-add-head">
      <span>★ One-click Add · pre-filled from catalog: <code>{entry.slug}</code></span>
      <Btn size="xs" onClick={onCancel}>✗ Cancel</Btn>
    </div>
    <McpServerForm mode="prefilled" initial={{
      url: entry.defaultBaseUrl, name: entry.slug, description: entry.short,
      authType: entry.authType, authConfig: entry.authConfig,
    }} compact/>
    <div style={{display:'flex', justifyContent:'flex-end', gap:6, marginTop:8}}>
      <Btn size="xs" onClick={onCancel}>Cancel</Btn>
      <Btn variant="primary" size="xs">Save & continue</Btn>
    </div>
  </div>
);

const AccessOAuthHint = ({status='waiting'}) => (
  <span className="access-oauth-hint">
    <span className="dot"/>
    {status === 'waiting' ? 'Waiting for OAuth confirmation in popup…' :
     status === 'connected' ? '✓ Connected · instance created' :
     'OAuth failed · retry'}
  </span>
);

const AccessRoleSelect = ({value='User'}) => (
  <div className="access-role-row">
    <span className="sm" style={{fontWeight:700}}>Approved role</span>
    <select defaultValue={value}>
      <option>User</option>
      <option>PowerUser</option>
      <option>Admin</option>
    </select>
    <span className="sm" style={{fontSize:10, color:'var(--ink-3)'}}>drives what this app can see in your resources</span>
  </div>
);

const AccessActionBar = ({checkedModels=0, totalModels=0, checkedMcps=0, totalMcps=0, blockers=0}) => {
  const total = totalModels + totalMcps;
  const checked = checkedModels + checkedMcps;
  const disabled = blockers > 0 || checked === 0;
  return (
    <div className="access-action-bar">
      <Btn>Deny</Btn>
      <div style={{display:'flex', alignItems:'center', gap:8}}>
        {disabled && <span className="access-action-hint">
          {blockers > 0 ? `${blockers} MCP${blockers>1?'s':''} need setup before approving` :
                          'select at least one resource'}
        </span>}
        <Btn variant="primary" title={disabled ? 'resolve prerequisites first' : undefined}>
          Approve {checked} of {total} resources
        </Btn>
      </div>
    </div>
  );
};

Object.assign(window, {Ph, Lines, Chip, Btn, Field, TL, Stars, Bar, Crumbs, Browser, Variant, Callout, SectionHead, ModelRow, DownloadsPanel, DownloadsMenu, ModelListRow, MobileHeader, MobileMenu, TabletFrame, PhoneFrame, ParamSection, PresetChipRow, QuantPicker, FitCheckCard, LiveConfigJson, DownloadProgressStrip, SliderWithMarks, TaskCategoryGrid, TaskCategoryCard, BrowseBySelector, OverlayShell, AliasRail, AliasMediumAnchors, DEFAULT_QUANTS, DEFAULT_ALIAS_CONFIG, TASK_CATEGORIES, PRESETS, ALIAS_SECTIONS, ArgsEditor, ArgsPalette, ArgsHelpPop, ARGS_HELP, ARGS_PRESETS, DEFAULT_ARG_LINES, PRESET_CATALOGUE, PresetGrid, PresetAndArgsSection, ModeToggle, ModeToggleCaption, KindChipRow, ModelsAddBrowseMenu, RankedRow, RankedModeCaption, RANKED_FIXTURE_CODING, groupIntoRankedRows, API_FORMATS, FIXTURE_OPENAI_MODELS, ApiFormatPicker, ApiKeyField, PrefixField, ForwardingModeRadio, ModelMultiSelect, ApiRail, ApiMediumAnchors,
  // MCP v30
  MCP_CATEGORIES, MCP_CATALOG_FIXTURE, MCP_SERVERS_FIXTURE, MCP_INSTANCES_FIXTURE, MCP_APPROVAL_FIXTURE, MCP_TOOLS_FIXTURE,
  McpCategoryChipRow, McpStatusFilter, mcpCardCta, McpCatalogCard, McpCatalogDrawer,
  McpAuthHeaderConfig, McpAuthOAuthConfig, McpServerForm, McpInstanceForm,
  McpServerListRow, McpApprovalRow, McpInstanceListRow, McpInstancePendingBanner,
  McpToolSidebar, McpToolExecutor, McpRail, McpMediumAnchors,
  // Access-request v31
  ACCESS_REQUEST_FIXTURE, ACCESS_MODELS_FIXTURE, accessModelState,
  AccessRequestHeader, AccessCapsEnvelope, AccessModelRow, AccessModelGroup,
  AccessMcpRow, AccessMcpInlineAddServer, AccessOAuthHint,
  AccessRoleSelect, AccessActionBar});
