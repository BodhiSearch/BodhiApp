// Screen 2 — Discover (HF repos + API providers, 3-column shell)

const DiscoverCard = ({kind='hf-repo', title, subtitle, caps=[], meta, cost, status, fit, fitLabel, selected, onClick}) => {
  const kindTone = kind==='provider' ? 'indigo' : kind==='provider-off' ? '' : 'leaf';
  const statusTone =
    status==='connected' ? 'leaf' :
    status==='oauth' ? 'saff' :
    status==='not connected' ? '' :
    status==='rate-limited' ? 'warn' : '';
  const fitTone = fit==='green' ? 'leaf' : fit==='yellow' ? 'warn' : fit==='red' ? 'warn' : '';
  const extraClass = kind==='provider-off' ? ' dashed' : '';
  return (
    <div className={`model-card${selected?' selected':''}${extraClass}`} onClick={onClick} style={{cursor:'pointer'}}>
      <div className="model-card-head">
        <Chip tone={kindTone} style={{fontSize:10}}>{kind==='provider-off' ? 'provider' : kind}</Chip>
        {status && <Chip tone={statusTone}>● {status}</Chip>}
        {fitLabel && <Chip tone={fitTone}>● {fitLabel}</Chip>}
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

        <div className="side-section-picker">
          <span className="side-section-picker-icon">▦</span>
          <span style={{flex:1}}>Models</span>
          <span className="sm">▾</span>
        </div>

        <div className="sub-nav">
          <div className="sub-nav-item">My Models</div>
          <div className="sub-nav-item active">Discover</div>
        </div>

        <DownloadsMenu active={sel==='downloads'} count={1} onClick={()=>setSel('downloads')}/>

        <div className="side-sec-label">Discover</div>
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
          <div className="side-filter-title">source</div>
          <div className="chips-col">
            <Chip>HuggingFace</Chip><Chip>OpenRouter</Chip>
            <Chip>NVIDIA NIM</Chip><Chip>Groq</Chip>
            <Chip>Together</Chip><Chip>HF Inference</Chip>
            <Chip>Anthropic</Chip><Chip>OpenAI</Chip><Chip>Google</Chip>
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
        <div className="side-filter-group">
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
            <div className="h1" style={{fontSize:20}}>Discover</div>
            <div className="sm">3.1M HF repos + 12 API providers · curated for M3 Max 36GB</div>
          </div>
          <div className="main-toolbar">
            <Chip on>All</Chip><Chip>Local</Chip><Chip>API</Chip>
            <span className="vsep"/>
            <Chip tone="leaf">Fits rig ✓</Chip>
            <span className="vsep"/>
            <Chip on>Likes</Chip><Chip>Downloads</Chip><Chip>Recent</Chip>
            <span className="vsep"/>
            <Chip on={view==='cards'} onClick={()=>setView('cards')}>▦ Cards</Chip>
            <Chip on={view==='list'} onClick={()=>setView('list')}>☰ List</Chip>
          </div>
        </div>

        <div style={{position:'relative'}}>
          <Field hint="Filter repos & providers — e.g. 'vision 7B apache' or 'claude tool-use'" filled
            right={<span className="sm">⌘K</span>}/>
          <Callout style={{top:-6, right:14}}>Unified local + API discovery</Callout>
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

        <div className={view==='list' ? 'cards-list' : 'cards-grid'}>
          <Row kind="hf-repo" title="Qwen/Qwen3.5-9B"
            subtitle="9B · ctx 32k · Apache-2 · HuggingFace"
            caps={['text2text','tool-use','reasoning']}
            meta={<>default <code>:Q4_K_M</code> · 5.6GB · 5 quants · ↓ 443k · ♥ 3.1k</>}
            fit="green" fitLabel="~38 t/s"
            selected={sel==='hf-qwen'}
            onClick={()=>setSel('hf-qwen')}/>
          <Row kind="provider" title="openai"
            subtitle={<><code>openai-responses</code> · key sk-…a71e</>}
            caps={['tool-use','vision','structured','reasoning','embedding']}
            cost="in $0.05 – $2.00 / M · 7 models"
            meta="gpt-5, gpt-5-mini, gpt-5-nano, o4-mini, +3 more"
            status="connected"
            selected={sel==='provider-openai'}
            onClick={()=>setSel('provider-openai')}/>
          <Row kind="provider-off" title="groq"
            subtitle={<><code>openai-completions</code> · bring-your-own-key</>}
            caps={['tool-use','speech','moderation']}
            cost="in $0.05 – $0.75 / M · 6 models"
            meta="llama-3.3-70b, llama-3.1-8b, qwen-2.5-32b, +3 more"
            status="not connected"
            selected={sel==='provider-groq'}
            onClick={()=>setSel('provider-groq')}/>

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
            status="not connected"/>
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
            status="not connected"/>
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
            status="not connected"/>
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
        </div>

        <div style={{textAlign:'center', marginTop:4}}>
          <Btn variant="ghost">Load more →</Btn>
        </div>
      </main>

      {/* ── RIGHT: collapsible detail panel (swaps by selection) ── */}
      <aside className="hub3col-right">
        {sel==='hf-qwen' && <HfRepoPanel/>}
        {sel==='provider-openai' && <ConnectedProviderPanel/>}
        {sel==='provider-groq' && <UnconnectedProviderPanel/>}
        {sel==='downloads' && <DownloadsPanel/>}
      </aside>
    </div>
  );
}

