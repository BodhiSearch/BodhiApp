// Shared wireframe primitives — exposed as window globals for other babel scripts.

const Ph = ({w='70%', h=7, style}) => (
  <div className="ph line" style={{width:w, height:h, ...style}} />
);

const Lines = ({rows=[60,80,40]}) => (
  <div style={{display:'flex', flexDirection:'column', gap:5}}>
    {rows.map((w,i) => <div key={i} className="ph line" style={{width: w+'%'}}/>) }
  </div>
);

const Chip = ({on, tone, children, style, onClick}) => (
  <span className={`chip${on?' on':''}${tone?' '+tone:''}`} style={{...(onClick?{cursor:'pointer'}:{}), ...style}} onClick={onClick}>{children}</span>
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

const Variant = ({label, tag, note, novel, className='', children}) => (
  <section className={`variant ${className}`}>
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

// Shared Downloads panel — used as the right-pane detail view on Hub + Discover
// when the user clicks the "↓ Downloads" menu entry.
const DownloadsPanel = () => (
  <>
    <div className="right-collapsed-rail">downloads · 1 active</div>
    <div className="right-topbar">
      <div style={{display:'flex', alignItems:'center', gap:6, flexWrap:'wrap'}}>
        <Chip tone="saff" style={{fontSize:10}}>downloads</Chip>
        <span className="h2" style={{margin:0}}>Downloads</span>
        <Chip tone="saff">● 1 active</Chip>
      </div>
      <Btn variant="ghost" size="xs" title="collapse">→</Btn>
    </div>
    <div className="sm">8 files · 94 GB on disk · last pull 12m ago</div>

    <div style={{display:'flex', gap:4, marginTop:8, flexWrap:'wrap'}}>
      <Chip on>All</Chip><Chip>Downloading (1)</Chip><Chip>Ready (7)</Chip><Chip>Failed</Chip>
    </div>

    <div className="h3">In progress</div>
    <div className="card" style={{padding:'8px 10px'}}>
      <div style={{display:'flex', alignItems:'center', gap:6, flexWrap:'wrap', marginBottom:4}}>
        <Chip tone="saff" style={{fontSize:10}}>downloading</Chip>
        <code style={{flex:1}}>unsloth/gemma-4-e4b-it:Q4_K_M</code>
        <span className="sm">2.40 / 3.99 GB</span>
      </div>
      <Bar pct={60}/>
      <div style={{display:'flex', justifyContent:'space-between', alignItems:'center', marginTop:4}}>
        <span className="sm">12 MB/s · eta 2m · started 1m ago</span>
        <div style={{display:'flex', gap:4}}>
          <Btn size="xs" title="pause">⏸</Btn>
          <Btn size="xs" title="cancel">✕</Btn>
        </div>
      </div>
    </div>

    <div className="h3">Recently completed</div>
    <div style={{display:'flex', flexDirection:'column', gap:4}}>
      <div className="card row" style={{padding:'5px 7px'}}>
        <Chip tone="leaf" style={{fontSize:10}}>ready</Chip>
        <code style={{flex:1}}>google/gemma-2-9b:Q4_K_M</code>
        <span className="sm">5.4 GB · 12m ago</span>
        <Btn variant="ghost" size="xs">open →</Btn>
      </div>
      <div className="card row" style={{padding:'5px 7px'}}>
        <Chip tone="leaf" style={{fontSize:10}}>ready</Chip>
        <code style={{flex:1}}>qwen/qwen3-14b:Q5_K_M</code>
        <span className="sm">10.1 GB · 2h ago</span>
        <Btn variant="ghost" size="xs">open →</Btn>
      </div>
      <div className="card row" style={{padding:'5px 7px'}}>
        <Chip tone="leaf" style={{fontSize:10}}>ready</Chip>
        <code style={{flex:1}}>LiquidAI/LFM2.5-1.2B:Q8_0</code>
        <span className="sm">1.3 GB · yesterday</span>
        <Btn variant="ghost" size="xs">open →</Btn>
      </div>
      <div className="card row" style={{padding:'5px 7px'}}>
        <Chip tone="warn" style={{fontSize:10}}>failed</Chip>
        <code style={{flex:1}}>meta/Llama-3.3-70B:Q4_K_M</code>
        <span className="sm">disk full · 2d ago</span>
        <Btn size="xs">retry</Btn>
      </div>
    </div>

    <div className="h3">Disk usage</div>
    <div style={{display:'grid', gridTemplateColumns:'auto 1fr', columnGap:8, rowGap:3}}>
      <span className="sm">models dir</span><span className="sm"><code>~/.bodhi/models</code></span>
      <span className="sm">on disk</span><span className="sm">94 GB / 1 TB <Bar pct={9}/></span>
      <span className="sm">cached</span><span className="sm">8 files · 2 broken cleanup candidates</span>
    </div>

    <div style={{display:'flex', gap:4, marginTop:10, flexWrap:'wrap'}}>
      <Btn variant="primary" size="xs">+ Pull from URL…</Btn>
      <Btn variant="ghost" size="xs">Open models folder ↗</Btn>
      <Btn variant="ghost" size="xs">Clear history</Btn>
    </div>
  </>
);

// Tiny sidebar row used on Hub + Discover to open the DownloadsPanel.
// Shows a live badge when something is in progress.
const DownloadsMenu = ({active, count=1, onClick}) => (
  <div className={`downloads-menu${active?' active':''}${count>0?' live':''}`} onClick={onClick}>
    <span className="downloads-menu-icon">↓</span>
    <span style={{flex:1}}>Downloads</span>
    {count > 0 && <span className="downloads-menu-badge">{count} ↓</span>}
  </div>
);

// Generic list-view row used by Hub + Discover when "☰ List" is selected.
// Accepts the same basic shape as ModelCard / DiscoverCard.
const ModelListRow = ({kind='file', title, subtitle, caps=[], meta, cost, status, fitLabel, fit, selected, onClick}) => {
  const kindTone =
    kind==='alias' ? 'saff' :
    kind==='provider' ? 'indigo' :
    kind==='provider-off' ? '' : 'leaf';
  const statusTone =
    status==='live' || status==='ready' || status==='fits' || status==='connected' ? 'leaf' :
    status==='oauth' ? 'saff' :
    status==='rate-limited' || status==='tight' ? 'warn' : '';
  const fitTone = fit==='green' ? 'leaf' : fit==='yellow' ? 'warn' : fit==='red' ? 'warn' : '';
  const extra = kind==='provider-off' ? ' dashed' : '';
  return (
    <div className={`model-list-row${selected?' selected':''}${extra}`} onClick={onClick}>
      <Chip tone={kindTone} style={{fontSize:10}}>{kind==='provider-off' ? 'provider' : kind}</Chip>
      <div className="mlr-title-cell">
        <div className="model-card-title" style={{fontSize:13, margin:0}}>{title}</div>
        {subtitle && <div className="sm">{subtitle}</div>}
      </div>
      <div className="mlr-caps-cell">
        {caps.map((c,i)=>(<Chip key={i}>{c}</Chip>))}
      </div>
      <div className="mlr-meta-cell sm">
        {cost && <div className="mlr-cost">{cost}</div>}
        {meta && <div>{meta}</div>}
      </div>
      <div className="mlr-status-cell">
        {status && <Chip tone={statusTone}>● {status}</Chip>}
        {fitLabel && <Chip tone={fitTone}>● {fitLabel}</Chip>}
      </div>
    </div>
  );
};

// Breadcrumb-style header for mobile + medium (replaces the old ☰-brand topbar).
// The whole path is tappable and opens MobileMenu.
const MobileHeader = ({active='My Models', dlCount=1, rightSlot}) => (
  <div className="m-bc-header">
    <div className="m-bc-path">
      <span>Bodhi</span>
      <span className="m-bc-sep">›</span>
      <span>Models</span>
      <span className="m-bc-sep">›</span>
      <span className="m-bc-active">{active}</span>
      <span className="m-bc-caret">▾</span>
    </div>
    {rightSlot || (
      <span className={`m-ico m-ico-dl${dlCount>0?' live':''}`} title="downloads">
        ↓{dlCount>0 && <span className="m-dl-badge">{dlCount}</span>}
      </span>
    )}
  </div>
);

// Nested app menu that drops from tapping the breadcrumb.
const MobileMenu = ({active='My Models', withDownloads=false, dlCount=1}) => (
  <div className="m-menu-overlay">
    <div className="m-menu">
      <div className="m-menu-item">Chat</div>
      <div className="m-menu-item expanded">
        <div className="m-menu-container">
          <div style={{display:'flex', justifyContent:'space-between', alignItems:'center'}}>
            <span>Models</span><span>▾</span>
          </div>
          <div className="m-menu-sub">
            <div className={`m-menu-sub-item${active==='My Models'?' active':''}`}>My Models</div>
            <div className={`m-menu-sub-item${active==='Discover'?' active':''}`}>Discover</div>
          </div>
        </div>
      </div>
      {withDownloads && (
        <div className="m-menu-item">
          <span>↓ Downloads</span>
          {dlCount>0 && <span className="m-menu-badge">{dlCount} ↓</span>}
        </div>
      )}
      <div className="m-menu-item">Agents</div>
      <div className="m-menu-item">Logs</div>
      <div className="m-menu-item">Settings</div>
    </div>
  </div>
);

// Tablet-shaped frame used for the medium-width wireframes.
const TabletFrame = ({label, children}) => (
  <div className="tablet-frame">
    <div className="tablet-label">{label}</div>
    <div className="tablet-screen">
      <div className="tablet-content">{children}</div>
    </div>
  </div>
);

Object.assign(window, {Ph, Lines, Chip, Btn, Field, TL, Stars, Bar, Crumbs, Browser, Variant, Callout, SectionHead, ModelRow, DownloadsPanel, DownloadsMenu, ModelListRow, MobileHeader, MobileMenu, TabletFrame});
