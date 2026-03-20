'use client';

import React, { useState } from 'react';

import { UserAccessRequest } from '@bodhiapp/ts-client';
import { Shield, Clock, CheckCircle, XCircle } from 'lucide-react';

import AppInitializer from '@/components/AppInitializer';
import { DataTable, Pagination } from '@/components/DataTable';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Skeleton } from '@/components/ui/skeleton';
import { TableCell } from '@/components/ui/table';
import { UserManagementTabs } from '@/components/UserManagementTabs';
import { useToastMessages } from '@/hooks/use-toast-messages';
import { useAllRequests, useApproveRequest, useRejectRequest } from '@/hooks/useAccessRequests';
import { useAuthenticatedUser } from '@/hooks/useUsers';
import { getAvailableRoles } from '@/lib/roles';
import { SortState } from '@/types/models';

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

function AllRequestRow({ request, userRole }: { request: UserAccessRequest; userRole: string }) {
  const [selectedRole, setSelectedRole] = useState<string>('resource_user');
  const { showSuccess, showError } = useToastMessages();

  const { mutate: approveRequest, isLoading: isApproving } = useApproveRequest({
    onSuccess: () => {
      showSuccess('Request Approved', `Access granted to ${request.username}`);
    },
    onError: (message) => {
      showError('Approval Failed', message);
    },
  });

  const { mutate: rejectRequest, isLoading: isRejecting } = useRejectRequest({
    onSuccess: () => {
      showSuccess('Request Rejected', `Access rejected for ${request.username}`);
    },
    onError: (message) => {
      showError('Rejection Failed', message);
    },
  });

  const handleApprove = () => {
    if (!selectedRole) return;
    approveRequest({ id: request.id, role: selectedRole });
  };

  const handleReject = () => {
    rejectRequest(request.id);
  };

  // Filter role options based on user's role hierarchy
  const availableRoles = getAvailableRoles(userRole);

  return (
    <>
      <TableCell className="font-medium" data-testid="request-username">
        {request.username}
      </TableCell>
      <TableCell className="hidden sm:table-cell" data-testid="request-date">
        {request.status === 'pending'
          ? new Date(request.created_at).toLocaleDateString()
          : new Date(request.updated_at).toLocaleDateString()}
      </TableCell>
      <TableCell data-testid={`request-status-${request.status}`}>{getStatusBadge(request.status)}</TableCell>
      <TableCell>
        {request.status !== 'pending' && request.reviewer ? (
          <span className="text-sm text-muted-foreground" data-testid="request-reviewer">
            {request.reviewer}
          </span>
        ) : request.status !== 'pending' ? (
          <span className="text-muted-foreground">-</span>
        ) : null}
      </TableCell>
      <TableCell>
        {request.status === 'pending' ? (
          <div className="flex flex-wrap gap-2">
            <Select
              value={selectedRole}
              onValueChange={setSelectedRole}
              data-testid={`role-select-${request.username}`}
            >
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
              onClick={handleApprove}
              disabled={isApproving || !selectedRole}
              data-testid={`approve-btn-${request.username}`}
            >
              {isApproving ? 'Approving...' : 'Approve'}
            </Button>
            <Button
              size="sm"
              variant="destructive"
              onClick={handleReject}
              disabled={isRejecting}
              data-testid={`reject-btn-${request.username}`}
            >
              {isRejecting ? 'Rejecting...' : 'Reject'}
            </Button>
          </div>
        ) : (
          <span className="text-muted-foreground">-</span>
        )}
      </TableCell>
    </>
  );
}

function AllRequestsContent() {
  const [page, setPage] = useState(1);
  const [pageSize] = useState(10);
  // Dummy sort values - no actual sorting functionality
  const dummySort: SortState = { column: '', direction: 'asc' };
  const noOpSortChange = () => {}; // No-op function
  const getItemId = (request: UserAccessRequest) => request.id;

  const { data: userInfo } = useAuthenticatedUser();
  const { data: requestsData, isLoading } = useAllRequests(page, pageSize);

  // Get user's role for filtering
  const userRole = typeof userInfo?.role === 'string' ? userInfo.role : '';

  const columns = [
    { id: 'username', name: 'Username', sorted: false },
    { id: 'date', name: 'Date', sorted: false, className: 'hidden sm:table-cell' },
    { id: 'status', name: 'Status', sorted: false },
    { id: 'reviewer', name: 'Reviewer', sorted: false },
    { id: 'actions', name: 'Actions', sorted: false },
  ];

  if (isLoading) {
    return (
      <Card data-testid="loading-skeleton">
        <CardHeader>
          <Skeleton className="h-6 w-48" />
          <Skeleton className="h-4 w-32" />
        </CardHeader>
        <CardContent>
          <div className="space-y-4">
            {Array.from({ length: 5 }).map((_, i) => (
              <Skeleton key={i} className="h-12 w-full" />
            ))}
          </div>
        </CardContent>
      </Card>
    );
  }

  const requests = requestsData?.requests || [];
  const total = requestsData?.total || 0;

  if (requests.length === 0) {
    return (
      <Card className="text-center py-8" data-testid="no-requests">
        <CardContent>
          <Shield className="h-12 w-12 mx-auto text-muted-foreground mb-4" />
          <h3 className="text-lg font-semibold mb-2">No Access Requests</h3>
          <p className="text-muted-foreground">No access requests have been submitted yet</p>
        </CardContent>
      </Card>
    );
  }

  const renderRow = (request: UserAccessRequest) => <AllRequestRow request={request} userRole={userRole} />;

  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center gap-2" data-testid="page-title">
          <Shield className="h-5 w-5" />
          All Access Requests
        </CardTitle>
        <CardDescription data-testid="request-count">
          {total} total {total === 1 ? 'request' : 'requests'}
        </CardDescription>
      </CardHeader>
      <CardContent>
        <div data-testid="requests-table">
          <DataTable
            columns={columns}
            data={requests}
            renderRow={renderRow}
            loading={isLoading}
            sort={dummySort}
            onSortChange={noOpSortChange}
            getItemId={getItemId}
          />
        </div>
        {total > pageSize && (
          <div className="mt-4" data-testid="pagination">
            <Pagination page={page} totalPages={Math.ceil(total / pageSize)} onPageChange={setPage} />
          </div>
        )}
      </CardContent>
    </Card>
  );
}

export default function AllRequestsPage() {
  return (
    <AppInitializer allowedStatus="ready" authenticated={true} minRole="manager">
      <div className="container mx-auto p-4" data-testid="all-requests-page">
        <UserManagementTabs />
        <AllRequestsContent />
      </div>
    </AppInitializer>
  );
}
