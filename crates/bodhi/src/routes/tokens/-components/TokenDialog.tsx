import { useState } from 'react';

import { TokenCreated } from '@bodhiapp/ts-client';
import { CheckCircle2 } from 'lucide-react';

import { CopyButton } from '@/components/CopyButton';
import { ShowHideInput } from '@/components/ShowHideInput';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Button } from '@/components/ui/button';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import '@/components/shell/new-token.css';

interface TokenDialogProps {
  token: TokenCreated;
  open: boolean;
  onClose: () => void;
}

export function TokenDialog({ token, open, onClose }: TokenDialogProps) {
  const [showToken, setShowToken] = useState(false);

  return (
    <Dialog open={open} onOpenChange={onClose}>
      <DialogContent className="sm:max-w-2xl" data-testid="token-dialog">
        <DialogHeader>
          <DialogTitle>
            <span className="nt-reveal-header">
              <CheckCircle2 />
              API Token Generated
            </span>
          </DialogTitle>
          <DialogDescription>Copy your API token now. You won&apos;t be able to see it again.</DialogDescription>
        </DialogHeader>

        <div className="space-y-4">
          <ShowHideInput
            value={token.token}
            shown={showToken}
            onToggle={() => setShowToken((v) => !v)}
            data-testid="token-value-input"
            actions={<CopyButton text={token.token} showToast={false} />}
          />
          <Alert variant="destructive">
            <AlertDescription>
              This token will not be shown again. Store it securely — for security reasons it cannot be displayed again.
            </AlertDescription>
          </Alert>
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
