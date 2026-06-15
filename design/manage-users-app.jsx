/* ═══════════════════════════════════════════════════
   MANAGE USERS — React App
   Tabs: Pending Requests · All Requests · All Users
═══════════════════════════════════════════════════ */

const MU_TWEAK_DEFAULTS = /*EDITMODE-BEGIN*/{
  "theme": "light"
}/*EDITMODE-END*/;

/* ── Sample data ── */
const INITIAL_USERS = [
  { id: 1, username: 'admin@email.com',          role: 'Admin',      isYou: true  },
  { id: 2, username: 'user@email.com',            role: 'Admin',      isYou: false },
  { id: 3, username: 'poweruser@email.com',       role: 'Power User', isYou: false },
  { id: 4, username: 'attacker-usera@test.local', role: 'User',       isYou: false },
];

const INITIAL_REQUESTS = [
  { id: 101, username: 'manager@email.com',    requestedDate: '07/05/2026', status: 'Pending',  role: 'User'  },
  { id: 102, username: 'dev1@company.io',      requestedDate: '05/05/2026', status: 'Approved', role: 'User'  },
  { id: 103, username: 'contractor@firm.net',  requestedDate: '03/05/2026', status: 'Rejected', role: 'User'  },
  { id: 104, username: 'intern@email.com',     requestedDate: '01/05/2026', status: 'Pending',  role: 'Power User' },
];

const ROLES = ['Admin', 'Power User', 'User'];

/* ── Icon helper ── */
function Icon({ name, size = 14, style = {} }) {
  const ref = React.useRef(null);
  React.useEffect(() => {
    if (!ref.current) return;
    ref.current.innerHTML = '';
    const el = document.createElement('i');
    el.setAttribute('data-lucide', name);
    ref.current.appendChild(el);
    lucide.createIcons({ nodes: [el] });
  }, [name, size]);
  return (
    <span ref={ref} style={{
      display: 'inline-flex', width: size, height: size,
      alignItems: 'center', justifyContent: 'center',
      flexShrink: 0, ...style
    }} />
  );
}

/* ── Sidebar tab list ── */
function SidebarTabs({ activeTab, setActiveTab, pendingCount }) {
  const tabs = [
    { id: 'pending', label: 'Pending Requests', icon: 'shield-check', badge: pendingCount > 0 ? pendingCount : null },
    { id: 'all',     label: 'All Requests',     icon: 'clock',        badge: null },
    { id: 'users',   label: 'All Users',         icon: 'users',        badge: null },
  ];

  return (
    <div className="mu-sidebar-tabs">
      <div className="mu-sidebar-tabs-label">Views</div>
      {tabs.map(tab => (
        <button
          key={tab.id}
          className={`mu-sidebar-tab${activeTab === tab.id ? ' active' : ''}`}
          onClick={() => setActiveTab(tab.id)}
        >
          <Icon name={tab.icon} size={14} />
          <span>{tab.label}</span>
          {tab.badge !== null && (
            <span className="mu-sidebar-tab-badge">{tab.badge}</span>
          )}
        </button>
      ))}
    </div>
  );
}

/* ── Status badge ── */
function StatusBadge({ status }) {
  if (status === 'Pending')  return <span className="mu-badge mu-badge-pending"><Icon name="clock" size={11} />{status}</span>;
  if (status === 'Approved') return <span className="mu-badge mu-badge-approved"><Icon name="check" size={11} />{status}</span>;
  if (status === 'Rejected') return <span className="mu-badge mu-badge-rejected"><Icon name="x" size={11} />{status}</span>;
  return <span className="mu-badge mu-badge-user">{status}</span>;
}

/* ── Role badge (display only) ── */
function RoleBadge({ role }) {
  const cls = role === 'Admin' ? 'mu-badge-admin' : role === 'Power User' ? 'mu-badge-power' : 'mu-badge-user';
  return <span className={`mu-badge ${cls}`}>{role}</span>;
}

