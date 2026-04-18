// Screen 1 — Models Hub (Unified catalog)

// Compact catalog row: type badge · name · inline meta · right-side status
const CatalogRow = ({kind='file', title, sub, meta=[], right, selected, children}) => {
  const badgeTone = kind==='alias' ? 'saff' : kind==='api' ? 'indigo' : 'leaf';
  const badgeLabel = kind==='alias' ? 'alias' : kind==='api' ? 'api' : 'file';
  return (
    <div className="card row" style={{background: selected?'var(--lotus-soft)':'#fff', padding:'7px 9px', gap:8}}>
      <Chip tone={badgeTone} style={{minWidth:44, textAlign:'center', fontSize:10}}>{badgeLabel}</Chip>
      <div style={{flex:1, minWidth:0}}>
        <div style={{display:'flex', gap:6, alignItems:'center', flexWrap:'wrap'}}>
          <span className="h2" style={{margin:0, fontSize:13}}>{title}</span>
          {children}
        </div>
        {sub && <div className="sm" style={{marginTop:1}}>{sub}</div>}
        {meta.length>0 && (
          <div style={{display:'flex', gap:8, flexWrap:'wrap', marginTop:3}}>
            {meta.map((m,i)=>(<span key={i} className="sm">{m}</span>))}
          </div>
        )}
      </div>
      {right && <div style={{display:'flex', flexDirection:'column', alignItems:'flex-end', gap:3}}>{right}</div>}
    </div>
  );
};

