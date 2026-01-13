'use client';

import React, { useState, useEffect } from 'react';

import { UserInfo } from '@bodhiapp/ts-client';

import { Badge } from '@/components/ui/badge';
import { TableCell } from '@/components/ui/table';
import { useToastMessages } from '@/hooks/use-toast-messages';
import { useChangeUserRole, useRemoveUser, AuthenticatedUser } from '@/hooks/useUsers';
import { getRoleLabel, getRoleBadgeVariant } from '@/lib/roles';

import { RemoveUserDialog } from './RemoveUserDialog';
import { RoleChangeDialog } from './RoleChangeDialog';
import { UserActionsCell } from './UserActionsCell';

function getRoleBadge(role: string) {
  const label = getRoleLabel(role);
  const variant = getRoleBadgeVariant(role);
  return <Badge variant={variant}>{label}</Badge>;
}

interface UserRowProps {
  user: UserInfo;
  currentUserRole: string;
  currentUsername: string;
  currentUserInfo?: AuthenticatedUser;
}

export function UserRow({ user, currentUserRole, currentUsername, currentUserInfo }: UserRowProps) {
  const [selectedRole, setSelectedRole] = useState<string>(typeof user.role === 'string' ? user.role : 'resource_user');
  const [showRoleDialog, setShowRoleDialog] = useState(false);
  const [showRemoveDialog, setShowRemoveDialog] = useState(false);

  // Sync selectedRole with user.role when props change
  useEffect(() => {
    const currentRole = typeof user.role === 'string' ? user.role : 'resource_user';
    setSelectedRole(currentRole);
  }, [user.role]);

  const { showSuccess, showError } = useToastMessages();

  const { mutate: changeRole, isLoading: isChangingRole } = useChangeUserRole({
    onSuccess: () => {
      setShowRoleDialog(false);
      showSuccess('Role Updated', `Role updated for ${user.username}`);
    },
    onError: (message) => {
      setSelectedRole(currentRole);
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
    setTimeout(() => {
      setShowRoleDialog(true);
    }, 0);
  };

  const confirmRoleChange = () => {
    changeRole({ userId: user.user_id, newRole: selectedRole });
  };

  const handleRemoveUser = () => {
    setTimeout(() => {
      setShowRemoveDialog(true);
    }, 0);
  };

  const confirmRemoveUser = () => {
    removeUser(user.user_id);
  };

  const currentRole = typeof user.role === 'string' ? user.role : 'resource_user';

  return (
    <>
      <TableCell className="font-medium" data-testid={`user-username-${user.username}`}>
        <span data-testid="user-username">{user.username}</span>
      </TableCell>
      <TableCell data-testid={`user-role-${user.username}`}>
        <span data-testid="user-role">{getRoleBadge(currentRole)}</span>
      </TableCell>
      <TableCell data-testid={`user-actions-${user.username}`}>
        <UserActionsCell
          username={user.username}
          currentRole={currentRole}
          selectedRole={selectedRole}
          currentUserRole={currentUserRole}
          currentUsername={currentUsername}
          currentUserInfo={currentUserInfo}
          userId={user.user_id}
          isRemoving={isRemoving}
          onRoleChange={handleRoleChange}
          onRemoveUser={handleRemoveUser}
        />
      </TableCell>

      <RoleChangeDialog
        open={showRoleDialog}
        onOpenChange={(open) => {
          if (!open && !isChangingRole) {
            setSelectedRole(currentRole);
          }
          setShowRoleDialog(open);
        }}
        username={user.username}
        currentRole={currentRole}
        newRole={selectedRole}
        isLoading={isChangingRole}
        onConfirm={confirmRoleChange}
      />

      <RemoveUserDialog
        open={showRemoveDialog}
        onOpenChange={setShowRemoveDialog}
        username={user.username}
        isLoading={isRemoving}
        onConfirm={confirmRemoveUser}
      />
    </>
  );
}
