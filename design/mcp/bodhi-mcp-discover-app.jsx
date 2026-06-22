/* ═══════════════════════════════════════════════════
   Bodhi MCP Discover — page root
   mcp/bodhi-mcp-discover-app.jsx   (load LAST of the discover modules)

   Owns page state (server list + mutations, role switch, filters,
   selection, config drawer) and assembles the shell with the sidebar,
   the main card/list area, the detail rail, and the config slide-over.

   Module load order (set in Bodhi MCP Discover v2.html):
     mcp-discover-data · mcp-discover-cards · mcp-discover-detail ·
     mcp-discover-config · bodhi-mcp-discover-app
═══════════════════════════════════════════════════ */
const { useState, useEffect } = React;
const MCP_MODE = window.MCP_MODE || 'explore';

function DiscoverApp() {
  const mode = MCP_MODE;
  const [servers, setServers] = useState(INITIAL_SERVERS);
  const [view, setView] = useState('list');
  const [stab, setStab] = useState('all');
  const [search, setSearch] = useState('');
  const [role, setRole] = useState('user');
  const [activeId, setActiveId] = useState(null);
  const [tab, setTab] = useState('about');
  const [configState, setConfigState] = useState(null);

  useEffect(() => {
    if (window.matchMedia('(max-width:767px)').matches) return;
    const base = mode === 'my-mcps' ? servers.filter(s => s.userInstances.length > 0) : servers;
    if (base[0]) setActiveId(base[0].id);
  }, []);

  const activeServer = servers.find(s => s.id === activeId) || null;
  const totalApprovals = servers.reduce((n, s) => n + s.approvalRequests.length, 0);

  const updateServer = (id, fn) => setServers(prev => prev.map(s => s.id === id ? fn({ ...s }) : s));

  const actions = {
    deleteInstance: (id, instId) => updateServer(id, s => ({ ...s, userInstances: s.userInstances.filter(i => i.id !== instId) })),
    deleteAuthConfig: (id, idx) => updateServer(id, s => ({ ...s, authConfigs: s.authConfigs.filter((_, i) => i !== idx) })),
    approveRequest: (id, email) => updateServer(id, s => ({ ...s, approvalRequests: s.approvalRequests.filter(r => r.email !== email), adminApproved: true })),
    rejectRequest: (id, email) => updateServer(id, s => ({ ...s, approvalRequests: s.approvalRequests.filter(r => r.email !== email) })),
    toggleDisabled: id => updateServer(id, s => ({ ...s, disabled: !s.disabled })),
    toggleApproved: id => updateServer(id, s => ({ ...s, adminApproved: !s.adminApproved })),
  };

  const matchesSearch = s => {
    if (!search) return true;
    const q = search.toLowerCase();
    return s.name.toLowerCase().includes(q) || s.publisher.toLowerCase().includes(q) || s.category.toLowerCase().includes(q);
  };
  const matchesStab = s => {
    if (stab === 'all') return true;
    if (mode === 'explore') return s.category === stab;        // category tabs
    if (stab === 'connected') return s.userInstances.some(i => i.status === 'connected');
    if (stab === 'pending') return s.userInstances.some(i => i.status === 'pending');
    if (stab === 'disabled') return s.disabled;
    return true;
  };
  // My MCPs is scoped to servers the user has added an instance for; the admin
  // Approval Requests tab reaches across the full catalog regardless of mode.
  const baseList = stab === 'approval_req'
    ? servers.filter(s => s.approvalRequests.length > 0)
    : (mode === 'my-mcps' ? servers.filter(s => s.userInstances.length > 0) : servers).filter(matchesStab);
  const visible = baseList.filter(matchesSearch);

  const headerActions = (
    <div className="role-badge" onClick={() => setRole(r => r === 'user' ? 'admin' : 'user')} title="Click to switch role">
      <Ic name="user-circle" size={11} /> Role: {role === 'admin' ? 'Admin' : 'User'} <Ic name="chevron-down" size={11} />
    </div>
  );

  return (
    <>
      <AppShell
        section="mcp" subPage={mode === 'my-mcps' ? 'my-mcps' : 'explore'} resizeKey="mcp"
        railWidth={380} railMin={320} railMax={540}
        breadcrumb={[{ label: 'Bodhi', href: 'Bodhi Chat.html' }, { label: 'MCP', href: 'Bodhi MCP My MCPs.html' }, { label: mode === 'my-mcps' ? 'My MCPs' : 'Explore', current: true }]}
        headerActions={headerActions}
        sidebar={<DiscoverSidebar mode={mode} stab={stab} setStab={setStab} role={role} totalApprovals={totalApprovals} />}
        contentClass="flush" mainScroll={false} railScroll={false}
        railHeader={activeServer ? <DiscoverRailHeader s={activeServer} setActiveId={setActiveId} /> : undefined}
        rail={activeServer ? <DetailPanel s={activeServer} role={role} tab={tab} setTab={setTab}
          onConfig={(s, m) => setConfigState({ server: s, mode: m })} actions={actions} /> : null}
      >
        <MainArea
          visible={visible} view={view} setView={setView} stab={stab} setStab={setStab} search={search} setSearch={setSearch}
          role={role} activeId={activeId} totalApprovals={totalApprovals} mode={mode}
          onOpen={(id, t) => { setActiveId(id); setTab(t || 'about'); }} />
      </AppShell>

      <ConfigDrawer state={configState} onClose={() => setConfigState(null)}
        onSave={(id, ac) => {
          if (ac && ac.name) {
            updateServer(id, s => {
              if (s.authConfigs.find(c => c.name === ac.name)) return s;
              const detail = ac.type === 'oauth' ? 'Dynamic registration' : ac.type === 'key' ? 'Header key' : 'Public access';
              return { ...s, authConfigs: [...s.authConfigs, { type: ac.type, name: ac.name, detail }], adminApproved: true };
            });
          }
          setConfigState(null);
        }} />
    </>
  );
}

/* main content — needs useShell for openRail, so it's its own component inside AppShell.
   Built on the shared list-page primitives: <ListToolbar> (category pills left +
   collapsible search) over a single .l-scroll region holding the card grid or the
   edge-to-edge <ListView>. */
function MainArea({ visible, view, setView, stab, setStab, search, setSearch, role, activeId, totalApprovals, mode, onOpen }) {
  const { openRail } = useShell();
  useListKeyNav();
  const open = (id, t) => { onOpen(id, t); openRail(); };
  const pg = usePagination(visible, 10, stab + '|' + search);

  const listHead = (
    <>
      <div className="lh-icon"></div>
      <div className="lh-name l-lh">Server</div>
      <div className="lh-cat l-lh">Category</div>
      <div className="lh-tools l-lh">Tools</div>
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
          <div className="l-empty"><Ic name="search-x" size={30} /><div className="l-empty-t">{mode === 'my-mcps' ? 'No MCPs added yet' : 'No servers match'}</div><div className="l-empty-s">{mode === 'my-mcps' ? 'Head to Explore to connect a server' : 'Try adjusting filters or search'}</div></div>
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
