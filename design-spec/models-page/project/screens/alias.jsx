// Screen 3 — Create / edit local alias (v21 · raw-args model)
//
// Learning: keeping a fixed form in sync with llama.cpp is painful — flags
// change frequently between releases. The alias stores raw llama-server
// cmdline args as lines of text. The UI surrounds that textarea with
// assistance (presets, palette built from parsed --help, per-line help,
// enum pickers, paste-command import, validation) without hiding the raw
// lines. OpenAI-compatible request defaults remain a stable fixed form.
//
// Four variants share the same section list:
//   1. Identity             (fixed form)
//   2. Model file           (fixed form · quant picker + download strip)
//   3. Preset               (optional · seeds Runtime args)
//   4. Runtime args         (ArgsEditor — raw llama-server lines)
//   5. Request defaults     (fixed OpenAI-compat form)

// ── 1. Identity ───────────────────────────────────────────────────
const IdentityBody = () => (
  <>
    <Field label="Alias name" filled value="qwen-chat"/>
    <div style={{display:'flex', gap:6, marginTop:6}}>
      <Chip on>chat</Chip><Chip>tool-use</Chip><Chip>+ tag</Chip>
    </div>
    <div className="sm" style={{marginTop:4}}>Alias names can only contain lowercase, digits, and dashes.</div>
  </>
);

// ── 2. Model file ─────────────────────────────────────────────────
// Repo + Snapshot live here. Quant is the same input as filename so we
// don't show a separate filename field — the Quant picker below *is* the
// file selector.
const ModelFileBody = () => (
  <>
    <div style={{display:'grid', gridTemplateColumns:'1fr 1fr', gap:6}}>
      <Field label="Repo" filled value="Qwen/Qwen3.5-9B-GGUF" right={<span className="sm">▾</span>}/>
      <Field label="Snapshot" filled value="116f76…d44ec8ff1" right={<span className="sm">▾</span>}/>
    </div>
    <div className="h3" style={{marginTop:8}}>Quant · picks the file</div>
    <QuantPicker quants={DEFAULT_QUANTS} selected=":Q4_K_M"/>
    <div className="sm" style={{marginTop:4}}>
      Selecting a quant sets the filename — e.g. <code>:Q4_K_M</code> → <code>Qwen3.5-9B-Q4_K_M.gguf</code>. No separate filename input.
    </div>
    <DownloadProgressStrip state="queued" pct={0}/>
  </>
);

// ── 5. Request defaults (OpenAI schema — stable, fixed form is safe) ──
const RequestDefaultsBody = () => (
  <>
    <div className="sm" style={{marginBottom:6}}>
      Applied as defaults on every chat request — overridable per-call. Stable OpenAI schema, so a fixed form is safe here.
    </div>
    <div style={{display:'grid', gridTemplateColumns:'1fr 1fr', gap:6}}>
      <Field label="temperature" filled value="0.7"/>
      <Field label="top_p" filled value="0.95"/>
      <Field label="max_tokens" hint="unset"/>
      <Field label="seed" hint="unset"/>
      <Field label="frequency_penalty" filled value="0"/>
      <Field label="presence_penalty" filled value="0"/>
      <Field label="stop" hint="[ ]"/>
      <Field label="user" hint="unset"/>
      <Field label="response_format" filled value="auto"/>
      <Field label="tool_choice default" filled value="auto"/>
    </div>
    <Field label="system preamble" ta hint="You are a helpful assistant…" style={{marginTop:6}}/>
  </>
);

// ── Right panel (standalone + medium) ────────────────────────────
const AliasRightPanel = ({compact=false}) => (
  <div className="alias-right-panel">
    <FitCheckCard/>
    <div className="h3" style={{marginTop:4}}>Live config preview</div>
    <LiveConfigJson config={DEFAULT_ALIAS_CONFIG}/>
    {!compact && (
      <>
        <div className="h3">Tips</div>
        <div className="sm">
          • Preset chips seed Runtime args — you can tweak any line after.<br/>
          • Palette is built live from <code>llama-server --help</code>, so new flags show up on upgrade.<br/>
          • <code>⚠</code> marks unknown flags · doesn't block save.<br/>
          • "Paste command" accepts a full <code>llama-server …</code> line.
        </div>
        <Callout style={{position:'static', display:'inline-block', marginTop:6, fontSize:10}}>★ right panel: fit + JSON + tips</Callout>
      </>
    )}
  </div>
);

