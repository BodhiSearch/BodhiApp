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

function DiscoverApp() {
  const [servers, setServers] = useState(INITIAL_SERVERS);
  const [view, setView] = useState('list');
  const [stab, setStab] = useState('all');
  const [search, setSearch] = useState('');
  const [role, setRole] = useState('user');
  const [activeId, setActiveId] = useState(null);
  const [tab, setTab] = useState('about');
  const [configState, setConfigState] = useState(null);

  useEffect(() => {
    if (!window.matchMedia('(max-width:767px)').matches) setActiveId(INITIAL_SERVERS[0].id);
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

  const matchesFilter = s => {
    if (stab === 'mine' && s.userInstances.length === 0) return false;
    if (stab === 'connected' && s.userInstances.every(i => i.status !== 'connected')) return false;
    if (stab === 'approved' && !s.adminApproved) return false;
    if (stab === 'pending' && s.userInstances.every(i => i.status !== 'pending')) return false;
    if (stab === 'not' && s.userInstances.some(i => i.status === 'connected' || i.status === 'pending')) return false;
    if (stab === 'approval_req' && !s.approvalRequests.length) return false;
    if (search) {
      const q = search.toLowerCase();
      if (!s.name.toLowerCase().includes(q) && !s.publisher.toLowerCase().includes(q) && !s.category.toLowerCase().includes(q)) return false;
    }
    return true;
  };
  const visible = servers.filter(matchesFilter);

  const STABS = [
    { id: 'all', label: 'Explore' },
    { id: 'mine', label: 'My Instances', catCls: 'c-leaf' },
    { id: 'approved', label: 'Admin-approved', catCls: 'c-indigo' },
    { id: 'connected', label: 'Connected', catCls: 'c-leaf' },
    { id: 'not', label: 'Not connected' },
    { id: 'pending', label: 'Pending', catCls: 'c-saffron' },
  ];

  const headerActions = (
    <div className="role-badge" onClick={() => setRole(r => r === 'user' ? 'admin' : 'user')} title="Click to switch role">
      <Ic name="user-circle" size={11} /> Role: {role === 'admin' ? 'Admin' : 'User'} <Ic name="chevron-down" size={11} />
    </div>
  );

  return (
    <>
      <AppShell
        section="mcp" subPage="discover" resizeKey="mcp"
        railWidth={380} railMin={320} railMax={540}
        breadcrumb={[{ label: 'Bodhi', href: 'Bodhi Chat.html' }, { label: 'MCP', href: 'Bodhi MCP Discover v2.html' }, { label: 'All MCPs', current: true }]}
        headerActions={headerActions}
        sidebar={<DiscoverSidebar />}
        contentClass="flush" mainScroll={false} railScroll={false}
        railHeader={activeServer ? <DiscoverRailHeader s={activeServer} setActiveId={setActiveId} /> : undefined}
        rail={activeServer ? <DetailPanel s={activeServer} role={role} tab={tab} setTab={setTab}
          onConfig={(s, m) => setConfigState({ server: s, mode: m })} actions={actions} /> : null}
      >
        <MainArea
          visible={visible} view={view} setView={setView} stab={stab} setStab={setStab} search={search} setSearch={setSearch}
          role={role} activeId={activeId} STABS={STABS} totalApprovals={totalApprovals}
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
function MainArea({ visible, view, setView, stab, setStab, search, setSearch, role, activeId, STABS, totalApprovals, onOpen }) {
  const { openRail } = useShell();
  useListKeyNav();
  const open = (id, t) => { onOpen(id, t); openRail(); };

  const cats = STABS.map(t => ({ id: t.id, label: t.label, cls: t.catCls }));
  if (role === 'admin' && totalApprovals > 0) {
    cats.push({ id: 'approval_req', label: 'Approval Requests', cls: 'c-saffron', badge: totalApprovals });
  }

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
        categories={cats} category={stab} onCategory={setStab}
        search={search} onSearch={setSearch}
        searchPlaceholder="Search MCP servers by name, publisher, or tag…"
        actions={
          <button className={'l-iconbtn' + (view === 'cards' ? ' on' : '')}
                  title={view === 'cards' ? 'Switch to list view' : 'Switch to card view'}
                  onClick={() => setView(v => v === 'cards' ? 'list' : 'cards')}>
            <Ic name={view === 'cards' ? 'list' : 'layout-grid'} size={15} />
          </button>
        } />
      <div className="l-scroll">
        {visible.length === 0 ? (
          <div className="l-empty"><Ic name="search-x" size={30} /><div className="l-empty-t">No servers match</div><div className="l-empty-s">Try adjusting filters or search</div></div>
        ) : view === 'cards' ? (
          <div className="l-cardgrid">{visible.map(s => <McpCard key={s.id} s={s} role={role} active={activeId === s.id} onOpen={open} />)}</div>
        ) : (
          <ListView head={listHead}>
            {visible.map(s => <McpRow key={s.id} s={s} role={role} active={activeId === s.id} onOpen={open} />)}
          </ListView>
        )}
      </div>
    </div>
  );
}

ReactDOM.createRoot(document.getElementById('root')).render(<DiscoverApp />);
