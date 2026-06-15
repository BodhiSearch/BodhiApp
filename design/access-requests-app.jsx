/* ═══════════════════════════════════════════════════
   ACCESS REQUESTS — List page React app
═══════════════════════════════════════════════════ */
const { useState, useEffect, useRef } = React;

const SAMPLE_REQUESTS = [
  {
    id: 'req_1', appName: 'Research Copilot', appInitial: 'R', appColor: '#3E4AA8',
    appDesc: 'Summarises papers, organises Notion pages, pulls market data via web search.',
    verified: true, status: 'pending',
    slots: 3, mcps: 4, role: 'user',
    requestedAt: '2 hours ago', decidedAt: null,
  },
  {
    id: 'req_2', appName: 'DataSync Pro', appInitial: 'D', appColor: '#5E6AD2',
    appDesc: 'Syncs structured data between databases and spreadsheets via API.',
    verified: false, status: 'approved',
    slots: 2, mcps: 1, role: 'user',
    requestedAt: '3 days ago', decidedAt: '3 days ago',
    approvedModels: 2, approvedMcps: 1,
  },
  {
    id: 'req_3', appName: 'AutoReporter', appInitial: 'A', appColor: '#2F7D1F',
    appDesc: 'Generates weekly business reports by querying local models and file data.',
    verified: true, status: 'approved',
    slots: 1, mcps: 0, role: 'user',
    requestedAt: '1 week ago', decidedAt: '1 week ago',
    approvedModels: 1, approvedMcps: 0,
  },
  {
    id: 'req_4', appName: 'Sketch AI', appInitial: 'S', appColor: '#B02A52',
    appDesc: 'Generative image + text co-pilot for design teams.',
    verified: false, status: 'denied',
    slots: 3, mcps: 2, role: 'power',
    requestedAt: '2 weeks ago', decidedAt: '2 weeks ago',
  },
  {
    id: 'req_5', appName: 'DevBot', appInitial: 'DV', appColor: '#0F6F67',
    appDesc: 'Code review, PR summaries, and issue triage for engineering teams.',
    verified: true, status: 'pending',
    slots: 2, mcps: 3, role: 'user',
    requestedAt: '5 hours ago', decidedAt: null,
  },
];

function Icon({ name, size = 14 }) {
  const ref = useRef(null);
  useEffect(() => {
    if (!ref.current) return;
    ref.current.innerHTML = '';
    const el = document.createElement('i');
    el.setAttribute('data-lucide', name);
    ref.current.appendChild(el);
    if (window.lucide) window.lucide.createIcons({ nodes: [el] });
  }, [name]);
  return <span ref={ref} style={{ display:'inline-flex', width:size, height:size, alignItems:'center', justifyContent:'center', flexShrink:0 }} />;
}

const STATUS_ICON = { pending: 'clock', approved: 'check-circle-2', denied: 'x-circle' };

function RequestCard({ req, onRevoke }) {
  useEffect(() => { if (window.lucide) window.lucide.createIcons(); });

  const isApproved = req.status === 'approved';
  const isPending  = req.status === 'pending';
  const isDenied   = req.status === 'denied';

  return (
    <div className={`req-card ${req.status}`}>
      <div className="req-card-body">

        {/* Avatar */}
        <div className="req-app-avatar" style={{ background: req.appColor }}>
          {req.appInitial}
        </div>

        {/* Identity */}
        <div className="req-identity">
          <div className="req-app-name-row">
            <span className="req-app-name">{req.appName}</span>
            {req.verified && <span className="tag tag-leaf">✓ verified</span>}
            <span className="tag tag-muted">3rd-party</span>
          </div>
          <div className="req-app-desc">{req.appDesc}</div>
        </div>

        {/* Resources */}
        <div className="req-divider"></div>
        <div className="req-resources">
          <div className="req-resource-row">
            <Icon name="cpu" size={12} />
            {isApproved
              ? <span>{req.approvedModels} of {req.slots} model slot{req.slots !== 1 ? 's' : ''} approved</span>
              : <span>{req.slots} model slot{req.slots !== 1 ? 's' : ''} requested</span>
            }
          </div>
          <div className="req-resource-row">
            <Icon name="plug" size={12} />
            {isApproved
              ? <span>{req.approvedMcps} of {req.mcps} MCP{req.mcps !== 1 ? 's' : ''} approved</span>
              : <span>{req.mcps} MCP server{req.mcps !== 1 ? 's' : ''} requested</span>
            }
          </div>
          <div className="req-resource-row">
            <Icon name="shield" size={12} />
            <span>Role: {req.role === 'power' ? 'Power User' : 'User'}</span>
          </div>
        </div>

        {/* Date */}
        <div className="req-divider"></div>
        <div className="req-date">
          <div className="req-date-label">
            {isPending ? 'Received' : isDenied ? 'Denied' : 'Approved'}
          </div>
          <div className="req-date-main">{isPending ? req.requestedAt : req.decidedAt}</div>
        </div>

        {/* Status + actions */}
        <div className="req-actions">
          <span className={`req-status ${req.status}`}>
            <Icon name={STATUS_ICON[req.status]} size={11} />
            {req.status.charAt(0).toUpperCase() + req.status.slice(1)}
          </span>
          {isPending && (
            <a href="Bodhi Access Request.html">
              <button className="btn-review">
                <Icon name="clipboard-check" size={12} /> Review
              </button>
            </a>
          )}
          {(isApproved || isDenied) && (
            <button className="btn-view">
              <Icon name="eye" size={12} /> View
            </button>
          )}
        </div>
      </div>

      {/* Approved footer strip */}
      {isApproved && (
        <div className="req-revoke-row">
          <span>Access granted · last active {req.requestedAt}</span>
          <button className="btn-revoke-access" onClick={() => onRevoke(req.id)}>
            <Icon name="shield-off" size={11} /> Revoke access
          </button>
        </div>
      )}
    </div>
  );
}

