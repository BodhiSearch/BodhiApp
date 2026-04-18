// Screen 3 — Download / Pull flow
function DownloadA() {
  return (
    <Browser url="bodhi.local/models/files/pull">
      <Crumbs items={['Bodhi','Models','Download']}/>
      <div className="h1" style={{fontSize:20, marginBottom:4}}>Download model</div>
      <div className="sm" style={{marginBottom:10}}>Paste a HuggingFace URL, repo, or search — we'll resolve files for you.</div>
      <Field label="🔗 HF URL, repo, or search" filled hint="huggingface.co/unsloth/Qwen3.5-9B-GGUF" />
      <div style={{marginTop:8}}>
        <div className="h3">Matches</div>
        <div className="card row">
          <div className="ph thumb"/>
          <div style={{flex:1}}>
            <div className="h2" style={{margin:0}}>unsloth/Qwen3.5-9B-GGUF</div>
            <span className="sm">Qwen3.5 9B · 7 files · 443k downloads</span>
          </div>
          <Chip tone="leaf">fits</Chip>
        </div>
      </div>
      <div className="h3" style={{marginTop:10}}>Pick a quantization</div>
      <table className="tbl">
        <thead><tr><th></th><th>File</th><th>Size</th><th>Quality</th><th>Speed</th><th>Fit</th><th></th></tr></thead>
        <tbody>
          <tr><td><input type="radio"/></td><td>Q4_K_M</td><td>5.4 GB</td><td><Bar pct={72}/></td><td><Bar pct={85}/></td><td><TL tone="green">~48 t/s</TL></td><td><Chip tone="saff">⭐ recommended</Chip></td></tr>
          <tr><td><input type="radio" defaultChecked/></td><td>Q6_K</td><td>7.5 GB</td><td><Bar pct={88}/></td><td><Bar pct={70}/></td><td><TL tone="green">~38 t/s</TL></td><td><span className="sm">best balance</span></td></tr>
          <tr><td><input type="radio"/></td><td>Q8_0</td><td>9.2 GB</td><td><Bar pct={95}/></td><td><Bar pct={55}/></td><td><TL tone="yellow">~22 t/s</TL></td><td></td></tr>
          <tr><td><input type="radio"/></td><td>F16</td><td>18 GB</td><td><Bar pct={100}/></td><td><Bar pct={20}/></td><td><TL tone="red">won't fit</TL></td><td></td></tr>
        </tbody>
      </table>
      <Callout style={{position:'static', display:'inline-block', margin:'8px 0'}}>★ quality vs size vs speed, at a glance</Callout>
      <div style={{display:'flex', gap:6, justifyContent:'flex-end'}}>
        <Btn>Cancel</Btn>
        <Btn variant="primary">↓ Pull 7.5 GB</Btn>
      </div>
      <div className="divider"/>
      <div className="h3">Active downloads</div>
      <div className="card">
        <div style={{display:'flex', justifyContent:'space-between', alignItems:'center'}}>
          <span className="h2" style={{margin:0}}>gemma-4-e4b-Q4_K_M.gguf</span>
          <span className="sm">2.4 / 3.99 GB · 12 MB/s · 2m left</span>
        </div>
        <Bar pct={60}/>
      </div>
    </Browser>
  );
}

function DownloadB() {
  return (
    <Browser url="bodhi.local/models/files">
      <Crumbs items={['Bodhi','Models','Files']}/>
      <div style={{display:'flex', justifyContent:'space-between', alignItems:'baseline', flexWrap:'wrap', gap:6}}>
        <div className="h1" style={{fontSize:20}}>Model files · 8 · 94 GB</div>
        <Btn variant="primary">+ Pull new</Btn>
      </div>
      <div style={{display:'flex', gap:6, marginTop:6, flexWrap:'wrap'}}>
        <Chip on>All</Chip><Chip>Ready</Chip><Chip>Downloading (1)</Chip><Chip>Broken</Chip>
      </div>
      <div className="divider"/>
      <div className="h3">Downloading</div>
      <div className="card">
        <div style={{display:'flex', gap:8, alignItems:'center'}}>
          <div className="ph thumb"/>
          <div style={{flex:1}}>
            <div className="h2" style={{margin:0}}>unsloth/gemma-4-e4b-it · Q4_K_M</div>
            <span className="sm">2.40 / 3.99 GB · 12 MB/s · eta 2m</span>
            <Bar pct={60}/>
          </div>
          <Btn size="xs">⏸</Btn><Btn size="xs">✕</Btn>
        </div>
      </div>
      <div className="h3" style={{marginTop:10}}>Ready · 7</div>
      {[
        ['prism-ml','Bonsai-8B','Q6_K','1.08 GB'],
        ['LiquidAI','LFM2.5-1.2B','Q8_0','1.16 GB'],
        ['unsloth','NVIDIA-Nemotron-3-Nano-4B','Q8_0','3.94 GB'],
        ['unsloth','Nemotron-3-Nano-30B','Q4_K_M','16.96 GB'],
        ['ggml-org','Qwen3-1.7B-GGUF','Q8_0','2.02 GB'],
        ['Ai3-VE','context-1-20b','Q4_K_M','14.72 GB'],
      ].map((r,i)=>(
        <div key={i} className="card row">
          <div className="ph thumb"/>
          <div style={{flex:1, minWidth:0}}>
            <div className="h2" style={{margin:0}}>{r[0]}/{r[1]}</div>
            <span className="sm">{r[2]} · {r[3]} · last used 2d ago</span>
          </div>
          <Chip tone="leaf">● ready</Chip>
          <Btn size="xs">Use</Btn>
          <Btn size="xs">⋯</Btn>
        </div>
      ))}
      <Callout style={{position:'static', display:'inline-block', margin:'8px 0'}}>★ merges "files" + "pull" into one page with filters</Callout>
    </Browser>
  );
}

