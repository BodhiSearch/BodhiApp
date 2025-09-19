'use client';

import React, { useState } from 'react';
import AppInitializer from '@/components/AppInitializer';
import { UserManagementTabs } from '@/components/UserManagementTabs';
import { DataTable, Pagination } from '@/components/DataTable';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { TableCell } from '@/components/ui/table';
import { Skeleton } from '@/components/ui/skeleton';
import { Alert, AlertDescription } from '@/components/ui/alert';
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from '@/components/ui/alert-dialog';
import { useAllUsers, useChangeUserRole, useRemoveUser } from '@/hooks/useAccessRequest';
import { useUser } from '@/hooks/useQuery';
import { useToastMessages } from '@/hooks/use-toast-messages';
import { UserInfo, UserInfoResponse } from '@bodhiapp/ts-client';
import { Users, AlertCircle, Trash2 } from 'lucide-react';
import { getRoleLabel, getRoleBadgeVariant, getAvailableRoles, getRoleLevel } from '@/lib/roles';
import { SortState } from '@/types/models';

function getRoleBadge(role: string) {
  const label = getRoleLabel(role);
  const variant = getRoleBadgeVariant(role);
  return <Badge variant={variant}>{label}</Badge>;
}

function UserRow({
  user,
  currentUserRole,
  currentUsername,
  currentUserInfo,
}: {
  user: UserInfoResponse;
  currentUserRole: string;
  currentUsername: string;
  currentUserInfo: any;
}) {
  const [selectedRole, setSelectedRole] = useState<string>(typeof user.role === 'string' ? user.role : 'resource_user');
  const [showRoleDialog, setShowRoleDialog] = useState(false);
  const [showRemoveDialog, setShowRemoveDialog] = useState(false);
  const { showSuccess, showError } = useToastMessages();

  const { mutate: changeRole, isLoading: isChangingRole } = useChangeUserRole({
    onSuccess: () => {
      setShowRoleDialog(false);
      // selectedRole already has the new value, keep it
      showSuccess('Role Updated', `Role updated for ${user.username}`);
    },
    onError: (message) => {
      setSelectedRole(currentRole); // Reset to original on error
      setShowRoleDialog(false);
      showError('Update Failed', message);
    },
  });

  const { mutate: removeUser, isLoading: isRemoving } = useRemoveUser({
    onSuccess: () => {
      setShowRemoveDialog(false);
      showSuccess('User Removed', `User access removed for ${user.username}`);
    },
    onError: (message) => {
      showError('Removal Failed', message);
    },
  });

  const handleRoleChange = (newRole: string) => {
    setSelectedRole(newRole);
    // Use setTimeout to ensure keyboard event completes before dialog opens
    setTimeout(() => {
      setShowRoleDialog(true);
    }, 0);
  };

  const confirmRoleChange = () => {
    changeRole({ userId: user.user_id, newRole: selectedRole });
  };

  const handleRemoveUser = () => {
    setShowRemoveDialog(true);
  };

  const confirmRemoveUser = () => {
    removeUser(user.user_id);
  };

  // Filter role options based on current user's role hierarchy
  const availableRoles = getAvailableRoles(currentUserRole);

  const currentRole = typeof user.role === 'string' ? user.role : 'resource_user';

  // Check if this is the current user (self-modification prevention)
  // Use multiple comparison methods to ensure proper identification
  const isCurrentUser =
    user.username?.trim() === currentUsername?.trim() ||
    user.username === currentUserInfo?.username ||
    (currentUserInfo?.email && user.username === currentUserInfo.email) ||
    (currentUserInfo?.user_id && user.user_id === currentUserInfo.user_id);

  // Check if target user has higher role (hierarchy enforcement)
  const targetUserLevel = getRoleLevel(currentRole);
  const currentUserLevel = getRoleLevel(currentUserRole);
  const canModifyUser = !isCurrentUser && targetUserLevel <= currentUserLevel;

  // Show actions only if user can be modified
  // Safety check: If we can't identify current user properly, disable all actions for safety
  const hasValidCurrentUserInfo = currentUsername && currentUserRole;
  const showActions = hasValidCurrentUserInfo && canModifyUser;

  return (
    <>
      <TableCell className="font-medium" data-testid={`user-username-${user.username}`}>
        <span data-testid="user-username">{user.username}</span>
      </TableCell>
      <TableCell data-testid={`user-role-${user.username}`}>
        <span data-testid="user-role">{getRoleBadge(currentRole)}</span>
      </TableCell>
      <TableCell data-testid={`user-status-${user.username}`}>
        <Badge variant="outline" data-testid="user-status">
          Active
        </Badge>
      </TableCell>
      <TableCell data-testid={`user-actions-${user.username}`}>
        {showActions ? (
          <div className="flex flex-wrap gap-2" data-testid={`user-actions-container-${user.username}`}>
            <Select value={selectedRole} onValueChange={handleRoleChange} data-testid={`role-select-${user.username}`}>
              <SelectTrigger className="w-32" data-testid={`role-select-trigger-${user.username}`}>
                <SelectValue />
              </SelectTrigger>
              <SelectContent data-testid={`role-select-content-${user.username}`}>
                {availableRoles.map((role) => (
                  <SelectItem
                    key={role.value}
                    value={role.value}
                    data-testid={`role-option-${role.value}-${user.username}`}
                  >
                    {role.label}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
            <Button
              size="sm"
              variant="destructive"
              onClick={handleRemoveUser}
              disabled={isRemoving}
              className="gap-1"
              data-testid={`remove-user-btn-${user.username}`}
            >
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
        ) : (
          <div className="text-sm text-muted-foreground" data-testid={`no-actions-${user.username}`}>
            <span data-testid={isCurrentUser ? 'current-user-indicator' : 'restricted-user-indicator'}>
              {isCurrentUser ? 'You' : 'Restricted'}
            </span>
          </div>
        )}
      </TableCell>

      {/* Role Change Confirmation Dialog */}
      <AlertDialog
        open={showRoleDialog}
        onOpenChange={(open) => {
          if (!open && !isChangingRole) {
            // Dialog is closing and not due to successful change
            // Reset to current role
            setSelectedRole(currentRole);
          }
          setShowRoleDialog(open);
        }}
      >
        <AlertDialogContent data-testid="role-change-dialog">
          <AlertDialogHeader>
            <AlertDialogTitle data-testid="role-change-title">Change User Role</AlertDialogTitle>
            <AlertDialogDescription data-testid="role-change-description">
              Are you sure you want to change {user.username}'s role from {getRoleLabel(currentRole)} to{' '}
              {getRoleLabel(selectedRole)}?
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel data-testid="role-change-cancel" disabled={isChangingRole}>
              Cancel
            </AlertDialogCancel>
            <AlertDialogAction onClick={confirmRoleChange} disabled={isChangingRole} data-testid="role-change-confirm">
              {isChangingRole ? 'Changing...' : 'Change Role'}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>

      {/* Remove User Confirmation Dialog */}
      <AlertDialog open={showRemoveDialog} onOpenChange={setShowRemoveDialog}>
        <AlertDialogContent data-testid="remove-user-dialog">
          <AlertDialogHeader>
            <AlertDialogTitle data-testid="remove-user-title">Remove User Access</AlertDialogTitle>
            <AlertDialogDescription data-testid="remove-user-description">
              Are you sure you want to remove {user.username}'s access? This action cannot be undone.
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel data-testid="remove-user-cancel" disabled={isRemoving}>
              Cancel
            </AlertDialogCancel>
            <AlertDialogAction
              onClick={confirmRemoveUser}
              disabled={isRemoving}
              className="bg-destructive text-destructive-foreground hover:bg-destructive/90"
              data-testid="remove-user-confirm"
            >
              {isRemoving ? 'Removing...' : 'Remove User'}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </>
  );
}

function UsersContent() {
  const [page, setPage] = useState(1);
  const [pageSize] = useState(10);

  // Dummy sort values - no actual sorting functionality
  const dummySort: SortState = { column: '', direction: 'asc' };
  const noOpSortChange = () => {}; // No-op function
  const getItemId = (user: UserInfoResponse) => user.username;

  const { data: currentUserInfo, isLoading: isLoadingUser } = useUser();
  const { data: usersData, isLoading: isLoadingUsers, error } = useAllUsers(page, pageSize);

  // Get current user's role and username for filtering
  const currentUserRole = typeof currentUserInfo?.role === 'string' ? currentUserInfo.role : '';
  const currentUsername = typeof currentUserInfo?.username === 'string' ? currentUserInfo.username : '';

  const columns = [
    { id: 'username', name: 'Username', sorted: false },
    { id: 'role', name: 'Role', sorted: false },
    { id: 'status', name: 'Status', sorted: false },
    { id: 'actions', name: 'Actions', sorted: false },
  ];

  // Show loading state if either users or current user info is loading
  if (isLoadingUsers || isLoadingUser) {
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
    const errorMessage = error?.response?.data?.error?.message || 'Failed to load users. Please try again later.';
    return (
      <Alert variant="destructive">
        <AlertCircle className="h-4 w-4" />
        <AlertDescription>{errorMessage}</AlertDescription>
      </Alert>
    );
  }

  const users = usersData?.users || [];
  const total = usersData?.total || 0;

  return (
    <div className="space-y-4">
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Users className="h-5 w-5" />
            All Users
          </CardTitle>
          <CardDescription>Manage user access and roles</CardDescription>
        </CardHeader>
        <CardContent>
          {users.length === 0 ? (
            <div className="text-center py-8">
              <Users className="h-12 w-12 mx-auto text-muted-foreground mb-4" />
              <h3 className="text-lg font-semibold mb-2">No Users</h3>
              <p className="text-muted-foreground">No users found</p>
            </div>
          ) : (
            <>
              <DataTable
                columns={columns}
                data={users}
                renderRow={(user) => (
                  <UserRow
                    user={user}
                    currentUserRole={currentUserRole}
                    currentUsername={currentUsername}
                    currentUserInfo={currentUserInfo}
                  />
                )}
                loading={isLoadingUsers}
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
        <UserManagementTabs />
        <UsersContent />
      </div>
    </AppInitializer>
  );
}
