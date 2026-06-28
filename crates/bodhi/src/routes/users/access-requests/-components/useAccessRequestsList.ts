import { useCallback, useMemo, useState } from 'react';

import { UserAccessRequest } from '@bodhiapp/ts-client';

import { useCollapsibleSearch } from '@/components/shell';
import { useToastMessages } from '@/hooks/useToastMessages';
import { useApproveRequest, useGetAuthenticatedUser, useListAllRequests, useRejectRequest } from '@/hooks/users';
import { useViewTransition } from '@/hooks/useViewTransition';
import { getAvailableRoles } from '@/lib/roles';
import { REQUEST_FILTER_TABS, RequestFilter } from '@/routes/users/access-requests/-components/utils';

export function useAccessRequestsList() {
  const [page, setPage] = useState(1);
  const [pageSize] = useState(10);
  const [filter, setFilter] = useState<RequestFilter>('pending');
  const [search, setSearch] = useState('');
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [selectedRole, setSelectedRole] = useState<string>('resource_user');

  const { showSuccess, showError } = useToastMessages();
  const { data: userInfo } = useGetAuthenticatedUser();
  const { data: requestsData, isLoading } = useListAllRequests(page, pageSize);

  const userRole = typeof userInfo?.role === 'string' ? userInfo.role : '';
  const availableRoles = useMemo(() => getAvailableRoles(userRole), [userRole]);

  const { mutate: approveRequest, isPending: isApproving } = useApproveRequest({
    onSuccess: () => showSuccess('Request Approved', 'Access granted'),
    onError: (message) => showError('Approval Failed', message),
  });
  const { mutate: rejectRequest, isPending: isRejecting } = useRejectRequest({
    onSuccess: () => showSuccess('Request Rejected', 'Access rejected'),
    onError: (message) => showError('Rejection Failed', message),
  });

  const onApprove = useCallback(
    (req: UserAccessRequest) => approveRequest({ id: req.id, role: selectedRole }),
    [approveRequest, selectedRole]
  );
  const onReject = useCallback((req: UserAccessRequest) => rejectRequest(req.id), [rejectRequest]);

  const withViewTransition = useViewTransition();
  const selectRequest = useCallback(
    (id: string | null) => withViewTransition(() => setSelectedId(id)),
    [withViewTransition]
  );

  const requests = useMemo(() => requestsData?.requests ?? [], [requestsData]);
  const total = requestsData?.total ?? 0;

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

  const searchNode = useCollapsibleSearch({
    value: search,
    onChange: setSearch,
    placeholder: 'Search requests by email…',
    toggleTestId: 'requests-search-toggle',
    closeTestId: 'requests-search-close',
  });

  const visible = useMemo(() => {
    const q = search.trim().toLowerCase();
    return requests.filter((r) => {
      if (filter !== 'all' && r.status !== filter) return false;
      if (!q) return true;
      return r.username.toLowerCase().includes(q);
    });
  }, [requests, filter, search]);

  const selected = useMemo(() => requests.find((r) => r.id === selectedId) ?? null, [requests, selectedId]);

  return {
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
  };
}
