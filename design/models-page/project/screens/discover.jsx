// Screen 2 — Local model discovery (HF-style browse)
function DiscoverA() {
  return (
    <Browser url="bodhi.local/models/discover">
      <Crumbs items={['Bodhi','Models','Discover local']}/>
      <div className="split w">
        <aside>
          <div className="h3">Task</div>
          {['Text→Text','Vision','Tool-use','Embedding','Speech','Image'].map(t=>(
            <div key={t} style={{marginBottom:4}}><Chip>{t}</Chip></div>
          ))}
          <div className="h3">Parameters</div>
          <div className="slider">
            <div className="tick" style={{left:'5%'}}/><div className="tick" style={{left:'25%'}}/>
            <div className="tick" style={{left:'55%'}}/><div className="tick" style={{left:'85%'}}/>
            <div className="thumb" style={{left:'25%'}}/><div className="thumb" style={{left:'55%'}}/>
          </div>
          <div className="sm">1B — 32B</div>
          <div className="h3">Capability</div>
          {['🔧 Tool use','👁 Vision','🧠 Reasoning','💬 Chat'].map(t=>(
            <div key={t} style={{marginBottom:4}}><Chip>{t}</Chip></div>
          ))}
          <div className="h3">License</div>
          {['Apache-2','MIT','Llama','Gemma'].map(t=>(
            <div key={t} style={{marginBottom:4}}><Chip>{t}</Chip></div>
          ))}
          <div className="h3">Format</div>
          <div><Chip tone="leaf">GGUF ✓</Chip></div>
          <div className="sm" style={{marginTop:4}}>others soon</div>
        </aside>
        <div>
          <div style={{display:'flex', justifyContent:'space-between', alignItems:'center', gap:8, flexWrap:'wrap'}}>
            <Field hint="Filter 2.8M models…" filled style={{flex:1, minWidth:140}}/>
            <div style={{display:'flex', gap:4}}>
              <Chip tone="leaf">Fits my rig</Chip>
              <Chip on>Trending</Chip><Chip>Likes</Chip><Chip>Downloads</Chip>
            </div>
          </div>
          <div className="divider"/>
          {[
            ['Qwen','Qwen3.5-9B','9B','Text→Text','443k','415','green','~38 tok/s'],
            ['unsloth','Nemotron-3-Nano-30B','30B','Text→Text','133k','39','yellow','~6 tok/s · tight'],
            ['google','gemma-4-e2b','2B','Vision+Text','3.8M','2.1k','green','~85 tok/s'],
            ['LiquidAI','LFM2.5-1.2B','1.2B','Text→Text','28k','91','green','~110 tok/s'],
            ['unsloth','Qwen3.5-35B','35B','Text→Text','443k','415','red','won\'t fit'],
          ].map((r,i)=>(
            <ModelRow key={i} org={r[0]} name={r[1]} size={r[2]} task={r[3]} dl={r[4]} likes={r[5]} fit={r[6]} fitLabel={r[7]}/>
          ))}
          <div style={{textAlign:'center', marginTop:8}}><Btn variant="ghost">Load more…</Btn></div>
        </div>
      </div>
    </Browser>
  );
}

