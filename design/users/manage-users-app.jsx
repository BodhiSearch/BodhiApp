/* ═══════════════════════════════════════════════════
   MANAGE USERS — Settings page (on AppShell)
   Sidebar views: Access Requests (Pending zone + History zone) · All Users
   manage-users-app.jsx  (load after bodhi-app-shell.jsx + bodhi-list.jsx + tweaks-panel.jsx)
═══════════════════════════════════════════════════ */

const MU_TWEAK_DEFAULTS = /*EDITMODE-BEGIN*/{
  "tenancy": "multi"
}/*EDITMODE-END*/;

const MU_TENANTS = [
  { id: 'acme',      name: 'Acme Corp',         role: 'Admin',      plan: 'Enterprise' },
  { id: 'northwind', name: 'Northwind Trading', role: 'Power User', plan: 'Team' },
  { id: 'initech',   name: 'Initech Labs',      role: 'User',       plan: 'Free' },
];

const Ic = ShellIcon;

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

const TABS = [
  { id: 'requests', label: 'Access Requests', icon: 'shield-check' },
  { id: 'users',    label: 'All Users',       icon: 'users' },
];

/* ── Sidebar view list (collapse-aware) ── */
function ManageUsersSidebar({ activeTab, setActiveTab, pendingCount }) {
  const { collapsed } = useShell();
  const tabs = TABS.map(t => ({ ...t, badge: t.id === 'requests' && pendingCount > 0 ? pendingCount : null }));

  if (collapsed) {
    return (
      <>
        {tabs.map(tab => (
          <button key={tab.id}
            className={`shell-railbtn shell-tip${activeTab === tab.id ? ' on' : ''}`}
            data-tip={tab.label}
            onClick={() => setActiveTab(tab.id)}>
            <Ic name={tab.icon} size={18} />
            {tab.badge !== null && <span className="rb-badge">{tab.badge}</span>}
          </button>
        ))}
      </>
    );
  }

  return (
    <div className="mu-sidebar-tabs">
      <div className="mu-sidebar-tabs-label">Views</div>
      {tabs.map(tab => (
        <button
          key={tab.id}
          className={`mu-sidebar-tab${activeTab === tab.id ? ' active' : ''}`}
          onClick={() => setActiveTab(tab.id)}
        >
          <Ic name={tab.icon} size={14} />
          <span>{tab.label}</span>
          {tab.badge !== null && <span className="mu-sidebar-tab-badge">{tab.badge}</span>}
        </button>
      ))}
    </div>
  );
}

/* ── Badges ── */
function StatusBadge({ status }) {
  if (status === 'Pending')  return <span className="mu-badge mu-badge-pending"><Ic name="clock" size={11} />{status}</span>;
  if (status === 'Approved') return <span className="mu-badge mu-badge-approved"><Ic name="check" size={11} />{status}</span>;
  if (status === 'Rejected') return <span className="mu-badge mu-badge-rejected"><Ic name="x" size={11} />{status}</span>;
  return <span className="mu-badge mu-badge-user">{status}</span>;
}
function RoleBadge({ role }) {
  const cls = role === 'Admin' ? 'mu-badge-admin' : role === 'Power User' ? 'mu-badge-power' : 'mu-badge-user';
  return <span className={`mu-badge ${cls}`}>{role}</span>;
}