function AccessRequestsApp() {
  const [requests, setRequests] = useState(SAMPLE_REQUESTS);
  const [filter,   setFilter]   = useState('all');
  const [search,   setSearch]   = useState('');

  useEffect(() => { if (window.lucide) window.lucide.createIcons(); });

  const handleRevoke = id => setRequests(p => p.map(r => r.id === id ? { ...r, status:'denied', decidedAt:'just now' } : r));

  const counts = {
    all:      requests.length,
    pending:  requests.filter(r => r.status === 'pending').length,
    approved: requests.filter(r => r.status === 'approved').length,
    denied:   requests.filter(r => r.status === 'denied').length,
  };

  const visible = requests.filter(r => {
    if (filter !== 'all' && r.status !== filter) return false;
    if (search) {
      const q = search.toLowerCase();
      if (!r.appName.toLowerCase().includes(q) && !r.appDesc.toLowerCase().includes(q)) return false;
    }
    return true;
  });

  return (
    <div className="app">
      <BodhiSidebar section="api-keys" subPage="access-requests" />
      <main className="main">

        {/* Topbar */}
        <div className="topbar">
          <nav className="breadcrumb">
            <a className="bc-seg" href="#">Bodhi</a>
            <Icon name="chevron-right" size={10} />
            <a className="bc-seg" href="App Tokens.html">API Keys</a>
            <Icon name="chevron-right" size={10} />
            <span className="bc-current">Access Requests</span>
          </nav>
          {counts.pending > 0 && (
            <div style={{ marginLeft:'auto', display:'flex', alignItems:'center', gap:6, fontSize:12, fontWeight:600, color:'var(--c-saffron-text)', background:'var(--c-saffron-bg)', border:'1px solid var(--c-saffron-bd)', borderRadius:99, padding:'3px 10px' }}>
              <Icon name="clock" size={12} />
              {counts.pending} pending review
            </div>
          )}
        </div>

        {/* Body */}
        <div className="page-body">
          <div className="page-body-inner">

            <div className="page-header">
              <div className="page-header-text">
                <div className="page-title">Access Requests</div>
                <div className="page-subtitle">Third-party apps requesting access to your models and MCP servers.</div>
              </div>
            </div>

            {/* Toolbar */}
            <div className="toolbar">
              <div className="filter-tabs">
                {[
                  { id:'all',      label:'All',      count: counts.all },
                  { id:'pending',  label:'Pending',  count: counts.pending },
                  { id:'approved', label:'Approved', count: counts.approved },
                  { id:'denied',   label:'Denied',   count: counts.denied },
                ].map(tab => (
                  <button key={tab.id} className={`filter-tab${filter === tab.id ? ' active' : ''}`} onClick={() => setFilter(tab.id)}>
                    {tab.label}
                    <span className="tab-count">{tab.count}</span>
                  </button>
                ))}
              </div>
              <div className="toolbar-spacer"></div>
              <div className="search-wrap">
                <span className="search-icon"><Icon name="search" size={12} /></span>
                <input className="search-input" placeholder="Search apps…" value={search} onChange={e => setSearch(e.target.value)} />
              </div>
            </div>

            {/* List */}
            {visible.length === 0 ? (
              <div className="empty-state">
                <div className="empty-icon"><Icon name="shield-check" size={32} /></div>
                <div className="empty-title">{search ? 'No requests match your search' : `No ${filter === 'all' ? '' : filter + ' '}requests`}</div>
                <div className="empty-sub">{search ? 'Try a different search term.' : "When apps request access to your resources, they'll appear here."}</div>
              </div>
            ) : (
              <div className="req-list">
                {visible.map(req => (
                  <RequestCard key={req.id} req={req} onRevoke={handleRevoke} />
                ))}
              </div>
            )}

          </div>
        </div>

      </main>
    </div>
  );
}

ReactDOM.createRoot(document.getElementById('app-root')).render(<AccessRequestsApp />);
