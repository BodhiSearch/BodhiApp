import { useCallback, useMemo, useState } from 'react';

import { UserInfo } from '@bodhiapp/ts-client';

import { Pagination } from '@/components/DataTable';
import { ShellFilterTabs, ShellIcon, useCollapsibleSearch, useShellChrome } from '@/components/shell';
import '@/components/shell/api-keys.css';
import '@/components/shell/list.css';
import '@/components/shell/manage-users.css';
import { Badge } from '@/components/ui/badge';
import { Input } from '@/components/ui/input';
import { Skeleton } from '@/components/ui/skeleton';
import { useGetAppInfo } from '@/hooks/info';
import { toast } from '@/hooks/use-toast';
import { useToastMessages } from '@/hooks/use-toast-messages';
import {
  AuthenticatedUser,
  useChangeUserRole,
  useGetAuthenticatedUser,
  useListUsers,
  useRemoveUser,
} from '@/hooks/users';
import { useViewTransition } from '@/hooks/useViewTransition';
import { copyToClipboard, getClipboardUnavailableMessage } from '@/lib/clipboard';
import { getAvailableRoles, getRoleBadgeVariant, getRoleLabel, getRoleLevel, ROLE_OPTIONS } from '@/lib/roles';

const USERS_BREADCRUMB = [
  { label: 'Bodhi' },
  { label: 'Users', href: '/users/access-requests/' },
  { label: 'Manage Users', current: true },
];

type RoleFilter = 'all' | string;

const AVATAR_COLORS = ['#3E4AA8', '#0F6F67', '#B02A52', '#2F7D1F', '#5E6AD2', '#9A5B12'];
function avatarColor(seed: string) {
  let h = 0;
  for (let i = 0; i < seed.length; i++) h = (h * 31 + seed.charCodeAt(i)) >>> 0;
  return AVATAR_COLORS[h % AVATAR_COLORS.length];
}
function initials(name: string) {
  const local = (name.split('@')[0] || name).replace(/[^a-zA-Z0-9]/g, '');
  return (local.slice(0, 2) || name.slice(0, 2)).toUpperCase();
}

function isSelf(user: UserInfo, me?: AuthenticatedUser) {
  return me?.auth_status === 'logged_in' && user.user_id === me.user_id;
}
/** Mirror of UserActionsCell: self can't be modified; target must not outrank the actor. */
function canModify(user: UserInfo, me: AuthenticatedUser | undefined, myRole: string) {
  if (isSelf(user, me)) return false;
  return getRoleLevel(user.role ?? '') <= getRoleLevel(myRole);
}

interface UserRowProps {
  user: UserInfo;
  active: boolean;
  self: boolean;
  onSelect: () => void;
}

function UserRow({ user, active, self, onSelect }: UserRowProps) {
  return (
    <div
      className={`l-listrow mu-row${active ? ' active' : ''}`}
      onClick={onSelect}
      data-testid={`user-row-${user.username}`}
    >
      <div className="mu-icon">
        <div className="mu-avatar" style={{ background: avatarColor(user.username) }}>
          {initials(user.username)}
        </div>
      </div>
      <div className="mu-id">
        <div className="mu-username" data-testid="user-username">
          <span data-testid={`user-username-${user.username}`}>{user.username}</span>
        </div>
      </div>
      <div className="mu-role-cell" data-testid={`user-role-${user.username}`}>
        <Badge variant={getRoleBadgeVariant(user.role ?? '')} data-testid="user-role">
          {getRoleLabel(user.role ?? '')}
        </Badge>
        {self && <span className="mu-you-label">You</span>}
      </div>
    </div>
  );
}

