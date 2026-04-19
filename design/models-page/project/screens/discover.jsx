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

      <div className="h3">Quants · pull</div>
      <div style={{display:'flex', flexDirection:'column', gap:4}}>
        <div className="card row" style={{padding:'5px 7px', borderColor:'var(--ink)'}}>
          <Chip tone="saff" style={{fontSize:10}}>default</Chip>
          <code style={{flex:1}}>:Q4_K_M</code>
          <span className="sm">5.6 GB</span>
          <TL tone="green">~38 t/s</TL>
          <Btn variant="primary" size="xs">pull</Btn>
        </div>
        <div className="card row" style={{padding:'5px 7px'}}>
          <code style={{flex:1}}>:Q5_K_M</code>
          <span className="sm">6.8 GB</span>
          <TL tone="green">~30 t/s</TL>
          <Btn size="xs">pull</Btn>
        </div>
        <div className="card row" style={{padding:'5px 7px'}}>
          <code style={{flex:1}}>:Q6_K</code>
          <span className="sm">7.9 GB</span>
          <TL tone="green">~24 t/s</TL>
          <Btn size="xs">pull</Btn>
        </div>
        <div className="card row" style={{padding:'5px 7px'}}>
          <code style={{flex:1}}>:Q8_0</code>
          <span className="sm">9.6 GB</span>
          <TL tone="yellow">~16 t/s</TL>
          <Btn size="xs">pull</Btn>
        </div>
        <div className="card row" style={{padding:'5px 7px'}}>
          <code style={{flex:1}}>:F16</code>
          <span className="sm">18 GB</span>
          <TL tone="warn">won't fit</TL>
          <Btn variant="ghost" size="xs">pull</Btn>
        </div>
      </div>

      <div className="h3">README snippet</div>
      <div className="sm" style={{fontStyle:'italic', borderLeft:'2px dashed var(--line-soft)', paddingLeft:8}}>
        Qwen3.5-9B is a 9B instruction-tuned model with improved tool-use, 32k context, and strong reasoning. Drop-in replacement for Qwen2.5-9B.
      </div>

      <div style={{display:'flex', gap:4, marginTop:10, flexWrap:'wrap'}}>
        <Btn variant="primary" size="xs">Pull default (Q4_K_M)</Btn>
        <Btn variant="ghost" size="xs">★ Favorite</Btn>
        <Btn variant="ghost" size="xs">HF ↗</Btn>
      </div>
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

