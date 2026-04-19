// Screen 1 — Models Hub (Unified catalog · 3-column chat-app shell)

const ModelCard = ({kind='file', title, subtitle, caps=[], meta, cost, status, selected, onClick}) => {
  const badgeTone = kind==='alias' ? 'saff' : kind==='provider' ? 'indigo' : 'leaf';
  const statusTone =
    status==='live' ? 'leaf' :
    status==='ready' ? 'leaf' :
    status==='oauth' ? 'saff' :
    status==='rate-limited' ? 'warn' :
    status==='tight' ? 'warn' :
    status==='fits' ? 'leaf' : '';
  return (
    <div className={`model-card${selected?' selected':''}`} onClick={onClick} style={{cursor:'pointer'}}>
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

// ── Right-panel variants ───────────────────────────────────────

function AliasPanel() {
  return (
    <>
      <div className="right-collapsed-rail">my-gemma · alias</div>
      <div className="right-topbar">
        <div style={{display:'flex', alignItems:'center', gap:6, flexWrap:'wrap'}}>
          <Chip tone="saff" style={{fontSize:10}}>alias</Chip>
          <span className="h2" style={{margin:0}}>my-gemma</span>
        </div>
        <Btn variant="ghost" size="xs" title="collapse">→</Btn>
      </div>
      <div className="sm">user-defined · created 2026-02-11 · last used 12m ago</div>

      <div style={{display:'flex', gap:4, marginTop:8, flexWrap:'wrap'}}>
        <Chip on>Overview</Chip><Chip>Config</Chip><Chip>Usage</Chip>
      </div>

      <div className="h3">Underlying model file</div>
      <div className="card row" style={{padding:'5px 7px'}}>
        <Chip tone="leaf" style={{fontSize:10}}>file</Chip>
        <code style={{flex:1}}>google/gemma-2-9b:Q4_K_M</code>
        <span className="sm">5.4 GB</span>
        <Btn variant="ghost" size="xs">open →</Btn>
      </div>

      <div className="h3">Runtime config</div>
      <div style={{display:'grid', gridTemplateColumns:'auto 1fr', columnGap:8, rowGap:3}}>
        <span className="sm">ctx</span><span className="sm"><b>16384</b> tokens</span>
        <span className="sm">gpu layers</span><span className="sm">28 / 32</span>
        <span className="sm">temperature</span><span className="sm">0.7</span>
        <span className="sm">top-p</span><span className="sm">0.9</span>
        <span className="sm">stop words</span><span className="sm">[]</span>
        <span className="sm">system prompt</span><span className="sm" style={{fontStyle:'italic'}}>"You are a concise, helpful assistant…"</span>
      </div>

      <div className="h3">Capabilities</div>
      <div style={{display:'flex', gap:4, flexWrap:'wrap'}}>
        <Chip tone="leaf">text→text</Chip><Chip tone="leaf">tool-use</Chip>
      </div>

      <div className="h3">Usage</div>
      <div style={{display:'grid', gridTemplateColumns:'auto 1fr', columnGap:8, rowGap:3}}>
        <span className="sm">sessions (30d)</span><span className="sm"><b>142</b></span>
        <span className="sm">tokens (30d)</span><span className="sm">1.8M in · 420k out</span>
        <span className="sm">avg t/s</span><span className="sm">~36</span>
      </div>

      <div style={{display:'flex', gap:4, marginTop:10, flexWrap:'wrap'}}>
        <Btn variant="primary" size="xs">Open in chat</Btn>
        <Btn size="xs">Edit alias</Btn>
        <Btn variant="ghost" size="xs">Duplicate</Btn>
        <Btn variant="ghost" size="xs" style={{color:'var(--warn)'}}>Delete</Btn>
      </div>
    </>
  );
}

function FilePanel() {
  return (
    <>
      <div className="right-collapsed-rail">google/gemma-2-9b:Q4_K_M · file</div>
      <div className="right-topbar">
        <div style={{display:'flex', alignItems:'center', gap:6, flexWrap:'wrap'}}>
          <Chip tone="leaf" style={{fontSize:10}}>file</Chip>
          <span className="h2" style={{margin:0, fontSize:15}}>google/gemma-2-9b:Q4_K_M</span>
        </div>
        <Btn variant="ghost" size="xs" title="collapse">→</Btn>
      </div>
      <div className="sm">5.4 GB · HuggingFace · Gemma Terms · downloaded 2026-02-10</div>

      <div style={{display:'flex', gap:4, marginTop:8, flexWrap:'wrap'}}>
        <Chip on>Overview</Chip><Chip>Other quants (4)</Chip><Chip>Aliases (2)</Chip><Chip>Usage</Chip>
      </div>

      <div className="h3">Parent repo</div>
      <div className="sm"><code>google/gemma-2-9b</code> · 8.5B params · released 2024-06</div>

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
        <span className="sm">rig fit</span><span className="sm"><TL tone="green">~38 t/s · fits comfortably</TL></span>
      </div>

      <div className="h3">Other quants from this repo</div>
      <div style={{display:'flex', flexDirection:'column', gap:4}}>
        <div className="card row" style={{padding:'5px 7px'}}>
          <Chip tone="leaf" style={{fontSize:10}}>downloaded</Chip>
          <code style={{flex:1}}>:Q5_K_M</code>
          <span className="sm">6.6 GB</span>
          <TL tone="green">~30 t/s</TL>
          <Btn variant="ghost" size="xs">open →</Btn>
        </div>
        <div className="card row" style={{padding:'5px 7px'}}>
          <span className="sm" style={{minWidth:'68px'}}>remote</span>
          <code style={{flex:1}}>:Q3_K_M</code>
          <span className="sm">4.3 GB</span>
          <TL tone="green">~48 t/s</TL>
          <Btn size="xs">pull</Btn>
        </div>
        <div className="card row" style={{padding:'5px 7px'}}>
          <span className="sm" style={{minWidth:'68px'}}>remote</span>
          <code style={{flex:1}}>:Q6_K</code>
          <span className="sm">7.8 GB</span>
          <TL tone="green">~24 t/s</TL>
          <Btn size="xs">pull</Btn>
        </div>
        <div className="card row" style={{padding:'5px 7px'}}>
          <span className="sm" style={{minWidth:'68px'}}>remote</span>
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
          <span className="sm">ctx 16k</span>
          <Btn variant="ghost" size="xs">open →</Btn>
        </div>
      </div>

      <div style={{display:'flex', gap:4, marginTop:10, flexWrap:'wrap'}}>
        <Btn variant="primary" size="xs">+ New alias</Btn>
        <Btn size="xs">Open in chat</Btn>
        <Btn variant="ghost" size="xs">HF ↗</Btn>
      </div>
    </>
  );
}

function ProviderPanel() {
  return (
    <>
      <div className="right-collapsed-rail">openai · provider</div>
      <div className="right-topbar">
        <div style={{display:'flex', alignItems:'center', gap:6, flexWrap:'wrap'}}>
          <Chip tone="indigo" style={{fontSize:10}}>provider</Chip>
          <span className="h2" style={{margin:0}}>openai</span>
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
          <code>codex-latest</code>
          <span className="sm">code · tool</span>
          <span className="sm">plan</span><span className="sm">plan</span><span className="sm">—</span>
          <TL tone="yellow">41</TL>
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
        <span className="sm">auth</span><span className="sm">api-key (sk-…a71e) · <a className="sm" href="#">rotate</a></span>
        <span className="sm">base url</span><span className="sm">api.openai.com/v1</span>
        <span className="sm">status</span><span className="sm"><TL tone="green">live · last ping 2s ago</TL></span>
      </div>

      <div style={{display:'flex', gap:4, marginTop:10, flexWrap:'wrap'}}>
        <Btn variant="primary" size="xs">+ Add as alias</Btn>
        <Btn size="xs">Test connection</Btn>
        <Btn variant="ghost" size="xs">Edit key</Btn>
        <Btn variant="ghost" size="xs" style={{color:'var(--warn)'}}>Remove</Btn>
      </div>
    </>
  );
}

function HubB() {
  const [sel, setSel] = React.useState('file-gemma-q4');
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
          <div className="sub-nav-item active">My Models</div>
          <div className="sub-nav-item">Discover</div>
        </div>

        <Btn variant="primary" style={{width:'100%', justifyContent:'center'}}>+ Add model ▾</Btn>

        <div className="side-sec-label">Models</div>
        <div className="side-nav">
          <div className="side-nav-item active">All models <span className="badge">14</span></div>
          <div className="side-nav-item">Recently used <span className="badge">4</span></div>
          <div className="side-nav-item">Favorites <span className="badge">5</span></div>
          <div className="side-nav-item">Downloads <span className="badge">2</span></div>
        </div>

        <div className="side-sec-label">Sources</div>
        <div className="side-nav">
          <div className="side-nav-item">Local files <span className="badge">8</span></div>
          <div className="side-nav-item">Aliases <span className="badge">2</span></div>
          <div className="side-nav-item">API providers <span className="badge">4</span></div>
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
            <div className="sm">8 files · 2 aliases · 4 providers · 14 total</div>
          </div>
          <div className="main-toolbar">
            <Chip on>All 14</Chip><Chip>Files 8</Chip><Chip>Aliases 2</Chip><Chip>Providers 4</Chip>
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

        <div className="active-filters">
          <span className="active-filters-label">filters:</span>
          <span className="filter-tag">capability: tool-use <span className="x">×</span></span>
          <span className="filter-tag">size: Fits rig ✓ <span className="x">×</span></span>
          <span className="filter-tag">format: openai-responses <span className="x">×</span></span>
          <span className="active-filters-clear">clear all</span>
        </div>

        <div className="cards-grid">
          <ModelCard kind="alias" title="my-gemma"
            subtitle={<>→ <code>google/gemma-2-9b:Q4_K_M</code></>}
            caps={['text→text','tool-use']}
            meta="ctx 16k · gpu 28 layers · used 12m ago"
            status="ready"
            selected={sel==='alias-my-gemma'}
            onClick={()=>setSel('alias-my-gemma')}/>
          <ModelCard kind="alias" title="code-beast"
            subtitle={<>→ <code>qwen/qwen3-14b:Q5_K_M</code></>}
            caps={['text→text','tool-use','structured']}
            meta="ctx 32k · stop [</done>]"
            status="ready"/>

          <ModelCard kind="file" title="google/gemma-2-9b:Q4_K_M"
            subtitle="5.4 GB · 8.5B · HuggingFace"
            caps={['text2text','tool-use']}
            meta="~38 t/s · 3 sibling quants · ↓ 2026-02-10"
            status="fits"
            selected={sel==='file-gemma-q4'}
            onClick={()=>setSel('file-gemma-q4')}/>
          <ModelCard kind="file" title="google/gemma-2-9b:Q5_K_M"
            subtitle="6.6 GB · 8.5B · HuggingFace"
            caps={['text2text','tool-use']}
            meta="~30 t/s · 3 sibling quants"
            status="fits"/>
          <ModelCard kind="file" title="qwen/qwen3-14b:Q5_K_M"
            subtitle="10.1 GB · 14B · ctx 32k"
            caps={['text2text','tool-use','long-ctx']}
            meta="~18 t/s · 3 sibling quants"
            status="tight"/>
          <ModelCard kind="file" title="qwen/qwen3-14b:Q4_K_M"
            subtitle="8.2 GB · 14B · ctx 32k"
            caps={['text2text','tool-use','long-ctx']}
            meta="~24 t/s · 3 sibling quants"
            status="fits"/>
          <ModelCard kind="file" title="LiquidAI/LFM2.5-1.2B:Q8_0"
            subtitle="1.3 GB · 1.2B · edge"
            caps={['text2text']}
            meta="~85 t/s · 2 sibling quants"
            status="fits"/>
          <ModelCard kind="file" title="Qwen/Qwen2.5-VL-7B:Q4_K_M"
            subtitle="4.7 GB · vision-language"
            caps={['multimodal','vision']}
            meta="~32 t/s · 3 sibling quants"
            status="fits"/>
          <ModelCard kind="file" title="nomic-ai/nomic-embed-text-v2:F16"
            subtitle="274 MB · 768-dim"
            caps={['text-embedding']}
            meta="fast · 1 quant"/>
          <ModelCard kind="file" title="deepseek/DeepSeek-R1-14B:Q5_K_M"
            subtitle="10.2 GB · reasoning"
            caps={['text2text','reasoning']}
            meta="~19 t/s · 3 sibling quants"
            status="fits"/>

          <ModelCard kind="provider" title="openai"
            subtitle={<><code>openai-responses</code> · key sk-…a71e</>}
            caps={['tool-use','vision','structured','reasoning','embedding']}
            cost="in $0.05 – $2.00 / M · 7 models"
            meta="gpt-5, gpt-5-mini, gpt-5-nano, o4-mini, +3 more"
            status="live"
            selected={sel==='provider-openai'}
            onClick={()=>setSel('provider-openai')}/>
          <ModelCard kind="provider" title="anthropic"
            subtitle={<><code>anthropic-oauth</code> · Claude Pro</>}
            caps={['tool-use','vision','structured','reasoning']}
            cost="in $0.80 – $15 / M · 5 models"
            meta="claude-opus-4, claude-sonnet-4.5, claude-haiku-4.5, +2 more"
            status="live"/>
          <ModelCard kind="provider" title="google"
            subtitle={<><code>google-gemini</code> · key AIza…</>}
            caps={['tool-use','vision','long-ctx','embedding']}
            cost="in $0.075 – $1.25 / M · 4 models"
            meta="gemini-2.5-pro, gemini-2.5-flash, gemini-2.5-flash-lite, +1 more"
            status="rate-limited"/>
          <ModelCard kind="provider" title="openrouter"
            subtitle={<><code>openai-completions</code> · key sk-or-…</>}
            caps={['tool-use','vision','reasoning']}
            cost="varies by model · 100+ models"
            meta="meta-llama-3.3-70b, mistral-large, deepseek-r1, +97 more"
            status="live"/>
        </div>

        <div style={{textAlign:'center', marginTop:4}}>
          <Btn variant="ghost">Load more →</Btn>
        </div>
      </main>

      {/* ── RIGHT: collapsible detail panel (swaps by selection) ── */}
      <aside className="hub3col-right">
        {sel==='alias-my-gemma' && <AliasPanel/>}
        {sel==='file-gemma-q4' && <FilePanel/>}
        {sel==='provider-openai' && <ProviderPanel/>}
      </aside>
    </div>
  );
}

window.HubScreens = [
  {label:'B · Unified catalog', tag:'balanced', note:'3-column shell with entity-aware detail panel. Click alias/file/provider cards to swap the right-hand view.', novel:'selection-driven right panel (alias | file | provider)', component:HubB},
];