// ── Mobile variant ────────────────────────────────────────────

// Action button that replaces the DL icon in the Discover header.
// Opens a dropdown menu listing browse modes + Downloads + Leaderboards.
const DiscoverActionsBtn = () => (
  <span className="m-ico m-ico-action" title="browse & actions">⋯ ▾</span>
);

// The dropdown that drops from the Discover header action.
const DiscoverActionsMenu = () => (
  <div className="m-menu-overlay" style={{justifyContent:'flex-end', paddingRight:6, paddingTop:40}}>
    <div className="m-menu" style={{width:'74%'}}>
      <div className="m-menu-item">↑ Trending</div>
      <div className="m-menu-item">★ New launches</div>
      <div className="m-menu-item">
        <span>↓ Downloads</span>
        <span className="m-menu-badge">1</span>
      </div>
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

const DiscoverMobileSubbar = ({filtersCount=5}) => (
  <>
    <div className="t-search" style={{fontSize:10}}><span>Filter repos & providers…</span><span>⌘K</span></div>
    <div className="m-toolbar">
      <span className="m-filter-btn">Filters <span className="m-filter-badge">{filtersCount}</span> ▾</span>
      <span className="sm">Sort: Likes ▾</span>
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
        <MobileHeader active="Discover" rightSlot={<DiscoverActionsBtn/>}/>
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
        <MobileHeader active="Discover" rightSlot={<DiscoverActionsBtn/>}/>
        <MobileMenu active="Discover"/>
        <Callout style={{position:'static', fontSize:9, margin:'4px 0'}}>breadcrumb menu · Models expands · Discover active</Callout>
      </PhoneFrame>

      {/* ── 3. Filters sheet ── */}
      <PhoneFrame label="3 · Filters sheet">
        <div className="phone-dim">
          <MobileHeader active="Discover" rightSlot={<DiscoverActionsBtn/>}/>
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
          <MobileHeader active="Discover" rightSlot={<DiscoverActionsBtn/>}/>
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
        <MobileHeader active="Discover" rightSlot={<DiscoverActionsBtn/>}/>
        <DiscoverActionsMenu/>
        <Callout style={{position:'static', fontSize:9, margin:'4px 0'}}>tap ⋯ ▾ · Trending / New launches / Downloads / Leaderboards ›</Callout>
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

const DiscoverMediumToolbar = ({filtersCount=5}) => (
  <>
    <div className="t-search" style={{fontSize:11}}><span>Filter repos & providers — e.g. 'vision 7B apache' or 'claude tool-use'…</span><span>⌘K</span></div>
    <div className="m-toolbar">
      <span className="m-filter-btn">Filters <span className="m-filter-badge">{filtersCount}</span> ▾</span>
      <span className="sm">Scope: All ▾</span>
      <span className="sm">Sort: Likes ▾</span>
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
        <MobileHeader active="Discover" rightSlot={<DiscoverActionsBtn/>}/>
        <DiscoverMediumToolbar/>
        <div className="t-layout-fixed">
          <DiscoverMediumGrid/>
          <DiscoverMediumRightPanel/>
        </div>
      </TabletFrame>

      {/* 2. Filters sheet (compact centered) */}
      <TabletFrame label="2 · Filters sheet (from Filters ▾)">
        <div className="phone-dim">
          <MobileHeader active="Discover" rightSlot={<DiscoverActionsBtn/>}/>
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
        <MobileHeader active="Discover" rightSlot={<DiscoverActionsBtn/>}/>
        <DiscoverMediumToolbar/>
        <div className="t-layout-fixed">
          <DiscoverMediumGrid/>
          <DiscoverMediumRightPanel/>
        </div>
        <DiscoverActionsMenu/>
      </TabletFrame>
    </div>
  );
}

window.DiscoverScreens = [
  {label:'A · HF repos + providers', tag:'balanced', note:'Same 3-column shell as Hub. Repo-level HF cards (with default-quant badge) mixed with API provider cards (connected + not-yet-connected). Three demo cards are clickable.', novel:'repo + provider unified grid, connected vs unconnected provider states', component:DiscoverA},
  {label:'A · Medium (tablet)', tag:'medium', note:'Breadcrumb header + ⋯ ▾ action. 2-col grid + fixed right detail panel. Filters sheet now includes Specialization group (single-select · Coding shown active · Clear). Active-filter pill shows "Coding · sort ⭐ HumanEval". 3 frames.', novel:'Specialization filter in sheet · sort follows benchmark · Clear affordance', component:DiscoverMedium},
  {label:'A · Mobile', tag:'mobile', note:'Breadcrumb menu. ⋯ ▾ header action. Filters sheet: Specialization (single-select) / source / capability / size / cost / license / format. Specialization pill visible in active-filters strip. Five frames.', novel:'Specialization as single-select filter with Clear and bench-driven sort', component:DiscoverMobile},
];
