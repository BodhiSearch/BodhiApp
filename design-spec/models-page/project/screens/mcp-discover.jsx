// MCP Discover · v30
// Browse the Bodhi-curated MCP catalog. Each card resolves to one of five states
// (catalog-only / pending / approved / connected / disabled) that drives its CTA.
// Five variants: Desktop / Medium / Mobile + two overlay demos showing the
// McpServerForm and McpInstanceForm launched on top of the Discover grid.

// Shared: the card grid + filter chrome used inside every variant. We accept a
// `columns` override so Medium/Mobile can force a different grid layout.
function McpDiscoverBody({columns='auto', showDrawer=true, role='user', slice}) {
  const [category, setCategory] = React.useState('all');
  const [status, setStatus] = React.useState('all');
  const [search, setSearch] = React.useState('');
  const [open, setOpen] = React.useState('notion');

  const filtered = MCP_CATALOG_FIXTURE.filter(e =>
    (category === 'all' || e.category === category) &&
    (status === 'all' || e.state === status) &&
    (!search || e.name.toLowerCase().includes(search.toLowerCase()))
  );
  const visible = slice ? filtered.slice(0, slice) : filtered;
  const instanceFor = (slug) => MCP_INSTANCES_FIXTURE.find(i => i.slug === slug);
  const opened = MCP_CATALOG_FIXTURE.find(e => e.slug === open);

  const gridClass = columns === 'two' ? 'mcp-card-grid two-col' : columns === 'one' ? 'mcp-card-grid one-col' : 'mcp-card-grid';

  return (
    <>
      <div className="mcp-filters">
        <div className="mcp-filters-row">
          <div className="mcp-search-field">
            <span>🔍</span>
            <span style={{flex:1, color: search ? 'var(--ink)' : 'var(--ink-4)'}}>{search || 'Search MCP servers by name, publisher, or tag…'}</span>
          </div>
          <McpStatusFilter active={status} onChange={setStatus}/>
        </div>
        <McpCategoryChipRow active={category} onChange={setCategory}/>
      </div>
      <div className={showDrawer ? 'mcp-discover-layout' : 'mcp-discover-layout no-drawer'}>
        <div>
          <div className={gridClass}>
            {visible.map(e => (
              <McpCatalogCard key={e.slug}
                entry={e}
                role={role}
                instance={e.state === 'connected' ? instanceFor(e.slug) : undefined}
                onOpen={() => setOpen(e.slug)}/>
            ))}
          </div>
          <Callout style={{position:'static', display:'inline-block', marginTop:8}}>
            ★ Cards span all 5 states · CTA changes: admin one-click Add · user Submit-for-Approval · + Add MCP Server · View instance · Re-enable
          </Callout>
        </div>
        {showDrawer && opened && <McpCatalogDrawer entry={opened} activeTab="capabilities"/>}
      </div>
    </>
  );
}

// ── 1. Desktop (Standalone) ─────────────────────────────────────
function McpDiscoverDesktop() {
  return (
    <Browser url="bodhi.local/mcps/discover">
      <Crumbs items={['Bodhi','MCP','Discover']}/>
      <div style={{display:'flex', alignItems:'baseline', justifyContent:'space-between', gap:10, marginBottom:6}}>
        <div>
          <div className="h1" style={{fontSize:20, marginBottom:2}}>MCP Discover</div>
          <div className="sm">Bodhi-curated catalog · click a card for capabilities · one-click Add or Submit-for-Approval</div>
        </div>
        <div style={{display:'flex', gap:6}}>
          <Btn size="xs">Role: User ▾</Btn>
        </div>
      </div>
      <McpDiscoverBody/>
    </Browser>
  );
}

// ── 2. Overlay · Server form (admin pre-fill) ───────────────────
function McpDiscoverOverlayServer() {
  const entry = MCP_CATALOG_FIXTURE.find(e => e.slug === 'linear');
  const context = (
    <>
      <span className="sm" style={{color:'var(--ink)'}}>Adding to app</span>
      <Chip tone="indigo" style={{fontSize:10}}>catalog entry</Chip>
      <code>linear</code>
      <span className="sm" style={{marginLeft:'auto'}}>admin · one-click pre-fill</span>
    </>
  );
  const body = (
    <McpServerForm mode="prefilled" initial={{
      url: entry.defaultBaseUrl, name: entry.slug, description: entry.short,
      authType: entry.authType, authConfig: entry.authConfig,
    }}/>
  );
  const footer = (
    <>
      <Btn variant="ghost" size="xs">🔌 Test connection</Btn>
      <Btn variant="ghost" size="xs">Open full page ↗</Btn>
      <Btn>Cancel</Btn>
      <Btn variant="primary">Save MCP Server</Btn>
    </>
  );
  return (
    <>
      <OverlayShell title="Register MCP Server · from catalog" context={context} body={body} footer={footer}/>
      <Callout style={{position:'static', display:'inline-block', marginTop:8}}>
        ★ Admin clicks a card CTA on Discover · this overlay opens with all OAuth fields pre-filled from the catalog entry
      </Callout>
    </>
  );
}