function DiscoverB() {
  return (
    <Browser url="bodhi.local/models/discover">
      <Crumbs items={['Bodhi','Models','Discover']}/>
      <div style={{display:'flex', justifyContent:'space-between', alignItems:'baseline', flexWrap:'wrap', gap:6}}>
        <div className="h1" style={{fontSize:20}}>Discover · curated for M3 Max 36GB</div>
        <div style={{display:'flex', gap:4}}>
          <Chip tone="leaf">Fits ✓</Chip><Chip>All</Chip>
        </div>
      </div>
      <div className="sm" style={{marginBottom:8}}>Ranked by quality × fit × popularity. Hardware detected automatically.</div>
      <div className="h3">🏆 Top of leaderboard this week</div>
      <div style={{display:'grid', gridTemplateColumns:'1fr 1fr', gap:8}}>
        {[
          ['Qwen3.5-9B','Qwen','9B','#2 chatbot arena','92.4','~38 tok/s','green'],
          ['gemma-4-e2b','google','2B','#7 open-weights','88.1','~85 tok/s','green'],
          ['LFM2.5-1.2B','LiquidAI','1.2B','#1 tiny models','81.3','~110 tok/s','green'],
          ['Nemotron-3-30B','NVIDIA','30B','#4 reasoning','94.2','~6 tok/s','yellow'],
        ].map((m,i)=>(
          <div key={i} className="card">
            <div style={{display:'flex', alignItems:'center', gap:6}}>
              <div className="ph thumb" style={{width:32,height:32}}/>
              <div style={{flex:1, minWidth:0}}>
                <div className="h2" style={{margin:0}}>{m[1]}/{m[0]}</div>
                <span className="sm">{m[3]} · score {m[4]}</span>
              </div>
              <Chip tone="saff">{m[2]}</Chip>
            </div>
            <div style={{display:'flex', justifyContent:'space-between', alignItems:'center'}}>
              <TL tone={m[6]}>{m[5]}</TL>
              <Btn size="xs" variant="primary">Add →</Btn>
            </div>
          </div>
        ))}
      </div>
      <div className="h3" style={{marginTop:14}}>By capability</div>
      <div style={{display:'flex', gap:6, flexWrap:'wrap'}}>
        {['💬 Chat (124)','🧰 Tool-use (48)','👁 Vision (31)','🔊 Speech (12)','🎨 Image-gen (22)','🧮 Embedding (60)','🔢 Math (18)'].map(t=>
          <Chip key={t}>{t}</Chip>
        )}
      </div>
      <Callout style={{position:'static', display:'inline-block', margin:'10px 0'}}>★ leaderboard-first, not an endless scroll</Callout>
    </Browser>
  );
}

function DiscoverC() {
  return (
    <Browser url="bodhi.local/models/discover">
      <div className="h1" style={{fontSize:20, marginBottom:4}}>Tell us about your use case</div>
      <div className="sm" style={{marginBottom:10}}>We'll narrow 2.8M models down to a shortlist of 3–5.</div>
      <div className="card">
        <div className="h3" style={{marginTop:0}}>1. What for?</div>
        <div style={{display:'flex', gap:6, flexWrap:'wrap'}}>
          <Chip on>💬 General chat</Chip><Chip>🧑‍💻 Code</Chip><Chip>📄 Summarize docs</Chip>
          <Chip>🧰 Agent/tools</Chip><Chip>👁 Image Q&amp;A</Chip>
        </div>
        <div className="h3">2. How long are your inputs?</div>
        <div className="slider">
          <div className="thumb" style={{left:'40%'}}/>
        </div>
        <div style={{display:'flex', justifyContent:'space-between'}} className="sm"><span>short (4K)</span><span>32K</span><span>128K+</span></div>
        <div className="h3">3. Your speed vs. quality preference</div>
        <div className="slider"><div className="thumb" style={{left:'65%'}}/></div>
        <div style={{display:'flex', justifyContent:'space-between'}} className="sm"><span>fast/smaller</span><span>balanced</span><span>best/slower</span></div>
        <div className="h3">4. Privacy</div>
        <div style={{display:'flex', gap:6}}>
          <Chip on>🔒 Local only</Chip><Chip>☁︎ API fine</Chip><Chip>Mix</Chip>
        </div>
      </div>
      <div className="h2" style={{margin:'14px 0 6px'}}>✨ Our picks</div>
      <ModelRow name="Qwen3.5-9B" org="Qwen" size="9B" fitLabel="~38 tok/s · balanced" highlight>
        <Chip tone="saff">match 94%</Chip>
      </ModelRow>
      <ModelRow name="gemma-4-e2b" org="google" size="2B" fitLabel="~85 tok/s">
        <Chip tone="saff">match 88%</Chip>
      </ModelRow>
      <ModelRow name="LFM2.5-1.2B" org="LiquidAI" size="1.2B" fitLabel="~110 tok/s">
        <Chip tone="saff">match 79%</Chip>
      </ModelRow>
      <Callout style={{position:'static', display:'inline-block', marginTop:8}}>★ guided needs → recommendation</Callout>
    </Browser>
  );
}

window.DiscoverScreens = [
  {label:'A · HF-style facets', tag:'familiar', note:'Sidebar filters + ranked list. Adds a "Fits my rig" chip and tok/s estimates on every row.', novel:'per-row tok/s estimate from detected hardware', component:DiscoverA},
  {label:'B · Leaderboard grid', tag:'balanced', note:'Curated hero grid, not a flat list. Ranks by quality × fit × popularity.', novel:'hardware-aware default ranking', component:DiscoverB},
  {label:'C · Guided picker', tag:'bold', note:'Conversational form → 3–5 recommendations. Good for novices and hardware-curious users.', novel:'guided needs → match-score', component:DiscoverC},
];