function DownloadC() {
  return (
    <Browser url="bodhi.local/models/Qwen--Qwen3.5-9B-GGUF">
      <Crumbs items={['Bodhi','Models','Qwen/Qwen3.5-9B-GGUF']}/>
      <div style={{display:'flex', gap:12, alignItems:'flex-start', flexWrap:'wrap'}}>
        <div className="ph thumb" style={{width:56, height:56}}/>
        <div style={{flex:1, minWidth:200}}>
          <div className="h1" style={{fontSize:22, margin:0}}>Qwen/Qwen3.5-9B-GGUF</div>
          <span className="sm">Updated 2d ago · 443k downloads · ♥ 415 · Apache-2</span>
          <div style={{display:'flex', gap:4, marginTop:5}}>
            <Chip tone="leaf">9B</Chip><Chip tone="indigo">Text→Text</Chip><Chip tone="saff">🔧 Tool</Chip>
          </div>
        </div>
      </div>
      <div className="divider"/>
      <div className="h2">Choose quantization</div>
      <div className="sm" style={{marginBottom:6}}>We default to Q6_K — best quality/size for your 36GB. Drag the slider, or pick a file.</div>
      <div style={{padding:'6px 4px 0'}}>
        <div className="slider">
          {[8,22,38,55,72,88].map((x,i)=><div key={i} className="tick" style={{left: x+'%'}}/>)}
          <div className="thumb" style={{left:'55%'}}/>
        </div>
        <div style={{display:'flex', justifyContent:'space-between'}} className="sm">
          <span>Q2_K</span><span>Q4_K_S</span><span>Q4_K_M</span><span style={{fontWeight:700, color:'var(--lotus)'}}>Q6_K ⭐</span><span>Q8_0</span><span>F16</span>
        </div>
      </div>
      <div className="card" style={{marginTop:10, background:'var(--lotus-soft)'}}>
        <div style={{display:'flex', justifyContent:'space-between'}}>
          <div><div className="h2" style={{margin:0}}>Q6_K · 7.5 GB</div><span className="sm">Near-lossless vs F16 · fits with 28GB headroom</span></div>
          <div style={{textAlign:'right'}}>
            <div className="sm">~38 tok/s</div>
            <TL tone="green">fits easy</TL>
          </div>
        </div>
        <div style={{display:'flex', gap:14, marginTop:4}}>
          <div style={{flex:1}}><span className="lbl">Quality</span><Bar pct={88}/></div>
          <div style={{flex:1}}><span className="lbl">Speed</span><Bar pct={70}/></div>
          <div style={{flex:1}}><span className="lbl">Size</span><Bar pct={42}/></div>
        </div>
      </div>
      <Callout style={{position:'static', display:'inline-block', margin:'8px 0'}}>★ interactive quant slider w/ live tradeoffs</Callout>
      <div style={{display:'flex', gap:6, justifyContent:'flex-end', marginTop:6, flexWrap:'wrap'}}>
        <Btn>View all 7 files</Btn>
        <Btn variant="primary">↓ Download Q6_K · 7.5 GB</Btn>
      </div>
    </Browser>
  );
}

window.DownloadScreens = [
  {label:'A · URL-first pull', tag:'familiar', note:'Paste repo, resolve files, pick quant from a table with quality/speed/fit bars.', novel:'per-quant fit indicator & tok/s preview', component:DownloadA},
  {label:'B · Downloads as library', tag:'balanced', note:'Unified Files page with filter chips: ready / downloading / broken. Status lives with the file.', novel:null, component:DownloadB},
  {label:'C · Interactive quant slider', tag:'bold', note:'Model page with a slider across quantization levels — tradeoffs update live.', novel:'interactive quality↔size↔speed slider', component:DownloadC},
];
