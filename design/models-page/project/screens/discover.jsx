// Unified Models page — v25 · HF repos + local aliases/files/api-models + providers
// (Filename remains discover.jsx to avoid index.html churn; the SCREEN is now
// "Models" — see app.jsx. Tabs/variants are the three width variants of the
// unified page.)

const DiscoverCard = ({kind='hf-repo', title, subtitle, caps=[], meta, cost, status, fit, fitLabel, selected, onClick,
                        localBadge, backlink, catalogAliases, directoryAttribution}) => {
  const kindTone =
    kind==='alias' ? 'saff' :
    kind==='file' ? 'leaf' :
    kind==='api-model' ? 'indigo' :
    kind==='provider' ? 'indigo' :
    kind==='provider-off' ? '' :
    kind==='hf-repo' ? 'leaf' : 'leaf';
  const statusTone =
    status==='ready' || status==='connected' || status==='fits' || status==='live' ? 'leaf' :
    status==='oauth' ? 'saff' :
    status==='not connected' ? '' :
    status==='rate-limited' || status==='tight' ? 'warn' : '';
  const fitTone = fit==='green' ? 'leaf' : fit==='yellow' ? 'warn' : fit==='red' ? 'warn' : '';
  const extraClass = kind==='provider-off' ? ' dashed' : '';
  const kindLabel = kind==='provider-off' ? 'provider' : kind;
  return (
    <div className={`model-card${selected?' selected':''}${extraClass}`} onClick={onClick} style={{cursor:'pointer'}}>
      <div className="model-card-head">
        <Chip tone={kindTone} style={{fontSize:10}}>{kindLabel}</Chip>
        {status && <Chip tone={statusTone}>● {status}</Chip>}
        {fitLabel && <Chip tone={fitTone}>● {fitLabel}</Chip>}
        {localBadge && <span className="row-local-badge">local</span>}
      </div>
      <div className="model-card-title">{title}</div>
      {subtitle && <div className="sm">{subtitle}</div>}
      {caps.length>0 && (
        <div style={{display:'flex', gap:3, flexWrap:'wrap'}}>
          {caps.map((c,i)=>(<Chip key={i}>{c}</Chip>))}
        </div>
      )}
      {cost && <div className="model-card-cost">{cost}</div>}
      {meta && <div className="model-card-meta">{meta}</div>}
      {backlink && <div><span className="row-backlink">↗ {backlink}</span></div>}
      {catalogAliases && catalogAliases.count>0 && (
        <div style={{marginTop:3}}>
          <span className="row-catalog-aliases-badge">✓ {catalogAliases.count} local aliases ↗</span>
        </div>
      )}
      {directoryAttribution && (
        <div className="row-directory-attribution">from Bodhi directory · <code>api.getbodhi.app</code></div>
      )}
    </div>
  );
};

// ── Right-panel variants ───────────────────────────────────────

function HfRepoPanel() {
  return (
    <>
      <div className="right-collapsed-rail">Qwen/Qwen3.5-9B · repo</div>
      <div className="right-topbar">
        <div style={{display:'flex', alignItems:'center', gap:6, flexWrap:'wrap'}}>
          <Chip tone="leaf" style={{fontSize:10}}>hf-repo</Chip>
          <span className="h2" style={{margin:0}}>Qwen/Qwen3.5-9B</span>
        </div>
        <Btn variant="ghost" size="xs" title="collapse">→</Btn>
      </div>
      <div className="sm">HuggingFace · Apache-2 · 9B · released 2025-08 · not yet downloaded</div>

      <div style={{display:'flex', gap:4, marginTop:8, flexWrap:'wrap'}}>
        <Chip on>Overview</Chip><Chip>Quants (5)</Chip><Chip>README</Chip><Chip>Leaderboard</Chip>
      </div>

      <div className="h3">Capabilities</div>
      <div style={{display:'flex', gap:4, flexWrap:'wrap'}}>
        <Chip tone="leaf">text2text</Chip><Chip tone="leaf">tool-use</Chip>
        <Chip>reasoning</Chip><Chip>structured-output</Chip>
      </div>

      <div className="h3">Specs</div>
      <div style={{display:'grid', gridTemplateColumns:'auto 1fr', columnGap:8, rowGap:3}}>
        <span className="sm">ctx</span><span className="sm"><b>32768</b> tokens</span>
        <span className="sm">arch</span><span className="sm">qwen3 · GQA</span>
        <span className="sm">scores</span><span className="sm">#2 Arena · 92.4 MMLU-Pro</span>
        <span className="sm">popularity</span><span className="sm">↓ 443k · ♥ 3.1k</span>
        <span className="sm">rig fit</span><span className="sm"><TL tone="green">Q4/Q5 fit · Q8 tight</TL></span>
      </div>

      <div className="h3">Quants · add as alias</div>
      <div style={{display:'flex', flexDirection:'column', gap:4}}>
        <div className="card row" style={{padding:'5px 7px', borderColor:'var(--ink)'}}>
          <Chip tone="saff" style={{fontSize:10}}>default</Chip>
          <code style={{flex:1}}>:Q4_K_M</code>
          <span className="sm">5.6 GB</span>
          <TL tone="green">~38 t/s</TL>
          <Btn variant="primary" size="xs">+ alias</Btn>
        </div>
        <div className="card row" style={{padding:'5px 7px'}}>
          <code style={{flex:1}}>:Q5_K_M</code>
          <span className="sm">6.8 GB</span>
          <TL tone="green">~30 t/s</TL>
          <Btn size="xs">+ alias</Btn>
        </div>
        <div className="card row" style={{padding:'5px 7px'}}>
          <code style={{flex:1}}>:Q6_K</code>
          <span className="sm">7.9 GB</span>
          <TL tone="green">~24 t/s</TL>
          <Btn size="xs">+ alias</Btn>
        </div>
        <div className="card row" style={{padding:'5px 7px'}}>
          <code style={{flex:1}}>:Q8_0</code>
          <span className="sm">9.6 GB</span>
          <TL tone="yellow">~16 t/s</TL>
          <Btn size="xs">+ alias</Btn>
        </div>
        <div className="card row" style={{padding:'5px 7px'}}>
          <code style={{flex:1}}>:F16</code>
          <span className="sm">18 GB</span>
          <TL tone="warn">won't fit</TL>
          <Btn variant="ghost" size="xs">+ alias</Btn>
        </div>
      </div>
      <div className="sm" style={{marginTop:4}}>★ `+ alias` opens the alias form overlay prefilled with this quant · downloads on save.</div>

      <div className="h3">README snippet</div>
      <div className="sm" style={{fontStyle:'italic', borderLeft:'2px dashed var(--line-soft)', paddingLeft:8}}>
        Qwen3.5-9B is a 9B instruction-tuned model with improved tool-use, 32k context, and strong reasoning. Drop-in replacement for Qwen2.5-9B.
      </div>

      <div style={{display:'flex', gap:4, marginTop:10, flexWrap:'wrap'}}>
        <Btn variant="primary" size="xs">+ Add to Bodhi</Btn>
        <Btn size="xs">Pull file only</Btn>
        <Btn variant="ghost" size="xs">★ Favorite</Btn>
        <Btn variant="ghost" size="xs">HF ↗</Btn>
      </div>
      <Callout style={{position:'static', display:'inline-block', marginTop:6, fontSize:10}}>★ `+ Add to Bodhi` opens alias overlay prefilled with repo + default quant</Callout>
    </>
  );
}

