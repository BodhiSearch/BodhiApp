'use client';

import { Button } from '@/components/ui/button';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { getAvailableRoles, getRoleLevel } from '@/lib/roles';
import { AuthenticatedUser } from '@/hooks/useAuthenticatedUser';
import { Trash2 } from 'lucide-react';

interface UserActionsCellProps {
  username: string;
  currentRole: string;
  selectedRole: string;
  currentUserRole: string;
  currentUsername: string;
  currentUserInfo?: AuthenticatedUser;
  userId: string;
  isRemoving: boolean;
  onRoleChange: (newRole: string) => void;
  onRemoveUser: () => void;
}

export function UserActionsCell({
  username,
  currentRole,
  selectedRole,
  currentUserRole,
  currentUsername,
  currentUserInfo,
  userId,
  isRemoving,
  onRoleChange,
  onRemoveUser,
}: UserActionsCellProps) {
  // Filter role options based on current user's role hierarchy
  const availableRoles = getAvailableRoles(currentUserRole);

  // Check if this is the current user (self-modification prevention)
  const isCurrentUser = currentUserInfo?.auth_status === 'logged_in' && userId === currentUserInfo.user_id;

  // Check if target user has higher role (hierarchy enforcement)
  const targetUserLevel = getRoleLevel(currentRole);
  const currentUserLevel = getRoleLevel(currentUserRole);
  const canModifyUser = !isCurrentUser && targetUserLevel <= currentUserLevel;

  // Show actions only if user can be modified
  const hasValidCurrentUserInfo = currentUsername && currentUserRole;
  const showActions = hasValidCurrentUserInfo && canModifyUser;

  if (!showActions) {
    return (
      <div className="text-sm text-muted-foreground" data-testid={`no-actions-${username}`}>
        <span data-testid={isCurrentUser ? 'current-user-indicator' : 'restricted-user-indicator'}>
          {isCurrentUser ? 'You' : 'Restricted'}
        </span>
      </div>
    );
  }

  return (
    <div className="flex flex-wrap gap-2" data-testid={`user-actions-container-${username}`}>
      <Select value={selectedRole} onValueChange={onRoleChange} data-testid={`role-select-${username}`}>
        <SelectTrigger className="w-32" data-testid={`role-select-trigger-${username}`}>
          <SelectValue />
        </SelectTrigger>
        <SelectContent data-testid={`role-select-content-${username}`}>
          {availableRoles.map((role) => (
            <SelectItem key={role.value} value={role.value} data-testid={`role-option-${role.value}-${username}`}>
              {role.label}
            </SelectItem>
          ))}
        </SelectContent>
      </Select>
      <Button
        size="sm"
        variant="destructive"
        onClick={onRemoveUser}
        disabled={isRemoving}
        className="gap-1"
        data-testid={`remove-user-btn-${username}`}
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
  );
}
