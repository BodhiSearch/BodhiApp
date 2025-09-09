'use client';

import React, { useState } from 'react';
import Link from 'next/link';
import { usePathname } from 'next/navigation';
import AppInitializer from '@/components/AppInitializer';
import { DataTable, Pagination } from '@/components/DataTable';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { TableCell } from '@/components/ui/table';
import { Skeleton } from '@/components/ui/skeleton';
import { ROUTE_ACCESS_REQUESTS_PENDING, ROUTE_ACCESS_REQUESTS_ALL, ROUTE_USERS } from '@/lib/constants';
import { useAllRequests, useApproveRequest, useRejectRequest } from '@/hooks/useAccessRequest';
import { useUser } from '@/hooks/useQuery';
import { useToastMessages } from '@/hooks/use-toast-messages';
import { UserAccessRequestDto } from '@bodhiapp/ts-client';
import { Shield, Clock, CheckCircle, XCircle } from 'lucide-react';
import { ROLE_OPTIONS, getAvailableRoles } from '@/lib/roles';

function NavigationLinks() {
  const pathname = usePathname();

  const linkClass = (path: string) =>
    pathname === path
      ? 'font-bold text-primary border-b-2 border-primary pb-1'
      : 'text-muted-foreground hover:text-foreground';

  return (
    <div className="flex gap-4 mb-6">
      <Link href={ROUTE_ACCESS_REQUESTS_PENDING} className={linkClass(ROUTE_ACCESS_REQUESTS_PENDING)}>
        Pending Requests
      </Link>
      <Link href={ROUTE_ACCESS_REQUESTS_ALL} className={linkClass(ROUTE_ACCESS_REQUESTS_ALL)}>
        All Requests
      </Link>
      <Link href={ROUTE_USERS} className={linkClass(ROUTE_USERS)}>
        All Users
      </Link>
    </div>
  );
}

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

function AllRequestRow({ request, userRole }: { request: UserAccessRequestDto; userRole: string }) {
  const [selectedRole, setSelectedRole] = useState<string>('resource_user');
  const { showSuccess, showError } = useToastMessages();

  const { mutate: approveRequest, isLoading: isApproving } = useApproveRequest({
    onSuccess: () => {
      showSuccess('Request Approved', `Access granted to ${request.email}`);
    },
    onError: (message) => {
      showError('Approval Failed', message);
    },
  });

  const { mutate: rejectRequest, isLoading: isRejecting } = useRejectRequest({
    onSuccess: () => {
      showSuccess('Request Rejected', `Access rejected for ${request.email}`);
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
      <TableCell className="font-medium">{request.email}</TableCell>
      <TableCell className="hidden sm:table-cell">{new Date(request.created_at).toLocaleDateString()}</TableCell>
      <TableCell>{getStatusBadge(request.status)}</TableCell>
      <TableCell>
        {request.status === 'pending' ? (
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
  const { data: userInfo } = useUser();
  const { data: requestsData, isLoading } = useAllRequests(page, pageSize);

  // Get user's role for filtering
  const userRole = typeof userInfo?.role === 'string' ? userInfo.role : '';

  const columns = [
    { id: 'email', name: 'Email', sorted: false },
    { id: 'created_at', name: 'Requested Date', sorted: true, className: 'hidden sm:table-cell' },
    { id: 'status', name: 'Status', sorted: false },
    { id: 'actions', name: 'Actions', sorted: false },
  ];

  if (isLoading) {
    return (
      <Card>
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
      <Card className="text-center py-8">
        <CardContent>
          <Shield className="h-12 w-12 mx-auto text-muted-foreground mb-4" />
          <h3 className="text-lg font-semibold mb-2">No Access Requests</h3>
          <p className="text-muted-foreground">No access requests have been submitted yet</p>
        </CardContent>
      </Card>
    );
  }

  const renderRow = (request: UserAccessRequestDto) => <AllRequestRow request={request} userRole={userRole} />;

  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <Shield className="h-5 w-5" />
          All Access Requests
        </CardTitle>
        <CardDescription>
          {total} total {total === 1 ? 'request' : 'requests'}
        </CardDescription>
      </CardHeader>
      <CardContent>
        <DataTable columns={columns} data={requests} renderRow={renderRow} />
        {total > pageSize && (
          <div className="mt-4">
            <Pagination currentPage={page} totalPages={Math.ceil(total / pageSize)} onPageChange={setPage} />
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
        <NavigationLinks />
        <AllRequestsContent />
      </div>
    </AppInitializer>
  );
}
