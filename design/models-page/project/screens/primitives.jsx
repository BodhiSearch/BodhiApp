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

const Btn = ({variant='', size='', children, style, title}) => (
  <button className={`btn ${variant} ${size}`} style={style} title={title}>{children}</button>
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

// Generic list-view row used by Hub + Discover when "☰ List" is selected.
// Accepts the same basic shape as ModelCard / DiscoverCard.
const ModelListRow = ({kind='file', title, subtitle, caps=[], meta, cost, status, fitLabel, fit, selected, onClick}) => {
  const kindTone =
    kind==='alias' ? 'saff' :
    kind==='provider' ? 'indigo' :
    kind==='provider-off' ? '' : 'leaf';
  const statusTone =
    status==='live' || status==='ready' || status==='fits' || status==='connected' ? 'leaf' :
    status==='oauth' ? 'saff' :
    status==='rate-limited' || status==='tight' ? 'warn' : '';
  const fitTone = fit==='green' ? 'leaf' : fit==='yellow' ? 'warn' : fit==='red' ? 'warn' : '';
  const extra = kind==='provider-off' ? ' dashed' : '';
  return (
    <div className={`model-list-row${selected?' selected':''}${extra}`} onClick={onClick}>
      <Chip tone={kindTone} style={{fontSize:10}}>{kind==='provider-off' ? 'provider' : kind}</Chip>
      <div className="mlr-title-cell">
        <div className="model-card-title" style={{fontSize:13, margin:0}}>{title}</div>
        {subtitle && <div className="sm">{subtitle}</div>}
      </div>
      <div className="mlr-caps-cell">
        {caps.map((c,i)=>(<Chip key={i}>{c}</Chip>))}
      </div>
      <div className="mlr-meta-cell sm">
        {cost && <div className="mlr-cost">{cost}</div>}
        {meta && <div>{meta}</div>}
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
const MobileMenu = ({active='My Models', withDownloads=false, dlCount=1}) => (
  <div className="m-menu-overlay">
    <div className="m-menu">
      <div className="m-menu-item">Chat</div>
      <div className="m-menu-item expanded">
        <div className="m-menu-container">
          <div style={{display:'flex', justifyContent:'space-between', alignItems:'center'}}>
            <span>Models</span><span>▾</span>
          </div>
          <div className="m-menu-sub">
            <div className={`m-menu-sub-item${active==='My Models'?' active':''}`}>My Models</div>
            <div className={`m-menu-sub-item${active==='Discover'?' active':''}`}>Discover</div>
          </div>
        </div>
      </div>
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

// Tablet-shaped frame used for the medium-width wireframes.
const TabletFrame = ({label, children}) => (
  <div className="tablet-frame">
    <div className="tablet-label">{label}</div>
    <div className="tablet-screen">
      <div className="tablet-content">{children}</div>
    </div>
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

Object.assign(window, {Ph, Lines, Chip, Btn, Field, TL, Stars, Bar, Crumbs, Browser, Variant, Callout, SectionHead, ModelRow, DownloadsPanel, DownloadsMenu, ModelListRow, MobileHeader, MobileMenu, TabletFrame, ParamSection, PresetChipRow, QuantPicker, FitCheckCard, LiveConfigJson, DownloadProgressStrip, SliderWithMarks, TaskCategoryGrid, TaskCategoryCard, BrowseBySelector, OverlayShell, AliasRail, AliasMediumAnchors, DEFAULT_QUANTS, DEFAULT_ALIAS_CONFIG, TASK_CATEGORIES, PRESETS, ALIAS_SECTIONS, ArgsEditor, ArgsPalette, ArgsHelpPop, ARGS_HELP, ARGS_PRESETS, DEFAULT_ARG_LINES, PRESET_CATALOGUE, PresetGrid, PresetAndArgsSection});
