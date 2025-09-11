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
import { Alert, AlertDescription } from '@/components/ui/alert';
import { ROUTE_ACCESS_REQUESTS_PENDING, ROUTE_ACCESS_REQUESTS_ALL, ROUTE_USERS } from '@/lib/constants';
import { useAllUsers, useChangeUserRole, useRemoveUser } from '@/hooks/useAccessRequest';
import { useUser } from '@/hooks/useQuery';
import { useToastMessages } from '@/hooks/use-toast-messages';
import { UserInfo } from '@bodhiapp/ts-client';
import { Users, AlertCircle, Trash2 } from 'lucide-react';
import {
  ROLE_OPTIONS,
  getRoleLabel,
  getRoleBadgeVariant,
  getAvailableRoles,
  getRoleLevel,
  getCleanRoleName,
} from '@/lib/roles';
import { SortState } from '@/types/models';

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

function getRoleBadge(role: string) {
  const label = getRoleLabel(role);
  const variant = getRoleBadgeVariant(role);
  return <Badge variant={variant}>{label}</Badge>;
}

function UserRow({ user, currentUserRole }: { user: UserInfo; currentUserRole: string }) {
  const [selectedRole, setSelectedRole] = useState<string>(typeof user.role === 'string' ? user.role : 'resource_user');
  const { showSuccess, showError } = useToastMessages();

  const { mutate: changeRole } = useChangeUserRole({
    onSuccess: () => {
      showSuccess('Role Updated', `Role updated for ${user.email}`);
    },
    onError: (message) => {
      showError('Update Failed', message);
    },
  });

  const { mutate: removeUser, isLoading: isRemoving } = useRemoveUser({
    onSuccess: () => {
      showSuccess('User Removed', `User access removed for ${user.email}`);
    },
    onError: (message) => {
      showError('Removal Failed', message);
    },
  });

  const handleRoleChange = (newRole: string) => {
    setSelectedRole(newRole);
    changeRole({ userId: user.email || '', newRole });
  };

  const handleRemoveUser = () => {
    if (!user.email) return;
    removeUser(user.email);
  };

  // Filter role options based on current user's role hierarchy
  const availableRoles = getAvailableRoles(currentUserRole);

  const currentRole = typeof user.role === 'string' ? user.role : 'resource_user';
  const lastLogin = user.logged_in ? new Date().toLocaleDateString() : 'Never'; // Mock data

  return (
    <>
      <TableCell className="font-medium">{user.email}</TableCell>
      <TableCell>{getRoleBadge(currentRole)}</TableCell>
      <TableCell>
        <Badge variant="outline">Active</Badge>
      </TableCell>
      <TableCell className="hidden md:table-cell">{lastLogin}</TableCell>
      <TableCell className="hidden md:table-cell">
        {new Date().toLocaleDateString()} {/* Mock created date */}
      </TableCell>
      <TableCell>
        <div className="flex flex-wrap gap-2">
          <Select value={selectedRole} onValueChange={handleRoleChange}>
            <SelectTrigger className="w-32">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              {availableRoles.map((role) => (
                <SelectItem key={role.value} value={role.value}>
                  {role.label}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
          <Button size="sm" variant="destructive" onClick={handleRemoveUser} disabled={isRemoving} className="gap-1">
            {isRemoving ? (
              'Removing...'
            ) : (
              <>
                <Trash2 className="h-3 w-3" />
                Remove
              </>
            )}
          </Button>
        </div>
      </TableCell>
    </>
  );
}

function UsersContent() {
  const [page, setPage] = useState(1);
  const [pageSize] = useState(10);

  // Dummy sort values - no actual sorting functionality
  const dummySort: SortState = { column: '', direction: 'asc' };
  const noOpSortChange = () => {}; // No-op function
  const getItemId = (user: UserInfo) => user.email || 'unknown';

  const { data: currentUserInfo } = useUser();
  const { data: usersData, isLoading, error } = useAllUsers(page, pageSize);

  // Get current user's role for filtering
  const currentUserRole = typeof currentUserInfo?.role === 'string' ? currentUserInfo.role : '';

  const columns = [
    { id: 'email', name: 'Email', sorted: false },
    { id: 'role', name: 'Role', sorted: false },
    { id: 'status', name: 'Status', sorted: false },
    { id: 'last_login', name: 'Last Login', sorted: false, className: 'hidden md:table-cell' },
    { id: 'created_at', name: 'Created', sorted: false, className: 'hidden md:table-cell' },
    { id: 'actions', name: 'Actions', sorted: false },
  ];

  if (isLoading) {
    return (
      <Card>
        <CardHeader>
          <Skeleton className="h-6 w-32" />
          <Skeleton className="h-4 w-48" />
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

  if (error) {
    return (
      <Alert variant="destructive">
        <AlertCircle className="h-4 w-4" />
        <AlertDescription>Failed to load users. User management functionality is not yet implemented.</AlertDescription>
      </Alert>
    );
  }

  const users = usersData?.users || [];
  const total = usersData?.total || 0;

  // Show placeholder message since user management API is not yet implemented
  return (
    <div className="space-y-4">
      <Alert>
        <AlertCircle className="h-4 w-4" />
        <AlertDescription>
          User management functionality will be implemented in a future release. This page shows the planned interface
          design.
        </AlertDescription>
      </Alert>

      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Users className="h-5 w-5" />
            All Users
          </CardTitle>
          <CardDescription>Manage user access and roles (Coming Soon)</CardDescription>
        </CardHeader>
        <CardContent>
          {users.length === 0 ? (
            <div className="text-center py-8">
              <Users className="h-12 w-12 mx-auto text-muted-foreground mb-4" />
              <h3 className="text-lg font-semibold mb-2">No Users</h3>
              <p className="text-muted-foreground">User management API not yet implemented</p>
            </div>
          ) : (
            <>
              <DataTable
                columns={columns}
                data={users}
                renderRow={(user) => <UserRow user={user} currentUserRole={currentUserRole} />}
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
            </>
          )}
        </CardContent>
      </Card>
    </div>
  );
}

export default function UsersPage() {
  return (
    <AppInitializer allowedStatus="ready" authenticated={true} minRole="manager">
      <div className="container mx-auto p-4" data-testid="users-page">
        <NavigationLinks />
        <UsersContent />
      </div>
    </AppInitializer>
  );
}