function UnconnectedProviderPanel() {
  return (
    <>
      <div className="right-collapsed-rail">groq · provider (not connected)</div>
      <div className="right-topbar">
        <div style={{display:'flex', alignItems:'center', gap:6, flexWrap:'wrap'}}>
          <Chip style={{fontSize:10}}>provider</Chip>
          <span className="h2" style={{margin:0}}>groq</span>
          <Chip>● not connected</Chip>
        </div>
        <Btn variant="ghost" size="xs" title="collapse">→</Btn>
      </div>
      <div className="sm">openai-completions · api-key · LPU-accelerated inference · 99.8% up (public SLA)</div>

      <div style={{display:'flex', gap:4, marginTop:8, flexWrap:'wrap'}}>
        <Chip on>Models (6)</Chip><Chip>Pricing</Chip><Chip>About</Chip>
      </div>

      <div className="h3">Available models (preview · connect to use)</div>
      <div className="provider-table">
        <div className="provider-table-row provider-table-head">
          <span>model</span><span>caps</span><span>in $/M</span><span>out $/M</span><span>cached</span><span>t/s</span><span></span>
        </div>
        <div className="provider-table-row disabled">
          <code>llama-3.3-70b-versatile</code>
          <span className="sm">tool · long-ctx</span>
          <span className="sm">0.59</span><span className="sm">0.79</span><span className="sm">—</span>
          <TL tone="green">276</TL>
          <Btn variant="ghost" size="xs" disabled>use</Btn>
        </div>
        <div className="provider-table-row disabled">
          <code>llama-3.1-8b-instant</code>
          <span className="sm">tool</span>
          <span className="sm">0.05</span><span className="sm">0.08</span><span className="sm">—</span>
          <TL tone="green">750</TL>
          <Btn variant="ghost" size="xs" disabled>use</Btn>
        </div>
        <div className="provider-table-row disabled">
          <code>qwen-2.5-32b</code>
          <span className="sm">tool · structured</span>
          <span className="sm">0.29</span><span className="sm">0.39</span><span className="sm">—</span>
          <TL tone="green">200</TL>
          <Btn variant="ghost" size="xs" disabled>use</Btn>
        </div>
        <div className="provider-table-row disabled">
          <code>deepseek-r1-distill-70b</code>
          <span className="sm">reasoning · tool</span>
          <span className="sm">0.75</span><span className="sm">0.99</span><span className="sm">—</span>
          <TL tone="green">180</TL>
          <Btn variant="ghost" size="xs" disabled>use</Btn>
        </div>
        <div className="provider-table-row disabled">
          <code>whisper-large-v3</code>
          <span className="sm">speech</span>
          <span className="sm">$0.11/hr</span><span className="sm">—</span><span className="sm">—</span>
          <TL tone="green">164×rt</TL>
          <Btn variant="ghost" size="xs" disabled>use</Btn>
        </div>
        <div className="provider-table-row disabled">
          <code>llama-guard-4-12b</code>
          <span className="sm">moderation</span>
          <span className="sm">0.20</span><span className="sm">0.20</span><span className="sm">—</span>
          <TL tone="green">480</TL>
          <Btn variant="ghost" size="xs" disabled>use</Btn>
        </div>
      </div>

      <div className="h3">About</div>
      <div className="sm" style={{fontStyle:'italic', borderLeft:'2px dashed var(--line-soft)', paddingLeft:8}}>
        Groq serves open-weight models on custom LPU silicon with industry-leading throughput. openai-completions-compatible endpoint; bring your own API key.
      </div>

      <div style={{display:'flex', gap:4, marginTop:10, flexWrap:'wrap'}}>
        <Btn variant="primary" size="xs">Configure →</Btn>
        <Btn variant="ghost" size="xs">groq.com ↗</Btn>
      </div>
      <div className="sm" style={{marginTop:4, color:'var(--ink-3)'}}>opens <b>Create API model</b> tab with groq preselected</div>
    </>
  );
}

function ConnectedProviderPanel() {
  return (
    <>
      <div className="right-collapsed-rail">openai · provider</div>
      <div className="right-topbar">
        <div style={{display:'flex', alignItems:'center', gap:6, flexWrap:'wrap'}}>
          <Chip tone="indigo" style={{fontSize:10}}>provider</Chip>
          <span className="h2" style={{margin:0}}>openai</span>
          <Chip tone="leaf">● connected</Chip>
        </div>
        <Btn variant="ghost" size="xs" title="collapse">→</Btn>
      </div>
      <div className="sm">openai-responses · key sk-…a71e · added 2026-01-22 · 99.9% up (30d)</div>

      <div style={{display:'flex', gap:4, marginTop:8, flexWrap:'wrap'}}>
        <Chip on>Models (7)</Chip><Chip>Connection</Chip><Chip>Usage</Chip>
      </div>

      <div className="h3">Available models</div>
      <div className="provider-table">
        <div className="provider-table-row provider-table-head">
          <span>model</span><span>caps</span><span>in $/M</span><span>out $/M</span><span>cached</span><span>t/s</span><span></span>
        </div>
        <div className="provider-table-row">
          <code>gpt-5</code>
          <span className="sm">tool · vision · reasoning</span>
          <span className="sm">1.25</span><span className="sm">10.00</span><span className="sm">0.125</span>
          <TL tone="green">64</TL>
          <Btn variant="ghost" size="xs">use</Btn>
        </div>
        <div className="provider-table-row">
          <code>gpt-5-mini</code>
          <span className="sm">tool · structured</span>
          <span className="sm">0.25</span><span className="sm">2.00</span><span className="sm">0.03</span>
          <TL tone="green">78</TL>
          <Btn variant="ghost" size="xs">use</Btn>
        </div>
        <div className="provider-table-row">
          <code>gpt-5-nano</code>
          <span className="sm">tool</span>
          <span className="sm">0.05</span><span className="sm">0.40</span><span className="sm">0.005</span>
          <TL tone="green">120</TL>
          <Btn variant="ghost" size="xs">use</Btn>
        </div>
        <div className="provider-table-row">
          <code>o4-mini</code>
          <span className="sm">reasoning · tool</span>
          <span className="sm">1.10</span><span className="sm">4.40</span><span className="sm">0.275</span>
          <TL tone="green">54</TL>
          <Btn variant="ghost" size="xs">use</Btn>
        </div>
        <div className="provider-table-row">
          <code>gpt-4.1</code>
          <span className="sm">tool · vision</span>
          <span className="sm">2.00</span><span className="sm">8.00</span><span className="sm">0.50</span>
          <TL tone="green">52</TL>
          <Btn variant="ghost" size="xs">use</Btn>
        </div>
        <div className="provider-table-row">
          <code>text-embedding-3-large</code>
          <span className="sm">embedding</span>
          <span className="sm">0.13</span><span className="sm">—</span><span className="sm">—</span>
          <TL tone="green">fast</TL>
          <Btn variant="ghost" size="xs">use</Btn>
        </div>
      </div>

      <div className="h3">Connection</div>
      <div style={{display:'grid', gridTemplateColumns:'auto 1fr', columnGap:8, rowGap:3}}>
        <span className="sm">format</span><span className="sm">openai-responses</span>
        <span className="sm">auth</span><span className="sm">api-key (sk-…a71e)</span>
        <span className="sm">base url</span><span className="sm">api.openai.com/v1</span>
        <span className="sm">status</span><span className="sm"><TL tone="green">live · last ping 2s ago</TL></span>
      </div>

      <div style={{display:'flex', gap:4, marginTop:10, flexWrap:'wrap'}}>
        <Btn variant="primary" size="xs">+ Add as alias</Btn>
        <Btn size="xs">Test connection</Btn>
        <Btn variant="ghost" size="xs">Manage in My Models</Btn>
      </div>
    </>
  );
}

// Specialization filter options (Task categories, single-select, default = All)
const SPECIALIZATIONS = [
  {k:'all',       label:'All',                 bench:null},
  {k:'chat',      label:'Chat · general',      bench:'Arena Elo'},
  {k:'coding',    label:'Coding',              bench:'HumanEval'},
  {k:'agent',     label:'Agentic · tool-use',  bench:'BFCL'},
  {k:'reason',    label:'Reasoning',           bench:'GPQA'},
  {k:'longctx',   label:'Long context',        bench:'RULER'},
  {k:'multiling', label:'Multilingual',        bench:'mMMLU'},
  {k:'vision',    label:'Vision + text',       bench:'MMMU'},
  {k:'embed',     label:'Text embedding',      bench:'MTEB'},
  {k:'memb',      label:'Multimodal embed',    bench:'MMEB'},
  {k:'small',     label:'Small & fast',        bench:'Open LLM LB'},
];

