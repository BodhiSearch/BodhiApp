// Screen 4 — Create/Edit Local Model Alias
function AliasA() {
  return (
    <Browser url="bodhi.local/models/alias/new">
      <Crumbs items={['Bodhi','Models','New alias']}/>
      <div className="h1" style={{fontSize:20, marginBottom:6}}>New model alias</div>
      <div className="sm" style={{marginBottom:10}}>Start simple — add tuning only if you need it.</div>
      <div className="h3">Basics</div>
      <Field label="Alias name" filled hint="qwen-long-ctx" />
      <div style={{display:'grid', gridTemplateColumns:'1fr 1fr', gap:8, marginTop:6}}>
        <Field label="Repo" filled hint="Qwen/Qwen3.5-9B-GGUF" right={<span className="sm">▾</span>}/>
        <Field label="Quant file" filled hint="Qwen3.5-9B-Q6_K.gguf" right={<span className="sm">▾</span>}/>
      </div>
      <div className="divider"/>
      <div style={{display:'flex', justifyContent:'space-between', alignItems:'center'}}>
        <div className="h3" style={{margin:0}}>Context &amp; server params</div>
        <span className="sm">▾ expand</span>
      </div>
      <div className="sm" style={{marginTop:4}}>Using defaults · ctx-size 4096, parallel 1</div>
      <div className="divider"/>
      <div style={{display:'flex', justifyContent:'space-between', alignItems:'center'}}>
        <div className="h3" style={{margin:0}}>Request params (sampling)</div>
        <span className="sm">▾ expand</span>
      </div>
      <div className="sm" style={{marginTop:4}}>Using defaults · temp 0.7, top_p 0.95</div>
      <div className="divider"/>
      <div style={{display:'flex', gap:6, justifyContent:'flex-end'}}>
        <Btn>Cancel</Btn>
        <Btn variant="primary">Create alias</Btn>
      </div>
      <Callout style={{position:'static', display:'inline-block', marginTop:8}}>★ progressive disclosure — advanced collapsed</Callout>
    </Browser>
  );
}

function AliasB() {
  return (
    <Browser url="bodhi.local/models/alias/new">
      <Crumbs items={['Bodhi','Models','New alias']}/>
      <div className="h1" style={{fontSize:20, marginBottom:6}}>New model alias</div>
      <div className="sm" style={{marginBottom:8}}>Start from a preset — tweak what matters.</div>
      <div className="h3">Preset</div>
      <div style={{display:'grid', gridTemplateColumns:'repeat(4,1fr)', gap:6}}>
        {[['💬','Chat','balanced'],['🧑‍💻','Coding','deterministic'],['📄','Long ctx','32K+'],['⚙️','Custom','empty']].map((p,i)=>(
          <div key={i} className="card" style={{textAlign:'center', background:i===0?'var(--lotus-soft)':'#fff'}}>
            <div style={{fontSize:20}}>{p[0]}</div>
            <div className="h2" style={{margin:0}}>{p[1]}</div>
            <span className="sm">{p[2]}</span>
          </div>
        ))}
      </div>
      <div className="divider"/>
      <div style={{display:'grid', gridTemplateColumns:'1.2fr 1fr', gap:12}}>
        <div>
          <Field label="Alias name" filled hint="qwen-chat"/>
          <div style={{marginTop:6}}>
            <Field label="Model" filled hint="Qwen/Qwen3.5-9B-GGUF · Q6_K" right={<span className="sm">▾</span>}/>
          </div>
          <div className="h3">Context</div>
          <div className="lbl">ctx-size</div>
          <div className="slider"><div className="thumb" style={{left:'25%'}}/></div>
          <div style={{display:'flex', justifyContent:'space-between'}} className="sm"><span>2K</span><span>8K ⭐</span><span>32K</span><span>128K</span></div>
          <div style={{marginTop:6}}><Field label="parallel" filled value="1"/></div>
          <div className="h3">Sampling</div>
          <div style={{display:'grid', gridTemplateColumns:'1fr 1fr', gap:6}}>
            <Field label="temperature" value="0.7" filled/>
            <Field label="top_p" value="0.95" filled/>
            <Field label="max_tokens" hint="unset"/>
            <Field label="seed" hint="unset"/>
          </div>
        </div>
        <div>
          <div className="h3" style={{marginTop:0}}>Live preview</div>
          <div className="wf" style={{background:'#f6f2e8'}}>
            <div className="code" style={{whiteSpace:'pre', lineHeight:1.4}}>{`{
  "alias": "qwen-chat",
  "repo": "Qwen/Qwen3.5-9B-GGUF",
  "file": "Qwen3.5-9B-Q6_K.gguf",
  "ctx_size": 8192,
  "parallel": 1,
  "temperature": 0.7,
  "top_p": 0.95
}`}</div>
          </div>
          <div className="h3">Fit check</div>
          <div className="card" style={{background:'var(--leaf-soft)'}}>
            <div className="h2" style={{margin:0}}>✓ Fits with 21 GB headroom</div>
            <span className="sm">Estimated ~38 tok/s on your rig</span>
          </div>
          <Callout style={{position:'static', display:'inline-block', marginTop:6}}>★ live config preview &amp; fit check</Callout>
        </div>
      </div>
      <div style={{display:'flex', gap:6, justifyContent:'flex-end', marginTop:10}}>
        <Btn>Cancel</Btn><Btn>Save &amp; test</Btn><Btn variant="primary">Create alias</Btn>
      </div>
    </Browser>
  );
}

