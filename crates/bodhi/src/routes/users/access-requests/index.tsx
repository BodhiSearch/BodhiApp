import React, { useMemo, useState } from 'react';

import { UserAccessRequest } from '@bodhiapp/ts-client';
import { createFileRoute } from '@tanstack/react-router';
import { Clock, CheckCircle, XCircle } from 'lucide-react';

import AppInitializer from '@/components/AppInitializer';
import { Pagination } from '@/components/DataTable';
import { ShellFilterTabs, ShellIcon, useShellChrome } from '@/components/shell';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Skeleton } from '@/components/ui/skeleton';
import '@/components/shell/api-keys.css';
import '@/components/shell/list.css';
import { useToastMessages } from '@/hooks/use-toast-messages';
import { useListAllRequests, useApproveRequest, useRejectRequest } from '@/hooks/users';
import { useGetAuthenticatedUser } from '@/hooks/users';
import { getAvailableRoles } from '@/lib/roles';

export const Route = createFileRoute('/users/access-requests/')({
  component: AccessRequestsPage,
});

function getStatusBadge(status: string) {
  switch (status) {
    case 'pending':
      return (
        <Badge variant="outline" className="gap-1">
          <Clock className="h-3 w-3" />
          Pending
        </Badge>
      );
    case 'approved':
      return (
        <Badge variant="default" className="gap-1">
          <CheckCircle className="h-3 w-3" />
          Approved
        </Badge>
      );
    case 'rejected':
      return (
        <Badge variant="destructive" className="gap-1">
          <XCircle className="h-3 w-3" />
          Rejected
        </Badge>
      );
    default:
      return <Badge variant="secondary">{status}</Badge>;
  }
}

/* ── V2 (AppShell) render — same hooks/mutations, restructured presentation ── */

const REQUEST_BREADCRUMB = [
  { label: 'Bodhi' },
  { label: 'API Keys', href: '/tokens/' },
  { label: 'Access Requests', current: true },
];

type RequestFilter = 'all' | 'pending' | 'approved' | 'rejected';

const REQUEST_FILTER_TABS: { id: RequestFilter; label: string }[] = [
  { id: 'all', label: 'All' },
  { id: 'pending', label: 'Pending' },
  { id: 'approved', label: 'Approved' },
  { id: 'rejected', label: 'Denied' },
];

function AllRequestsContentV2() {
  const [page, setPage] = useState(1);
  const [pageSize] = useState(10);
  const [filter, setFilter] = useState<RequestFilter>('all');

  const { data: userInfo } = useGetAuthenticatedUser();
  const { data: requestsData, isLoading } = useListAllRequests(page, pageSize);
  const userRole = typeof userInfo?.role === 'string' ? userInfo.role : '';

  const requests = useMemo(() => requestsData?.requests ?? [], [requestsData]);
  const total = requestsData?.total ?? 0;

  // counts derived during render from the fetched page
  const counts = useMemo(() => {
    const c = { all: requests.length, pending: 0, approved: 0, rejected: 0 };
    for (const r of requests) {
      if (r.status === 'pending') c.pending++;
      else if (r.status === 'approved') c.approved++;
      else if (r.status === 'rejected') c.rejected++;
    }
    return c;
  }, [requests]);

  const filterTabs = useMemo(() => REQUEST_FILTER_TABS.map((t) => ({ ...t, count: counts[t.id] })), [counts]);

  const visible = useMemo(
    () => (filter === 'all' ? requests : requests.filter((r) => r.status === filter)),
    [requests, filter]
  );

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

  useShellChrome({ breadcrumb: REQUEST_BREADCRUMB, headerActions });

  return (
    <div
      className="api-keys-screen l-page"
      data-testid="all-requests-page"
      data-pagestatus={isLoading ? 'loading' : 'ready'}
    >
      <div className="l-controls">
        <div className="l-toolbar">
          <ShellFilterTabs
            tabs={filterTabs}
            value={filter}
            onChange={setFilter}
            label="Filter access requests"
            testIdPrefix="requests-filter"
          />
          <div style={{ flex: 1 }} />
          <span data-testid="request-count" className="page-subtitle">
            {total} total {total === 1 ? 'request' : 'requests'}
          </span>
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
          <div className="empty-state" data-testid="no-requests">
            <div className="empty-icon">
              <ShellIcon name="shield-check" size={28} />
            </div>
            <div className="empty-title">No Access Requests</div>
            <div className="empty-sub">No access requests match this filter.</div>
          </div>
        ) : (
          <div className="l-listview">
            {visible.map((request) => (
              <AllRequestRowV2 key={request.id} request={request} userRole={userRole} />
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

function AllRequestRowV2({ request, userRole }: { request: UserAccessRequest; userRole: string }) {
  const [selectedRole, setSelectedRole] = useState<string>('resource_user');
  const { showSuccess, showError } = useToastMessages();

  const { mutate: approveRequest, isPending: isApproving } = useApproveRequest({
    onSuccess: () => showSuccess('Request Approved', `Access granted to ${request.username}`),
    onError: (message) => showError('Approval Failed', message),
  });
  const { mutate: rejectRequest, isPending: isRejecting } = useRejectRequest({
    onSuccess: () => showSuccess('Request Rejected', `Access rejected for ${request.username}`),
    onError: (message) => showError('Rejection Failed', message),
  });

  const availableRoles = getAvailableRoles(userRole);

  return (
    <div className="l-listrow" data-testid={`request-row-${request.username}`}>
      <div style={{ flex: 1, minWidth: 0 }}>
        <div data-testid="request-username" style={{ fontWeight: 600, fontSize: 13.5 }}>
          {request.username}
        </div>
        <div data-testid="request-date" className="page-subtitle">
          {request.status === 'pending'
            ? new Date(request.created_at).toLocaleDateString()
            : new Date(request.updated_at).toLocaleDateString()}
        </div>
      </div>
      <div data-testid={`request-status-${request.status}`} style={{ marginRight: 12 }}>
        {getStatusBadge(request.status)}
      </div>
      {request.status !== 'pending' && request.reviewer ? (
        <span className="page-subtitle" data-testid="request-reviewer" style={{ marginRight: 12 }}>
          {request.reviewer}
        </span>
      ) : null}
      {request.status === 'pending' ? (
        <div className="flex flex-wrap gap-2" onClick={(e) => e.stopPropagation()}>
          <Select value={selectedRole} onValueChange={setSelectedRole} data-testid={`role-select-${request.username}`}>
            <SelectTrigger className="w-32">
              <SelectValue placeholder="Select role" />
            </SelectTrigger>
            <SelectContent>
              {availableRoles.map((role) => (
                <SelectItem key={role.value} value={role.value}>
                  {role.label}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
          <Button
            size="sm"
            onClick={() => selectedRole && approveRequest({ id: request.id, role: selectedRole })}
            disabled={isApproving || !selectedRole}
            data-testid={`approve-btn-${request.username}`}
          >
            {isApproving ? 'Approving...' : 'Approve'}
          </Button>
          <Button
            size="sm"
            variant="destructive"
            onClick={() => rejectRequest(request.id)}
            disabled={isRejecting}
            data-testid={`reject-btn-${request.username}`}
          >
            {isRejecting ? 'Rejecting...' : 'Reject'}
          </Button>
        </div>
      ) : null}
    </div>
  );
}

export default function AccessRequestsPage() {
  return (
    <AppInitializer allowedStatus="ready" authenticated={true} minRole="manager">
      <AllRequestsContentV2 />
    </AppInitializer>
  );
}
