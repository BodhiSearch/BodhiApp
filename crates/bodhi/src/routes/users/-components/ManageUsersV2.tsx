import { useCallback, useMemo, useState } from 'react';

import {
  ShellFilterTabs,
  ShellIcon,
  ShellPagination,
  useCollapsibleSearch,
  useListKeyNav,
  useShellChrome,
} from '@/components/shell';
import '@/components/shell/api-keys.css';
import '@/components/shell/list.css';
import '@/components/shell/manage-users.css';
import { Skeleton } from '@/components/ui/skeleton';
import { useGetAuthenticatedUser, useListUsers } from '@/hooks/users';
import { useViewTransition } from '@/hooks/useViewTransition';
import { ROLE_OPTIONS } from '@/lib/roles';

import { InviteLinkAction } from './InviteLinkAction';
import { UserRailHeader } from './UserRailHeader';
import { UserRailPanel } from './UserRailPanel';
import { UserRow } from './UserRow';
import { isSelf } from './usersUtils';

const USERS_BREADCRUMB = [
  { label: 'Bodhi' },
  { label: 'Users', href: '/users/access-requests/' },
  { label: 'Manage Users', current: true },
];

type RoleFilter = 'all' | string;

function ManageUsersContent() {
  useListKeyNav();
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
            loading={isLoading}
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
        {total > pageSize && <ShellPagination minimal total={total} page={page} onPage={setPage} pageSize={pageSize} />}
      </div>
    </div>
  );
}

export function ManageUsersV2() {
  return <ManageUsersContent />;
}