/* ── Toast ── */
function Toast({ message, show, icon }) {
  return (
    <div className={`mu-toast${show ? ' show' : ''}`}>
      {icon && <Icon name={icon} size={13} />}
      {message}
    </div>
  );
}

/* ── Confirm dialog ── */
function ConfirmDialog({ title, body, confirmLabel, confirmClass, onConfirm, onCancel }) {
  return (
    <div className="mu-overlay" onClick={onCancel}>
      <div className="mu-dialog" onClick={e => e.stopPropagation()}>
        <div className="mu-dialog-title">{title}</div>
        <div className="mu-dialog-body">{body}</div>
        <div className="mu-dialog-actions">
          <button className="mu-btn mu-btn-ghost" onClick={onCancel}>Cancel</button>
          <button className={`mu-btn ${confirmClass}`} onClick={onConfirm}>{confirmLabel}</button>
        </div>
      </div>
    </div>
  );
}

/* ══════════════════════════════════════════════════
   TAB: ALL USERS
══════════════════════════════════════════════════ */
function AllUsersTab({ users, setUsers, showToast }) {
  const [confirm, setConfirm] = React.useState(null); // { userId, username }

  function handleRoleChange(id, newRole) {
    setUsers(prev => prev.map(u => u.id === id ? { ...u, role: newRole } : u));
    showToast('Role updated', 'check');
  }

  function handleRemoveClick(user) {
    setConfirm({ userId: user.id, username: user.username });
  }

  function handleRemoveConfirm() {
    setUsers(prev => prev.filter(u => u.id !== confirm.userId));
    showToast('User removed', 'trash-2');
    setConfirm(null);
  }

  return (
    <>
      <div className="mu-card">
        {/* Header */}
        <div className="mu-card-header">
          <div>
            <div className="mu-card-title-group">
              <span className="mu-card-icon"><Icon name="users" size={20} /></span>
              <h2 className="mu-card-title">All Users</h2>
            </div>
            <p className="mu-card-sub">Manage user access and roles</p>
          </div>
        </div>

        {users.length === 0 ? (
          <div className="mu-empty">
            <div className="mu-empty-icon"><Icon name="users" size={20} /></div>
            <div className="mu-empty-title">No users yet</div>
            <div className="mu-empty-sub">Approved users will appear here.</div>
          </div>
        ) : (
          <table className="mu-table">
            <thead className="mu-table-head">
              <tr>
                <th className="mu-col-user">Username</th>
                <th className="mu-col-role">Role</th>
                <th className="mu-col-actions">Actions</th>
              </tr>
            </thead>
            <tbody>
              {users.map(user => (
                <tr key={user.id} className="mu-row">
                  <td className="mu-col-user">
                    <span className="mu-username">{user.username}</span>
                  </td>
                  <td className="mu-col-role">
                    <RoleBadge role={user.role} />
                  </td>
                  <td className="mu-col-actions">
                    <div className="mu-actions-cell">
                      {user.isYou ? (
                        <span className="mu-you-label">You</span>
                      ) : (
                        <>
                          <select
                            className="mu-role-select"
                            value={user.role}
                            onChange={e => handleRoleChange(user.id, e.target.value)}
                          >
                            {ROLES.map(r => <option key={r} value={r}>{r}</option>)}
                          </select>
                          <button
                            className="mu-btn mu-btn-remove"
                            onClick={() => handleRemoveClick(user)}
                          >
                            <Icon name="trash-2" size={13} />
                            Remove
                          </button>
                        </>
                      )}
                    </div>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
      </div>

      {confirm && (
        <ConfirmDialog
          title="Remove user?"
          body={<>Are you sure you want to remove <strong style={{ fontFamily: 'var(--font-mono)', fontSize: 12 }}>{confirm.username}</strong>? They will lose all access immediately.</>}
          confirmLabel="Remove"
          confirmClass="mu-btn-remove"
          onConfirm={handleRemoveConfirm}
          onCancel={() => setConfirm(null)}
        />
      )}
    </>
  );
}

/* ══════════════════════════════════════════════════
   TAB: PENDING REQUESTS
══════════════════════════════════════════════════ */
function PendingRequestsTab({ requests, setRequests, setUsers, showToast }) {
  const pending = requests.filter(r => r.status === 'Pending');
  const [roleMap, setRoleMap] = React.useState(() => {
    const m = {};
    requests.forEach(r => { m[r.id] = r.role; });
    return m;
  });
  const [confirm, setConfirm] = React.useState(null); // { type: 'approve'|'reject', req }

  function handleRoleChange(id, val) {
    setRoleMap(prev => ({ ...prev, [id]: val }));
  }

  function handleApprove(req) {
    setConfirm({ type: 'approve', req });
  }

  function handleReject(req) {
    setConfirm({ type: 'reject', req });
  }

  function confirmAction() {
    const { type, req } = confirm;
    if (type === 'approve') {
      setRequests(prev => prev.map(r => r.id === req.id ? { ...r, status: 'Approved', role: roleMap[req.id] || r.role } : r));
      setUsers(prev => [...prev, { id: Date.now(), username: req.username, role: roleMap[req.id] || req.role, isYou: false }]);
      showToast('Request approved', 'check-circle-2');
    } else {
      setRequests(prev => prev.map(r => r.id === req.id ? { ...r, status: 'Rejected' } : r));
      showToast('Request rejected', 'x-circle');
    }
    setConfirm(null);
  }

  return (
    <>
      <div className="mu-card">
        <div className="mu-card-header">
          <div>
            <div className="mu-card-title-group">
              <span className="mu-card-icon"><Icon name="shield-check" size={20} /></span>
              <h2 className="mu-card-title">Pending Access Requests</h2>
            </div>
            <p className="mu-card-sub">
              {pending.length === 0
                ? 'No requests awaiting review'
                : `${pending.length} request${pending.length > 1 ? 's' : ''} awaiting review`}
            </p>
          </div>
        </div>

        {pending.length === 0 ? (
          <div className="mu-empty">
            <div className="mu-empty-icon"><Icon name="shield-check" size={20} /></div>
            <div className="mu-empty-title">All caught up</div>
            <div className="mu-empty-sub">No pending access requests at this time.</div>
          </div>
        ) : (
          <table className="mu-table">
            <thead className="mu-table-head">
              <tr>
                <th className="mu-col-user">Username</th>
                <th className="mu-col-date">Requested Date</th>
                <th className="mu-col-status">Status</th>
                <th className="mu-col-actions">Actions</th>
              </tr>
            </thead>
            <tbody>
              {pending.map(req => (
                <tr key={req.id} className="mu-row">
                  <td className="mu-col-user">
                    <span className="mu-username">{req.username}</span>
                  </td>
                  <td className="mu-col-date">
                    <span className="mu-date">{req.requestedDate}</span>
                  </td>
                  <td className="mu-col-status">
                    <StatusBadge status={req.status} />
                  </td>
                  <td className="mu-col-actions">
                    <div className="mu-actions-cell">
                      <select
                        className="mu-role-select"
                        value={roleMap[req.id] || req.role}
                        onChange={e => handleRoleChange(req.id, e.target.value)}
                      >
                        {ROLES.map(r => <option key={r} value={r}>{r}</option>)}
                      </select>
                      <button className="mu-btn mu-btn-approve" onClick={() => handleApprove(req)}>
                        <Icon name="check" size={13} />
                        Approve
                      </button>
                      <button className="mu-btn mu-btn-reject" onClick={() => handleReject(req)}>
                        <Icon name="x" size={13} />
                        Reject
                      </button>
                    </div>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
      </div>

      {confirm && (
        <ConfirmDialog
          title={confirm.type === 'approve' ? 'Approve request?' : 'Reject request?'}
          body={
            confirm.type === 'approve'
              ? <><strong style={{ fontFamily: 'var(--font-mono)', fontSize: 12 }}>{confirm.req.username}</strong> will be granted <strong>{roleMap[confirm.req.id] || confirm.req.role}</strong> access and added to All Users.</>
              : <>Reject the access request from <strong style={{ fontFamily: 'var(--font-mono)', fontSize: 12 }}>{confirm.req.username}</strong>? They will not be granted access.</>
          }
          confirmLabel={confirm.type === 'approve' ? 'Approve' : 'Reject'}
          confirmClass={confirm.type === 'approve' ? 'mu-btn-approve' : 'mu-btn-reject'}
          onConfirm={confirmAction}
          onCancel={() => setConfirm(null)}
        />
      )}
    </>
  );
}

/* ══════════════════════════════════════════════════
   TAB: ALL REQUESTS
══════════════════════════════════════════════════ */
function AllRequestsTab({ requests }) {
  const [filter, setFilter] = React.useState('All');
  const statuses = ['All', 'Pending', 'Approved', 'Rejected'];

  const filtered = filter === 'All' ? requests : requests.filter(r => r.status === filter);

  return (
    <div className="mu-card">
      <div className="mu-card-header">
        <div>
          <div className="mu-card-title-group">
            <span className="mu-card-icon"><Icon name="clock" size={20} /></span>
            <h2 className="mu-card-title">All Requests</h2>
          </div>
          <p className="mu-card-sub">Complete history of access requests</p>
        </div>
        {/* Filter pills */}
        <div style={{ display: 'flex', alignItems: 'center', gap: 4 }}>
          {statuses.map(s => (
            <button
              key={s}
              onClick={() => setFilter(s)}
              style={{
                height: 28, padding: '0 12px', borderRadius: 99,
                fontSize: 11.5, fontWeight: 600,
                border: filter === s ? '1px solid hsl(var(--ring))' : '1px solid hsl(var(--border))',
                background: filter === s ? 'var(--c-lotus-bg)' : 'transparent',
                color: filter === s ? 'var(--c-lotus-text)' : 'hsl(var(--muted-foreground))',
                cursor: 'pointer', transition: 'all 120ms', fontFamily: 'inherit',
              }}
            >
              {s}
            </button>
          ))}
        </div>
      </div>

      {filtered.length === 0 ? (
        <div className="mu-empty">
          <div className="mu-empty-icon"><Icon name="inbox" size={20} /></div>
          <div className="mu-empty-title">No requests</div>
          <div className="mu-empty-sub">No {filter.toLowerCase()} requests found.</div>
        </div>
      ) : (
        <table className="mu-table">
          <thead className="mu-table-head">
            <tr>
              <th className="mu-col-user">Username</th>
              <th className="mu-col-date">Requested Date</th>
              <th className="mu-col-role">Role</th>
              <th style={{ textAlign: 'left' }}>Status</th>
            </tr>
          </thead>
          <tbody>
            {filtered.map(req => (
              <tr key={req.id} className="mu-row">
                <td className="mu-col-user">
                  <span className="mu-username">{req.username}</span>
                </td>
                <td className="mu-col-date">
                  <span className="mu-date">{req.requestedDate}</span>
                </td>
                <td className="mu-col-role">
                  <RoleBadge role={req.role} />
                </td>
                <td>
                  <StatusBadge status={req.status} />
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </div>
  );
}

/* ══════════════════════════════════════════════════
   MAIN APP
══════════════════════════════════════════════════ */
function ManageUsersApp() {
  const [tweaks, setTweak] = useTweaks(MU_TWEAK_DEFAULTS);

  const [activeTab, setActiveTab] = React.useState('pending');
  const [users, setUsers] = React.useState(INITIAL_USERS);
  const [requests, setRequests] = React.useState(INITIAL_REQUESTS);

  const [toast, setToast] = React.useState({ show: false, message: '', icon: '' });
  const toastTimer = React.useRef(null);

  function showToast(message, icon = 'check') {
    clearTimeout(toastTimer.current);
    setToast({ show: true, message, icon });
    toastTimer.current = setTimeout(() => setToast(t => ({ ...t, show: false })), 2200);
  }

  const pendingCount = requests.filter(r => r.status === 'Pending').length;

  React.useEffect(() => {
    document.documentElement.setAttribute('data-theme', tweaks.theme);
  }, [tweaks.theme]);

  React.useEffect(() => { lucide.createIcons(); });

  return (
    <div className="mu-app">

      {/* ══ SIDEBAR ══ */}
      <BodhiSidebar section="settings">
        <SidebarTabs activeTab={activeTab} setActiveTab={setActiveTab} pendingCount={pendingCount} />
      </BodhiSidebar>

      {/* ══ MAIN ══ */}
      <main className="mu-main">

        {/* Topbar */}
        <div className="mu-topbar">
          <div className="mu-breadcrumb">
            <span>Bodhi</span>
            <i data-lucide="chevron-right" className="mu-bc-sep"></i>
            <span>Manage Users</span>
            <i data-lucide="chevron-right" className="mu-bc-sep"></i>
            <span className="mu-bc-curr">
              {activeTab === 'pending' ? 'Pending Requests' : activeTab === 'all' ? 'All Requests' : 'All Users'}
            </span>
          </div>
          <div className="mu-topbar-right">
            <button className="mu-icon-btn" title="View on GitHub">
              <svg viewBox="0 0 24 24" fill="currentColor" width="15" height="15"><path d="M12 2C6.477 2 2 6.484 2 12.017c0 4.425 2.865 8.18 6.839 9.504.5.092.682-.217.682-.483 0-.237-.008-.868-.013-1.703-2.782.605-3.369-1.343-3.369-1.343-.454-1.158-1.11-1.466-1.11-1.466-.908-.62.069-.608.069-.608 1.003.07 1.531 1.032 1.531 1.032.892 1.53 2.341 1.088 2.91.832.092-.647.35-1.088.636-1.338-2.22-.253-4.555-1.113-4.555-4.951 0-1.093.39-1.988 1.029-2.688-.103-.253-.446-1.272.098-2.65 0 0 .84-.27 2.75 1.026A9.564 9.564 0 0112 6.844c.85.004 1.705.115 2.504.337 1.909-1.296 2.747-1.027 2.747-1.027.546 1.379.202 2.398.1 2.651.64.7 1.028 1.595 1.028 2.688 0 3.848-2.339 4.695-4.566 4.943.359.309.678.92.678 1.855 0 1.338-.012 2.419-.012 2.747 0 .268.18.58.688.482A10.019 10.019 0 0022 12.017C22 6.484 17.522 2 12 2z"/></svg>
            </button>
          </div>
        </div>

        {/* Scroll area */}
        <div className="mu-scroll">
          <div style={{ maxWidth: 900, margin: '0 auto', paddingTop: 24 }}>
            {activeTab === 'pending' && (
              <PendingRequestsTab
                requests={requests}
                setRequests={setRequests}
                setUsers={setUsers}
                showToast={showToast}
              />
            )}
            {activeTab === 'all' && (
              <AllRequestsTab requests={requests} />
            )}
            {activeTab === 'users' && (
              <AllUsersTab
                users={users}
                setUsers={setUsers}
                showToast={showToast}
              />
            )}
          </div>
        </div>
      </main>

      {/* ══ TWEAKS ══ */}
      <TweaksPanel>
        <TweakSection title="Theme">
          <TweakRadio
            value={tweaks.theme}
            options={[{ label: 'Light', value: 'light' }, { label: 'Dark', value: 'dark' }]}
            onChange={v => setTweak('theme', v)}
          />
        </TweakSection>
      </TweaksPanel>

      {/* Toast */}
      <Toast show={toast.show} message={toast.message} icon={toast.icon} />
    </div>
  );
}

const muRoot = ReactDOM.createRoot(document.getElementById('root'));
muRoot.render(<ManageUsersApp />);
