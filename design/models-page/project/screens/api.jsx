// Screen 5 — Create/Edit API Model
function ApiA() {
  return (
    <Browser url="bodhi.local/models/api/new">
      <Crumbs items={['Bodhi','Models','New API model']}/>
      <div className="h1" style={{fontSize:20, marginBottom:6}}>New API model</div>
      <Field label="API format" filled value="OpenAI — Completions" right={<span className="sm">▾</span>}/>
      <div style={{marginTop:6}}>
        <Field label="Base URL" filled value="https://api.openai.com/v1" />
        <span className="sm">Auto-filled · swap for OpenRouter, HF, Groq, etc.</span>
      </div>
      <div style={{marginTop:6}}>
        <Field label="API key · required" filled hint="sk-…" right={<span className="sm">👁</span>}/>
      </div>
      <div className="divider"/>
      <div className="h3" style={{marginTop:0}}>Optional</div>
      <div style={{display:'grid', gridTemplateColumns:'1fr 1fr', gap:8}}>
        <Field label="Model prefix" hint="e.g. openai/" />
        <Field label="Forwarding" value="Forward all" right={<span className="sm">▾</span>}/>
      </div>
      <div style={{display:'flex', gap:6, justifyContent:'space-between', marginTop:10, alignItems:'center', flexWrap:'wrap'}}>
        <Btn>🔌 Test connection</Btn>
        <div style={{display:'flex', gap:6}}>
          <Btn>Cancel</Btn><Btn variant="primary">Save — use all models</Btn>
        </div>
      </div>
      <Callout style={{position:'static', display:'inline-block', marginTop:8}}>★ save with just key — everything else auto-filled</Callout>
    </Browser>
  );
}

function ApiB() {
  return (
    <Browser url="bodhi.local/models/api/new">
      <Crumbs items={['Bodhi','Models','API','New']}/>
      <div className="h1" style={{fontSize:20, marginBottom:6}}>Connect Google Gemini</div>
      <div className="sm" style={{marginBottom:8}}>Steps collapse as you complete them.</div>

      <div className="card" style={{borderColor:'var(--lotus)', background:'var(--lotus-soft)'}}>
        <div style={{display:'flex', justifyContent:'space-between'}}>
          <div className="h2" style={{margin:0}}>1 · Authentication</div>
          <Chip>editing</Chip>
        </div>
        <Field label="API key" filled hint="AIza…" right={<span className="sm">👁</span>} />
        <span className="sm">Get one at <span className="code">aistudio.google.com/apikey</span></span>
        <div style={{display:'flex', gap:6, marginTop:6}}>
          <Btn size="xs">🔌 Test</Btn>
          <Btn size="xs" variant="primary">Continue →</Btn>
        </div>
      </div>

      <div className="card" style={{marginTop:8, opacity:.6}}>
        <div style={{display:'flex', justifyContent:'space-between'}}>
          <div className="h2" style={{margin:0}}>2 · Choose models</div>
          <Chip>fetch after auth</Chip>
        </div>
        <span className="sm">We'll list Gemini models once your key is verified.</span>
      </div>
      <div className="card" style={{marginTop:8, opacity:.6}}>
        <div style={{display:'flex', justifyContent:'space-between'}}>
          <div className="h2" style={{margin:0}}>3 · Routing &amp; prefix (optional)</div>
          <Chip>skip</Chip>
        </div>
        <span className="sm">Prefix like <span className="code">gmn/</span> and forwarding rules.</span>
      </div>
      <Callout style={{position:'static', display:'inline-block', marginTop:10}}>★ stepper — fields appear as you need them</Callout>
    </Browser>
  );
}

function ApiC() {
  return (
    <Browser url="bodhi.local/models/api/new">
      <Crumbs items={['Bodhi','Models','API','Anthropic (Claude Code OAuth)']}/>
      <div style={{display:'flex', alignItems:'center', gap:10}}>
        <div className="ph thumb" style={{background:'#d97557'}}/>
        <div>
          <div className="h1" style={{fontSize:20, margin:0}}>Anthropic · Claude Code OAuth</div>
          <span className="sm">api.anthropic.com · auto-filled · 5 models</span>
        </div>
        <Chip tone="leaf" style={{marginLeft:'auto'}}>● connected</Chip>
      </div>
      <div className="divider"/>
      <div style={{display:'grid', gridTemplateColumns:'1fr 1fr', gap:10}}>
        <div>
          <div className="h3" style={{marginTop:0}}>Auth</div>
          <div className="card">
            <div style={{display:'flex', justifyContent:'space-between'}}>
              <span className="h2" style={{margin:0}}>OAuth token</span>
              <Chip tone="leaf">valid · 30d</Chip>
            </div>
            <span className="sm">Re-auth in your browser if this expires.</span>
            <div style={{display:'flex', gap:4, marginTop:6}}>
              <Btn size="xs">Re-auth</Btn>
              <Btn size="xs">Revoke</Btn>
            </div>
          </div>
          <div className="h3">Routing</div>
          <div className="card">
            <div style={{display:'flex', gap:6, alignItems:'center'}}>
              <Chip on>Forward all</Chip>
              <Chip>Selected only</Chip>
            </div>
            <div style={{marginTop:6}}>
              <Field label="Prefix" filled value="cc-"/>
              <span className="sm">→ cc-opus-4, cc-sonnet-4.5…</span>
            </div>
          </div>
        </div>
        <div>
          <div className="h3" style={{marginTop:0}}>Models (5 selected)</div>
          <div className="card" style={{padding:6}}>
            {[['claude-opus-4.1','✓','$15/$75'],
              ['claude-sonnet-4.5','✓','$3/$15'],
              ['claude-haiku-4.5','✓','$0.80/$4'],
              ['claude-sonnet-3.7','✓','$3/$15'],
              ['claude-haiku-3.5','','$0.80/$4']
            ].map((r,i)=>(
              <div key={i} style={{display:'flex', alignItems:'center', justifyContent:'space-between', padding:'5px 4px', borderBottom: i<4?'1px dashed var(--line-soft)':'none'}}>
                <span className="md"><input type="checkbox" defaultChecked={r[1]==='✓'} /> {r[0]}</span>
                <span className="sm">{r[2]} <Chip tone="leaf">🔧</Chip></span>
              </div>
            ))}
          </div>
          <Callout style={{position:'static', display:'inline-block', marginTop:6}}>★ cost + tool-capability per model</Callout>
        </div>
      </div>
      <div style={{display:'flex', gap:6, justifyContent:'flex-end', marginTop:10}}>
        <Btn>Cancel</Btn><Btn variant="primary">Update</Btn>
      </div>
    </Browser>
  );
}

window.ApiScreens = [
  {label:'A · One-form minimal', tag:'familiar', note:'Format + base URL + key. Base URL auto-fills per format. Save with just a key.', novel:'save-with-key-only default', component:ApiA},
  {label:'B · Stepper', tag:'balanced', note:'Auth → fetch models → route. Next step unlocks when prior is valid.', novel:null, component:ApiB},
  {label:'C · Provider-aware editor', tag:'bold', note:'Branded header, cost per model, OAuth lifecycle in one card.', novel:'per-model cost + capability inline', component:ApiC},
];