function DiscoverA() {
  const [sel, setSel] = React.useState('hf-qwen');
  const [view, setView] = React.useState('cards');
  const [spec, setSpec] = React.useState('coding'); // wireframe demo: show active Specialization
  const [mode, setMode] = React.useState('all');    // v25: My | All (demo on All so duality is visible)
  const specMeta = SPECIALIZATIONS.find(s => s.k === spec) || SPECIALIZATIONS[0];
  const Row = view==='list' ? ModelListRow : DiscoverCard;
  return (
    <div className="hub3col">

      {/* ── LEFT: navigation + filters ── */}
      <aside className="hub3col-left">
        <div className="side-brand">
          <span className="brand-dot">◉</span>
          <div style={{display:'flex', flexDirection:'column', lineHeight:1}}>
            <span>Bodhi</span>
            <span className="sm" style={{letterSpacing:1, fontSize:9, color:'var(--ink-3)'}}>AI GATEWAY</span>
          </div>
        </div>

        <div className="side-section-picker active">
          <span className="side-section-picker-icon">▦</span>
          <span style={{flex:1, fontWeight:700}}>Models</span>
        </div>

        <DownloadsMenu active={sel==='downloads'} count={1} onClick={()=>setSel('downloads')}/>

        <div className="side-sec-label">Browse</div>
        <div className="side-nav">
          <div className="side-nav-item active">Trending <span className="badge">↑</span></div>
          <div className="side-nav-item">New launches <span className="badge">★</span></div>
        </div>

        <div className="side-sec-label">Filters</div>
        <div className="side-filter-group">
          <div className="side-filter-title" style={{display:'flex', justifyContent:'space-between', alignItems:'center'}}>
            <span>specialization · single-select</span>
            {spec!=='all' && <span className="active-filters-clear" onClick={()=>setSpec('all')}>clear</span>}
          </div>
          <div className="chips-col">
            {SPECIALIZATIONS.map(s => (
              <Chip key={s.k}
                on={spec===s.k}
                onClick={()=>setSpec(s.k)}
                style={spec===s.k && s.k!=='all' ? {background:'var(--indigo-soft)', fontWeight:700} : undefined}>
                {s.label}{s.bench && spec===s.k ? ` · ${s.bench}` : ''}
              </Chip>
            ))}
          </div>
          {spec!=='all' && (
            <div className="sm" style={{marginTop:4, fontStyle:'italic'}}>
              Main grid filtered to <b>{specMeta.label}</b> · sorted by <b>{specMeta.bench}</b>
            </div>
          )}
        </div>
        <div className="side-filter-group">
          <div className="side-filter-title">kind · multi-select</div>
          <div className="chips-col">
            <Chip on>All</Chip>
            <Chip>Aliases</Chip><Chip>Files</Chip><Chip>API models</Chip>
            <Chip>Providers</Chip>{mode==='all' && <Chip>HF repos</Chip>}
          </div>
        </div>
        <div className="side-filter-group">
          <div className="side-filter-title">source</div>
          <div className="chips-col">
            <Chip>HuggingFace</Chip>
            <Chip>Bodhi Directory <span className="sm" style={{color:'var(--ink-4)'}}>· api.getbodhi.app</span></Chip>
            <Chip>OpenAI</Chip><Chip>Anthropic</Chip><Chip>Groq</Chip>
            <Chip>Together</Chip><Chip>NVIDIA NIM</Chip><Chip>HF Inference</Chip>
            <Chip>OpenRouter</Chip><Chip>Google</Chip>
          </div>
        </div>
        <div className="side-filter-group">
          <div className="side-filter-title">capability</div>
          <div className="chips-col">
            <Chip>tool-use</Chip><Chip>vision</Chip><Chip>structured</Chip>
            <Chip>embedding</Chip><Chip>speech</Chip><Chip>image-gen</Chip>
            <Chip>reasoning</Chip>
          </div>
        </div>
        <div className="side-filter-group">
          <div className="side-filter-title">size · rig</div>
          <div className="chips-col">
            <Chip tone="leaf">Fits rig ✓</Chip>
            <Chip>&lt; 5GB</Chip><Chip>5–15GB</Chip><Chip>&gt; 15GB</Chip>
            <Chip>ctx ≥ 32k</Chip>
          </div>
        </div>
        <div className={`side-filter-group${mode==='my'?' filter-group-disabled':''}`}>
          <div className="side-filter-title">cost · api</div>
          <div className="chips-col">
            <Chip>Free / OSS</Chip><Chip>$&lt;1 / M</Chip>
            <Chip>$1–5</Chip><Chip>$&gt;5</Chip>
            <Chip>≥99% up</Chip>
          </div>
        </div>
        <div className="side-filter-group">
          <div className="side-filter-title">license</div>
          <div className="chips-col">
            <Chip>Apache-2</Chip><Chip>MIT</Chip><Chip>Llama</Chip>
            <Chip>Gemma</Chip><Chip>CC-BY</Chip><Chip>Proprietary</Chip>
          </div>
        </div>
        <div className="side-filter-group">
          <div className="side-filter-title">format</div>
          <div className="chips-col">
            <Chip tone="leaf">GGUF ✓</Chip>
            <Chip>openai-responses</Chip><Chip>anthropic-messages</Chip>
            <Chip>openrouter</Chip>
          </div>
        </div>

        <div className="side-sec-label">Leaderboard</div>
        <div className="side-nav">
          <div className="side-nav-item">Chatbot Arena</div>
          <div className="side-nav-item">MMLU-Pro</div>
          <div className="side-nav-item">HumanEval</div>
          <div className="side-nav-item">Tool-use</div>
          <div className="side-nav-item">Vision (MMMU)</div>
        </div>
      </aside>

      {/* ── CENTER: search + scope + card grid ── */}
      <main className="hub3col-main">
        <div className="main-topbar">
          <div>
            <div className="h1" style={{fontSize:20}}>Models</div>
            <div className="sm">Local + API + remote · one catalog · M3 Max 36GB</div>
          </div>
          <div style={{position:'relative', display:'flex', gap:6}}>
            <Btn variant="primary" size="xs">+ ▾ Add model</Btn>
            <Btn size="xs">⋯ ▾ Browse</Btn>
            {/* Wireframe: always render the Add&Browse menu so users see its shape */}
            <ModelsAddBrowseMenu/>
          </div>
        </div>

        {/* ── Toolbar row 1: mode toggle ── */}
        <ModeToggle mode={mode} onChange={setMode}
          localCount={14}
          catalogCount="3.1M"
          directoryCount={23}/>
        <ModeToggleCaption mode={mode}/>

        {/* ── Toolbar row 2: kind chips + sort + view ── */}
        <div className="main-toolbar" style={{marginBottom:4}}>
          <KindChipRow mode={mode} active={['all']}/>
          <span className="vsep"/>
          <span className="sm">sort:</span>
          {mode==='my'
            ? <><Chip on>Recently used</Chip><Chip>Name</Chip><Chip>Size</Chip></>
            : <><Chip on>Likes</Chip><Chip>Downloads</Chip><Chip>Recent</Chip>
              {spec!=='all' && <Chip tone="indigo">⭐ {specMeta.bench}</Chip>}</>}
          <span className="vsep"/>
          <Chip on={view==='cards'} onClick={()=>setView('cards')}>▦ Cards</Chip>
          <Chip on={view==='list'} onClick={()=>setView('list')}>☰ List</Chip>
        </div>

        <div style={{position:'relative'}}>
          <Field hint="Filter anything — 'vision 7B apache', 'claude tool-use', 'my aliases'…" filled
            right={<span className="sm">⌘K</span>}/>
        </div>

        <div className="active-filters">
          <span className="active-filters-label">filters:</span>
          {spec!=='all' && (
            <span className="filter-tag" style={{background:'var(--indigo-soft)', fontWeight:700}}>
              Specialization: {specMeta.label} · sort ⭐ {specMeta.bench}
              <span className="x" onClick={()=>setSpec('all')}>× clear</span>
            </span>
          )}
          <span className="filter-tag">capability: tool-use <span className="x">×</span></span>
          <span className="filter-tag">size: Fits rig ✓ <span className="x">×</span></span>
          <span className="filter-tag">license: Apache-2 <span className="x">×</span></span>
          <span className="active-filters-clear">clear all</span>
        </div>

        {/* Ranked display mode: a benchmark sort is active (see specs/models.md §8).
            Rows collapse to model-level with local-file dedup + api-config stack. */}
        {spec!=='all' && (
          <RankedModeCaption benchmark={specMeta.bench} specLabel={specMeta.label}/>
        )}

        <div className={view==='list' ? 'cards-list' : 'cards-grid'}>
          {spec!=='all' ? (
            groupIntoRankedRows(specMeta.bench, mode).map((entry) => (
              <RankedRow key={entry.rank} entry={entry}
                selected={sel===`rank-${entry.rank}`}
                onClick={()=>setSel(`rank-${entry.rank}`)}/>
            ))
          ) : (<>
          {/* ── Local entities (always shown; labelled `local` in All mode) ── */}
          <Row kind="alias" title="my-gemma"
            subtitle="google/gemma-2-9b:Q4_K_M · ctx 16k"
            caps={['text→text','tool-use']}
            meta="preset: chat · last used 12m ago"
            status="ready"
            backlink="catalog · google/gemma-2-9b-GGUF"
            localBadge={mode==='all'}
            selected={sel==='alias-my-gemma'}
            onClick={()=>setSel('alias-my-gemma')}/>
          <Row kind="alias" title="code-beast"
            subtitle="Qwen/Qwen2.5-Coder-14B:Q5_K_M · ctx 32k"
            caps={['text→text','tool-use','structured']}
            meta="preset: coding · last used yesterday"
            status="ready"
            backlink="catalog · Qwen/Qwen2.5-Coder-14B-GGUF"
            localBadge={mode==='all'}
            selected={sel==='alias-code-beast'}
            onClick={()=>setSel('alias-code-beast')}/>
          <Row kind="file" title="google/gemma-2-9b:Q5_K_M"
            subtitle="6.6 GB · downloaded · no alias"
            caps={['text2text','tool-use']}
            meta="+ Create alias · orphan file"
            fit="green" fitLabel="~30 t/s"
            backlink="catalog · google/gemma-2-9b-GGUF"
            localBadge={mode==='all'}
            selected={sel==='file-gemma-q5'}
            onClick={()=>setSel('file-gemma-q5')}/>
          <Row kind="api-model" title="openai/gpt-5-mini"
            subtitle="configured · system preset + tool overrides"
            caps={['tool-use','structured']}
            cost="in $0.25 · out $2.00 / M"
            meta="preset: agent · last used 3h ago"
            status="live"
            backlink="openai provider"
            localBadge={mode==='all'}
            selected={sel==='api-gpt5mini'}
            onClick={()=>setSel('api-gpt5mini')}/>
          <Row kind="provider" title="openai"
            subtitle={<><code>openai-responses</code> · key sk-…a71e</>}
            caps={['tool-use','vision','structured','reasoning','embedding']}
            cost="in $0.05 – $2.00 / M · 7 models"
            meta="✓ 1 api-model configured · gpt-5, gpt-5-mini, gpt-5-nano, o4-mini, +3"
            status="connected"
            localBadge={mode==='all'}
            selected={sel==='provider-openai'}
            onClick={()=>setSel('provider-openai')}/>

          {/* ── Catalog entities (All mode only) ── */}
          {mode==='all' && <>
          <Row kind="hf-repo" title="Qwen/Qwen3.5-9B"
            subtitle="9B · ctx 32k · Apache-2 · HuggingFace"
            caps={['text2text','tool-use','reasoning']}
            meta={<>default <code>:Q4_K_M</code> · 5.6GB · 5 quants · ↓ 443k · ♥ 3.1k</>}
            fit="green" fitLabel="~38 t/s"
            selected={sel==='hf-qwen'}
            onClick={()=>setSel('hf-qwen')}/>
          <Row kind="provider-off" title="groq"
            subtitle={<><code>openai-completions</code> · bring-your-own-key</>}
            caps={['tool-use','speech','moderation']}
            cost="in $0.05 – $0.75 / M · 6 models"
            meta="llama-3.3-70b, llama-3.1-8b, qwen-2.5-32b, +3 more"
            status="not connected"
            directoryAttribution
            selected={sel==='provider-groq'}
            onClick={()=>setSel('provider-groq')}/>

          <Row kind="hf-repo" title="google/gemma-2-9b-GGUF"
            subtitle="8.5B · Gemma T&C · HuggingFace"
            caps={['text2text','tool-use']}
            meta={<>default <code>:Q4_K_M</code> · 5.4GB · 5 quants · ↓ 2.1M · ♥ 4.4k</>}
            fit="green" fitLabel="~38 t/s"
            catalogAliases={{count:1}}/>
          <Row kind="hf-repo" title="Qwen/Qwen2.5-Coder-14B-GGUF"
            subtitle="14B · Apache-2 · HuggingFace"
            caps={['text2text','structured','reasoning']}
            meta={<>default <code>:Q4_K_M</code> · 9.0GB · 5 quants · ↓ 511k · ♥ 2.8k</>}
            fit="green" fitLabel="~22 t/s"
            catalogAliases={{count:1}}/>
          <Row kind="hf-repo" title="google/gemma-4-e2b"
            subtitle="2B · vision · Gemma T&C · HuggingFace"
            caps={['multimodal','vision']}
            meta={<>default <code>:Q4_K_M</code> · 1.4GB · 3 quants · ↓ 3.8M · ♥ 2.1k</>}
            fit="green" fitLabel="~85 t/s"/>
          <Row kind="provider-off" title="openrouter"
            subtitle={<><code>openai-completions</code> · multi-provider routing</>}
            caps={['tool-use','vision','reasoning']}
            cost="varies · 100+ models"
            meta="meta-llama-3.3-70b, mistral-large, deepseek-r1, +97 more"
            status="not connected"
            directoryAttribution/>
          <Row kind="hf-repo" title="unsloth/Nemotron-3-Nano-30B"
            subtitle="30B · ctx 128k · NVIDIA · HuggingFace"
            caps={['text2text','reasoning','long-ctx']}
            meta={<>default <code>:Q4_K_M</code> · 17GB · 4 quants · ↓ 133k</>}
            fit="yellow" fitLabel="~6 t/s · tight"/>
          <Row kind="provider-off" title="anthropic"
            subtitle={<><code>anthropic-oauth</code> · Claude Pro / key</>}
            caps={['tool-use','vision','structured','reasoning']}
            cost="in $0.80 – $15 / M · 5 models"
            meta="claude-opus-4, claude-sonnet-4.5, claude-haiku-4.5, +2 more"
            status="not connected"
            directoryAttribution/>
          <Row kind="hf-repo" title="LiquidAI/LFM2.5-1.2B"
            subtitle="1.2B · edge · Apache-2 · HuggingFace"
            caps={['text2text']}
            meta={<>default <code>:Q8_0</code> · 1.3GB · 3 quants · ↓ 28k</>}
            fit="green" fitLabel="~110 t/s"/>
          <Row kind="provider-off" title="nvidia-nim"
            subtitle={<><code>openai-completions</code> · NVIDIA NIM · key</>}
            caps={['text2text','tool-use','long-ctx']}
            cost="in $0.60 – $1.80 / M · 18 models"
            meta="nemotron-4-340b, llama-3.3-70b-nim, mistral-large-nim, +15 more"
            status="not connected"
            directoryAttribution/>
          <Row kind="hf-repo" title="Qwen/Qwen2.5-VL-7B"
            subtitle="7B · vision-language · Apache-2 · HuggingFace"
            caps={['multimodal','vision']}
            meta={<>default <code>:Q4_K_M</code> · 4.7GB · 4 quants · ↓ 612k</>}
            fit="green" fitLabel="~32 t/s"/>
          <Row kind="provider-off" title="together"
            subtitle={<><code>openai-completions</code> · Together AI · key</>}
            caps={['text2text','tool-use','long-ctx','image-gen']}
            cost="in $0.10 – $3.50 / M · 40+ models"
            meta="qwen-2.5-72b, deepseek-v3, flux-1.1-pro, +37 more"
            status="not connected"/>
          <Row kind="hf-repo" title="deepseek/DeepSeek-R1-Distill-14B"
            subtitle="14B · reasoning · MIT · HuggingFace"
            caps={['text2text','reasoning']}
            meta={<>default <code>:Q5_K_M</code> · 10.2GB · 4 quants · ↓ 1.1M</>}
            fit="green" fitLabel="~19 t/s"/>
          <Row kind="provider-off" title="huggingface"
            subtitle={<><code>openai-completions</code> · HF Inference · key</>}
            caps={['text2text','embedding','image-gen']}
            cost="PAYG + Pro $9/mo · 500+ models"
            meta="mistral-large, llama-3.3, stable-diffusion-xl, +497 more"
            status="not connected"/>
          <Row kind="hf-repo" title="meta/Llama-3.3-70B-Instruct"
            subtitle="70B · ctx 128k · Llama-3 license · HuggingFace"
            caps={['text2text','tool-use','long-ctx']}
            meta={<>default <code>:Q4_K_M</code> · 40GB · 5 quants · ↓ 2.4M</>}
            fit="red" fitLabel="won't fit"/>
          <Row kind="provider-off" title="google-gemini"
            subtitle={<><code>google-gemini</code> · key AIza…</>}
            caps={['tool-use','vision','long-ctx','embedding']}
            cost="in $0.075 – $1.25 / M · 4 models"
            meta="gemini-2.5-pro, gemini-2.5-flash, gemini-2.5-flash-lite, +1 more"
            status="not connected"/>
          <Row kind="hf-repo" title="nomic-ai/nomic-embed-text-v2"
            subtitle="F16 · 274MB · 768-dim · HuggingFace"
            caps={['text-embedding']}
            meta={<>default <code>:F16</code> · 274MB · 1 quant · ↓ 5.1M</>}
            fit="green" fitLabel="embed fast"/>
          </>}
          </>)}
        </div>

        <div style={{textAlign:'center', marginTop:4}}>
          <Btn variant="ghost">{mode==='my' ? 'Switch to All Models to browse catalog →' : 'Load more →'}</Btn>
        </div>
      </main>

      {/* ── RIGHT: collapsible detail panel (dispatches by row kind) ── */}
      <aside className="hub3col-right">
        {/* Local entities — panels from hub.jsx */}
        {(sel==='alias-my-gemma' || sel==='alias-code-beast') && <AliasPanel/>}
        {sel==='file-gemma-q5' && <FilePanel/>}
        {/* api-model reuses the connected-provider panel (highlight on the specific model happens later) */}
        {sel==='api-gpt5mini' && <ConnectedProviderPanel/>}
        {sel==='provider-openai' && <ConnectedProviderPanel/>}

        {/* Catalog entities — panels from this file */}
        {sel==='hf-qwen' && <HfRepoPanel/>}
        {sel==='provider-groq' && <UnconnectedProviderPanel/>}

        {/* Ranked-mode selections — dispatch by the entry's canonical kind */}
        {sel && typeof sel === 'string' && sel.startsWith('rank-') && (() => {
          const rank = Number(sel.slice(5));
          const entry = RANKED_FIXTURE_CODING.find(e => e.rank === rank);
          if (!entry) return null;
          const k = entry.dispatchKind;
          if (k === 'alias') return <AliasPanel/>;
          if (k === 'file') return <FilePanel/>;
          if (k === 'api-model' || k === 'provider-connected') return <ConnectedProviderPanel/>;
          if (k === 'provider-unconnected') return <UnconnectedProviderPanel/>;
          if (k === 'hf-repo') return <HfRepoPanel/>;
          return null;
        })()}

        {/* Downloads rail click */}
        {sel==='downloads' && <DownloadsPanel/>}
      </aside>
    </div>
  );
}

