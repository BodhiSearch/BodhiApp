// Screen 1 — Models Hub (Unified catalog · 3-column chat-app shell)

const ModelCard = ({kind='file', title, subtitle, caps=[], meta, cost, status, selected}) => {
  const badgeTone = kind==='alias' ? 'saff' : kind==='api' ? 'indigo' : 'leaf';
  const statusTone =
    status==='live' ? 'leaf' :
    status==='ready' ? 'leaf' :
    status==='oauth' ? 'saff' :
    status==='rate-limited' ? 'warn' :
    status==='tight' ? 'warn' :
    status==='fits' ? 'leaf' : '';
  return (
    <div className={`model-card${selected?' selected':''}`}>
      <div className="model-card-head">
        <Chip tone={badgeTone} style={{fontSize:10}}>{kind}</Chip>
        {status && <Chip tone={statusTone}>● {status}</Chip>}
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

function HubB() {
  return (
    <div className="hub3col">

      {/* ── LEFT: navigation + filters ── */}
      <aside className="hub3col-left">
        <div className="side-brand">
          <span className="brand-dot">◉</span>
          <span>Bodhi</span>
        </div>

        <Btn variant="primary" style={{width:'100%', justifyContent:'center'}}>+ Add model ▾</Btn>

        <div className="side-sec-label">Models</div>
        <div className="side-nav">
          <div className="side-nav-item active">All models <span className="badge">29</span></div>
          <div className="side-nav-item">Recently used <span className="badge">4</span></div>
          <div className="side-nav-item">Favorites <span className="badge">5</span></div>
          <div className="side-nav-item">Downloads <span className="badge">2</span></div>
        </div>

        <div className="side-sec-label">Sources</div>
        <div className="side-nav">
          <div className="side-nav-item">Local files <span className="badge">14</span></div>
          <div className="side-nav-item">Aliases <span className="badge">9</span></div>
          <div className="side-nav-item">API providers <span className="badge">6</span></div>
        </div>

        <div className="side-sec-label">Filters</div>
        <div className="side-filter-group">
          <div className="side-filter-title">capability</div>
          <div className="chips-col">
            <Chip>tool-use</Chip><Chip>vision</Chip><Chip>structured</Chip>
            <Chip>embedding</Chip><Chip>speech</Chip><Chip>image-gen</Chip>
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
            <Chip>$&lt;1 / M</Chip><Chip>$1–5</Chip><Chip>$&gt;5</Chip>
            <Chip>≥99% up</Chip>
          </div>
        </div>
        <div className="side-filter-group">
          <div className="side-filter-title">format</div>
          <div className="chips-col">
            <Chip>openai-completions</Chip><Chip>openai-responses</Chip>
            <Chip>anthropic-messages</Chip><Chip>google-gemini</Chip>
            <Chip tone="saff">openai-codex-oauth</Chip>
            <Chip tone="saff">anthropic-oauth</Chip>
          </div>
        </div>
      </aside>

      {/* ── CENTER: search + scope + card grid ── */}
      <main className="hub3col-main">
        <div className="main-topbar">
          <div>
            <div className="h1" style={{fontSize:20}}>Models</div>
            <div className="sm">14 files · 9 aliases · 6 API · 29 total</div>
          </div>
          <div className="main-toolbar">
            <Chip on>All 29</Chip><Chip>Files 14</Chip><Chip>Aliases 9</Chip><Chip>API 6</Chip>
            <span className="vsep"/>
            <Chip>Local only</Chip><Chip>API only</Chip>
            <span className="vsep"/>
            <Chip on>▦ Cards</Chip><Chip>☰ List</Chip>
          </div>
        </div>

        <div style={{position:'relative'}}>
          <Field hint="Search models, providers, or 'I want to summarize PDFs'" filled
            right={<span className="sm">⌘K</span>}/>
          <Callout style={{top:-6, right:14}}>Unified omni-search</Callout>
        </div>

        <div className="cards-grid">
          <ModelCard kind="alias" title="my-gemma"
            subtitle={<>→ <code>google/gemma-2-9b:Q4_K_M</code></>}
            caps={['text→text','tool-use']}
            meta="ctx 16k · gpu 28 layers"
            status="ready"/>
          <ModelCard kind="alias" title="code-beast"
            subtitle={<>→ <code>qwen/qwen3-14b:Q5_K_M</code></>}
            caps={['text→text','tool-use','structured']}
            meta="ctx 32k · stop [</done>]"
            status="ready"/>
          <ModelCard kind="file" title="google/gemma-2-9b"
            subtitle="8.5B params · HuggingFace"
            caps={['text2text','tool-use']}
            meta="3 quants · Q4 5.4GB ~38 t/s · ↓ 3.8M · ♥ 12.4k"
            status="fits" selected/>
          <ModelCard kind="file" title="qwen/qwen3-14b"
            subtitle="14B · ctx 32k"
            caps={['text2text','tool-use','long-ctx']}
            meta="4 quants · Q5 ~18 t/s · ↓ 886k"
            status="tight"/>
          <ModelCard kind="file" title="LiquidAI/LFM2.5-1.2B"
            subtitle="1.2B · edge"
            caps={['text2text']}
            meta="3 quants · Q8 ~85 t/s · ↓ 44k"/>
          <ModelCard kind="file" title="Qwen/Qwen2.5-VL-7B"
            subtitle="Q4_K_M · 4.7GB"
            caps={['multimodal','vision']}
            meta="vision-language · ↓ 612k · ~32 t/s"/>
          <ModelCard kind="file" title="nomic-ai/nomic-embed-text-v2"
            subtitle="F16 · 274MB · 768-dim"
            caps={['text-embedding']}
            meta="↓ 5.1M · fast"/>
          <ModelCard kind="api" title="anthropic/claude-sonnet-4.5"
            subtitle={<><code>anthropic-oauth</code> · Claude Pro</>}
            caps={['tool-use','vision','structured']}
            cost="in $3 / out $15 / cached $0.30 per M"
            meta="52 t/s · 99.7% up · oauth ↺ 12d"
            status="live"/>
          <ModelCard kind="api" title="openai/gpt-5-mini"
            subtitle={<><code>openai-responses</code> · key sk-…a71e</>}
            caps={['tool-use','structured']}
            cost="in $0.25 / out $2 / cached $0.03"
            meta="78 t/s · 99.9% up"
            status="live"/>
          <ModelCard kind="api" title="openai/codex-latest"
            subtitle={<><code>openai-codex-oauth</code> · ChatGPT Plus</>}
            caps={['code','tool-use']}
            cost="included in plan"
            meta="~41 t/s"
            status="oauth"/>
          <ModelCard kind="api" title="google/gemini-2.5-flash-lite"
            subtitle={<><code>google-gemini</code> · key AIza…</>}
            caps={['multimodal','vision']}
            cost="in $0.075 / out $0.30"
            meta="110 t/s · 98.2% up"
            status="rate-limited"/>
          <ModelCard kind="api" title="anthropic/claude-haiku-4.5"
            subtitle={<><code>anthropic-messages</code> · key</>}
            caps={['tool-use','vision']}
            cost="in $0.80 / out $4 / cached $0.08"
            meta="140 t/s · 99.8% up"
            status="live"/>
        </div>

        <div style={{textAlign:'center', marginTop:4}}>
          <Btn variant="ghost">Load 17 more →</Btn>
        </div>
      </main>

      {/* ── RIGHT: collapsible detail panel ── */}
      <aside className="hub3col-right">
        <div className="right-collapsed-rail">google/gemma-2-9b · details</div>

        <div className="right-topbar">
          <div style={{display:'flex', alignItems:'center', gap:6, flexWrap:'wrap'}}>
            <Chip tone="leaf" style={{fontSize:10}}>file</Chip>
            <span className="h2" style={{margin:0}}>google/gemma-2-9b</span>
          </div>
          <Btn variant="ghost" size="xs" title="collapse">→</Btn>
        </div>
        <div className="sm">HuggingFace · Gemma Terms · 8.5B · released 2024-06</div>

        <div style={{display:'flex', gap:4, marginTop:8, flexWrap:'wrap'}}>
          <Chip on>Overview</Chip><Chip>Quant files</Chip><Chip>Aliases (2)</Chip><Chip>Usage</Chip>
        </div>

        <div className="h3">Capabilities</div>
        <div style={{display:'flex', gap:4, flexWrap:'wrap'}}>
          <Chip tone="leaf">text2text</Chip><Chip tone="leaf">tool-use</Chip>
          <Chip>structured-output</Chip><Chip>json-mode</Chip>
        </div>

        <div className="h3">Specs</div>
        <div style={{display:'grid', gridTemplateColumns:'auto 1fr', columnGap:8, rowGap:3}}>
          <span className="sm">ctx</span><span className="sm"><b>8192</b> tokens</span>
          <span className="sm">vocab</span><span className="sm">256k</span>
          <span className="sm">arch</span><span className="sm">gemma2 · GQA</span>
          <span className="sm">rig fit</span><span className="sm"><TL tone="green">Q4/Q5 fit · Q8 tight</TL></span>
        </div>

        <div className="h3">Quant variants</div>
        <div style={{display:'flex', flexDirection:'column', gap:4}}>
          <div className="card row" style={{padding:'5px 7px'}}>
            <code style={{flex:1}}>:Q4_K_M</code>
            <span className="sm">5.4 GB</span>
            <TL tone="green">~38 t/s</TL>
            <Btn size="xs">use</Btn>
          </div>
          <div className="card row" style={{padding:'5px 7px'}}>
            <code style={{flex:1}}>:Q5_K_M</code>
            <span className="sm">6.6 GB</span>
            <TL tone="green">~30 t/s</TL>
            <Btn size="xs">use</Btn>
          </div>
          <div className="card row" style={{padding:'5px 7px'}}>
            <code style={{flex:1}}>:Q8_0</code>
            <span className="sm">9.1 GB</span>
            <TL tone="yellow">~18 t/s</TL>
            <Btn size="xs">pull</Btn>
          </div>
        </div>

        <div className="h3">Aliases pointing here</div>
        <div style={{display:'flex', flexDirection:'column', gap:4}}>
          <div className="card row" style={{padding:'5px 7px'}}>
            <Chip tone="saff" style={{fontSize:10}}>alias</Chip>
            <code style={{flex:1}}>my-gemma</code>
            <span className="sm">:Q4_K_M · 16k</span>
          </div>
          <div className="card row" style={{padding:'5px 7px'}}>
            <Chip tone="saff" style={{fontSize:10}}>alias</Chip>
            <code style={{flex:1}}>gemma-writer</code>
            <span className="sm">:Q5_K_M</span>
          </div>
        </div>

        <div style={{display:'flex', gap:4, marginTop:10, flexWrap:'wrap'}}>
          <Btn variant="primary" size="xs">+ New alias</Btn>
          <Btn size="xs">Open in chat</Btn>
          <Btn variant="ghost" size="xs">HF ↗</Btn>
        </div>
      </aside>
    </div>
  );
}

window.HubScreens = [
  {label:'B · Unified catalog', tag:'balanced', note:'3-column chat-app shell: nav + filters left, search + scope + card grid center, collapsible details right.', novel:'card/list toggle with sticky filter sidebar', component:HubB},
];