// ── 3. Overlay · Instance form (user create) ────────────────────
function McpDiscoverOverlayInstance() {
  const context = (
    <>
      <span className="sm" style={{color:'var(--ink)'}}>Creating instance of</span>
      <Chip tone="indigo" style={{fontSize:10}}>approved server</Chip>
      <code>linear</code>
      <span className="sm" style={{marginLeft:'auto'}}>user · one-click Add MCP Server</span>
    </>
  );
  const body = (
    <McpInstanceForm initial={{serverSlug:'linear', name:'linear', slug:'linear', authState:'connected'}}/>
  );
  const footer = (
    <>
      <Btn variant="ghost" size="xs">Open full page ↗</Btn>
      <Btn>Cancel</Btn>
      <Btn variant="primary">Create MCP instance</Btn>
    </>
  );
  return (
    <>
      <OverlayShell title="Add MCP Server · instance" context={context} body={body} footer={footer}/>
      <Callout style={{position:'static', display:'inline-block', marginTop:8}}>
        ★ User opens Discover → clicks `+ Add MCP Server` on an approved card · same form reachable full-page from My MCPs
      </Callout>
    </>
  );
}

// ── 4. Medium · tablet ──────────────────────────────────────────
function McpDiscoverMedium() {
  return (
    <TabletFrame label="Tablet · 2-column grid">
      <MobileHeader active="MCP Discover"/>
      <div style={{padding:'8px 10px'}}>
        <div className="h1" style={{fontSize:16, marginBottom:2}}>MCP Discover</div>
        <div className="sm" style={{fontSize:11, marginBottom:6}}>Drawer collapses to full-screen sheet on row tap.</div>
        <McpDiscoverBody columns="two" showDrawer={false} slice={8}/>
      </div>
    </TabletFrame>
  );
}

// ── 5. Mobile · phone ───────────────────────────────────────────
function McpDiscoverMobile() {
  return (
    <div className="phone-deck">
      <PhoneFrame label="1 · Grid + category sheet">
        <MobileHeader active="MCP Discover"/>
        <div style={{padding:'6px 8px'}}>
          <div className="h1" style={{fontSize:14, marginBottom:2}}>MCP Discover</div>
          <McpDiscoverBody columns="one" showDrawer={false} slice={4}/>
        </div>
      </PhoneFrame>
      <PhoneFrame label="2 · Detail sheet (tapping a card)">
        <MobileHeader active="Notion · detail"/>
        <div style={{padding:'6px 8px'}}>
          <McpCatalogDrawer entry={MCP_CATALOG_FIXTURE[0]} activeTab="capabilities"/>
        </div>
      </PhoneFrame>
    </div>
  );
}

window.McpDiscoverScreens = [
  {label:'A · Desktop · Standalone', tag:'balanced',
    note:'Curated grid of 12 cards across 5 derived states (catalog-only / pending / approved / connected / disabled). Filter chrome: search + 10 category chips + 5 status pills. Right-drawer shows Capabilities / Connection / Stats / Metadata for the opened card. CTA per card adapts to viewer role + state.',
    novel:'five-state CTA contract · back-link drawer · role-aware one-click Add',
    component:McpDiscoverDesktop},
  {label:'A · Overlay · Server form (admin pre-fill)', tag:'balanced',
    note:'Admin clicks CTA on a catalog-only card → this overlay opens pre-filled from the catalog entry. Save creates the MCP server registry entry for the whole app. Matches production download (24).png.',
    novel:'admin one-click add from catalog',
    component:McpDiscoverOverlayServer},
  {label:'A · Overlay · Instance form (user add)', tag:'balanced',
    note:'Once a server is approved, user clicks + Add MCP Server on the card → this overlay opens. OAuth connect step shows Connected state with Client ID + Disconnect button. Matches production download (25).png.',
    novel:'same form launches full-page from My MCPs',
    component:McpDiscoverOverlayInstance},
  {label:'A · Medium · tablet', tag:'medium',
    note:'Two-column card grid; no right drawer (card tap opens full-screen sheet); status + category filters wrap.',
    novel:'card grid at tablet width',
    component:McpDiscoverMedium},
  {label:'A · Mobile', tag:'mobile',
    note:'Two frames: single-column card stack on a phone, and the card-detail full-screen sheet that replaces the desktop drawer.',
    novel:'detail sheet replaces drawer at phone width',
    component:McpDiscoverMobile},
];
