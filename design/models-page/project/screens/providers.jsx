// Screen 6 — API Provider Directory
function ProvidersA() {
  const provs = [
    ['OpenAI','api.openai.com','#10a37f','Key','$3/$10','★★★★★','official'],
    ['Anthropic','api.anthropic.com','#d97557','Key/OAuth','$3/$15','★★★★★','OAuth'],
    ['Google Gemini','generativelanguage.googleapis.com','#4285f4','Key','$0.15/$0.60','★★★★☆','free tier'],
    ['OpenRouter','openrouter.ai/api/v1','#111','Key','varies','★★★★☆','200+ models'],
    ['Groq','api.groq.com','#f55036','Key','$0.27/$0.27','★★★★★','fast'],
    ['HuggingFace','api-inference.huggingface.co','#ffcc4d','Key','free/low','★★★☆☆','community'],
    ['Together AI','api.together.xyz','#0f62fe','Key','$0.20/$0.80','★★★★☆','OSS hosted'],
    ['Cerebras','api.cerebras.ai','#e8444d','Key','$0.10/$0.10','★★★★★','fastest'],
    ['NVIDIA NIM','integrate.api.nvidia.com','#76b900','Key','free trial','★★★★☆','ent.'],
  ];
  return (
    <Browser url="bodhi.local/models/api/providers">
      <Crumbs items={['Bodhi','Models','API providers']}/>
      <div style={{display:'flex', justifyContent:'space-between', alignItems:'baseline', flexWrap:'wrap', gap:6}}>
        <div className="h1" style={{fontSize:20}}>Connect an AI inference service</div>
        <Field hint="Search providers…" filled style={{maxWidth:220}}/>
      </div>
      <div className="sm" style={{marginBottom:8}}>Pick a card to autofill base URL &amp; format — then just paste a key.</div>
      <div style={{display:'grid', gridTemplateColumns:'repeat(3,1fr)', gap:8}}>
        {provs.map((p,i)=>(
          <div key={i} className="card">
            <div style={{display:'flex', alignItems:'center', gap:6}}>
              <div className="ph thumb" style={{width:30, height:30, background:p[2]}}/>
              <div style={{flex:1, minWidth:0}}>
                <div className="h2" style={{margin:0}}>{p[0]}</div>
                <span className="sm" style={{fontSize:10}}>{p[1]}</span>
              </div>
              <Chip tone="saff">{p[6]}</Chip>
            </div>
            <div style={{display:'flex', justifyContent:'space-between'}}>
              <span className="sm">{p[3]} · {p[4]}/1M</span>
              <span className="stars" style={{fontSize:11}}>{p[5]}</span>
            </div>
            <Btn size="xs" variant="primary" style={{alignSelf:'flex-start'}}>+ Connect</Btn>
          </div>
        ))}
      </div>
      <Callout style={{position:'static', display:'inline-block', marginTop:8}}>★ one-click setup — no copying URLs</Callout>
    </Browser>
  );
}

