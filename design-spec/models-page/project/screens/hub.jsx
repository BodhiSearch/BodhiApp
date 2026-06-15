// Local-side detail panels used by the unified Models page.
// v25: the Models Hub SCREEN has been collapsed into the unified Models page
// (see discover.jsx). This file now only holds the three right-panel components
// that still render when an alias / file / connected provider row is clicked:
//   · AliasPanel    — user's alias config (runtime + usage + capabilities)
//   · FilePanel     — downloaded GGUF (sibling quants, aliases pointing here)
//   · ProviderPanel — connected API provider with model list + connection info
// These are kept in hub.jsx (rather than moved) so the existing <script> tag in
// index.html continues to load them without needing further surgery.

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

// Expose the panels on window — the unified Models page dispatches to them.
Object.assign(window, {AliasPanel, FilePanel, ProviderPanel});
