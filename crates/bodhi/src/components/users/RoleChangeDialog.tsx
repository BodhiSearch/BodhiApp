'use client';

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
import { getRoleLabel } from '@/lib/roles';

interface RoleChangeDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  username: string;
  currentRole: string;
  newRole: string;
  isLoading: boolean;
  onConfirm: () => void;
}

export function RoleChangeDialog({
  open,
  onOpenChange,
  username,
  currentRole,
  newRole,
  isLoading,
  onConfirm,
}: RoleChangeDialogProps) {
  return (
    <AlertDialog open={open} onOpenChange={onOpenChange}>
      <AlertDialogContent data-testid="role-change-dialog">
        <AlertDialogHeader>
          <AlertDialogTitle data-testid="role-change-title">Change User Role</AlertDialogTitle>
          <AlertDialogDescription data-testid="role-change-description">
            Are you sure you want to change {username}&apos;s role from {getRoleLabel(currentRole)} to{' '}
            {getRoleLabel(newRole)}?
          </AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter>
          <AlertDialogCancel data-testid="role-change-cancel" disabled={isLoading}>
            Cancel
          </AlertDialogCancel>
          <AlertDialogAction onClick={onConfirm} disabled={isLoading} data-testid="role-change-confirm">
            {isLoading ? 'Changing...' : 'Change Role'}
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  );
}