// Section numbering — 4 sections now (Preset merged into Runtime args)
const SECTIONS = [
  {k:'identity',    n:1, label:'Identity'},
  {k:'model',       n:2, label:'Model file'},
  {k:'presetArgs',  n:3, label:'Preset & Runtime args'},
  {k:'request',     n:4, label:'Request defaults'},
];
const NewAliasRail = ({active='runtime'}) => (
  <div className="alias-rail">
    <div className="alias-rail-title">Sections</div>
    {SECTIONS.map(s => (
      <div key={s.k} className={`alias-rail-item${active===s.k?' active':''}`}>
        <span className="alias-rail-item-num">{s.n}</span>
        <span>{s.label}</span>
      </div>
    ))}
  </div>
);
const NewAliasAnchors = ({active='runtime'}) => (
  <div className="alias-medium-anchor-row">
    {SECTIONS.map(s => (
      <span key={s.k} className={`alias-medium-anchor${active===s.k?' active':''}`}>
        {s.n} · {s.label}
      </span>
    ))}
  </div>
);

// ── 1. AliasStandalone (desktop) ─────────────────────────────────
function AliasStandalone() {
  return (
    <Browser url="bodhi.local/models/alias/new">
      <Crumbs items={['Bodhi','Models','New alias']}/>
      <div style={{display:'flex', alignItems:'baseline', justifyContent:'space-between', gap:10, marginBottom:6}}>
        <div>
          <div className="h1" style={{fontSize:20, marginBottom:2}}>New model alias</div>
          <div className="sm">Runtime args stay raw to avoid drifting with llama.cpp releases · OpenAI request defaults are a fixed form.</div>
        </div>
        <div style={{display:'flex', gap:6}}>
          <Btn>Cancel</Btn>
          <Btn>Save &amp; test</Btn>
          <Btn variant="primary">Create alias</Btn>
        </div>
      </div>
      <div className="alias-3col">
        <NewAliasRail active="presetArgs"/>
        <div>
          <ParamSection n={1} title="Identity" summary="alias: qwen-chat · 2 tags" open>
            <IdentityBody/>
          </ParamSection>
          <ParamSection n={2} title="Model file" summary="Qwen/Qwen3.5-9B-GGUF · :Q4_K_M · ✓ recommended" open>
            <ModelFileBody/>
          </ParamSection>
          <PresetAndArgsSection n={3} selected="ragLong" open={true}/>
          <ParamSection n={4} title="Request defaults · OpenAI-compat" summary="temp 0.7 · top_p 0.95 · max_tokens —" open>
            <RequestDefaultsBody/>
          </ParamSection>
          <Callout style={{position:'static', display:'inline-block', marginTop:8}}>
            ★ merged section · preset chips always visible · editor collapses · raw textarea with hover help + wavy underline for unknown flags
          </Callout>
        </div>
        <AliasRightPanel/>
      </div>
    </Browser>
  );
}

// ── 2. AliasOverlay (desktop) ─────────────────────────────────────
function AliasOverlay() {
  const context = (
    <>
      <span className="sm" style={{color:'var(--ink)'}}>Adding</span>
      <Chip tone="leaf" style={{fontSize:10}}>hf-repo</Chip>
      <code>Qwen/Qwen3.5-9B-GGUF</code>
      <span className="sm">·</span>
      <Chip tone="saff" style={{fontSize:10}}>:Q4_K_M</Chip>
      <span className="sm" style={{marginLeft:'auto'}}>prefilled from Discover</span>
    </>
  );
  const body = (
    <>
      <DownloadProgressStrip state="queued"/>
      <ParamSection n={1} title="Identity" summary="alias: qwen-chat" open>
        <IdentityBody/>
      </ParamSection>
      <PresetAndArgsSection n={3} selected="chat" open={false} compact={true}/>
      <ParamSection n={2} title="Model file" summary="Q4_K_M · 5.6 GB · will download"/>
      <ParamSection n={4} title="Request defaults" summary="temp 0.7 · top_p 0.95"/>
    </>
  );
  const footer = (
    <>
      <span className="fit-mini">✓ Fits · ~38 tok/s est. · 21 GB free</span>
      <Btn variant="ghost" size="xs">Open full page ↗</Btn>
      <Btn>Cancel</Btn>
      <Btn variant="primary">Create &amp; download</Btn>
    </>
  );
  return <OverlayShell title="Add to Bodhi" context={context} body={body} footer={footer}/>;
}

