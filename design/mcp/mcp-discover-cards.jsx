/* ═══════════════════════════════════════════════════
   Bodhi MCP Discover — rows + sidebar filters
   mcp/mcp-discover-cards.jsx   (load after mcp-catalog.jsx)

   The list presentation of a server (McpRow) and the sidebar filter
   groups. Rows are STATUS-ONLY: they show state and open the detail rail
   on click. Every action (connect / request / configure) lives in the
   rail. Exports: McpRow, DiscoverSidebar.
═══════════════════════════════════════════════════ */

/* ══ Row ══ */
function McpRow({ s, role, active, onOpen }) {
  return (
    <ListRow className={statusClass(s)} active={active} onSelect={() => onOpen(s.id)} label={`Open ${s.name}`}>
      <div className="row-icon"><div className="row-icon-box" style={{ background: s.iconBg, color: s.iconColor, borderColor: s.iconBg }}>{s.icon}</div></div>
      <div className="row-body">
        <div className="row-name">{s.name}</div>
        <div className="row-pub">{s.publisher}{s.verified && <Ic name="badge-check" size={10} color="var(--c-indigo-text)" />}</div>
      </div>
      <div className="row-cat"><CatBadge s={s} /></div>
      <div className="row-auth"><AuthBadges auths={s.auth} /></div>
      <div className="row-stat"><StatusLine s={s} role={role} /></div>
      <div className="row-act"><span className="card-open"><Ic name="chevron-right" size={14} /></span></div>
    </ListRow>
  );
}

/* ══ Sidebar filters ══
   My MCPs → registered servers only, with a Scope toggle (all / mine).
   Explore → discovery filters (category functional, rest decorative). */
const AuthFilter = () => (
  <ShellFilterGroup icon="key-round" label="Auth Type" chips={[
    { label: 'Any', defaultOn: true }, { label: 'OAuth', color: 'indigo' }, { label: 'API Key', color: 'saffron' }, { label: 'Public' }]} />
);

function DiscoverSidebar({ mode = 'explore', stab = 'all', setStab, myScope = 'all', setMyScope, role = 'user' }) {
  if (mode === 'my-mcps') {
    return (
      <>
        <ShellFilterGroup icon="filter" label="Scope" single value={myScope} onSelect={setMyScope} chips={[
          { id: 'all', label: 'Configured' },
          { id: 'mine', label: 'Connected', color: 'leaf' },
        ]} />
        <ShellFilterGroup icon="shapes" label="Category" single value={stab} onSelect={setStab} chips={[
          { id: 'all', label: 'All' }, { id: 'Productivity', label: 'Productivity', color: 'lotus' },
          { id: 'Dev Tools', label: 'Dev Tools', color: 'indigo' }, { id: 'Search & Web', label: 'Search & Web', color: 'saffron' },
          { id: 'Browser', label: 'Browser', color: 'teal' }, { id: 'Data', label: 'Data', color: 'leaf' },
          { id: 'Comms', label: 'Comms', color: 'indigo' }, { id: 'Memory', label: 'Memory' },
        ]} />
        <AuthFilter />
      </>
    );
  }
  const catChips = [
    { id: 'all', label: 'All' }, { id: 'Productivity', label: 'Productivity', color: 'lotus' },
    { id: 'Dev Tools', label: 'Dev Tools', color: 'indigo' }, { id: 'Search & Web', label: 'Search & Web', color: 'saffron' },
    { id: 'Browser', label: 'Browser', color: 'teal' }, { id: 'Data', label: 'Data', color: 'leaf' },
    { id: 'Comms', label: 'Comms', color: 'indigo' }, { id: 'Memory', label: 'Memory' },
  ];
  return (
    <>
      <ShellFilterGroup icon="shapes" label="Category" single value={stab} onSelect={setStab} chips={catChips} />
      <AuthFilter />
      <ShellFilterGroup icon="shield-check" label="Availability" chips={[
        { label: 'All', defaultOn: true }, { label: 'Configured', color: 'leaf' }, { label: 'Not added' }]} />
      <ShellFilterGroup icon="badge-check" label="Publisher" chips={[
        { label: 'Verified ✓' }, { label: 'Official' }, { label: 'Community' }]} />
    </>
  );
}

Object.assign(window, { McpRow, DiscoverSidebar });