function DiscoverA() {
  const [sel, setSel] = React.useState('hf-qwen');
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
            <Chip on>▦ Cards</Chip><Chip>☰ List</Chip>
          </div>
        </div>

        <div style={{position:'relative'}}>
          <Field hint="Filter repos & providers — e.g. 'vision 7B apache' or 'claude tool-use'" filled
            right={<span className="sm">⌘K</span>}/>
          <Callout style={{top:-6, right:14}}>Unified local + API discovery</Callout>
        </div>

        <div className="active-filters">
          <span className="active-filters-label">filters:</span>
          <span className="filter-tag">capability: tool-use <span className="x">×</span></span>
          <span className="filter-tag">size: Fits rig ✓ <span className="x">×</span></span>
          <span className="filter-tag">license: Apache-2 <span className="x">×</span></span>
          <span className="active-filters-clear">clear all</span>
        </div>

        <div className="cards-grid">
          <DiscoverCard title="Qwen/Qwen3.5-9B"
            subtitle="9B · ctx 32k · Apache-2 · HuggingFace"
            caps={['text2text','tool-use','reasoning']}
            meta={<>default <code>:Q4_K_M</code> · 5.6GB · 5 quants · ↓ 443k · ♥ 3.1k</>}
            fit="green" fitLabel="~38 t/s"
            selected={sel==='hf-qwen'}
            onClick={()=>setSel('hf-qwen')}/>
          <DiscoverCard kind="provider" title="openai"
            subtitle={<><code>openai-responses</code> · key sk-…a71e</>}
            caps={['tool-use','vision','structured','reasoning','embedding']}
            cost="in $0.05 – $2.00 / M · 7 models"
            meta="gpt-5, gpt-5-mini, gpt-5-nano, o4-mini, +3 more"
            status="connected"
            selected={sel==='provider-openai'}
            onClick={()=>setSel('provider-openai')}/>
          <DiscoverCard kind="provider-off" title="groq"
            subtitle={<><code>openai-completions</code> · bring-your-own-key</>}
            caps={['tool-use','speech','moderation']}
            cost="in $0.05 – $0.75 / M · 6 models"
            meta="llama-3.3-70b, llama-3.1-8b, qwen-2.5-32b, +3 more"
            status="not connected"
            selected={sel==='provider-groq'}
            onClick={()=>setSel('provider-groq')}/>

          <DiscoverCard title="google/gemma-4-e2b"
            subtitle="2B · vision · Gemma T&C · HuggingFace"
            caps={['multimodal','vision']}
            meta={<>default <code>:Q4_K_M</code> · 1.4GB · 3 quants · ↓ 3.8M · ♥ 2.1k</>}
            fit="green" fitLabel="~85 t/s"/>
          <DiscoverCard kind="provider-off" title="openrouter"
            subtitle={<><code>openai-completions</code> · multi-provider routing</>}
            caps={['tool-use','vision','reasoning']}
            cost="varies · 100+ models"
            meta="meta-llama-3.3-70b, mistral-large, deepseek-r1, +97 more"
            status="not connected"/>
          <DiscoverCard title="unsloth/Nemotron-3-Nano-30B"
            subtitle="30B · ctx 128k · NVIDIA · HuggingFace"
            caps={['text2text','reasoning','long-ctx']}
            meta={<>default <code>:Q4_K_M</code> · 17GB · 4 quants · ↓ 133k</>}
            fit="yellow" fitLabel="~6 t/s · tight"/>
          <DiscoverCard kind="provider-off" title="anthropic"
            subtitle={<><code>anthropic-oauth</code> · Claude Pro / key</>}
            caps={['tool-use','vision','structured','reasoning']}
            cost="in $0.80 – $15 / M · 5 models"
            meta="claude-opus-4, claude-sonnet-4.5, claude-haiku-4.5, +2 more"
            status="not connected"/>
          <DiscoverCard title="LiquidAI/LFM2.5-1.2B"
            subtitle="1.2B · edge · Apache-2 · HuggingFace"
            caps={['text2text']}
            meta={<>default <code>:Q8_0</code> · 1.3GB · 3 quants · ↓ 28k</>}
            fit="green" fitLabel="~110 t/s"/>
          <DiscoverCard kind="provider-off" title="nvidia-nim"
            subtitle={<><code>openai-completions</code> · NVIDIA NIM · key</>}
            caps={['text2text','tool-use','long-ctx']}
            cost="in $0.60 – $1.80 / M · 18 models"
            meta="nemotron-4-340b, llama-3.3-70b-nim, mistral-large-nim, +15 more"
            status="not connected"/>
          <DiscoverCard title="Qwen/Qwen2.5-VL-7B"
            subtitle="7B · vision-language · Apache-2 · HuggingFace"
            caps={['multimodal','vision']}
            meta={<>default <code>:Q4_K_M</code> · 4.7GB · 4 quants · ↓ 612k</>}
            fit="green" fitLabel="~32 t/s"/>
          <DiscoverCard kind="provider-off" title="together"
            subtitle={<><code>openai-completions</code> · Together AI · key</>}
            caps={['text2text','tool-use','long-ctx','image-gen']}
            cost="in $0.10 – $3.50 / M · 40+ models"
            meta="qwen-2.5-72b, deepseek-v3, flux-1.1-pro, +37 more"
            status="not connected"/>
          <DiscoverCard title="deepseek/DeepSeek-R1-Distill-14B"
            subtitle="14B · reasoning · MIT · HuggingFace"
            caps={['text2text','reasoning']}
            meta={<>default <code>:Q5_K_M</code> · 10.2GB · 4 quants · ↓ 1.1M</>}
            fit="green" fitLabel="~19 t/s"/>
          <DiscoverCard kind="provider-off" title="huggingface"
            subtitle={<><code>openai-completions</code> · HF Inference · key</>}
            caps={['text2text','embedding','image-gen']}
            cost="PAYG + Pro $9/mo · 500+ models"
            meta="mistral-large, llama-3.3, stable-diffusion-xl, +497 more"
            status="not connected"/>
          <DiscoverCard title="meta/Llama-3.3-70B-Instruct"
            subtitle="70B · ctx 128k · Llama-3 license · HuggingFace"
            caps={['text2text','tool-use','long-ctx']}
            meta={<>default <code>:Q4_K_M</code> · 40GB · 5 quants · ↓ 2.4M</>}
            fit="red" fitLabel="won't fit"/>
          <DiscoverCard kind="provider-off" title="google-gemini"
            subtitle={<><code>google-gemini</code> · key AIza…</>}
            caps={['tool-use','vision','long-ctx','embedding']}
            cost="in $0.075 – $1.25 / M · 4 models"
            meta="gemini-2.5-pro, gemini-2.5-flash, gemini-2.5-flash-lite, +1 more"
            status="not connected"/>
          <DiscoverCard title="nomic-ai/nomic-embed-text-v2"
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

window.DiscoverScreens = [
  {label:'A · HF repos + providers', tag:'balanced', note:'Same 3-column shell as Hub. Repo-level HF cards (with default-quant badge) mixed with API provider cards (connected + not-yet-connected). Three demo cards are clickable.', novel:'repo + provider unified grid, connected vs unconnected provider states', component:DiscoverA},
];