/* ── Toast ── */
function Toast({ message, show, icon }) {
  return (
    <div className={`mu-toast${show ? ' show' : ''}`}>
      {icon && <Ic name={icon} size={13} />}
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
   VIEW: ACCESS REQUESTS  (Pending zone + History zone)
══════════════════════════════════════════════════ */
function AccessRequestsView({ requests, setRequests, setUsers, showToast }) {
  const [search, setSearch] = React.useState('');
  const [roleMap, setRoleMap] = React.useState(() => {
    const m = {}; requests.forEach(r => { m[r.id] = r.role; }); return m;
  });
  const [confirm, setConfirm] = React.useState(null);

  const q = search.trim().toLowerCase();
  const match = r => !q || r.username.toLowerCase().includes(q);
  const pending = requests.filter(r => r.status === 'Pending' && match(r));
  const history = requests.filter(r => r.status !== 'Pending' && match(r));

  const setRole = (id, val) => setRoleMap(prev => ({ ...prev, [id]: val }));

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
    <div className="l-page">
      <ListToolbar search={search} onSearch={setSearch} searchPlaceholder="Search requests by username…" />
      <div className="l-scroll">

        <div className="l-sectionhead">
          <span className="l-sectionhead-t">Pending</span>
          <span className="l-sectionhead-n warn">{pending.length}</span>
          <span className="l-sectionhead-sub">awaiting your review</span>
        </div>
        {pending.length === 0 ? (
          <div className="mu-zone-empty"><Ic name="shield-check" size={15} /> {q ? 'No matching pending requests.' : 'All caught up — no pending requests.'}</div>
        ) : (
          <ListView head={
            <>
              <div className="mu-user l-lh">Username</div>
              <div className="mu-date l-lh">Requested</div>
              <div className="mu-act l-lh mu-act-lh">Actions</div>
            </>
          }>
            {pending.map(req => (
              <div className="l-listrow mu-listrow" key={req.id}>
                <div className="mu-user"><span className="mu-username">{req.username}</span></div>
                <div className="mu-date"><span className="mu-date-val">{req.requestedDate}</span></div>
                <div className="mu-act">
                  <select className="mu-role-select" value={roleMap[req.id] || req.role} onChange={e => setRole(req.id, e.target.value)}>
                    {ROLES.map(r => <option key={r} value={r}>{r}</option>)}
                  </select>
                  <button className="mu-btn mu-btn-approve" onClick={() => setConfirm({ type: 'approve', req })}><Ic name="check" size={13} /> Approve</button>
                  <button className="mu-btn mu-btn-reject" onClick={() => setConfirm({ type: 'reject', req })}><Ic name="x" size={13} /> Reject</button>
                </div>
              </div>
            ))}
          </ListView>
        )}

        <div className="l-sectionhead">
          <span className="l-sectionhead-t">History</span>
          <span className="l-sectionhead-n">{history.length}</span>
          <span className="l-sectionhead-sub">decided requests</span>
        </div>
        {history.length === 0 ? (
          <div className="mu-zone-empty"><Ic name="inbox" size={15} /> {q ? 'No matching history.' : 'No decided requests yet.'}</div>
        ) : (
          <ListView head={
            <>
              <div className="mu-user l-lh">Username</div>
              <div className="mu-date l-lh">Requested</div>
              <div className="mu-role l-lh">Role</div>
              <div className="mu-status l-lh">Status</div>
            </>
          }>
            {history.map(req => (
              <div className="l-listrow mu-listrow" key={req.id}>
                <div className="mu-user"><span className="mu-username">{req.username}</span></div>
                <div className="mu-date"><span className="mu-date-val">{req.requestedDate}</span></div>
                <div className="mu-role"><RoleBadge role={req.role} /></div>
                <div className="mu-status"><StatusBadge status={req.status} /></div>
              </div>
            ))}
          </ListView>
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
    </div>
  );
}

/* ══════════════════════════════════════════════════
   VIEW: ALL USERS  (list only — details/actions in the rail)
══════════════════════════════════════════════════ */
function AllUsersView({ users, search, setSearch, selId, onSelect }) {
  const { openRail } = useShell();
  useListKeyNav();
  const select = id => { onSelect(id); openRail(); };
  const [cat, setCat] = React.useState('all');

  const q = search.trim().toLowerCase();
  const byRole = role => users.filter(u => u.role === role).length;
  const counts = { all: users.length, Admin: byRole('Admin'), 'Power User': byRole('Power User'), User: byRole('User') };
  const visible = users.filter(u =>
    (cat === 'all' || u.role === cat) && (!q || u.username.toLowerCase().includes(q)));

  return (
    <div className="l-page">
      <ListToolbar
        categories={[
          { id: 'all',        label: 'All',        badge: counts.all },
          { id: 'Admin',      label: 'Admin',      badge: counts.Admin },
          { id: 'Power User', label: 'Power User', badge: counts['Power User'] },
          { id: 'User',       label: 'User',       badge: counts.User },
        ]}
        category={cat} onCategory={setCat}
        search={search} onSearch={setSearch} searchPlaceholder="Search users by username…" />
      <div className="l-scroll">
        {visible.length === 0 ? (
          <div className="l-empty">
            <Ic name="users" size={30} />
            <div className="l-empty-t">{q ? 'No users match your search' : 'No users yet'}</div>
            <div className="l-empty-s">{q ? 'Try a different search term.' : 'Approved users will appear here.'}</div>
          </div>
        ) : (
          <ListView head={
            <>
              <div className="mu-user l-lh">Username</div>
              <div className="mu-role l-lh">Role</div>
            </>
          }>
            {visible.map(user => (
              <ListRow className="mu-listrow" key={user.id} active={user.id === selId}
                       onSelect={() => select(user.id)} label={`Open user ${user.username}`}>
                <div className="mu-user"><span className="mu-username">{user.username}</span></div>
                <div className="mu-role">
                  <RoleBadge role={user.role} />
                  {user.isYou && <span className="mu-you-label" style={{ marginLeft: 8 }}>You</span>}
                </div>
              </ListRow>
            ))}
          </ListView>
        )}
      </div>
    </div>
  );
}

function userInitial(username) { return (username || '?').trim().charAt(0).toUpperCase(); }

/* ── Rail header (railHeader slot) ── */
function UserDetailHeader({ user, onClose }) {
  return (
    <div className="dp-head">
      <div className="dp-head-icon" style={{ background: 'hsl(var(--accent))', color: 'hsl(var(--accent-foreground))' }}>{userInitial(user.username)}</div>
      <div className="dp-head-body">
        <div className="dp-head-title mono">{user.username}</div>
        <div className="dp-head-sub">{user.role}{user.isYou ? ' · You' : ''}</div>
      </div>
      <button className="dp-close" onClick={onClose} title="Close"><Ic name="x" size={15} /></button>
    </div>
  );
}

/* ── Rail body (rail slot) — role form + remove ── */
function UserDetailPanel({ user, onSave, onRemove }) {
  const [draftRole, setDraftRole] = React.useState(user ? user.role : 'User');
  const [confirm, setConfirm] = React.useState(false);
  React.useEffect(() => { setDraftRole(user ? user.role : 'User'); setConfirm(false); }, [user && user.id]);

  const dirty = draftRole !== user.role;

  return (
    <div className="dp-panel">
      <div className="dp-status-row">
        <RoleBadge role={user.role} />
        {user.isYou && <span className="mu-you-label" style={{ marginLeft: 'auto' }}>This is you</span>}
      </div>

      <div className="dp-body">
        <div className="dp-section">
          <div className="dp-sec-lbl">Account</div>
          <div className="dp-rows">
            <div className="dp-row"><span className="dp-row-k"><Ic name="at-sign" size={13} /> Username</span><span className="dp-row-v mono">{user.username}</span></div>
            <div className="dp-row"><span className="dp-row-k"><Ic name="shield" size={13} /> Current role</span><span className="dp-row-v">{user.role}</span></div>
          </div>
        </div>

        {user.isYou ? (
          <div className="dp-section">
            <div className="dp-sec-lbl">Role</div>
            <div className="dp-field-hint">You can't change your own role or remove your own account. Ask another admin if you need changes.</div>
          </div>
        ) : (
          <div className="dp-section">
            <div className="dp-sec-lbl">Change role</div>
            <div className="dp-field">
              <select className="mu-role-select" style={{ width: '100%' }} value={draftRole} onChange={e => setDraftRole(e.target.value)}>
                {ROLES.map(r => <option key={r} value={r}>{r}</option>)}
              </select>
              <span className="dp-field-hint">Updating the role changes what this user can access across Bodhi.</span>
            </div>
          </div>
        )}
      </div>

      {!user.isYou && (
        <div className="dp-foot">
          <button className="dp-btn dp-btn-accent" disabled={!dirty} onClick={() => onSave(user.id, draftRole)}>
            <Ic name="check" size={14} /> {dirty ? 'Save changes' : 'Saved'}
          </button>
          {confirm ? (
            <button className="dp-btn dp-btn-danger" style={{ borderColor: 'hsl(var(--destructive))', background: 'rgba(220,38,38,.05)', color: 'hsl(var(--destructive))' }}
                    onClick={() => onRemove(user.id)}><Ic name="trash-2" size={14} /> Confirm remove</button>
          ) : (
            <button className="dp-btn dp-btn-danger" onClick={() => setConfirm(true)}><Ic name="trash-2" size={14} /> Remove user</button>
          )}
          {confirm && <div className="dp-field-hint" style={{ textAlign: 'center' }}>They'll lose all access immediately. Click again to confirm.</div>}
        </div>
      )}
    </div>
  );
}

/* ── GitHub header action ── */
function GitHubBtn() {
  return (
    <button className="mu-icon-btn" title="View on GitHub">
      <svg viewBox="0 0 24 24" fill="currentColor" width="15" height="15"><path d="M12 2C6.477 2 2 6.484 2 12.017c0 4.425 2.865 8.18 6.839 9.504.5.092.682-.217.682-.483 0-.237-.008-.868-.013-1.703-2.782.605-3.369-1.343-3.369-1.343-.454-1.158-1.11-1.466-1.11-1.466-.908-.62.069-.608.069-.608 1.003.07 1.531 1.032 1.531 1.032.892 1.53 2.341 1.088 2.91.832.092-.647.35-1.088.636-1.338-2.22-.253-4.555-1.113-4.555-4.951 0-1.093.39-1.988 1.029-2.688-.103-.253-.446-1.272.098-2.65 0 0 .84-.27 2.75 1.026A9.564 9.564 0 0112 6.844c.85.004 1.705.115 2.504.337 1.909-1.296 2.747-1.027 2.747-1.027.546 1.379.202 2.398.1 2.651.64.7 1.028 1.595 1.028 2.688 0 3.848-2.339 4.695-4.566 4.943.359.309.678.92.678 1.855 0 1.338-.012 2.419-.012 2.747 0 .268.18.58.688.482A10.019 10.019 0 0022 12.017C22 6.484 17.522 2 12 2z"/></svg>
    </button>
  );
}

/* ══════════════════════════════════════════════════
   MAIN APP
══════════════════════════════════════════════════ */
function ManageUsersApp() {
  const [tweaks, setTweak] = useTweaks(MU_TWEAK_DEFAULTS);

  const [users, setUsers] = React.useState(INITIAL_USERS);
  const [userSearch, setUserSearch] = React.useState('');
  const [selUserId, setSelUserId] = React.useState(null);

  const [toast, setToast] = React.useState({ show: false, message: '', icon: '' });
  const toastTimer = React.useRef(null);

  function showToast(message, icon = 'check') {
    clearTimeout(toastTimer.current);
    setToast({ show: true, message, icon });
    toastTimer.current = setTimeout(() => setToast(t => ({ ...t, show: false })), 2200);
  }

  function handleRoleSave(id, newRole) {
    setUsers(prev => prev.map(u => u.id === id ? { ...u, role: newRole } : u));
    showToast('Role updated', 'check');
  }
  function handleRemoveUser(id) {
    setUsers(prev => prev.filter(u => u.id !== id));
    setSelUserId(s => s === id ? null : s);
    showToast('User removed', 'trash-2');
  }

  const selUser = users.find(u => u.id === selUserId) || null;

  return (
    <>
      <AppShell
        section="users" subPage="all-users" resizeKey="users"
        user={{
          initials: 'YO', name: 'Yogesh', email: 'yogesh@email.com', role: 'Admin',
          multiTenant: tweaks.tenancy === 'multi',
          tenants: MU_TENANTS, currentTenantId: 'acme',
        }}
        contentClass="flush" mainScroll={false} railScroll={false}
        breadcrumb={[
          { label: 'Bodhi', href: 'Bodhi Chat.html' },
          { label: 'Users', href: 'User Access Requests.html' },
          { label: 'All Users', current: true },
        ]}
        headerActions={<GitHubBtn />}
        rail={selUser ? <UserDetailPanel user={selUser} onSave={handleRoleSave} onRemove={handleRemoveUser} /> : undefined}
        railHeader={selUser ? <UserDetailHeader user={selUser} onClose={() => setSelUserId(null)} /> : undefined}
      >
        <AllUsersView users={users} search={userSearch} setSearch={setUserSearch} selId={selUserId} onSelect={setSelUserId} />
      </AppShell>

      <TweaksPanel>
        <TweakSection title="Server mode">
          <TweakRadio
            value={tweaks.tenancy}
            options={[{ label: 'Single-tenant', value: 'single' }, { label: 'Multi-tenant', value: 'multi' }]}
            onChange={v => setTweak('tenancy', v)}
          />
          <div style={{ fontSize: 11, color: 'hsl(var(--muted-foreground))', marginTop: 8, lineHeight: 1.5 }}>
            Multi-tenant lets the same account belong to several organizations. Open the user menu (bottom-left) to switch tenant.
          </div>
        </TweakSection>
      </TweaksPanel>

      <Toast show={toast.show} message={toast.message} icon={toast.icon} />
    </>
  );
}

const muRoot = ReactDOM.createRoot(document.getElementById('root'));
muRoot.render(<ManageUsersApp />);
