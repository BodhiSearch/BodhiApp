/* ═══════════════════════════════════════════════════
   USER ACCESS REQUESTS — list page + right detail panel (on AppShell)
   user-access-requests-app.jsx  (load after bodhi-app-shell.jsx + bodhi-list.jsx)
   People requesting access to this Bodhi instance. Admin assigns a role,
   then Approves or Rejects. The role picker + Approve/Reject stay in the
   row (pending) and repeat in the right detail panel (rail).
═══════════════════════════════════════════════════ */
const { useState } = React;
const Ic = ShellIcon;

const ROLES = ['User', 'Power User', 'Admin'];

const AVATAR_COLORS = ['#3E4AA8', '#0F6F67', '#B02A52', '#2F7D1F', '#5E6AD2', '#9A5B12'];
function avatarColor(seed) {
  let h = 0;
  for (let i = 0; i < seed.length; i++) h = (h * 31 + seed.charCodeAt(i)) >>> 0;
  return AVATAR_COLORS[h % AVATAR_COLORS.length];
}
function initials(email) {
  const local = (email.split('@')[0] || email).replace(/[^a-zA-Z0-9]/g, '');
  return (local.slice(0, 2) || email.slice(0, 2)).toUpperCase();
}

const SAMPLE_REQUESTS = [
  { id:'usr_1', email:'admin@email.com',      role:'User',       status:'pending',  requestedAt:'12/06/2026', requestedAgo:'2 hours ago', decidedAt:null },
  { id:'usr_2', email:'manager@email.com',    role:'Power User', status:'pending',  requestedAt:'11/06/2026', requestedAgo:'1 day ago',   decidedAt:null },
  { id:'usr_3', email:'dev1@company.io',      role:'User',       status:'pending',  requestedAt:'10/06/2026', requestedAgo:'2 days ago',  decidedAt:null },
  { id:'usr_4', email:'lead@company.io',      role:'Power User', status:'approved', requestedAt:'05/06/2026', requestedAgo:'1 week ago',  decidedAt:'05/06/2026' },
  { id:'usr_5', email:'intern@email.com',     role:'User',       status:'approved', requestedAt:'02/06/2026', requestedAgo:'2 weeks ago', decidedAt:'03/06/2026' },
  { id:'usr_6', email:'contractor@firm.net',  role:'User',       status:'denied',   requestedAt:'01/06/2026', requestedAgo:'2 weeks ago', decidedAt:'01/06/2026' },
];

const STATUS_ICON = { pending: 'clock', approved: 'check-circle-2', denied: 'x-circle' };
const statusAccent = s => s === 'pending' ? 'var(--c-saffron-bd)' : s === 'approved' ? 'var(--c-leaf-bd)' : 'hsl(var(--border))';
const whenText = req => req.status === 'pending'
  ? `Requested ${req.requestedAgo}`
  : `${req.status === 'denied' ? 'Denied' : 'Approved'} ${req.decidedAt}`;

function StatusChip({ status }) {
  return (
    <span className={`ua-status ${status}`}>
      <Ic name={STATUS_ICON[status]} size={11} />
      {status.charAt(0).toUpperCase() + status.slice(1)}
    </span>
  );
}

function RoleSelect({ value, onChange, className = 'ua-role-select' }) {
  return (
    <select className={className} value={value} onClick={e => e.stopPropagation()}
            onChange={e => { e.stopPropagation(); onChange(e.target.value); }}>
      {ROLES.map(r => <option key={r} value={r}>{r}</option>)}
    </select>
  );
}

