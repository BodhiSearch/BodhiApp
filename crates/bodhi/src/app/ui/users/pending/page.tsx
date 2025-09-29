'use client';

import AppInitializer from '@/components/AppInitializer';
import { UserManagementTabs } from '@/components/UserManagementTabs';
import { DataTable, Pagination } from '@/components/DataTable';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Skeleton } from '@/components/ui/skeleton';
import { TableCell } from '@/components/ui/table';
import { useToastMessages } from '@/hooks/use-toast-messages';
import { UserAccessRequest } from '@bodhiapp/ts-client';
import { Shield, Clock } from 'lucide-react';
import { getAvailableRoles } from '@/lib/roles';
import { SortState } from '@/types/models';
import { useApproveRequest, usePendingRequests, useRejectRequest } from '@/hooks/useAccessRequests';
import { useAuthenticatedUser } from '@/hooks/useUsers';
import { useState } from 'react';

function PendingRequestRow({ request, userRole }: { request: UserAccessRequest; userRole: string }) {
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
      <TableCell className="font-medium">{request.username}</TableCell>
      <TableCell>{new Date(request.created_at).toLocaleDateString()}</TableCell>
      <TableCell>
        <Badge variant="outline" className="gap-1">
          <Clock className="h-3 w-3" />
          Pending
        </Badge>
      </TableCell>
      <TableCell>
        <div className="flex flex-wrap gap-2">
          <Select value={selectedRole} onValueChange={setSelectedRole}>
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
          <Button size="sm" onClick={handleApprove} disabled={isApproving || !selectedRole}>
            {isApproving ? 'Approving...' : 'Approve'}
          </Button>
          <Button size="sm" variant="destructive" onClick={handleReject} disabled={isRejecting}>
            {isRejecting ? 'Rejecting...' : 'Reject'}
          </Button>
        </div>
      </TableCell>
    </>
  );
}

function PendingRequestsContent() {
  const [page, setPage] = useState(1);
  const [pageSize] = useState(10);
  // Dummy sort values - no actual sorting functionality
  const dummySort: SortState = { column: '', direction: 'asc' };
  const noOpSortChange = () => {}; // No-op function
  const getItemId = (request: UserAccessRequest) => request.id.toString();

  const { data: userInfo } = useAuthenticatedUser();
  const { data: requestsData, isLoading, error } = usePendingRequests(page, pageSize);

  // Get user's role for filtering
  const userRole = typeof userInfo?.role === 'string' ? userInfo.role : '';

  const columns = [
    { id: 'username', name: 'Username', sorted: false },
    { id: 'created_at', name: 'Requested Date', sorted: false, className: 'hidden sm:table-cell' },
    { id: 'status', name: 'Status', sorted: false },
    { id: 'actions', name: 'Actions', sorted: false },
  ];

  // Handle error state - show empty state instead of error
  if (error) {
    return (
      <Card className="text-center py-8">
        <CardContent>
          <Shield className="h-12 w-12 mx-auto text-muted-foreground mb-4" />
          <h3 className="text-lg font-semibold mb-2">No Pending Requests</h3>
          <p className="text-muted-foreground">All access requests have been reviewed</p>
        </CardContent>
      </Card>
    );
  }

  if (isLoading) {
    return (
      <Card>
        <CardHeader>
          <Skeleton className="h-6 w-48" />
          <Skeleton className="h-4 w-32" />
        </CardHeader>
        <CardContent>
          <div className="space-y-4">
            {Array.from({ length: 3 }).map((_, i) => (
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
      <Card className="text-center py-8">
        <CardContent>
          <Shield className="h-12 w-12 mx-auto text-muted-foreground mb-4" />
          <h3 className="text-lg font-semibold mb-2">No Pending Requests</h3>
          <p className="text-muted-foreground">All access requests have been reviewed</p>
        </CardContent>
      </Card>
    );
  }

  const renderRow = (request: UserAccessRequest) => <PendingRequestRow request={request} userRole={userRole} />;

  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <Shield className="h-5 w-5" />
          Pending Access Requests
        </CardTitle>
        <CardDescription>
          {total} {total === 1 ? 'request' : 'requests'} awaiting review
        </CardDescription>
      </CardHeader>
      <CardContent>
        <DataTable
          columns={columns}
          data={requests}
          renderRow={renderRow}
          loading={isLoading}
          sort={dummySort}
          onSortChange={noOpSortChange}
          getItemId={getItemId}
        />
        {total > pageSize && (
          <div className="mt-4">
            <Pagination page={page} totalPages={Math.ceil(total / pageSize)} onPageChange={setPage} />
          </div>
        )}
      </CardContent>
    </Card>
  );
}

export default function PendingRequestsPage() {
  return (
    <AppInitializer allowedStatus="ready" authenticated={true} minRole="manager">
      <div className="container mx-auto p-4" data-testid="pending-requests-page">
        <UserManagementTabs />
        <PendingRequestsContent />
      </div>
    </AppInitializer>
  );
}