// ── Mobile variant ────────────────────────────────────────────

// Header action button — single "+ ▾" on mobile/medium that drops a grouped
// Add + Browse menu (the same dropdown rendered as ModelsAddBrowseMenu on
// desktop).
const DiscoverActionsBtn = () => (
  <span className="m-ico m-ico-add" title="add & browse">+ ▾</span>
);

// Grouped menu (Add + Browse) opened from the `+ ▾` header button.
const DiscoverActionsMenu = () => (
  <div className="m-menu-overlay" style={{justifyContent:'flex-end', paddingRight:6, paddingTop:40}}>
    <div className="m-menu" style={{width:'76%'}}>
      <div className="m-menu-section-head">Add model</div>
      <div className="m-menu-item">Add by HF repo</div>
      <div className="m-menu-item">Paste URL</div>
      <div className="m-menu-item">Add API provider</div>
      <div className="m-menu-item">Add API model</div>
      <div className="m-menu-section-head">Browse</div>
      <div className="m-menu-item">↑ Trending</div>
      <div className="m-menu-item">★ New launches</div>
      <div className="m-menu-item expanded">
        <div className="m-menu-container">
          <div style={{display:'flex', justifyContent:'space-between', alignItems:'center'}}>
            <span>🏆 Leaderboards</span><span>▾</span>
          </div>
          <div className="m-menu-sub">
            <div className="m-menu-sub-item">Chatbot Arena</div>
            <div className="m-menu-sub-item">MMLU-Pro</div>
            <div className="m-menu-sub-item">HumanEval</div>
            <div className="m-menu-sub-item">Tool-use</div>
            <div className="m-menu-sub-item">Vision (MMMU)</div>
          </div>
        </div>
      </div>
    </div>
  </div>
);