function RequestRow({ req, selected, onSelect, onRole, onApprove, onReject }) {
  const pending = req.status === 'pending';
  return (
    <div className={`l-listrow ua-row accent ${req.status}${selected ? ' active' : ''}`}
         style={{ '--row-accent': statusAccent(req.status) }} onClick={() => onSelect(req.id)}>
      <div className="ua-icon"><div className="ua-avatar" style={{ background: avatarColor(req.email) }}>{initials(req.email)}</div></div>

      <div className="ua-id">
        <div className="ua-email">{req.email}</div>
        <div className="ua-sub">{whenText(req)}</div>
      </div>

      <div className="ua-status-cell"><StatusChip status={req.status} /></div>

      <div className="ua-role-cell">
        {pending
          ? <RoleSelect value={req.role} onChange={r => onRole(req.id, r)} />
          : <span className="ua-role-static"><Ic name="shield" size={11} /> {req.role}</span>}
      </div>

      <div className="ua-act">
        {pending && (
          <>
            <button className="ua-approve" onClick={e => { e.stopPropagation(); onApprove(req.id); }}>
              <Ic name="check" size={13} /> Approve
            </button>
            <button className="ua-reject" onClick={e => { e.stopPropagation(); onReject(req.id); }}>
              Reject
            </button>
          </>
        )}
      </div>
    </div>
  );
}

/* ── Rail header (railHeader slot) ── */
function RequestDetailHeader({ req, onClose }) {
  return (
    <div className="dp-head">
      <div className="dp-head-icon" style={{ background: avatarColor(req.email) }}>{initials(req.email)}</div>
      <div className="dp-head-body">
        <div className="dp-head-title">{req.email}</div>
        <div className="dp-head-sub">User access request</div>
      </div>
      <button className="dp-close" onClick={onClose} title="Close"><Ic name="x" size={15} /></button>
    </div>
  );
}

/* ── Rail body (rail slot) ── */
function RequestDetailPanel({ req, onRole, onApprove, onReject }) {
  const pending  = req.status === 'pending';
  const approved = req.status === 'approved';

  return (
    <div className="dp-panel">
      <div className="dp-status-row">
        <StatusChip status={req.status} />
        <span className="dp-head-sub" style={{ marginLeft: 'auto' }}>{whenText(req)}</span>
      </div>

      <div className="dp-body">
        <div className="dp-section">
          <div className="dp-sec-lbl">Account</div>
          <div className="dp-rows">
            <div className="dp-row"><span className="dp-row-k"><Ic name="at-sign" size={13} /> Email</span><span className="dp-row-v mono">{req.email}</span></div>
            <div className="dp-row"><span className="dp-row-k"><Ic name="calendar" size={13} /> Requested</span><span className="dp-row-v">{req.requestedAt}</span></div>
          </div>
        </div>

        <div className="dp-section">
          <div className="dp-sec-lbl">{pending ? 'Assign role' : 'Role'}</div>
          {pending ? (
            <div className="dp-field">
              <RoleSelect value={req.role} onChange={r => onRole(req.id, r)} className="ua-role-select dp-role-select" />
              <span className="dp-field-hint">The role is granted to this user when you approve the request.</span>
            </div>
          ) : (
            <div className="dp-resource"><Ic name="shield" size={14} /> {req.role}</div>
          )}
        </div>

        <div className="dp-section">
          <div className="dp-sec-lbl">Timeline</div>
          <div className="dp-rows">
            <div className="dp-row"><span className="dp-row-k"><Ic name="clock" size={13} /> Requested</span><span className="dp-row-v">{req.requestedAt}</span></div>
            {req.decidedAt && (
              <div className="dp-row"><span className="dp-row-k"><Ic name={approved ? 'check' : 'x'} size={13} /> {approved ? 'Approved' : 'Denied'}</span><span className="dp-row-v">{req.decidedAt}</span></div>
            )}
          </div>
        </div>
      </div>

      <div className="dp-foot">
        {pending ? (
          <div className="dp-foot-row">
            <button className="dp-btn dp-btn-approve" onClick={() => onApprove(req.id)}><Ic name="check" size={14} /> Approve</button>
            <button className="dp-btn dp-btn-danger" onClick={() => onReject(req.id)}><Ic name="x" size={14} /> Reject</button>
          </div>
        ) : (
          <div className="ua-decided-note">
            <Ic name={approved ? 'check-circle-2' : 'x-circle'} size={14} />
            <span>{approved ? 'Approved' : 'Rejected'} {req.decidedAt}</span>
          </div>
        )}
      </div>
    </div>
  );
}