function AliasC() {
  return (
    <Browser url="bodhi.local/models/alias/new">
      <Crumbs items={['Bodhi','Models','New alias']}/>
      <div className="h1" style={{fontSize:20, marginBottom:4}}>Tune alias: qwen3.5-9B</div>
      <div className="sm" style={{marginBottom:8}}>Three knobs. Everything else is opinionated defaults.</div>
      <div className="card">
        <div style={{display:'flex', justifyContent:'space-between'}}>
          <div className="h2" style={{margin:0}}>Personality</div>
          <span className="sm" style={{color:'var(--lotus)'}}>temp 0.7 · top_p 0.95</span>
        </div>
        <div className="slider"><div className="thumb" style={{left:'50%'}}/></div>
        <div style={{display:'flex', justifyContent:'space-between'}} className="sm"><span>deterministic (code)</span><span>balanced</span><span>creative (writing)</span></div>
      </div>
      <div className="card" style={{marginTop:8}}>
        <div style={{display:'flex', justifyContent:'space-between'}}>
          <div className="h2" style={{margin:0}}>Memory</div>
          <span className="sm" style={{color:'var(--lotus)'}}>ctx-size 16K · parallel 1</span>
        </div>
        <div className="slider"><div className="thumb" style={{left:'35%'}}/></div>
        <div style={{display:'flex', justifyContent:'space-between'}} className="sm"><span>short (fast)</span><span>16K</span><span>128K (slow)</span></div>
        <span className="sm">Memory use: 11 GB · fits ✓</span>
      </div>
      <div className="card" style={{marginTop:8}}>
        <div style={{display:'flex', justifyContent:'space-between'}}>
          <div className="h2" style={{margin:0}}>Response length cap</div>
          <span className="sm" style={{color:'var(--lotus)'}}>max_tokens 2048</span>
        </div>
        <div className="slider"><div className="thumb" style={{left:'22%'}}/></div>
      </div>
      <div className="h3" style={{marginTop:14}}>Advanced (hidden by default)</div>
      <div style={{display:'flex', gap:4, flexWrap:'wrap'}}>
        {['frequency_penalty','presence_penalty','seed','stop','user','llama-server args'].map(x=><Chip key={x}>{x}</Chip>)}
      </div>
      <Callout style={{position:'static', display:'inline-block', marginTop:8}}>★ 3 plain-english knobs; raw params behind a chip</Callout>
      <div style={{display:'flex', gap:6, justifyContent:'flex-end', marginTop:10}}>
        <Btn>Cancel</Btn><Btn variant="primary">Save alias</Btn>
      </div>
    </Browser>
  );
}

window.AliasScreens = [
  {label:'A · Progressive disclosure', tag:'familiar', note:'Basics visible, server params + sampling collapsed. Press save to use defaults.', novel:null, component:AliasA},
  {label:'B · Preset + live preview', tag:'balanced', note:'Start from a preset. Config JSON previewed live, fit check beside it.', novel:'live JSON + fit-check preview', component:AliasB},
  {label:'C · Three plain-english knobs', tag:'bold', note:'Personality / Memory / Length sliders. Raw llama-server args behind a chip.', novel:'plain-english sliders replace 10+ fields', component:AliasC},
];