// ── 3. AliasMedium (tablet) ───────────────────────────────────────
function AliasMedium() {
  return (
    <TabletFrame label="Medium · New alias · args-editor">
      <MobileHeader active="My Models" rightSlot={<span className="m-ico m-ico-add">+ ▾</span>}/>
      <div style={{padding:'8px 10px', borderBottom:'1.3px solid var(--ink)'}}>
        <div className="h1" style={{fontSize:16, margin:0}}>New model alias</div>
        <div className="sm">Raw args · preset-seeded · live --help palette.</div>
      </div>
      <div style={{padding:'8px 10px', flex:1, minHeight:0, overflow:'auto'}}>
        <NewAliasAnchors active="presetArgs"/>
        <div className="alias-medium-layout">
          <div>
            <ParamSection n={1} title="Identity" summary="qwen-chat · chat,tool-use" open>
              <IdentityBody/>
            </ParamSection>
            <ParamSection n={2} title="Model file" summary=":Q4_K_M · will download" open>
              <div style={{display:'grid', gridTemplateColumns:'1fr 1fr', gap:6}}>
                <Field label="Repo" filled value="Qwen/Qwen3.5-9B-GGUF"/>
                <Field label="Snapshot" filled value="116f76…ec8ff1"/>
              </div>
              <div className="h3" style={{marginTop:6}}>Quant · picks the file</div>
              <QuantPicker quants={DEFAULT_QUANTS.slice(0,4)} selected=":Q4_K_M"/>
            </ParamSection>
            <PresetAndArgsSection n={3} selected="maxPerf" open={true} compact={true}/>
            <ParamSection n={4} title="Request defaults" summary="temp 0.7 · top_p 0.95"/>
            <div style={{display:'flex', gap:6, justifyContent:'flex-end', marginTop:8}}>
              <Btn size="xs">Cancel</Btn>
              <Btn size="xs">Save &amp; test</Btn>
              <Btn variant="primary" size="xs">Create alias</Btn>
            </div>
          </div>
          <div className="alias-right-panel">
            <FitCheckCard/>
            <div className="h3">Live JSON</div>
            <LiveConfigJson config={DEFAULT_ALIAS_CONFIG}/>
          </div>
        </div>
      </div>
    </TabletFrame>
  );
}

// ── 4. AliasMobile ────────────────────────────────────────────────
const MobileAliasSection = ({n, title, summary, open}) => (
  <div className="alias-mobile-sec">
    <div className="alias-mobile-sec-head">
      <span><b>{n}</b> · {title}</span>
      <span>{open?'▾':'▸'}</span>
    </div>
    {!open && <div className="alias-mobile-sec-summary">{summary}</div>}
  </div>
);

