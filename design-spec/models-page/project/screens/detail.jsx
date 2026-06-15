// Archived 2026-04-19 — Model Detail absorbed into unified Models page.
// Not loaded by index.html. Kept on disk for archival reference (git-recoverable).
// The three variants here (side-drawer / full-page / bottom-sheet) were all for
// hf-repo entities; the Models page already dispatches `HfRepoPanel` as a
// right-drawer on desktop and a bottom-sheet on mobile, covering Variants A + C.
// Variant B's unique surface (benchmark bars, quant slider, community rating)
// was dropped by user decision — can be added to HfRepoPanel if required later.
// See specs/models.md §13 + specs/shared-primitives.md §6.

// Screen 7 — Model Detail (archived)
function DetailA() {
  return (
    <div style={{position:'relative'}}>
      <Browser url="bodhi.local/models/discover" style={{opacity:.45, pointerEvents:'none'}}>
        <Crumbs items={['Bodhi','Models','Discover']}/>
        <div className="sm">(list behind drawer)</div>
        <ModelRow name="gemma-4-e2b" org="google" size="2B"/>
        <ModelRow name="Qwen3.5-9B" org="Qwen" size="9B" highlight/>
        <ModelRow name="LFM2.5-1.2B" org="LiquidAI" size="1.2B"/>
      </Browser>
      <div style={{position:'absolute', top:6, right:6, bottom:6, width:'58%', minWidth:280, background:'#fff', border:'1.5px solid var(--ink)', borderRadius:'10px', padding:12, overflow:'auto', boxShadow:'-4px 4px 0 rgba(26,26,34,.15)'}}>
        <div style={{display:'flex', justifyContent:'space-between', alignItems:'center'}}>
          <span className="sm">← back · Qwen/Qwen3.5-9B</span>
          <span className="sm">✕</span>
        </div>
        <div style={{display:'flex', gap:10, alignItems:'center', marginTop:6}}>
          <div className="ph thumb" style={{width:44,height:44}}/>
          <div style={{flex:1}}>
            <div className="h1" style={{fontSize:17, margin:0}}>Qwen3.5-9B</div>
            <span className="sm">Qwen · updated 2d ago · Apache-2</span>
          </div>
          <Btn size="xs" variant="primary">↓ Download</Btn>
        </div>
        <div style={{display:'flex', gap:4, marginTop:6, flexWrap:'wrap'}}>
          <Chip tone="leaf">9B</Chip><Chip tone="indigo">Text→Text</Chip><Chip tone="saff">🔧 Tool</Chip><Chip>MCP ✓</Chip>
        </div>
        <div style={{display:'grid', gridTemplateColumns:'repeat(3,1fr)', gap:6, marginTop:8}}>
          <div className="card" style={{padding:6}}><span className="sm">Downloads</span><div className="h2" style={{margin:0}}>443k</div></div>
          <div className="card" style={{padding:6}}><span className="sm">Likes</span><div className="h2" style={{margin:0}}>♥ 415</div></div>
          <div className="card" style={{padding:6}}><span className="sm">Rating</span><div className="h2" style={{margin:0}}><Stars n={4.5}/></div></div>
        </div>
        <div className="h3">Fit on your rig</div>
        <div className="card">
          <TL tone="green">Fits easy · ~38 tok/s at Q6_K</TL>
          <span className="sm">M3 Max 36GB · 21 GB headroom after load</span>
        </div>
        <div className="h3">README</div>
        <Lines rows={[90,80,60,85,40]}/>
        <div className="h3">Quantizations · 7 files</div>
        <table className="tbl">
          <thead><tr><th>File</th><th>Size</th><th>Fit</th><th></th></tr></thead>
          <tbody>
            <tr><td>Q4_K_M</td><td>5.4 GB</td><td><TL tone="green">~48 t/s</TL></td><td><Btn size="xs">↓</Btn></td></tr>
            <tr><td>Q6_K ⭐</td><td>7.5 GB</td><td><TL tone="green">~38 t/s</TL></td><td><Btn size="xs" variant="primary">↓</Btn></td></tr>
            <tr><td>Q8_0</td><td>9.2 GB</td><td><TL tone="yellow">~22 t/s</TL></td><td><Btn size="xs">↓</Btn></td></tr>
          </tbody>
        </table>
      </div>
    </div>
  );
}

