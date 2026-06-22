/* ═══════════════════════════════════════════════════
   Bodhi MCP Discover — cards, rows, sidebar filters
   mcp/mcp-discover-cards.jsx   (load after mcp-discover-data.jsx)

   The two list presentations of a server (McpCard for the grid view,
   McpRow for the edge-to-edge list view) and the decorative sidebar
   filter groups. Exports: McpCard, McpRow, DiscoverSidebar.
═══════════════════════════════════════════════════ */

/* ══ Card / Row ══ */
function McpCard({ s, role, active, onOpen }) {
  return (
    <div className={`l-card mcp-card ${statusClass(s)}${active ? ' active' : ''}`} onClick={() => onOpen(s.id)}>
      <div className="card-head">
        <div className="card-icon" style={{ background: s.iconBg }}><span style={{ color: s.iconColor, fontSize: 15, fontWeight: 800 }}>{s.icon}</span></div>
        <div className="card-title-block">
          <div className="card-name">{s.name}</div>
          <div className="card-publisher">{s.publisher}{s.verified && <Ic name="badge-check" size={10} color="var(--c-indigo-text)" />}</div>
        </div>
        <CatBadge s={s} />
      </div>
      <div className="card-desc">{s.desc}</div>
      <div className="card-meta">
        <span className="card-meta-item"><Ic name="wrench" size={11} />{s.tools != null ? s.tools : '?'} tools</span>
        <span className="card-meta-item"><Ic name="download" size={11} />{s.installs}</span>
        <AuthBadges auths={s.auth} />
        {role === 'admin' && s.approvalRequests.length > 0 && (
          <span className="auth-badge" style={{ background: 'var(--c-saffron-bg)', borderColor: 'var(--c-saffron-bd)', color: 'var(--c-saffron-text)' }}>
            <Ic name="users" size={10} />{s.approvalRequests.length} request{s.approvalRequests.length > 1 ? 's' : ''}
          </span>
        )}
      </div>
      {s.userInstances.length > 0 && (
        <div className="card-instances">
          {s.userInstances.map(inst => (
            <div className="card-instance-row" key={inst.id}>
              <span className={'inst-dot ' + inst.status}></span>
              <span className="inst-name">{inst.name}</span>
              <span className="inst-time">{inst.time}</span>
              {inst.status === 'connected' && (
                <button className="inst-play-btn" title="Open playground" onClick={e => { e.stopPropagation(); goToPlayground(inst.id, inst.name, s.id); }}><Ic name="play" size={11} /></button>
              )}
            </div>
          ))}
        </div>
      )}
      <div className="card-foot"><StatusLine s={s} /><Cta s={s} role={role} onOpen={onOpen} /></div>
    </div>
  );
}

function McpRow({ s, role, active, onOpen }) {
  return (
    <ListRow className={statusClass(s)} active={active} onSelect={() => onOpen(s.id)} label={`Open ${s.name}`}>
      <div className="row-icon"><div className="row-icon-box" style={{ background: s.iconBg, color: s.iconColor, borderColor: s.iconBg }}>{s.icon}</div></div>
      <div className="row-body">
        <div className="row-name">{s.name}</div>
        <div className="row-pub">{s.publisher}{s.verified && <Ic name="badge-check" size={10} color="var(--c-indigo-text)" />}</div>
      </div>
      <div className="row-cat"><CatBadge s={s} /></div>
      <div className="row-tools"><div className="row-tools-val">{s.tools != null ? s.tools : '?'}</div><div className="row-tools-lbl">tools</div></div>
      <div className="row-auth"><AuthBadges auths={s.auth} /></div>
      <div className="row-stat"><StatusLine s={s} /></div>
      <div className="row-act"><Cta s={s} role={role} onOpen={onOpen} /></div>
    </ListRow>
  );
}

/* ══ Sidebar filters (decorative + collapse-aware) ══
   My MCPs → narrow your own added servers (status / category / auth / transport).
   Explore → discovery filters (category / auth / publisher / availability). */
const CategoryFilter = () => (
  <ShellFilterGroup icon="shapes" label="Category" clearable chips={[
    { label: 'All', defaultOn: true }, { label: 'Productivity', color: 'lotus' }, { label: 'Dev Tools', color: 'indigo' },
    { label: 'Search & Web', color: 'saffron' }, { label: 'Browser', color: 'teal' }, { label: 'Data', color: 'leaf' },
    { label: 'AI & Content', color: 'teal' }, { label: 'Memory' }, { label: 'Comms', color: 'indigo' }]} />
);
const AuthFilter = () => (
  <ShellFilterGroup icon="key-round" label="Auth Type" chips={[
    { label: 'Any', defaultOn: true }, { label: 'OAuth', color: 'indigo' }, { label: 'API Key', color: 'saffron' }, { label: 'No auth' }]} />
);

function DiscoverSidebar({ mode = 'explore', stab = 'all', setStab, role = 'user', totalApprovals = 0 }) {
  if (mode === 'my-mcps') {
    const statusChips = [
      { id: 'all', label: 'All' }, { id: 'connected', label: 'Connected', color: 'leaf' },
      { id: 'pending', label: 'Pending', color: 'saffron' }, { id: 'disabled', label: 'Disabled' },
    ];
    if (role === 'admin' && totalApprovals > 0)
      statusChips.push({ id: 'approval_req', label: 'Approval Requests', color: 'saffron', badge: totalApprovals });
    return (
      <>
        <ShellFilterGroup icon="activity" label="Status" single value={stab} onSelect={setStab} chips={statusChips} />
        <CategoryFilter />
        <AuthFilter />
        <ShellFilterGroup icon="cable" label="Transport" chips={[
          { label: 'Any', defaultOn: true }, { label: 'streamable-http', color: 'indigo' }, { label: 'stdio', color: 'saffron' }]} />
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
        { label: 'All', defaultOn: true }, { label: 'Admin-approved', color: 'indigo' }, { label: 'Available' }]} />
      <ShellFilterGroup icon="badge-check" label="Publisher" chips={[
        { label: 'Verified ✓' }, { label: 'Official' }, { label: 'Community' }]} />
    </>
  );
}

Object.assign(window, { McpCard, McpRow, DiscoverSidebar });
