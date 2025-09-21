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

interface RemoveUserDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  username: string;
  isLoading: boolean;
  onConfirm: () => void;
}

export function RemoveUserDialog({ open, onOpenChange, username, isLoading, onConfirm }: RemoveUserDialogProps) {
  return (
    <AlertDialog open={open} onOpenChange={onOpenChange}>
      <AlertDialogContent data-testid="remove-user-dialog">
        <AlertDialogHeader>
          <AlertDialogTitle data-testid="remove-user-title">Remove User Access</AlertDialogTitle>
          <AlertDialogDescription data-testid="remove-user-description">
            Are you sure you want to remove {username}&apos;s access? This action cannot be undone.
          </AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter>
          <AlertDialogCancel data-testid="remove-user-cancel" disabled={isLoading}>
            Cancel
          </AlertDialogCancel>
          <AlertDialogAction
            onClick={onConfirm}
            disabled={isLoading}
            className="bg-destructive text-destructive-foreground hover:bg-destructive/90"
            data-testid="remove-user-confirm"
          >
            {isLoading ? 'Removing...' : 'Remove User'}
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  );
}
