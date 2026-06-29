/* ═══════════════════════════════════════════════════════════════
   Layout System — interactive spec / playground
   layout-system.jsx
   The controls below drive the very AppShell rendering this page.
═══════════════════════════════════════════════════════════════ */
const { useState } = React;

function Toggle({ on, set, label }){
  return (
    <button className={'ls-toggle'+(on?' on':'')} onClick={()=>set(v=>!v)}>
      <span className="ls-sw"/><span>{label}</span>
    </button>
  );
}
function Slider({ label, val, set, min, max, step=1, unit='px' }){
  return (
    <label className="ls-slider">
      <span className="ls-slider-top"><span>{label}</span><b>{val}{unit}</b></span>
      <input type="range" min={min} max={max} step={step} value={val} onChange={e=>set(+e.target.value)}/>
    </label>
  );
}

function LayoutSystem(){
  const [sbW, setSbW]   = useState(240);
  const [hH, setHH]     = useState(56);
  const [rW, setRW]     = useState(340);
  const [showRail, setShowRail]     = useState(true);
  const [showBanner, setShowBanner] = useState(false);
  const [overlay, setOverlay]       = useState(true);

  const banner = showBanner ? (
    <div className="shell-banner leaf">
      <ShellIcon name="info" size={16} color="var(--c-leaf-text)"/>
      <div className="txt" style={{color:'var(--c-leaf-text)'}}><strong>banner slot</strong> — a main-column sub-band that sits between the header and the toolbar band. Use it for alerts.</div>
    </div>
  ) : null;

  const rail = showRail ? (
    <div className="ls-rail">
      <p className="ls-rail-note">This is the <b>rail</b> — a first-class third column. Its header shares the same gridline as the sidebar and main.</p>
      <div className="ls-rail-card">railHeader → header band</div>
      <div className="ls-rail-card">rail → scrolling body</div>
    </div>
  ) : undefined;

  return (
    <>
      {overlay && (
        <div className="ls-overlay" aria-hidden="true">
          <div className="ls-gridline" style={{ top: hH+'px' }}><span>header band — {hH}px</span></div>
        </div>
      )}

      <AppShell
        section="settings" subPage={null}
        sidebarWidth={sbW} headerHeight={hH} railWidth={rW}
        breadcrumb={[{label:'Bodhi',href:'#'},{label:'Layout System',current:true}]}
        headerActions={<span className="ls-band-tag">header band</span>}
        sidebar={<div className="ls-sb-note">sidebar body<br/><span>(nav + page content, scrolls)</span></div>}
        rail={rail}
        railHeader={showRail ? <span className="ls-band-tag">rail header</span> : undefined}
        banner={banner}
        contentClass="wide"
      >
        <div className="ls-doc">
          <div className="shell-page-head">
            <div className="title">Bodhi App Shell</div>
            <div className="sub">One parameterized layout for every page. The page title is the first row of main content — it lines up with the sidebar’s nav. Sidebar, main, and the optional rail are columns of a CSS grid whose headers share one height, so the dividers form <b>continuous gridlines</b> across the whole app. Alignment is structural — not eyeballed.</div>
          </div>

          {/* CONTROLS */}
          <div className="ls-panel">
            <div className="ls-panel-h">Live controls <span>— these drive the shell around you</span></div>
            <div className="ls-grid">
              <Slider label="Sidebar width" val={sbW} set={setSbW} min={200} max={320}/>
              <Slider label="Header height" val={hH} set={setHH} min={48} max={72} step={2}/>
              <Slider label="Rail width" val={rW} set={setRW} min={280} max={420}/>
            </div>
            <div className="ls-toggles">
              <Toggle on={showRail} set={setShowRail} label="Rail column"/>
              <Toggle on={showBanner} set={setShowBanner} label="Banner slot"/>
              <Toggle on={overlay} set={setOverlay} label="Gridline overlay"/>
            </div>
          </div>

          {/* PRINCIPLE */}
          <div className="ls-section">
            <h2>The alignment contract</h2>
            <div className="ls-diagram">
              <div className="lsd-col"><span className="lsd-cell h">BRAND</span><span className="lsd-cell b opt">band · optional</span><span className="lsd-cell body">nav · footer</span></div>
              <div className="lsd-col main"><span className="lsd-cell h">HEADER / breadcrumb</span><span className="lsd-cell b opt">toolbar · optional</span><span className="lsd-cell body">content (scroll)</span></div>
              <div className="lsd-col"><span className="lsd-cell h">RAIL HEAD</span><span className="lsd-cell b opt">band · optional</span><span className="lsd-cell body">rail (scroll)</span></div>
            </div>
            <p className="ls-cap">Every column is a flex-column. Child 1 = <code>--shell-header-h</code>. An <b>optional</b> toolbar band (<code>--shell-band-h</code>) can follow for pages that need one. Shared variables ⇒ the horizontal rules line up everywhere.</p>
          </div>

          {/* PROPS */}
          <div className="ls-section">
            <h2>Component API · <code>&lt;AppShell&gt;</code></h2>
            <div className="ls-table">
              {[
                ['section / subPage','Highlights the primary nav + sub-pages'],
                ['sidebarWidth · railWidth','Column widths (px). Default 240 / 340'],
                ['headerHeight · bandHeight','Shared band heights (px). Default 56 / 52'],
                ['breadcrumb','Array of {label, href, current} or a node'],
                ['headerActions','Right side of the header band (main)'],
                ['sidebar','Page-specific body under the nav'],
                ['toolbar · sidebarToolbar · railToolbar','Per-column cells of the shared toolbar band'],
                ['banner','Main-column alert sub-band (under header)'],
                ['rail · railHeader','Enables + fills the third column'],
                ['contentClass','flush · narrow · wide — content sizing'],
              ].map(([p,d])=>(
                <div className="ls-tr" key={p}><code className="ls-prop">{p}</code><span className="ls-desc">{d}</span></div>
              ))}
            </div>
          </div>

          {/* RESPONSIVE */}
          <div className="ls-section">
            <h2>Responsive</h2>
            <div className="ls-bps">
              <div className="ls-bp"><b>≥ 1025px</b><span>Full grid — sidebar · main · rail.</span></div>
              <div className="ls-bp"><b>≤ 1024px</b><span>Rail folds into a right overlay drawer (toggle in header).</span></div>
              <div className="ls-bp"><b>≤ 768px</b><span>Sidebar folds into a left drawer behind a hamburger. Content goes full-bleed.</span></div>
            </div>
            <p className="ls-cap">Try it: drag your window narrower. The current viewport collapses columns automatically.</p>
          </div>

          <div className="ls-foot">Applied so far · <a href="Settings.html">App Settings</a> · <a href="Models-My-Models.html">Models</a> — review, then roll out to all pages.</div>
        </div>
      </AppShell>
    </>
  );
}

ReactDOM.createRoot(document.getElementById('root')).render(<LayoutSystem/>);