function UserRailHeader({ user, self, onClose }: { user: UserInfo; self: boolean; onClose: () => void }) {
  return (
    <div className="dp-head">
      <div className="dp-head-icon" style={{ background: avatarColor(user.username) }}>
        {initials(user.username)}
      </div>
      <div className="dp-head-body">
        <div className="dp-head-title mono">{user.username}</div>
        <div className="dp-head-sub">
          {getRoleLabel(user.role ?? '')}
          {self ? ' · You' : ''}
        </div>
      </div>
      <button className="dp-close" onClick={onClose} title="Close" data-testid="user-detail-close">
        <ShellIcon name="x" size={15} />
      </button>
    </div>
  );
}

interface UserRailPanelProps {
  user: UserInfo;
  me?: AuthenticatedUser;
  myRole: string;
}

function UserRailPanel({ user, me, myRole }: UserRailPanelProps) {
  const self = isSelf(user, me);
  const modifiable = canModify(user, me, myRole);
  const availableRoles = useMemo(() => getAvailableRoles(myRole), [myRole]);
  const { showSuccess, showError } = useToastMessages();

  const [draftRole, setDraftRole] = useState(user.role ?? '');
  const [confirmRemove, setConfirmRemove] = useState(false);

  // Reset when a different user is selected.
  const [trackedId, setTrackedId] = useState(user.user_id);
  if (trackedId !== user.user_id) {
    setTrackedId(user.user_id);
    setDraftRole(user.role ?? '');
    setConfirmRemove(false);
  }

  const changeRole = useChangeUserRole({
    onSuccess: () => showSuccess('Role updated', `${user.username} is now ${getRoleLabel(draftRole)}`),
    onError: (message) => showError('Error', message),
  });
  const removeUser = useRemoveUser({
    onSuccess: () => showSuccess('User removed', `${user.username} was removed`),
    onError: (message) => showError('Error', message),
  });

  const dirty = draftRole !== (user.role ?? '');
  const busy = changeRole.isPending || removeUser.isPending;

  return (
    <div className="dp-panel manage-users-rail" data-testid={`user-detail-${user.username}`}>
      <div className="dp-status-row">
        <Badge variant={getRoleBadgeVariant(user.role ?? '')}>{getRoleLabel(user.role ?? '')}</Badge>
        {self && (
          <span className="mu-you-label" style={{ marginLeft: 'auto' }}>
            This is you
          </span>
        )}
      </div>

      <div className="dp-body">
        <div className="dp-section">
          <div className="dp-sec-lbl">Account</div>
          <div className="dp-rows">
            <div className="dp-row">
              <span className="dp-row-k">
                <ShellIcon name="at-sign" size={13} /> Username
              </span>
              <span className="dp-row-v mono">{user.username}</span>
            </div>
            <div className="dp-row">
              <span className="dp-row-k">
                <ShellIcon name="shield" size={13} /> Current role
              </span>
              <span className="dp-row-v">{getRoleLabel(user.role ?? '')}</span>
            </div>
          </div>
        </div>

        {modifiable ? (
          <div className="dp-section">
            <div className="dp-sec-lbl">Change role</div>
            <div className="dp-field">
              <select
                className="dp-role-select"
                value={draftRole}
                onChange={(e) => setDraftRole(e.target.value)}
                data-testid={`role-select-${user.username}`}
              >
                {availableRoles.map((r) => (
                  <option key={r.value} value={r.value}>
                    {r.label}
                  </option>
                ))}
              </select>
              <span className="dp-field-hint">Updating the role changes what this user can access across Bodhi.</span>
            </div>
          </div>
        ) : (
          <div className="dp-section">
            <div className="dp-readonly-note" data-testid={`no-actions-${user.username}`}>
              <ShellIcon name={self ? 'user' : 'lock'} size={14} />
              <span data-testid={self ? 'current-user-indicator' : 'restricted-user-indicator'}>
                {self
                  ? "You can't change your own role or remove your own account. Ask another admin if you need changes."
                  : "This user outranks you, so you can't change their role or remove them."}
              </span>
            </div>
          </div>
        )}
      </div>

      {modifiable && (
        <div className="dp-foot">
          <button
            className="dp-btn dp-btn-accent"
            disabled={!dirty || busy}
            onClick={() => changeRole.mutate({ userId: user.user_id, newRole: draftRole })}
            data-testid={`save-role-${user.username}`}
          >
            <ShellIcon name="check" size={14} /> {dirty ? 'Save changes' : 'Saved'}
          </button>
          {confirmRemove ? (
            <button
              className="dp-btn dp-btn-danger"
              disabled={busy}
              onClick={() => removeUser.mutate(user.user_id)}
              data-testid={`remove-user-btn-${user.username}`}
            >
              <ShellIcon name="trash-2" size={14} /> Confirm remove
            </button>
          ) : (
            <button
              className="dp-btn dp-btn-danger"
              onClick={() => setConfirmRemove(true)}
              data-testid={`remove-user-btn-${user.username}`}
            >
              <ShellIcon name="trash-2" size={14} /> Remove user
            </button>
          )}
          {confirmRemove && (
            <div className="dp-field-hint" style={{ textAlign: 'center' }}>
              They&apos;ll lose all access immediately. Click again to confirm.
            </div>
          )}
        </div>
      )}
    </div>
  );
}