const DiscoverMobileSubbar = ({filtersCount=5, mode='all'}) => (
  <>
    <div style={{padding:'4px 6px'}}>
      <div className="mode-toggle" style={{padding:'3px 4px', marginBottom:4}}>
        <div className={`mode-toggle-option${mode==='my'?' active':''}`} style={{padding:'3px 6px', fontSize:11}}>
          <span className="dot" style={{width:8,height:8}}/>
          <span>My (14)</span>
        </div>
        <div className={`mode-toggle-option${mode==='all'?' active':''}`} style={{padding:'3px 6px', fontSize:11}}>
          <span className="dot" style={{width:8,height:8}}/>
          <span>All (3.1M + 23)</span>
        </div>
      </div>
    </div>
    <div className="t-search" style={{fontSize:10}}><span>Filter anything…</span><span>⌘K</span></div>
    <div className="m-toolbar">
      <span className="m-filter-btn">Filters <span className="m-filter-badge">{filtersCount}</span> ▾</span>
      <span className="sm">Sort: {mode==='my'?'Recently used':'Likes'} ▾</span>
      <span className="m-view-toggle"><Chip on>▦</Chip><Chip>☰</Chip></span>
    </div>
  </>
);

const DiscoverMobileCard = ({kind, title, subtitle, meta, cost, status, fit, fitLabel, selected}) => {
  const kindTone = kind==='provider' ? 'indigo' : kind==='provider-off' ? '' : 'leaf';
  const fitTone = fit==='green' ? 'leaf' : fit==='yellow' ? 'warn' : fit==='red' ? 'warn' : '';
  const extra = kind==='provider-off' ? ' dashed' : '';
  return (
    <div className={`m-card${selected?' selected':''}${extra}`}>
      <div style={{display:'flex', gap:4, alignItems:'center', flexWrap:'wrap'}}>
        <Chip tone={kindTone} style={{fontSize:9}}>{kind==='provider-off' ? 'provider' : kind}</Chip>
        {status && <Chip>● {status}</Chip>}
        {fitLabel && <Chip tone={fitTone}>● {fitLabel}</Chip>}
      </div>
      <div className="model-card-title" style={{fontSize:13}}>{title}</div>
      {subtitle && <div className="sm" style={{fontSize:10}}>{subtitle}</div>}
      {cost && <div className="model-card-cost" style={{fontSize:10, padding:'1px 5px'}}>{cost}</div>}
      {meta && <div className="sm" style={{fontSize:10, color:'var(--ink-3)'}}>{meta}</div>}
    </div>
  );
};