function HubB() {
  return (
    <Browser url="bodhi.local/models">
      {/* Header: title + counts + [+ Add ▾] */}
      <div style={{display:'flex', justifyContent:'space-between', alignItems:'flex-start', gap:10, marginBottom:6}}>
        <div>
          <div className="h1" style={{fontSize:22}}>Models</div>
          <div className="sm">14 model files · 9 aliases · 6 API models</div>
        </div>
        <div style={{position:'relative'}}>
          <Btn variant="primary">+ Add ▾</Btn>
          <Callout style={{top:-4, right:'100%', marginRight:6, whiteSpace:'nowrap'}}>dropdown: New alias · Connect API · Pull from HF · Import</Callout>
        </div>
      </div>

      {/* Type chips + scope toggle */}
      <div style={{display:'flex', justifyContent:'space-between', alignItems:'center', gap:6, flexWrap:'wrap', marginBottom:6}}>
        <div style={{display:'flex', gap:4, flexWrap:'wrap'}}>
          <Chip on>All 29</Chip><Chip>Files 14</Chip><Chip>Aliases 9</Chip><Chip>API 6</Chip>
        </div>
        <div style={{display:'flex', gap:4}}>
          <Chip>Local only</Chip><Chip>API only</Chip>
        </div>
      </div>

      {/* Search */}
      <div style={{position:'relative'}}>
        <Field hint="Search models, providers, or 'I want to summarize PDFs'" filled
          right={<span className="sm">⌘K</span>} />
        <Callout style={{top:-6, right:14}}>Unified omni-search</Callout>
      </div>

      {/* Filters — grouped chip rows */}
      <div style={{marginTop:10, display:'flex', flexDirection:'column', gap:5}}>
        <div style={{display:'flex', gap:6, alignItems:'center', flexWrap:'wrap'}}>
          <span className="lbl" style={{minWidth:72}}>capability</span>
          <Chip>tool-use</Chip><Chip>vision</Chip><Chip>structured-output</Chip>
          <Chip>embedding</Chip><Chip>speech</Chip><Chip>image-gen</Chip>
        </div>
        <div style={{display:'flex', gap:6, alignItems:'center', flexWrap:'wrap'}}>
          <span className="lbl" style={{minWidth:72}}>size · rig</span>
          <Chip tone="leaf">Fits my rig ✓</Chip>
          <Chip>&lt; 5GB</Chip><Chip>5–15GB</Chip><Chip>&gt; 15GB</Chip>
          <Chip>ctx ≥ 32k</Chip>
        </div>
        <div style={{display:'flex', gap:6, alignItems:'center', flexWrap:'wrap'}}>
          <span className="lbl" style={{minWidth:72}}>cost · api</span>
          <Chip>$&lt;1 / M out</Chip><Chip>$1–5</Chip><Chip>$&gt;5</Chip>
          <Chip>≥ 99% uptime</Chip>
        </div>
        <div style={{display:'flex', gap:6, alignItems:'center', flexWrap:'wrap'}}>
          <span className="lbl" style={{minWidth:72}}>format</span>
          <Chip>openai-completions</Chip><Chip>openai-responses</Chip>
          <Chip>anthropic-messages</Chip><Chip>google-gemini</Chip>
          <Chip tone="saff">openai-codex-oauth</Chip><Chip tone="saff">anthropic-oauth</Chip>
        </div>
        <Callout style={{position:'static', display:'inline-block', marginTop:4}}>★ More filters unlock as model metadata loads (license, quant, arch, context, multi-modal…)</Callout>
      </div>

      <div className="divider"/>

      {/* Split: unified list (2/3) + detail panel (1/3) */}
      <div style={{display:'grid', gridTemplateColumns:'1.9fr 1fr', gap:10}}>
        {/* LEFT — unified, type-badged rows */}
        <div style={{display:'flex', flexDirection:'column', gap:5}}>
          <CatalogRow kind="alias" title="my-gemma"
            sub={<>→ <code>google/gemma-2-9b:Q4_K_M</code> · ctx 16k · gpu 28 layers</>}
            right={<><Chip tone="leaf">● ready</Chip><span className="sm">edit</span></>}>
            <Chip>Text→Text</Chip><Chip>tool-use</Chip>
          </CatalogRow>

          <CatalogRow kind="alias" title="code-beast"
            sub={<>→ <code>qwen/qwen3-14b:Q5_K_M</code> · ctx 32k · stop [&lt;/done&gt;]</>}
            right={<><Chip tone="leaf">● ready</Chip><span className="sm">edit</span></>}>
            <Chip>Text→Text</Chip><Chip>tool-use</Chip><Chip>structured</Chip>
          </CatalogRow>

          <CatalogRow kind="file" title="google/gemma-2-9b" selected
            sub="3 quant variants · 8.5B params · ctx 8192"
            meta={[<>↓ 3.8M</>,<>♥ 12.4k</>,<TL tone="green">Q4 ~38 t/s</TL>]}
            right={<Chip>2 aliases</Chip>}>
            <Chip>text2text</Chip><Chip>tool-use</Chip>
          </CatalogRow>

          <CatalogRow kind="file" title="qwen/qwen3-14b"
            sub="4 quant variants · 14B · ctx 32768"
            meta={[<>↓ 886k</>,<TL tone="amber">Q5 ~18 t/s · tight</TL>]}
            right={<Chip>1 alias</Chip>}>
            <Chip>text2text</Chip><Chip>tool-use</Chip><Chip>long-ctx</Chip>
          </CatalogRow>

          <CatalogRow kind="file" title="LiquidAI/LFM2.5-1.2B"
            sub="3 quant variants · 1.2B · ctx 32k"
            meta={[<>↓ 44k</>,<TL tone="green">Q8 ~85 t/s</TL>]}>
            <Chip>text2text</Chip><Chip>edge</Chip>
          </CatalogRow>

          <CatalogRow kind="file" title="Qwen/Qwen2.5-VL-7B"
            sub="Q4_K_M · 4.7GB · vision-language"
            meta={[<>↓ 612k</>,<TL tone="green">Q4 ~32 t/s</TL>]}>
            <Chip>multimodal</Chip><Chip>vision</Chip>
          </CatalogRow>

          <CatalogRow kind="file" title="nomic-ai/nomic-embed-text-v2"
            sub="F16 · 274MB · 768-dim"
            meta={[<>↓ 5.1M</>,<TL tone="green">fast</TL>]}>
            <Chip>text-embedding</Chip>
          </CatalogRow>

          <CatalogRow kind="api" title="anthropic/claude-sonnet-4.5"
            sub={<>format <code>anthropic-oauth</code> · Claude Pro subscription</>}
            meta={[<b>in $3 / out $15 / cached $0.30</b>,<>52 t/s · 99.7% up</>]}
            right={<><Chip tone="leaf">● live</Chip><span className="sm">oauth ↺ 12d</span></>}>
            <Chip>tool-use</Chip><Chip>vision</Chip><Chip>structured</Chip>
          </CatalogRow>

          <CatalogRow kind="api" title="openai/gpt-5-mini"
            sub={<>format <code>openai-responses</code> · key sk-…a71e</>}
            meta={[<b>in $0.25 / out $2 / cached $0.03</b>,<>78 t/s · 99.9% up</>]}
            right={<Chip tone="leaf">● live</Chip>}>
            <Chip>tool-use</Chip><Chip>structured</Chip>
          </CatalogRow>

          <CatalogRow kind="api" title="openai/codex-latest"
            sub={<>format <code>openai-codex-oauth</code> · ChatGPT Plus</>}
            meta={[<b>included in plan</b>,<>~41 t/s</>]}
            right={<Chip tone="saff">● oauth</Chip>}>
            <Chip>code</Chip><Chip>tool-use</Chip>
          </CatalogRow>

          <CatalogRow kind="api" title="google/gemini-2.5-flash-lite"
            sub={<>format <code>google-gemini</code> · key AIza…</>}
            meta={[<b>in $0.075 / out $0.30</b>,<>110 t/s · 98.2% up</>]}
            right={<Chip tone="warn">● rate-limited</Chip>}>
            <Chip>multimodal</Chip><Chip>vision</Chip>
          </CatalogRow>

          <div style={{textAlign:'center', marginTop:4}}>
            <Btn variant="ghost">Load 18 more →</Btn>
          </div>
        </div>

        {/* RIGHT — detail panel for selected (google/gemma-2-9b) */}
        <div className="card" style={{position:'sticky', top:10, alignSelf:'start', padding:10}}>
          <div style={{display:'flex', gap:6, alignItems:'center', flexWrap:'wrap'}}>
            <Chip tone="leaf">file</Chip>
            <span className="h2" style={{margin:0}}>google/gemma-2-9b</span>
          </div>
          <div className="sm" style={{marginTop:2}}>HuggingFace · Gemma Terms · 8.5B params · released 2024-06</div>

          <div style={{display:'flex', gap:4, marginTop:8, flexWrap:'wrap'}}>
            <Chip on>Overview</Chip><Chip>Quant files</Chip><Chip>Aliases (2)</Chip><Chip>Usage</Chip>
          </div>

          <div className="h3" style={{marginTop:10}}>Capabilities</div>
          <div style={{display:'flex', gap:4, flexWrap:'wrap'}}>
            <Chip tone="leaf">text2text</Chip><Chip tone="leaf">tool-use</Chip>
            <Chip>structured-output</Chip><Chip>json-mode</Chip>
          </div>

          <div className="h3" style={{marginTop:10}}>Specs</div>
          <div style={{display:'grid', gridTemplateColumns:'auto 1fr', columnGap:8, rowGap:3}}>
            <span className="sm">ctx</span><span className="sm"><b>8192</b> tokens</span>
            <span className="sm">vocab</span><span className="sm">256k</span>
            <span className="sm">arch</span><span className="sm">gemma2 · GQA</span>
            <span className="sm">rig fit</span><span className="sm"><TL tone="green">Q4/Q5 fit · Q8 tight</TL></span>
          </div>

          <div className="h3" style={{marginTop:10}}>Quant variants</div>
          <div style={{display:'flex', flexDirection:'column', gap:4}}>
            <div className="card row" style={{padding:'5px 7px'}}>
              <code style={{flex:1}}>:Q4_K_M</code>
              <span className="sm">5.4 GB</span>
              <TL tone="green">~38 t/s</TL>
              <Btn size="sm">use</Btn>
            </div>
            <div className="card row" style={{padding:'5px 7px'}}>
              <code style={{flex:1}}>:Q5_K_M</code>
              <span className="sm">6.6 GB</span>
              <TL tone="green">~30 t/s</TL>
              <Btn size="sm">use</Btn>
            </div>
            <div className="card row" style={{padding:'5px 7px'}}>
              <code style={{flex:1}}>:Q8_0</code>
              <span className="sm">9.1 GB</span>
              <TL tone="amber">~18 t/s</TL>
              <Btn size="sm">pull</Btn>
            </div>
          </div>

          <div className="h3" style={{marginTop:10}}>Aliases pointing here</div>
          <div style={{display:'flex', flexDirection:'column', gap:4}}>
            <div className="card row" style={{padding:'5px 7px'}}>
              <Chip tone="saff" style={{fontSize:10}}>alias</Chip>
              <code style={{flex:1}}>my-gemma</code>
              <span className="sm">:Q4_K_M · 16k</span>
            </div>
            <div className="card row" style={{padding:'5px 7px'}}>
              <Chip tone="saff" style={{fontSize:10}}>alias</Chip>
              <code style={{flex:1}}>gemma-writer</code>
              <span className="sm">:Q5_K_M · system=…</span>
            </div>
          </div>

          <div style={{display:'flex', gap:4, marginTop:10, flexWrap:'wrap'}}>
            <Btn variant="primary" size="sm">+ New alias</Btn>
            <Btn size="sm">Open in chat</Btn>
            <Btn variant="ghost" size="sm">HF ↗</Btn>
          </div>
        </div>
      </div>
    </Browser>
  );
}

window.HubScreens = [
  {label:'B · Unified catalog', tag:'balanced', note:'Single searchable list with filter chips across local + API. Omni-search front and center.', novel:'single filter bar spans local + API models', component:HubB},
];