/** Invite-link header action — multi-tenant deployments only (gated on AppInfo.deployment). */
function InviteLinkAction() {
  const { data: appInfo } = useGetAppInfo();
  const [open, setOpen] = useState(false);
  const [copied, setCopied] = useState(false);

  if (appInfo?.deployment !== 'multi_tenant') {
    return null;
  }

  const inviteUrl = `${appInfo.url}/ui/login/?invite=${appInfo.client_id}`;

  const handleCopy = async () => {
    try {
      await copyToClipboard(inviteUrl);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch {
      setCopied(false);
      toast({
        title: 'Copy Failed',
        description: getClipboardUnavailableMessage() ?? 'Failed to copy invite URL',
        variant: 'destructive',
      });
    }
  };

  return (
    <div style={{ position: 'relative' }}>
      <button className="l-iconbtn" title="Invite users" onClick={() => setOpen((o) => !o)} data-testid="invite-toggle">
        <ShellIcon name="user-plus" size={15} />
      </button>
      {open && (
        <div
          style={{
            position: 'absolute',
            top: 'calc(100% + 6px)',
            right: 0,
            zIndex: 50,
            background: 'hsl(var(--popover))',
            border: '1px solid hsl(var(--border))',
            borderRadius: 10,
            padding: 12,
            boxShadow: 'var(--shadow-md)',
          }}
        >
          <div className="mu-invite-pop">
            <div className="mu-invite-pop-label">Invite link</div>
            <div className="mu-invite-pop-row">
              <Input readOnly value={inviteUrl} data-testid="invite-url-input" className="text-sm" />
              <button
                className="l-iconbtn"
                onClick={handleCopy}
                title="Copy invite link"
                data-testid="invite-copy-button"
              >
                <ShellIcon name={copied ? 'check' : 'copy'} size={14} />
              </button>
            </div>
            <div className="mu-invite-hint">Share this link to invite users to your organization.</div>
          </div>
        </div>
      )}
    </div>
  );
}

function ManageUsersContent() {
  const [page, setPage] = useState(1);
  const [pageSize] = useState(10);
  const [filter, setFilter] = useState<RoleFilter>('all');
  const [search, setSearch] = useState('');
  const [selectedId, setSelectedId] = useState<string | null>(null);

  const { data: me } = useGetAuthenticatedUser();
  const { data: usersData, isLoading } = useListUsers(page, pageSize);

  const myRole = typeof me?.role === 'string' ? me.role : '';

  const withViewTransition = useViewTransition();
  const select = useCallback((id: string | null) => withViewTransition(() => setSelectedId(id)), [withViewTransition]);

  const users = useMemo(() => usersData?.users ?? [], [usersData]);
  const total = usersData?.total_users ?? 0;

  // Per-page role counts (the list is server-paginated; counts reflect the current page only).
  const counts = useMemo(() => {
    const c: Record<string, number> = { all: users.length };
    ROLE_OPTIONS.forEach((r) => (c[r.value] = 0));
    for (const u of users) if (u.role) c[u.role] = (c[u.role] ?? 0) + 1;
    return c;
  }, [users]);

  const filterTabs = useMemo(
    () => [
      { id: 'all', label: 'All', count: counts.all },
      ...ROLE_OPTIONS.map((r) => ({ id: r.value, label: r.label, count: counts[r.value] })),
    ],
    [counts]
  );

  const searchNode = useCollapsibleSearch({
    value: search,
    onChange: setSearch,
    placeholder: 'Search users by username…',
    toggleTestId: 'users-search-toggle',
    closeTestId: 'users-search-close',
  });

  const visible = useMemo(() => {
    const q = search.trim().toLowerCase();
    return users.filter((u) => {
      if (filter !== 'all' && u.role !== filter) return false;
      if (!q) return true;
      return u.username.toLowerCase().includes(q);
    });
  }, [users, filter, search]);

  const selected = useMemo(() => users.find((u) => u.user_id === selectedId) ?? null, [users, selectedId]);

  const headerActions = useMemo(() => <InviteLinkAction />, []);
  const railHeader = useMemo(
    () =>
      selected ? <UserRailHeader user={selected} self={isSelf(selected, me)} onClose={() => select(null)} /> : null,
    [selected, me, select]
  );
  const rail = useMemo(
    () => (selected ? <UserRailPanel user={selected} me={me} myRole={myRole} /> : null),
    [selected, me, myRole]
  );

  useShellChrome({ breadcrumb: USERS_BREADCRUMB, headerActions, rail, railHeader, railDefaultOpen: false });

  return (
    <div
      className="manage-users-screen l-page"
      data-testid="users-page"
      data-pagestatus={isLoading ? 'loading' : 'ready'}
    >
      <div className="l-controls">
        {searchNode.row}
        <div className="l-toolbar">
          <ShellFilterTabs
            tabs={filterTabs}
            value={filter}
            onChange={setFilter}
            label="Filter users by role"
            testIdPrefix="users-filter"
          />
          <div className="l-tb-actions">{searchNode.toggle}</div>
        </div>
      </div>

      <div className="l-scroll" data-testid="users-table">
        {isLoading ? (
          <div style={{ padding: 16 }} data-testid="loading-skeleton">
            {Array.from({ length: 5 }).map((_, i) => (
              <Skeleton key={i} className="h-12 w-full mb-3" />
            ))}
          </div>
        ) : visible.length === 0 ? (
          <div className="empty-state" data-testid="no-users">
            <div className="empty-icon">
              <ShellIcon name="users" size={28} />
            </div>
            <div className="empty-title">No users</div>
            <div className="empty-sub">{search ? 'No users match your search.' : 'No users match this filter.'}</div>
          </div>
        ) : (
          <div className="l-listview">
            <div className="l-listhead">
              <div className="mu-icon" />
              <div className="l-lh mu-id">Username</div>
              <div className="l-lh mu-role-cell">Role</div>
            </div>
            {visible.map((user) => (
              <UserRow
                key={user.user_id}
                user={user}
                active={user.user_id === selectedId}
                self={isSelf(user, me)}
                onSelect={() => select(user.user_id)}
              />
            ))}
          </div>
        )}
        {total > pageSize && (
          <div style={{ padding: '14px 16px' }} data-testid="pagination">
            <Pagination page={page} totalPages={Math.ceil(total / pageSize)} onPageChange={setPage} />
          </div>
        )}
      </div>
    </div>
  );
}

export function ManageUsersV2() {
  return <ManageUsersContent />;
}
