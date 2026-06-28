import { useMemo } from 'react';

import { createFileRoute } from '@tanstack/react-router';

import AppInitializer from '@/components/AppInitializer';
import { EmptyState } from '@/components/EmptyState';
import { ShellFilterTabs, ShellIcon, ShellPagination, useListKeyNav, useShellChrome } from '@/components/shell';
import { Skeleton } from '@/components/ui/skeleton';
import '@/components/shell/api-keys.css';
import '@/components/shell/list.css';
import '@/components/shell/user-access-requests.css';
import { RequestDetailPanel } from '@/routes/users/access-requests/-components/RequestDetailPanel';
import { RequestRailHeader } from '@/routes/users/access-requests/-components/RequestRailHeader';
import { RequestRow } from '@/routes/users/access-requests/-components/RequestRow';
import { useAccessRequestsList } from '@/routes/users/access-requests/-components/useAccessRequestsList';
import { REQUEST_BREADCRUMB } from '@/routes/users/access-requests/-components/utils';

export const Route = createFileRoute('/users/access-requests/')({
  staticData: { section: 'users', subPage: 'access-requests' },
  component: AccessRequestsPage,
});

function AccessRequestsContent() {
  useListKeyNav();
  const {
    isLoading,
    page,
    setPage,
    pageSize,
    filter,
    setFilter,
    search,
    selectedId,
    selectedRole,
    setSelectedRole,
    availableRoles,
    isApproving,
    isRejecting,
    onApprove,
    onReject,
    selectRequest,
    total,
    counts,
    filterTabs,
    searchNode,
    visible,
    selected,
  } = useAccessRequestsList();

  const headerActions = useMemo(
    () =>
      counts.pending > 0 ? (
        <span className="tag tag-saffron" data-testid="pending-pill">
          <ShellIcon name="clock" size={12} />
          {counts.pending} pending review
        </span>
      ) : null,
    [counts.pending]
  );

  const railHeader = useMemo(
    () => (selected ? <RequestRailHeader req={selected} onClose={() => selectRequest(null)} /> : null),
    [selected, selectRequest]
  );
  const rail = useMemo(
    () =>
      selected ? (
        <RequestDetailPanel
          req={selected}
          roles={availableRoles}
          selectedRole={selectedRole}
          onRole={setSelectedRole}
          onApprove={onApprove}
          onReject={onReject}
          disabled={isApproving || isRejecting}
        />
      ) : null,
    [selected, availableRoles, selectedRole, setSelectedRole, onApprove, onReject, isApproving, isRejecting]
  );

  useShellChrome({
    breadcrumb: REQUEST_BREADCRUMB,
    headerActions,
    rail,
    railHeader,
    railDefaultOpen: false,
  });

  return (
    <div
      className="api-keys-screen l-page"
      data-testid="all-requests-page"
      data-pagestatus={isLoading ? 'loading' : 'ready'}
    >
      <div className="l-controls">
        {searchNode.row}
        <div className="l-toolbar">
          <ShellFilterTabs
            tabs={filterTabs}
            value={filter}
            onChange={setFilter}
            label="Filter access requests"
            testIdPrefix="requests-filter"
            loading={isLoading}
          />
          <div className="l-tb-actions">{searchNode.toggle}</div>
        </div>
      </div>

      <div className="l-scroll" data-testid="requests-table">
        {isLoading ? (
          <div style={{ padding: 16 }} data-testid="loading-skeleton">
            {Array.from({ length: 5 }).map((_, i) => (
              <Skeleton key={i} className="h-12 w-full mb-3" />
            ))}
          </div>
        ) : visible.length === 0 ? (
          <EmptyState
            icon="user-check"
            title="No Access Requests"
            sub={search ? 'No requests match your search.' : 'No access requests match this filter.'}
            testId="no-requests"
          />
        ) : (
          <div className="l-listview">
            <div className="l-listhead">
              <div className="ua-icon" />
              <div className="l-lh ua-id">User</div>
              <div className="l-lh ua-status-cell">Status</div>
              <div className="l-lh ua-role-cell">Role</div>
              <div className="ua-act" />
            </div>
            {visible.map((request) => (
              <RequestRow
                key={request.id}
                request={request}
                active={request.id === selectedId}
                roles={availableRoles}
                selectedRole={selectedRole}
                onRole={setSelectedRole}
                onSelect={() => selectRequest(request.id)}
                onApprove={onApprove}
                onReject={onReject}
                disabled={isApproving || isRejecting}
              />
            ))}
          </div>
        )}
        {total > pageSize && <ShellPagination minimal total={total} page={page} onPage={setPage} pageSize={pageSize} />}
      </div>
    </div>
  );
}

export default function AccessRequestsPage() {
  return (
    <AppInitializer allowedStatus="ready" authenticated={true} minRole="manager">
      <AccessRequestsContent />
    </AppInitializer>
  );
}
