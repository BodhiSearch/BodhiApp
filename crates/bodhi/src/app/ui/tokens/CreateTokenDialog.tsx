'use client';

import { TokenDialog } from '@/app/ui/tokens/TokenDialog';
import { TokenForm } from '@/app/ui/tokens/TokenForm';
import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle } from '@/components/ui/dialog';
import { ApiTokenResponse } from '@bodhiapp/ts-client';
import { useState } from 'react';

interface CreateTokenDialogProps {
  open: boolean;
  onClose: () => void;
}

export function CreateTokenDialog({ open, onClose }: CreateTokenDialogProps) {
  const [createdToken, setCreatedToken] = useState<ApiTokenResponse | null>(null);

  const handleClose = () => {
    setCreatedToken(null);
    onClose();
  };

  const handleTokenCreated = (token: ApiTokenResponse) => {
    setCreatedToken(token);
  };

  return (
    <Dialog open={open} onOpenChange={handleClose}>
      <DialogContent className="sm:max-w-lg" data-testid="create-token-dialog">
        {!createdToken ? (
          <>
            <DialogHeader>
              <DialogTitle>Create API Token</DialogTitle>
              <DialogDescription>Generate a new API token for programmatic access to the API.</DialogDescription>
            </DialogHeader>
            <TokenForm onTokenCreated={handleTokenCreated} />
          </>
        ) : (
          <TokenDialog token={createdToken} open={true} onClose={handleClose} />
        )}
      </DialogContent>
    </Dialog>
  );
}