function ProvidersB() {
  return (
    <Browser url="bodhi.local/models/api/compare">
      <div className="h1" style={{fontSize:20, marginBottom:6}}>Compare providers · GPT-class chat</div>
      <div className="sm" style={{marginBottom:8}}>Pick two or three — side-by-side cost, speed, reliability.</div>
      <table className="tbl">
        <thead><tr><th></th><th>OpenAI</th><th>Anthropic</th><th>Groq</th><th>OpenRouter</th></tr></thead>
        <tbody>
          <tr><td>Format</td><td>OpenAI</td><td>Anthropic</td><td>OpenAI</td><td>OpenAI</td></tr>
          <tr><td>Top model</td><td>gpt-5</td><td>opus-4.1</td><td>llama-3.3-70B</td><td>routes 200+</td></tr>
          <tr><td>In/Out / 1M</td><td>$5 / $15</td><td>$15 / $75</td><td>$0.60/$0.60</td><td>varies</td></tr>
          <tr><td>Speed</td><td><Bar pct={60}/></td><td><Bar pct={70}/></td><td><Bar pct={98}/></td><td><Bar pct={55}/></td></tr>
          <tr><td>Reliability</td><td>★★★★★</td><td>★★★★★</td><td>★★★★★</td><td>★★★★☆</td></tr>
          <tr><td>Tool use</td><td><TL tone="green">yes</TL></td><td><TL tone="green">yes</TL></td><td><TL tone="yellow">some</TL></td><td><TL tone="green">yes</TL></td></tr>
          <tr><td>Vision</td><td><TL tone="green">yes</TL></td><td><TL tone="green">yes</TL></td><td><TL tone="red">no</TL></td><td><TL tone="yellow">per model</TL></td></tr>
          <tr><td>Auth</td><td>Key</td><td>Key · OAuth</td><td>Key</td><td>Key</td></tr>
          <tr><td>User rating</td><td>★★★★★ 4.8</td><td>★★★★★ 4.9</td><td>★★★★☆ 4.5</td><td>★★★★☆ 4.3</td></tr>
          <tr><td></td><td><Btn size="xs" variant="primary">Connect</Btn></td><td><Btn size="xs" variant="primary">Connect</Btn></td><td><Btn size="xs" variant="primary">Connect</Btn></td><td><Btn size="xs" variant="primary">Connect</Btn></td></tr>
        </tbody>
      </table>
      <Callout style={{position:'static', display:'inline-block', marginTop:8}}>★ side-by-side cost/speed/capability</Callout>
    </Browser>
  );
}

function ProvidersC() {
  return (
    <Browser url="bodhi.local/models/api/providers">
      <div className="h1" style={{fontSize:20, marginBottom:6}}>What do you need?</div>
      <div className="sm" style={{marginBottom:8}}>We'll suggest providers ranked by match.</div>
      <div style={{display:'flex', gap:6, flexWrap:'wrap', marginBottom:8}}>
        <Chip on>⚡ Speed</Chip><Chip>💰 Cheap</Chip><Chip>🧠 Best reasoning</Chip>
        <Chip>👁 Vision</Chip><Chip>🔧 Tool-use</Chip><Chip>🔓 Open source</Chip>
      </div>
      <div className="h3">Top matches for speed</div>
      {[
        ['Cerebras','~1700 t/s · $0.10/1M','★★★★★','match 96%','#e8444d'],
        ['Groq','~800 t/s · $0.27/1M','★★★★★','match 93%','#f55036'],
        ['SambaNova','~700 t/s · free tier','★★★★☆','match 88%','#1f5fff'],
      ].map((p,i)=>(
        <div key={i} className="card row" style={{background:i===0?'var(--lotus-soft)':'#fff'}}>
          <div className="ph thumb" style={{background:p[4]}}/>
          <div style={{flex:1, minWidth:0}}>
            <div className="h2" style={{margin:0}}>{p[0]}</div>
            <span className="sm">{p[1]} · {p[2]}</span>
          </div>
          <Chip tone="saff">{p[3]}</Chip>
          <Btn size="xs" variant="primary">+ Add</Btn>
        </div>
      ))}
      <Callout style={{position:'static', display:'inline-block', marginTop:8}}>★ needs → ranked provider matches</Callout>
    </Browser>
  );
}

window.ProvidersScreens = [
  {label:'A · Logo gallery', tag:'familiar', note:'Card grid with auth, price, rating, speciality tag. One click autofills URL + format.', novel:null, component:ProvidersA},
  {label:'B · Comparison matrix', tag:'balanced', note:'Head-to-head table for cost, speed, capabilities, reliability, ratings.', novel:'capability/cost matrix', component:ProvidersB},
  {label:'C · Needs-based matcher', tag:'bold', note:'Pick your priority (speed/cheap/reasoning) → ranked provider shortlist.', novel:'priority-weighted provider ranking', component:ProvidersC},
];