function AliasMobile() {
  const phoneStyle = {
    width:280, height:560, border:'1.5px solid var(--ink)',
    borderRadius:22, background:'#fff', position:'relative', overflow:'hidden',
    boxShadow:'3px 3px 0 rgba(26,26,34,0.12)'
  };
  return (
    <div style={{display:'flex', gap:14, flexWrap:'wrap', justifyContent:'center', padding:10}}>
      {/* Frame 1 — stacked summaries */}
      <div>
        <div className="sm" style={{textAlign:'center', marginBottom:4, fontWeight:700}}>1 · Stacked sections</div>
        <div style={phoneStyle}>
          <MobileHeader active="My Models" rightSlot={<span className="m-ico m-ico-add">+ ▾</span>}/>
          <div style={{padding:'8px 10px'}}>
            <div className="h1" style={{fontSize:14, margin:'0 0 2px'}}>New alias</div>
            <div className="sm">Tap a section to edit.</div>
          </div>
          <div style={{padding:'0 10px', overflow:'auto', position:'absolute', top:80, left:0, right:0, bottom:48}}>
            <MobileAliasSection n={1} title="Identity" summary="qwen-chat · chat,tool-use" open/>
            <div className="alias-mobile-sec" style={{background:'var(--paper)', border:'1.5px solid var(--ink)'}}>
              <div style={{display:'grid', gridTemplateColumns:'1fr 1fr', gap:4}}>
                <Field label="alias" filled value="qwen-chat"/>
                <Field label="tags" filled value="2"/>
              </div>
            </div>
            <MobileAliasSection n={2} title="Model file" summary=":Q4_K_M · 5.6 GB · will download"/>
            <MobileAliasSection n={3} title="Preset & Runtime args" summary="RAG (long) · 7 lines · tap to edit"/>
            <MobileAliasSection n={4} title="Request defaults" summary="temp 0.7 · top_p 0.95"/>
          </div>
          <div className="alias-mobile-footer">
            <Btn size="xs">Cancel</Btn>
            <Btn variant="primary" size="xs">Create</Btn>
          </div>
        </div>
      </div>

      {/* Frame 2 — Runtime args sheet (the important one) */}
      <div>
        <div className="sm" style={{textAlign:'center', marginBottom:4, fontWeight:700}}>2 · Args editor sheet</div>
        <div style={phoneStyle}>
          <MobileHeader active="My Models" rightSlot={<span className="m-ico m-ico-add">+ ▾</span>}/>
          <div style={{padding:'8px 10px', filter:'blur(0.4px)', opacity:0.6}}>
            <div className="h1" style={{fontSize:14, margin:'0 0 2px'}}>New alias</div>
            <MobileAliasSection n={3} title="Preset & Runtime args" summary="editing…" open/>
          </div>
          <div className="m-sheet-editor" style={{maxHeight:'82%'}}>
            <div className="m-sheet-handle"/>
            <div className="h2" style={{margin:'0 0 2px'}}>Preset &amp; Runtime args</div>
            <div className="sm" style={{marginBottom:4}}>Pick a preset — seeds raw args below.</div>
            <PresetGrid selected="ragShort"/>
            <ArgsEditor selectedPreset="ragShort"
              lines={(ARGS_PRESETS['ragShort']||[]).map((t,i)=>{const[f,...r]=t.split(' ');return{flag:f,value:r.join(' ')||null,focused:i===1};})}
              count={`${(ARGS_PRESETS['ragShort']||[]).length} args`} compact/>
            <div className="m-sheet-actions" style={{marginTop:6}}>
              <Btn variant="primary" size="xs">Apply</Btn>
            </div>
          </div>
        </div>
      </div>

      {/* Frame 3 — Add overlay with args preview */}
      <div>
        <div className="sm" style={{textAlign:'center', marginBottom:4, fontWeight:700}}>3 · Add overlay</div>
        <div style={phoneStyle}>
          <MobileHeader active="Discover" rightSlot={<span className="m-ico m-ico-action">⋯ ▾</span>}/>
          <div style={{padding:'8px 10px', filter:'blur(0.4px)', opacity:0.55}}>
            <div className="h2" style={{margin:0}}>Qwen/Qwen3.5-9B</div>
            <div className="sm">HF · Apache-2 · 9B</div>
          </div>
          <div className="m-sheet-editor" style={{maxHeight:'74%'}}>
            <div className="m-sheet-handle"/>
            <div style={{display:'flex', alignItems:'center', justifyContent:'space-between'}}>
              <div className="h2" style={{margin:0}}>Add to Bodhi</div>
              <span className="sm">prefilled</span>
            </div>
            <div className="alias-overlay-context" style={{position:'static', border:'none', padding:'4px 0', background:'transparent'}}>
              <Chip tone="leaf" style={{fontSize:10}}>hf-repo</Chip>
              <code style={{fontSize:11}}>Qwen3.5-9B-GGUF</code>
              <Chip tone="saff" style={{fontSize:10}}>:Q4_K_M</Chip>
            </div>
            <DownloadProgressStrip state="queued"/>
            <MobileAliasSection n={1} title="Identity" summary="qwen-chat" open/>
            <div className="alias-mobile-sec" style={{background:'var(--paper)', border:'1.5px solid var(--ink)'}}>
              <Field label="alias" filled value="qwen-chat"/>
            </div>
            <MobileAliasSection n={3} title="Preset & Runtime args" summary="Chat · 4 lines seeded"/>
            <MobileAliasSection n={4} title="Request defaults" summary="temp 0.7 · top_p 0.95"/>
            <div className="m-sheet-actions" style={{marginTop:8}}>
              <Btn size="xs">Cancel</Btn>
              <Btn variant="primary" size="xs">Create &amp; download</Btn>
            </div>
            <div className="sm" style={{marginTop:4, textAlign:'center'}}>✓ Fits · ~38 tok/s</div>
          </div>
        </div>
      </div>
    </div>
  );
}

window.AliasScreens = [
  {label:'A · Standalone (desktop)', tag:'familiar',
    note:'Full page · Runtime args is a raw-line editor with preset chips, palette built from parsed llama-server --help, per-line enum pickers, inline help footer, paste-command import, validation strip. Request defaults (OpenAI-compat) stays a fixed form.',
    novel:'raw-args editor · palette from parsed --help · never drifts with llama.cpp releases',
    component:AliasStandalone},
  {label:'B · Overlay (desktop)', tag:'balanced',
    note:'Slide-over from Discover `+ Add to Bodhi`. Compact ArgsEditor (single-col, palette stacked below) seeded by preset. Other sections collapse to summary lines.',
    novel:'args-editor works in overlay too · preset-driven quick-add',
    component:AliasOverlay},
  {label:'C · Medium (tablet)', tag:'medium',
    note:'Anchor chips replace rail. 2-col main layout + fixed right fit/JSON panel. ArgsEditor compact (palette below lines). All sections visible.',
    novel:'compact args-editor · anchors + right fit panel',
    component:AliasMedium},
  {label:'D · Mobile', tag:'mobile',
    note:'Three frames: (1) stacked section summaries, (2) bottom-sheet ArgsEditor in compact mode — preset chips + lines + palette, (3) Add overlay from Discover with prefilled repo+quant, Create & download.',
    novel:'args-editor in bottom-sheet · preset chips + palette stay usable on mobile',
    component:AliasMobile},
];