function DiscoverMobile() {
  return (
    <div className="phone-deck">
      {/* ── 1. Browse state ── */}
      <PhoneFrame label="1 · Browse">
        <MobileHeader active="Models" rightSlot={<DiscoverActionsBtn/>}/>
        <DiscoverMobileSubbar/>
        <div className="active-filters" style={{fontSize:10, padding:'2px 0'}}>
          <span className="active-filters-label">filters:</span>
          <span className="filter-tag">tool-use <span className="x">×</span></span>
          <span className="filter-tag">Fits rig ✓ <span className="x">×</span></span>
        </div>
        <div className="m-grid">
          <DiscoverMobileCard title="Qwen/Qwen3.5-9B" subtitle="9B · Apache-2 · HF" meta="default :Q4_K_M · 5.6GB · 5 quants · ↓443k" fit="green" fitLabel="~38 t/s" selected/>
          <DiscoverMobileCard kind="provider" title="openai" subtitle="openai-responses · 7 models" cost="in $0.05 – $2.00 / M" status="connected"/>
          <DiscoverMobileCard kind="provider-off" title="groq" subtitle="openai-completions · 6 models" cost="in $0.05 – $0.75 / M" status="not connected"/>
          <DiscoverMobileCard title="google/gemma-4-e2b" subtitle="2B · Gemma T&C · HF" meta="default :Q4_K_M · 1.4GB · 3 quants" fit="green" fitLabel="~85 t/s"/>
          <DiscoverMobileCard kind="provider-off" title="openrouter" subtitle="openai-completions · 100+ models" cost="varies"/>
        </div>
        <Callout style={{position:'static', fontSize:9, margin:'4px 0'}}>repos + providers unified · same sheet UX as Hub</Callout>
      </PhoneFrame>

      {/* ── 2. Header menu open ── */}
      <PhoneFrame label="2 · Menu">
        <MobileHeader active="Models" rightSlot={<DiscoverActionsBtn/>}/>
        <MobileMenu active="Models"/>
        <Callout style={{position:'static', fontSize:9, margin:'4px 0'}}>breadcrumb menu · Models expands · Discover active</Callout>
      </PhoneFrame>

      {/* ── 3. Filters sheet ── */}
      <PhoneFrame label="3 · Filters sheet">
        <div className="phone-dim">
          <MobileHeader active="Models" rightSlot={<DiscoverActionsBtn/>}/>
          <DiscoverMobileSubbar/>
          <div className="m-grid dim">
            <DiscoverMobileCard title="Qwen/Qwen3.5-9B" subtitle="9B · Apache-2" fit="green" fitLabel="~38 t/s"/>
            <DiscoverMobileCard kind="provider" title="openai" subtitle="7 models"/>
          </div>
        </div>
        <div className="m-sheet m-sheet-tall">
          <div className="m-sheet-handle"/>
          <div style={{display:'flex', alignItems:'center', justifyContent:'space-between', gap:6}}>
            <span className="h2" style={{margin:0, fontSize:13}}>Filters</span>
            <span className="sm" style={{fontSize:10}}>5 active · <a href="#" style={{textDecoration:'underline', color:'var(--ink-3)'}}>clear all</a></span>
          </div>
          <div className="m-filter-groups">
            <div className="side-filter-title" style={{fontSize:11, display:'flex', justifyContent:'space-between'}}>
              <span>specialization · single-select</span>
              <span className="active-filters-clear" style={{fontSize:10}}>clear</span>
            </div>
            <div style={{display:'flex', gap:3, flexWrap:'wrap'}}>
              <Chip>All</Chip>
              <Chip on style={{background:'var(--indigo-soft)', fontWeight:700}}>Coding · HumanEval</Chip>
              <Chip>Chat</Chip><Chip>Agent</Chip><Chip>Reasoning</Chip>
              <Chip>Long ctx</Chip><Chip>Vision</Chip><Chip>Embed</Chip><Chip>Small</Chip>
            </div>
            <div className="side-filter-title" style={{fontSize:11, marginTop:6}}>source</div>
            <div style={{display:'flex', gap:3, flexWrap:'wrap'}}>
              <Chip on>HuggingFace</Chip><Chip>OpenRouter</Chip><Chip>NVIDIA NIM</Chip>
              <Chip>Groq</Chip><Chip>Together</Chip><Chip>Anthropic</Chip><Chip>OpenAI</Chip>
            </div>
            <div className="side-filter-title" style={{fontSize:11, marginTop:6}}>capability</div>
            <div style={{display:'flex', gap:3, flexWrap:'wrap'}}>
              <Chip on>tool-use</Chip><Chip>vision</Chip><Chip>structured</Chip>
              <Chip>embedding</Chip><Chip>reasoning</Chip>
            </div>
            <div className="side-filter-title" style={{fontSize:11, marginTop:6}}>size · rig</div>
            <div style={{display:'flex', gap:3, flexWrap:'wrap'}}>
              <Chip tone="leaf" on>Fits rig ✓</Chip>
              <Chip>&lt; 5GB</Chip><Chip>5–15GB</Chip><Chip>&gt; 15GB</Chip>
            </div>
            <div className="side-filter-title" style={{fontSize:11, marginTop:6}}>cost · api</div>
            <div style={{display:'flex', gap:3, flexWrap:'wrap'}}>
              <Chip>Free / OSS</Chip><Chip>$&lt;1 / M</Chip>
              <Chip>$1–5</Chip><Chip>$&gt;5</Chip><Chip>≥99% up</Chip>
            </div>
            <div className="side-filter-title" style={{fontSize:11, marginTop:6}}>license</div>
            <div style={{display:'flex', gap:3, flexWrap:'wrap'}}>
              <Chip on>Apache-2</Chip><Chip>MIT</Chip><Chip>Llama</Chip>
              <Chip>Gemma</Chip><Chip>Proprietary</Chip>
            </div>
            <div className="side-filter-title" style={{fontSize:11, marginTop:6}}>format</div>
            <div style={{display:'flex', gap:3, flexWrap:'wrap'}}>
              <Chip tone="leaf" on>GGUF ✓</Chip>
              <Chip>openai-responses</Chip><Chip>anthropic-messages</Chip>
              <Chip>openrouter</Chip>
            </div>
          </div>
          <div className="m-sheet-actions">
            <Btn variant="ghost" size="xs">Cancel</Btn>
            <Btn variant="primary" size="xs">Apply · 5 filters</Btn>
          </div>
        </div>
      </PhoneFrame>

      {/* ── 4. Detail bottom sheet (HF repo) ── */}
      <PhoneFrame label="4 · Repo sheet">
        <div className="phone-dim">
          <MobileHeader active="Models" rightSlot={<DiscoverActionsBtn/>}/>
          <DiscoverMobileSubbar/>
          <div className="m-grid dim">
            <DiscoverMobileCard title="Qwen/Qwen3.5-9B" subtitle="9B · Apache-2" meta="~38 t/s" selected/>
            <DiscoverMobileCard kind="provider" title="openai" subtitle="7 models"/>
          </div>
        </div>
        <div className="m-sheet">
          <div className="m-sheet-handle"/>
          <div style={{display:'flex', alignItems:'center', gap:6, flexWrap:'wrap'}}>
            <Chip tone="leaf" style={{fontSize:9}}>hf-repo</Chip>
            <span className="h2" style={{margin:0, fontSize:13}}>Qwen/Qwen3.5-9B</span>
          </div>
          <div className="sm" style={{fontSize:10}}>Apache-2 · 9B · #2 Arena · ↓443k</div>
          <div style={{display:'flex', gap:3, marginTop:4, flexWrap:'wrap'}}>
            <Chip tone="leaf">text2text</Chip><Chip tone="leaf">tool-use</Chip><Chip>reasoning</Chip>
          </div>
          <div className="h3" style={{fontSize:10, margin:'6px 0 3px'}}>Quants · pull</div>
          <div style={{display:'flex', flexDirection:'column', gap:3}}>
            <div className="card row" style={{padding:'3px 5px', fontSize:10}}>
              <Chip tone="saff" style={{fontSize:9}}>default</Chip>
              <code style={{flex:1, fontSize:10}}>:Q4_K_M</code>
              <span className="sm" style={{fontSize:10}}>5.6GB</span>
              <Btn variant="primary" size="xs">pull</Btn>
            </div>
            <div className="card row" style={{padding:'3px 5px', fontSize:10}}>
              <code style={{flex:1, fontSize:10}}>:Q5_K_M</code>
              <span className="sm" style={{fontSize:10}}>6.8GB</span>
              <Btn size="xs">pull</Btn>
            </div>
          </div>
          <div className="sm" style={{fontSize:9, color:'var(--ink-3)', marginTop:4, textAlign:'center'}}>↑ swipe up for README · leaderboard · more quants</div>
        </div>
      </PhoneFrame>

      {/* ── 5. Header action menu ── */}
      <PhoneFrame label="5 · Header action">
        <MobileHeader active="Models" rightSlot={<DiscoverActionsBtn/>}/>
        <DiscoverActionsMenu/>
        <Callout style={{position:'static', fontSize:9, margin:'4px 0'}}>tap ⋯ ▾ · Trending / New launches / Downloads / Leaderboards ›</Callout>
      </PhoneFrame>

      {/* ── 6. Ranked display mode ── */}
      <PhoneFrame label="6 · Ranked (Coding)">
        <MobileHeader active="Models" rightSlot={<DiscoverActionsBtn/>}/>
        <DiscoverMobileSubbar/>
        <div style={{padding:'0 4px'}}>
          <RankedModeCaption benchmark="HumanEval" specLabel="Coding"/>
          <div style={{display:'flex', flexDirection:'column', gap:3}}>
            {groupIntoRankedRows('HumanEval', 'all').slice(0,4).map(entry => (
              <RankedRow key={entry.rank} entry={entry}/>
            ))}
          </div>
          <Callout style={{position:'static', fontSize:9, margin:'4px 0'}}>local downloads dedup · API configs stack · absolute rank preserved</Callout>
        </div>
      </PhoneFrame>
    </div>
  );
}

