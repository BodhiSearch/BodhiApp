'use client';

import { Alert, AlertDescription } from '@/components/ui/alert';
import { Button } from '@/components/ui/button';
import { CopyButton } from '@/components/CopyButton';
import { ShowHideInput } from '@/components/ShowHideInput';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { ApiTokenResponse } from '@bodhiapp/ts-client';
import { Shield } from 'lucide-react';
import { useState } from 'react';

interface TokenDialogProps {
  token: ApiTokenResponse;
  open: boolean;
  onClose: () => void;
}

export function TokenDialog({ token, open, onClose }: TokenDialogProps) {
  const [showToken, setShowToken] = useState(false);

  const toggleShowToken = () => {
    setShowToken(!showToken);
  };

  return (
    <Dialog open={open} onOpenChange={onClose}>
      <DialogContent className="sm:max-w-md" data-testid="token-dialog">
        <DialogHeader>
          <DialogTitle>API Token Generated</DialogTitle>
          <DialogDescription>Copy your API token now. You won&apos;t be able to see it again.</DialogDescription>
        </DialogHeader>

        <div className="space-y-4">
          <Alert variant="destructive">
            <Shield className="h-4 w-4" />
            <AlertDescription>
              Make sure to copy your token now and store it securely. For security reasons, it cannot be displayed
              again.
            </AlertDescription>
          </Alert>

          <ShowHideInput
            value={token.token}
            shown={showToken}
            onToggle={toggleShowToken}
            actions={<CopyButton text={token.token} showToast={false} />}
            data-testid="token-value-input"
          />
        </div>

        <DialogFooter className="sm:justify-start">
          <Button type="button" variant="secondary" onClick={onClose} data-testid="token-dialog-done">
            Done
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
