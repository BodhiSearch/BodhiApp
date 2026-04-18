// Shared wireframe primitives — exposed as window globals for other babel scripts.

const Ph = ({w='70%', h=7, style}) => (
  <div className="ph line" style={{width:w, height:h, ...style}} />
);

const Lines = ({rows=[60,80,40]}) => (
  <div style={{display:'flex', flexDirection:'column', gap:5}}>
    {rows.map((w,i) => <div key={i} className="ph line" style={{width: w+'%'}}/>) }
  </div>
);

const Chip = ({on, tone, children, style}) => (
  <span className={`chip${on?' on':''}${tone?' '+tone:''}`} style={style}>{children}</span>
);

const Btn = ({variant='', size='', children, style, title}) => (
  <button className={`btn ${variant} ${size}`} style={style} title={title}>{children}</button>
);

const Field = ({label, value, hint, filled, ta, style, right}) => (
  <div style={style}>
    {label && <div className="lbl" style={{marginBottom:3}}>{label}</div>}
    <div className={`field ${filled?'filled':''} ${ta?'ta':''}`}>
      <span>{value || <span style={{color:'var(--ink-4)'}}>{hint || 'placeholder'}</span>}</span>
      {right}
    </div>
  </div>
);

const TL = ({tone='green', children}) => <span className={`tl ${tone}`}>{children}</span>;

const Stars = ({n=4.5}) => {
  const full = Math.floor(n), half = n - full >= 0.5;
  return <span className="stars">{'★'.repeat(full)}{half?'½':''}{'☆'.repeat(5-full-(half?1:0))}</span>;
};

const Bar = ({pct=60, tone='leaf'}) => (
  <div className="bar"><span style={{width: pct+'%', background: `var(--${tone}-soft, var(--leaf-soft))`}}/></div>
);

const Crumbs = ({items=['Bodhi','Models']}) => (
  <div className="crumbs">
    {items.map((it,i)=>(
      <React.Fragment key={i}>
        <span className={i===0?'c-home':''}>{it}</span>
        {i<items.length-1 && <span>›</span>}
      </React.Fragment>
    ))}
  </div>
);

const Browser = ({url='bodhi.local/models', children, style}) => (
  <div className="wf-browser" style={style}>
    <div className="wf-browser-bar">
      <span className="wf-dot r"/><span className="wf-dot y"/><span className="wf-dot g"/>
      <span style={{marginLeft:8}}>{url}</span>
    </div>
    <div className="wf-body">{children}</div>
  </div>
);

const Variant = ({label, tag, note, novel, children}) => (
  <section className="variant">
    <div className="variant-head">
      <span className="variant-label">{label}</span>
      {tag && <span className="variant-tag">{tag}</span>}
    </div>
    {note && <p className="variant-note">{note}</p>}
    {novel && <div style={{margin:'0 0 10px'}}><span className="novel">★ novel: {novel}</span></div>}
    {children}
  </section>
);

const Callout = ({style, children}) => (
  <div className="callout" style={style}>{children}</div>
);

const SectionHead = ({n, title, concept}) => (
  <div style={{display:'flex', alignItems:'baseline', gap:14, marginBottom:6, flexWrap:'wrap'}}>
    <span style={{fontFamily:'var(--hand)', fontSize:18, fontWeight:700, color:'var(--lotus)', border:'1.5px solid var(--ink)', borderRadius:'50%', width:36, height:36, display:'inline-flex', alignItems:'center', justifyContent:'center', background:'var(--lotus-soft)'}}>{n}</span>
    <h2 className="page-title">{title}</h2>
    {concept && <span className="concept">— {concept}</span>}
  </div>
);

// Small model row used across multiple screens
const ModelRow = ({name='Qwen3.5-9B', org='Qwen', task='Text→Text', size='9B', dl='443k', likes='415', fit='green', fitLabel='~38 tok/s', children, highlight}) => (
  <div className="card row" style={{background: highlight?'var(--lotus-soft)':'#fff'}}>
    <div className="ph thumb" style={{background:'linear-gradient(135deg,#ffd6e0,#d4d8f4)'}} />
    <div style={{flex:1, minWidth:0}}>
      <div style={{display:'flex', gap:6, alignItems:'center', flexWrap:'wrap'}}>
        <span className="h2" style={{margin:0}}>{org}/{name}</span>
        <Chip tone="leaf">{size}</Chip>
        <Chip>{task}</Chip>
      </div>
      <div style={{display:'flex', gap:10, alignItems:'center', flexWrap:'wrap', marginTop:3}}>
        <span className="sm">↓ {dl}</span>
        <span className="sm">♥ {likes}</span>
        <TL tone={fit}>{fitLabel}</TL>
        {children}
      </div>
    </div>
  </div>
);

Object.assign(window, {Ph, Lines, Chip, Btn, Field, TL, Stars, Bar, Crumbs, Browser, Variant, Callout, SectionHead, ModelRow});