// ── Medium / tablet variant ────────────────────────────────────

const TabletDiscoverMiniCard = ({kind, title, subtitle, fit, fitLabel, cost, status, selected}) => {
  const kindTone = kind==='provider' ? 'indigo' : kind==='provider-off' ? '' : 'leaf';
  const fitTone = fit==='green' ? 'leaf' : fit==='yellow' ? 'warn' : fit==='red' ? 'warn' : '';
  const extra = kind==='provider-off' ? ' dashed' : '';
  return (
    <div className={`m-card${selected?' selected':''}${extra}`} style={{fontSize:10}}>
      <div style={{display:'flex', gap:4, alignItems:'center', flexWrap:'wrap'}}>
        <Chip tone={kindTone} style={{fontSize:9}}>{kind==='provider-off' ? 'provider' : kind}</Chip>
        {status && <Chip>● {status}</Chip>}
        {fitLabel && <Chip tone={fitTone}>● {fitLabel}</Chip>}
      </div>
      <div className="model-card-title" style={{fontSize:12}}>{title}</div>
      {subtitle && <div className="sm" style={{fontSize:10}}>{subtitle}</div>}
      {cost && <div className="sm" style={{fontSize:10, color:'var(--ink-3)'}}>{cost}</div>}
    </div>
  );
};

const DiscoverMediumToolbar = ({filtersCount=5, mode='all'}) => (
  <>
    <div style={{padding:'4px 8px'}}>
      <div className="mode-toggle" style={{padding:'4px 6px', marginBottom:4}}>
        <div className={`mode-toggle-option${mode==='my'?' active':''}`} style={{padding:'4px 8px', fontSize:12}}>
          <span className="dot" style={{width:10,height:10}}/>
          <span>My Models (14)</span>
        </div>
        <div className={`mode-toggle-option${mode==='all'?' active':''}`} style={{padding:'4px 8px', fontSize:12}}>
          <span className="dot" style={{width:10,height:10}}/>
          <span>All Models (3.1M + 23 dir)</span>
        </div>
      </div>
    </div>
    <div className="t-search" style={{fontSize:11}}><span>Filter anything — 'vision 7B apache', 'my aliases', 'claude tool-use'…</span><span>⌘K</span></div>
    <div className="m-toolbar">
      <span className="m-filter-btn">Filters <span className="m-filter-badge">{filtersCount}</span> ▾</span>
      <span className="sm">kind: All ▾</span>
      <span className="sm">Sort: {mode==='my'?'Recently used':'Likes'} ▾</span>
      <span className="m-view-toggle"><Chip on>▦</Chip><Chip>☰</Chip></span>
    </div>
    <div className="active-filters" style={{fontSize:10, padding:'2px 0'}}>
      <span className="active-filters-label">filters:</span>
      <span className="filter-tag" style={{background:'var(--indigo-soft)', fontWeight:700}}>Coding · sort ⭐ HumanEval <span className="x">clear</span></span>
      <span className="filter-tag">tool-use <span className="x">×</span></span>
      <span className="filter-tag">Fits rig ✓ <span className="x">×</span></span>
      <span className="filter-tag">Apache-2 <span className="x">×</span></span>
      <span className="active-filters-clear">clear all</span>
    </div>
  </>
);

const DiscoverMediumGrid = () => (
  <div className="t-main">
    <div className="t-grid-2col">
      <TabletDiscoverMiniCard title="Qwen/Qwen3.5-9B" subtitle="9B · Apache-2 · default :Q4_K_M" fit="green" fitLabel="~38 t/s" selected/>
      <TabletDiscoverMiniCard kind="provider" title="openai" subtitle="openai-responses · 7 models" cost="in $0.05 – $2 / M" status="connected"/>
      <TabletDiscoverMiniCard kind="provider-off" title="groq" subtitle="openai-completions · 6 models" cost="in $0.05 – $0.75 / M" status="not connected"/>
      <TabletDiscoverMiniCard title="google/gemma-4-e2b" subtitle="2B · default :Q4_K_M · 1.4GB" fit="green" fitLabel="~85 t/s"/>
      <TabletDiscoverMiniCard kind="provider-off" title="openrouter" subtitle="100+ models" cost="varies"/>
      <TabletDiscoverMiniCard title="meta/Llama-3.3-70B" subtitle="70B · Llama-3 license" fit="red" fitLabel="won't fit"/>
    </div>
  </div>
);

const DiscoverMediumRightPanel = () => (
  <aside className="t-right-fixed">
    <div style={{display:'flex', alignItems:'center', gap:6, flexWrap:'wrap'}}>
      <Chip tone="leaf" style={{fontSize:10}}>hf-repo</Chip>
      <span className="h2" style={{margin:0, fontSize:12}}>Qwen/Qwen3.5-9B</span>
    </div>
    <div className="sm" style={{fontSize:10}}>Apache-2 · 9B · released 2025-08 · not yet downloaded</div>
    <div style={{display:'flex', gap:3, flexWrap:'wrap'}}>
      <Chip on style={{fontSize:9}}>Overview</Chip><Chip style={{fontSize:9}}>Quants (5)</Chip><Chip style={{fontSize:9}}>README</Chip>
    </div>
    <div className="h3">Capabilities</div>
    <div style={{display:'flex', gap:3, flexWrap:'wrap'}}>
      <Chip tone="leaf">text2text</Chip><Chip tone="leaf">tool-use</Chip><Chip>reasoning</Chip>
    </div>
    <div className="h3">Specs</div>
    <div style={{display:'grid', gridTemplateColumns:'auto 1fr', columnGap:8, rowGap:2, fontSize:10}}>
      <span className="sm">ctx</span><span className="sm"><b>32768</b></span>
      <span className="sm">arch</span><span className="sm">qwen3 · GQA</span>
      <span className="sm">scores</span><span className="sm">#2 Arena · 92.4 MMLU</span>
      <span className="sm">rig</span><span className="sm"><TL tone="green">Q4/Q5 fit</TL></span>
    </div>
    <div className="h3">Quants · pull</div>
    <div style={{display:'flex', flexDirection:'column', gap:3}}>
      <div className="card row" style={{padding:'3px 5px', fontSize:10}}>
        <Chip tone="saff" style={{fontSize:9}}>default</Chip>
        <code style={{flex:1, fontSize:10}}>:Q4_K_M</code>
        <span className="sm" style={{fontSize:10}}>5.6GB</span>
        <Btn variant="primary" size="xs">pull</Btn>
      </div>
      <div className="card row" style={{padding:'3px 5px', fontSize:10}}>
        <code style={{flex:1, fontSize:10}}>:Q5_K_M</code>
        <span className="sm" style={{fontSize:10}}>6.8GB</span>
        <Btn size="xs">pull</Btn>
      </div>
      <div className="card row" style={{padding:'3px 5px', fontSize:10}}>
        <code style={{flex:1, fontSize:10}}>:Q8_0</code>
        <span className="sm" style={{fontSize:10}}>9.6GB</span>
        <Btn size="xs">pull</Btn>
      </div>
    </div>
    <div style={{display:'flex', gap:4, marginTop:4, flexWrap:'wrap'}}>
      <Btn variant="primary" size="xs">Pull default</Btn>
      <Btn variant="ghost" size="xs">HF ↗</Btn>
    </div>
  </aside>
);

