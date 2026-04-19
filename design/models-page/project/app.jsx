// Main app: tabs + rendering of variants per screen
// v25: Models Hub + Discover collapsed into a single "Models" page.
// v27: Provider Directory absorbed into Models (specs/models.md §13).
// v28: Model Detail absorbed — right-drawer dispatch on Models page covers it.
const SCREENS = [
  {key:'models', title:'Models', concept:'local + API + remote · My / All · ranked leaderboards', list: () => window.ModelsScreens},
  {key:'alias', title:'Create local alias', concept:'tune llama.cpp runtime', list: () => window.AliasScreens},
  {key:'api', title:'Create API model', concept:'connect an inference service', list: () => window.ApiScreens},
  {key:'mcp-discover', title:'MCP Discover', concept:'Bodhi-curated catalog · one-click Add or Submit-for-Approval', list: () => window.McpDiscoverScreens},
  {key:'my-mcps', title:'My MCPs', concept:'instances you have connected', list: () => window.MyMcpsScreens},
  {key:'mcp-admin', title:'MCP Admin', concept:'registered servers + approval inbox', list: () => window.McpAdminScreens},
  {key:'mcp-playground', title:'MCP Playground', concept:'exercise tools on a connected instance', list: () => window.McpPlaygroundScreens},
];

// Migrate legacy tab keys (`hub`, `discover`, `providers`, `detail`) → `models` on first load.
(function migrateTabKey(){
  try {
    const cur = localStorage.getItem('bodhi-wf-tab');
    if (cur === 'hub' || cur === 'discover' || cur === 'providers' || cur === 'detail') {
      localStorage.setItem('bodhi-wf-tab', 'models');
    }
  } catch(e){}
})();

function App() {
  const firstTab = (typeof localStorage !== 'undefined' && localStorage.getItem('bodhi-wf-tab')) || 'hub';
  const [tab, setTab] = React.useState(firstTab);
  React.useEffect(() => {
    try { localStorage.setItem('bodhi-wf-tab', tab); } catch(e){}
  }, [tab]);
  const current = SCREENS.find(s => s.key === tab) || SCREENS[0];
  const variants = (current.list() || []);
  const idx = SCREENS.indexOf(current) + 1;
  return (
    <>
      <div className="page active" data-screen-label={`${String(idx).padStart(2,'0')} ${current.title}`}>
        <SectionHead n={idx} title={current.title} concept={current.concept}/>
        <p className="page-intro">
          3 wireframe variants below — familiar → balanced → bolder. Use the tabs to jump between screen families;
          Tweaks toggle (top-right button) hides annotations, color, or sketchy font.
        </p>
        <div className="variants">
          {variants.map((v, i) => {
            const C = v.component;
            return (
              <Variant key={i} label={v.label} tag={v.tag} note={v.note} novel={v.novel}>
                <C />
              </Variant>
            );
          })}
        </div>
      </div>
    </>
  );
}

function Tabs() {
  const firstTab = (typeof localStorage !== 'undefined' && localStorage.getItem('bodhi-wf-tab')) || 'hub';
  const [tab, setTab] = React.useState(firstTab);
  // Two-way: write when our local state changes, but also listen to page's state via a shared handler.
  React.useEffect(() => {
    const handler = (e) => { if (e.key === 'bodhi-wf-tab') setTab(e.newValue); };
    window.addEventListener('storage', handler);
    return () => window.removeEventListener('storage', handler);
  }, []);
  return (
    <>
      {SCREENS.map(s => (
        <button key={s.key} className={`tab ${tab===s.key?'active':''}`}
          onClick={() => {
            try { localStorage.setItem('bodhi-wf-tab', s.key); } catch(e){}
            setTab(s.key);
            // trigger App re-render by dispatching a synthetic storage event in this tab
            window.dispatchEvent(new Event('bodhi-tab-change'));
          }}>{s.title}</button>
      ))}
    </>
  );
}

// Simpler combined root so tabs + page share state
function Root() {
  const init = (typeof localStorage !== 'undefined' && localStorage.getItem('bodhi-wf-tab')) || 'hub';
  const [tab, setTab] = React.useState(init);
  React.useEffect(() => { try { localStorage.setItem('bodhi-wf-tab', tab); } catch(e){} }, [tab]);
  const current = SCREENS.find(s => s.key === tab) || SCREENS[0];
  const variants = current.list() || [];
  const idx = SCREENS.indexOf(current) + 1;
  return (
    <>
      {/* Tabs into the tabs container */}
      {ReactDOM.createPortal(
        SCREENS.map(s => (
          <button key={s.key} className={`tab ${tab===s.key?'active':''}`}
            onClick={() => setTab(s.key)}>{s.title}</button>
        )),
        document.getElementById('tabs')
      )}
      <div className="page active" data-screen-label={`${String(idx).padStart(2,'0')} ${current.title}`}>
        <SectionHead n={idx} title={current.title} concept={current.concept}/>
        <p className="page-intro">
          {variants.length > 1
            ? '3 wireframe variants — familiar → balanced → bolder. Responsive: stacks to 1 column on narrow screens. Toggle Tweaks (top-right) to hide annotations, texture, the sketchy font, or color accents.'
            : 'Selected direction — expand below. Toggle Tweaks (top-right) to hide annotations, texture, the sketchy font, or color accents.'}
        </p>
        <div className={`variants${variants.length===1?' single':''}${variants.some(v=>v.tag==='mobile'||v.tag==='medium')?' stacked':''}`}>
          {variants.map((v, i) => {
            const C = v.component;
            const responsiveClass = (v.tag==='mobile' || v.tag==='medium') ? 'variant-responsive' : '';
            return (
              <Variant key={i} label={v.label} tag={v.tag} note={v.note} novel={v.novel} className={responsiveClass}>
                <C />
              </Variant>
            );
          })}
        </div>
      </div>
    </>
  );
}

ReactDOM.createRoot(document.getElementById('pages')).render(<Root/>);
