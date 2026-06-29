/* ═══════════════════════════════════════════════════
   Bodhi MCP Discover — page root
   mcp/bodhi-mcp-discover-app.jsx   (load LAST of the discover modules)

   Owns page state (server list + mutations, role switch, filters,
   selection) and assembles the shell: sidebar, the card/list area, and
   the detail rail. All server config now lives on the dedicated New /
   Edit Server form page — no slide-over here.

   Module load order (set in the HTML):
     mcp-catalog · mcp-discover-cards · mcp-discover-detail ·
     bodhi-mcp-discover-app
═══════════════════════════════════════════════════ */
const { useState, useEffect } = React;
const MCP_MODE = window.MCP_MODE || 'explore';

function DiscoverApp() {
  const mode = MCP_MODE;
  const [servers, setServers] = useState(INITIAL_SERVERS);
  const [stab, setStab] = useState('all');           // category filter (both modes)
  const [myScope, setMyScope] = useState('all');     // my-mcps scope: all | mine
  const [search, setSearch] = useState('');
  const [role, setRole] = useState('user');
  const [activeId, setActiveId] = useState(null);

  const matchesCat = s => stab === 'all' || s.category === stab;
  const matchesSearch = s => {
    if (!search) return true;
    const q = search.toLowerCase();
    return s.name.toLowerCase().includes(q) || s.publisher.toLowerCase().includes(q) || s.category.toLowerCase().includes(q);
  };

  let baseList;
  if (mode === 'my-mcps') {
    baseList = servers.filter(s => s.registered);
    if (myScope === 'mine') baseList = baseList.filter(s => s.userInstances.length > 0);
  } else {
    baseList = servers;
  }
  const visible = baseList.filter(matchesCat).filter(matchesSearch);

  useEffect(() => {
    if (window.matchMedia('(max-width:767px)').matches) return;
    if (visible[0]) setActiveId(visible[0].id);
  }, []);

  const activeServer = servers.find(s => s.id === activeId) || null;
  const updateServer = (id, fn) => setServers(prev => prev.map(s => s.id === id ? fn({ ...s }) : s));

  const actions = {
    deleteInstance: (id, instId) => updateServer(id, s => ({ ...s, userInstances: s.userInstances.filter(i => i.id !== instId) })),
    requestServer: id => updateServer(id, s => ({ ...s, requestStatus: 'pending' })),
  };

  const headerActions = (
    <div className="role-badge" onClick={() => setRole(r => r === 'user' ? 'admin' : 'user')} title="Click to switch role">
      <Ic name="user-circle" size={11} /> Role: {role === 'admin' ? 'Admin' : 'User'} <Ic name="chevron-down" size={11} />
    </div>
  );

  return (
    <AppShell
      section="mcp" subPage={mode === 'my-mcps' ? 'my-mcps' : 'explore'} resizeKey="mcp"
      railWidth={380} railMin={320} railMax={540}
      breadcrumb={[{ label: 'Bodhi', href: 'Chat.html' }, { label: 'MCP', href: 'MCP-My-MCPs.html' }, { label: mode === 'my-mcps' ? 'My MCPs' : 'Explore', current: true }]}
      headerActions={headerActions}
      sidebar={<DiscoverSidebar mode={mode} stab={stab} setStab={setStab} myScope={myScope} setMyScope={setMyScope} role={role} />}
      contentClass="flush" mainScroll={false} railScroll={false}
      railHeader={activeServer ? <DiscoverRailHeader s={activeServer} setActiveId={setActiveId} /> : undefined}
      rail={activeServer ? <DetailPanel s={activeServer} role={role} actions={actions} /> : null}
    >
      <MainArea
        visible={visible} stab={stab} setStab={setStab} search={search} setSearch={setSearch}
        role={role} activeId={activeId} mode={mode}
        onOpen={(id) => { setActiveId(id); }} />
    </AppShell>
  );
}

/* main content — needs useShell for openRail, so it's its own component inside AppShell. */
function MainArea({ visible, stab, setStab, search, setSearch, role, activeId, mode, onOpen }) {
  const { openRail } = useShell();
  useListKeyNav();
  const open = (id) => { onOpen(id); openRail(); };
  const pg = usePagination(visible, 10, mode + '|' + stab + '|' + search);

  const listHead = (
    <>
      <div className="lh-icon"></div>
      <div className="lh-name l-lh">Server</div>
      <div className="lh-cat l-lh">Category</div>
      <div className="lh-auth l-lh">Auth</div>
      <div className="lh-stat l-lh">Status</div>
      <div className="lh-act"></div>
    </>
  );

  return (
    <div className="l-page">
      <ListToolbar
        searchMode="inline" searchKbd="⌘K"
        search={search} onSearch={setSearch}
        searchPlaceholder={mode === 'my-mcps' ? 'Search your MCPs by name or publisher…' : 'Search MCP servers by name, publisher, or tag…'} />
      <div className="l-scroll">
        {visible.length === 0 ? (
          <div className="l-empty"><Ic name="search-x" size={30} /><div className="l-empty-t">{mode === 'my-mcps' ? 'No MCPs match' : 'No servers match'}</div><div className="l-empty-s">{mode === 'my-mcps' ? 'Adjust the scope or head to Explore' : 'Try adjusting filters or search'}</div></div>
        ) : (
          <ListView head={listHead}>
            {pg.slice.map(s => <McpRow key={s.id} s={s} role={role} active={activeId === s.id} onOpen={open} />)}
          </ListView>
        )}
      </div>
      {visible.length > 0 &&
        <Pagination total={pg.total} page={pg.page} onPage={pg.setPage}
          pageSize={pg.pageSize} onPageSize={pg.setPageSize}
          pageSizeOptions={[10, 25, 50]} unit="servers" />
      }
    </div>
  );
}

ReactDOM.createRoot(document.getElementById('root')).render(<DiscoverApp />);