function UserRequestsMain({ requests, filter, setFilter, search, setSearch, counts, selId, onSelect, onRole, onApprove, onReject }) {
  const { openRail } = useShell();
  const select = id => { onSelect(id); openRail(); };

  const visible = requests.filter(r => {
    if (filter !== 'all' && r.status !== filter) return false;
    if (search && !r.email.toLowerCase().includes(search.toLowerCase())) return false;
    return true;
  });

  return (
    <div className="l-page">
      <ListToolbar
        categories={[
          { id: 'all',      label: 'All',      badge: counts.all },
          { id: 'pending',  label: 'Pending',  badge: counts.pending },
          { id: 'approved', label: 'Approved', badge: counts.approved },
          { id: 'denied',   label: 'Denied',   badge: counts.denied },
        ]}
        category={filter} onCategory={setFilter}
        search={search} onSearch={setSearch} searchPlaceholder="Search requests by email…" />

      <div className="l-scroll">
        {visible.length === 0 ? (
          <div className="l-empty">
            <Ic name="user-check" size={32} />
            <div className="l-empty-t">{search ? 'No requests match your search' : `No ${filter === 'all' ? '' : filter + ' '}requests`}</div>
            <div className="l-empty-s">{search ? 'Try a different search term.' : 'When people request access to this instance, they’ll appear here.'}</div>
          </div>
        ) : (
          <ListView head={
            <>
              <div className="ua-icon"></div>
              <div className="ua-id l-lh">User</div>
              <div className="ua-status-cell l-lh">Status</div>
              <div className="ua-role-cell l-lh">Role</div>
              <div className="ua-act"></div>
            </>
          }>
            {visible.map(req => (
              <RequestRow key={req.id} req={req} selected={req.id === selId}
                onSelect={select} onRole={onRole} onApprove={onApprove} onReject={onReject} />
            ))}
          </ListView>
        )}
      </div>
    </div>
  );
}

function UserRequestsApp() {
  const [requests, setRequests] = useState(SAMPLE_REQUESTS);
  const [filter,   setFilter]   = useState('all');
  const [search,   setSearch]   = useState('');
  const [selId,    setSelId]    = useState(null);

  React.useEffect(() => {
    if (!window.matchMedia('(max-width:767px)').matches) setSelId(SAMPLE_REQUESTS[0].id);
  }, []);

  const setRole    = (id, role) => setRequests(p => p.map(r => r.id === id ? { ...r, role } : r));
  const approve    = id => setRequests(p => p.map(r => r.id === id ? { ...r, status:'approved', decidedAt:'just now' } : r));
  const reject     = id => setRequests(p => p.map(r => r.id === id ? { ...r, status:'denied',   decidedAt:'just now' } : r));

  const counts = {
    all:      requests.length,
    pending:  requests.filter(r => r.status === 'pending').length,
    approved: requests.filter(r => r.status === 'approved').length,
    denied:   requests.filter(r => r.status === 'denied').length,
  };

  const selected = requests.find(r => r.id === selId) || null;

  return (
    <AppShell
      section="users" subPage="user-access-requests" resizeKey="users"
      contentClass="flush" mainScroll={false} railScroll={false}
      breadcrumb={[
        { label: 'Bodhi', href: 'Bodhi Chat.html' },
        { label: 'Users', href: 'User Access Requests.html' },
        { label: 'User Access Requests', current: true },
      ]}
      headerActions={
        counts.pending > 0 && (
          <div className="ua-pending-pill">
            <Ic name="clock" size={12} />
            {counts.pending} pending review
          </div>
        )
      }
      rail={selected ? <RequestDetailPanel req={selected} onRole={setRole} onApprove={approve} onReject={reject} /> : null}
      railHeader={selected ? <RequestDetailHeader req={selected} onClose={() => setSelId(null)} /> : undefined}
    >
      <UserRequestsMain requests={requests} filter={filter} setFilter={setFilter} search={search} setSearch={setSearch}
        counts={counts} selId={selId} onSelect={setSelId} onRole={setRole} onApprove={approve} onReject={reject} />
    </AppShell>
  );
}

ReactDOM.createRoot(document.getElementById('root')).render(<UserRequestsApp />);