function DetailB() {
  return (
    <Browser url="bodhi.local/models/Qwen--Qwen3.5-9B">
      <Crumbs items={['Bodhi','Models','Qwen/Qwen3.5-9B']}/>
      <div style={{display:'flex', gap:12, alignItems:'flex-start', flexWrap:'wrap'}}>
        <div className="ph thumb" style={{width:64,height:64}}/>
        <div style={{flex:1, minWidth:200}}>
          <div className="h1" style={{fontSize:22, margin:0}}>Qwen/Qwen3.5-9B</div>
          <span className="sm">Updated 2d ago · 443k ↓ · ♥ 415 · <Stars n={4.5}/> 4.5 (128)</span>
          <div style={{display:'flex', gap:4, marginTop:5, flexWrap:'wrap'}}>
            <Chip tone="leaf">9B params</Chip>
            <Chip tone="indigo">Text→Text</Chip>
            <Chip tone="saff">🔧 Tool-use</Chip>
            <Chip>🧠 Reasoning</Chip>
            <Chip>MCP ✓</Chip>
            <Chip>Apache-2</Chip>
          </div>
        </div>
        <div style={{display:'flex', flexDirection:'column', gap:4}}>
          <Btn variant="primary">↓ Download</Btn>
          <Btn>Create alias</Btn>
          <Btn variant="ghost">Add to shortlist</Btn>
        </div>
      </div>
      <div className="divider"/>
      <div style={{display:'grid', gridTemplateColumns:'1fr 1fr', gap:10}}>
        <div>
          <div className="h3" style={{marginTop:0}}>About</div>
          <Lines rows={[95,80,70,85,60]}/>
          <div className="h3">Capabilities</div>
          <div style={{display:'flex', flexWrap:'wrap', gap:4}}>
            <Chip tone="leaf">✓ Tool calling</Chip>
            <Chip tone="leaf">✓ Reasoning</Chip>
            <Chip tone="warn">~ Vision (via adapter)</Chip>
            <Chip>✗ Image gen</Chip>
          </div>
          <div className="h3">Benchmarks</div>
          <table className="tbl">
            <tbody>
              <tr><td>MMLU</td><td><Bar pct={76}/></td><td>76.4</td></tr>
              <tr><td>HumanEval</td><td><Bar pct={82}/></td><td>82.1</td></tr>
              <tr><td>GSM8K</td><td><Bar pct={88}/></td><td>88.0</td></tr>
              <tr><td>Arena</td><td><Bar pct={65}/></td><td>#2 · 9B class</td></tr>
            </tbody>
          </table>
        </div>
        <div>
          <div className="h3" style={{marginTop:0}}>Fit on M3 Max 36GB</div>
          <div className="card" style={{background:'var(--leaf-soft)'}}>
            <TL tone="green">Fits easy · Q6_K recommended</TL>
            <span className="sm">~38 tok/s · 7.5 GB RAM · loads in 4s</span>
          </div>
          <div className="h3">Quantizations</div>
          <div className="slider">
            {[8,22,38,55,72,88].map((x,i)=><div key={i} className="tick" style={{left:x+'%'}}/>)}
            <div className="thumb" style={{left:'55%'}}/>
          </div>
          <div style={{display:'flex', justifyContent:'space-between'}} className="sm">
            <span>Q2</span><span>Q4_S</span><span>Q4_M</span><span style={{fontWeight:700,color:'var(--lotus)'}}>Q6_K</span><span>Q8</span><span>F16</span>
          </div>
          <div className="h3">Community rating</div>
          <div className="card row">
            <span className="stars" style={{fontSize:18}}>★★★★½</span>
            <div style={{flex:1}}>
              <span className="sm">4.5 · 128 reviews</span>
              <Bar pct={90}/>
            </div>
            <Btn size="xs">Reviews</Btn>
          </div>
        </div>
      </div>
    </Browser>
  );
}

function DetailC() {
  return (
    <div style={{position:'relative'}}>
      <Browser url="bodhi.local/models/discover" style={{opacity:.4, pointerEvents:'none'}}>
        <div className="sm">(list behind drawer)</div>
        <ModelRow name="Qwen3.5-9B" org="Qwen" size="9B" highlight/>
      </Browser>
      <div style={{position:'absolute', bottom:0, left:0, right:0, background:'#fff', border:'1.5px solid var(--ink)', borderTopLeftRadius:16, borderTopRightRadius:16, padding:12, maxHeight:'85%', overflow:'auto'}}>
        <div style={{width:40, height:4, background:'var(--ink-3)', borderRadius:4, margin:'0 auto 8px'}}/>
        <div style={{display:'flex', gap:10, alignItems:'center'}}>
          <div className="ph thumb"/>
          <div style={{flex:1}}>
            <div className="h1" style={{fontSize:18, margin:0}}>Qwen3.5-9B</div>
            <span className="sm">Q6_K · 7.5 GB</span>
          </div>
          <Btn size="xs" variant="primary">↓</Btn>
        </div>
        <div style={{display:'flex', gap:4, marginTop:6, flexWrap:'wrap'}}>
          <Chip tone="leaf">fits</Chip><Chip tone="saff">🔧</Chip><Chip>🧠</Chip>
        </div>
        <div className="h3">At a glance</div>
        <div style={{display:'grid', gridTemplateColumns:'repeat(2,1fr)', gap:6}}>
          <div className="card" style={{padding:6}}><span className="sm">Speed</span><TL tone="green">~38 t/s</TL></div>
          <div className="card" style={{padding:6}}><span className="sm">Rating</span><Stars n={4.5}/></div>
          <div className="card" style={{padding:6}}><span className="sm">Downloads</span><div className="h2" style={{margin:0}}>443k</div></div>
          <div className="card" style={{padding:6}}><span className="sm">License</span><div className="h2" style={{margin:0}}>Apache-2</div></div>
        </div>
        <div className="h3">Capabilities</div>
        <div style={{display:'flex', gap:4, flexWrap:'wrap'}}>
          <Chip tone="leaf">Tools</Chip><Chip tone="leaf">Reasoning</Chip><Chip>MCP</Chip>
        </div>
        <div className="h3">README · tap to expand</div>
        <Lines rows={[90,70,55]}/>
        <Callout style={{position:'static', display:'inline-block', marginTop:6}}>★ mobile-first bottom-sheet</Callout>
      </div>
    </div>
  );
}

window.DetailScreens = [
  {label:'A · Right side-drawer', tag:'familiar', note:'List stays visible; detail slides in from the right. Core stats, quants table, fit check.', novel:null, component:DetailA},
  {label:'B · Full page', tag:'balanced', note:'Dedicated route when linked-to. Benchmarks, capabilities, reviews, fit — more breathing room.', novel:'benchmark bars + quant slider on detail', component:DetailB},
  {label:'C · Bottom sheet', tag:'bold', note:'Mobile-first — detail rises from bottom. Same info, touch-friendly.', novel:'mobile bottom-sheet pattern', component:DetailC},
];