function DiscoverMedium() {
  return (
    <div className="tablet-deck">
      {/* 1. Browse */}
      <TabletFrame label="1 · Browse · 2-col grid + fixed right panel">
        <MobileHeader active="Models" rightSlot={<DiscoverActionsBtn/>}/>
        <DiscoverMediumToolbar/>
        <div className="t-layout-fixed">
          <DiscoverMediumGrid/>
          <DiscoverMediumRightPanel/>
        </div>
      </TabletFrame>

      {/* 2. Filters sheet (compact centered) */}
      <TabletFrame label="2 · Filters sheet (from Filters ▾)">
        <div className="phone-dim">
          <MobileHeader active="Models" rightSlot={<DiscoverActionsBtn/>}/>
          <DiscoverMediumToolbar/>
          <div className="t-layout-fixed">
            <DiscoverMediumGrid/>
            <DiscoverMediumRightPanel/>
          </div>
        </div>
        <div className="m-sheet m-sheet-tall m-sheet-centered">
          <div style={{display:'flex', alignItems:'center', justifyContent:'space-between', gap:6}}>
            <span className="h2" style={{margin:0, fontSize:13}}>Filters</span>
            <span className="sm" style={{fontSize:10}}>5 active · <a href="#" style={{textDecoration:'underline', color:'var(--ink-3)'}}>clear all</a></span>
          </div>
          <div className="m-filter-groups">
            <div className="side-filter-title" style={{fontSize:11, display:'flex', justifyContent:'space-between'}}>
              <span>specialization · single-select</span>
              <span className="active-filters-clear" style={{fontSize:10}}>clear</span>
            </div>
            <div style={{display:'flex', gap:3, flexWrap:'wrap'}}>
              <Chip>All</Chip>
              <Chip on style={{background:'var(--indigo-soft)', fontWeight:700}}>Coding · HumanEval</Chip>
              <Chip>Chat</Chip><Chip>Agent</Chip><Chip>Reasoning</Chip>
              <Chip>Long ctx</Chip><Chip>Vision</Chip><Chip>Embed</Chip><Chip>Small</Chip>
            </div>
            <div className="side-filter-title" style={{fontSize:11, marginTop:6}}>source</div>
            <div style={{display:'flex', gap:3, flexWrap:'wrap'}}>
              <Chip on>HuggingFace</Chip><Chip>OpenRouter</Chip><Chip>Groq</Chip>
              <Chip>NVIDIA NIM</Chip><Chip>Together</Chip><Chip>Anthropic</Chip><Chip>OpenAI</Chip>
            </div>
            <div className="side-filter-title" style={{fontSize:11, marginTop:6}}>capability</div>
            <div style={{display:'flex', gap:3, flexWrap:'wrap'}}>
              <Chip on>tool-use</Chip><Chip>vision</Chip><Chip>structured</Chip>
              <Chip>embedding</Chip><Chip>reasoning</Chip>
            </div>
            <div className="side-filter-title" style={{fontSize:11, marginTop:6}}>size · rig</div>
            <div style={{display:'flex', gap:3, flexWrap:'wrap'}}>
              <Chip tone="leaf" on>Fits rig ✓</Chip>
              <Chip>&lt; 5GB</Chip><Chip>5–15GB</Chip><Chip>&gt; 15GB</Chip>
            </div>
            <div className="side-filter-title" style={{fontSize:11, marginTop:6}}>cost · api</div>
            <div style={{display:'flex', gap:3, flexWrap:'wrap'}}>
              <Chip>Free / OSS</Chip><Chip>$&lt;1 / M</Chip>
              <Chip>$1–5</Chip><Chip>$&gt;5</Chip><Chip>≥99% up</Chip>
            </div>
            <div className="side-filter-title" style={{fontSize:11, marginTop:6}}>license</div>
            <div style={{display:'flex', gap:3, flexWrap:'wrap'}}>
              <Chip on>Apache-2</Chip><Chip>MIT</Chip><Chip>Llama</Chip>
              <Chip>Gemma</Chip><Chip>Proprietary</Chip>
            </div>
            <div className="side-filter-title" style={{fontSize:11, marginTop:6}}>format</div>
            <div style={{display:'flex', gap:3, flexWrap:'wrap'}}>
              <Chip tone="leaf" on>GGUF ✓</Chip>
              <Chip>openai-responses</Chip><Chip>anthropic-messages</Chip>
              <Chip>openrouter</Chip>
            </div>
          </div>
          <div className="m-sheet-actions">
            <Btn variant="ghost" size="xs">Cancel</Btn>
            <Btn variant="primary" size="xs">Apply · 5 filters</Btn>
          </div>
        </div>
      </TabletFrame>

      {/* 3. Header action menu */}
      <TabletFrame label="3 · Header action (⋯ ▾)">
        <MobileHeader active="Models" rightSlot={<DiscoverActionsBtn/>}/>
        <DiscoverMediumToolbar/>
        <div className="t-layout-fixed">
          <DiscoverMediumGrid/>
          <DiscoverMediumRightPanel/>
        </div>
        <DiscoverActionsMenu/>
      </TabletFrame>

      {/* 4. Ranked display mode — Specialization Coding active */}
      <TabletFrame label="4 · Ranked (Coding · HumanEval)">
        <MobileHeader active="Models" rightSlot={<DiscoverActionsBtn/>}/>
        <DiscoverMediumToolbar/>
        <div style={{padding:'0 8px'}}>
          <RankedModeCaption benchmark="HumanEval" specLabel="Coding"/>
          <div style={{display:'flex', flexDirection:'column', gap:4}}>
            {groupIntoRankedRows('HumanEval', 'all').slice(0,6).map(entry => (
              <RankedRow key={entry.rank} entry={entry}/>
            ))}
          </div>
        </div>
      </TabletFrame>
    </div>
  );
}

// v27: unified Models page. Provider Directory absorbed; ranked display mode
// added (model-level rows when a benchmark sort is active). See specs/models.md.
window.ModelsScreens = [
  {label:'A · Models (desktop)', tag:'balanced',
    note:'Unified "Models" page · Models Hub + Discover + Provider Directory all collapsed. Top-of-toolbar mode toggle `[●] My Models · [ ] All Models` swaps the row set. Rows: local aliases/files/api-models/connected providers in My mode; + HF repos + directory providers (from api.getbodhi.app) in All mode. File-first rows carry `↗ catalog` backlinks; hf-repo rows show `✓ N local aliases ↗` when matching aliases exist; unconnected providers carry `from api.getbodhi.app` attribution. Sidebar filters unified: Specialization (single-select, Clear) / Kind / Source / Capability / Size·rig / Cost·api (greyed in My) / License / Format. v27: Specialization=Coding shown active → ranked display mode (model-level rows, local-file dedup, API-config stack, absolute rank numbers).',
    novel:'one page for local + API + remote · mode radio · duality links · directory attribution · ranked display mode',
    component:DiscoverA},
  {label:'A · Models (medium · tablet)', tag:'medium',
    note:'Breadcrumb `Bodhi › Models` (no sub-tab). Mode toggle at top of toolbar. Filters sheet covers unified filter set. 4 frames: grid / filter sheet / header-action menu (+ ▾ Add & Browse) / ranked display mode.',
    novel:'mode toggle compact on tablet · Add+Browse merged menu · ranked display mode',
    component:DiscoverMedium},
  {label:'A · Models (mobile)', tag:'mobile',
    note:'Breadcrumb menu shows Models as a single leaf (no My/Discover sub-tree). Mode toggle is compact pill at top of subbar. `+ ▾` header button opens grouped Add + Browse menu. Six frames: browse · breadcrumb menu · filters sheet · repo detail sheet · header-action menu · ranked (Coding) display mode.',
    novel:'single-leaf Models menu · compact mode pill · grouped add+browse menu · ranked display mode',
    component:DiscoverMobile},
];
