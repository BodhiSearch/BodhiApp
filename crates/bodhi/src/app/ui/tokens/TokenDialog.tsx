'use client';

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
import { TokenResponse } from '@/hooks/useApiTokens';
import { Check, Copy, Eye, EyeOff, Shield } from 'lucide-react';
import { useState } from 'react';

interface TokenDialogProps {
  token: TokenResponse;
  open: boolean;
  onClose: () => void;
}

export function TokenDialog({ token, open, onClose }: TokenDialogProps) {
  const [showToken, setShowToken] = useState(false);
  const [copied, setCopied] = useState(false);

  const handleCopy = async () => {
    await navigator.clipboard.writeText(token.offline_token);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  const toggleShowToken = () => {
    setShowToken(!showToken);
  };

  return (
    <Dialog open={open} onOpenChange={onClose}>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>API Token Generated</DialogTitle>
          <DialogDescription>
            Copy your API token now. You won&apos;t be able to see it again.
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-4">
          <Alert variant="destructive">
            <Shield className="h-4 w-4" />
            <AlertDescription>
              Make sure to copy your token now and store it securely. For
              security reasons, it cannot be displayed again.
            </AlertDescription>
          </Alert>

          <div className="relative">
            <div className="rounded-md bg-muted p-3 font-mono text-sm break-all">
              {showToken ? token.offline_token : '•'.repeat(40)}
            </div>
            <div className="absolute right-2 top-2 space-x-2">
              <Button
                variant="ghost"
                size="icon"
                onClick={toggleShowToken}
                type="button"
                data-testid="toggle-show-token"
              >
                {showToken ? (
                  <EyeOff className="h-4 w-4" />
                ) : (
                  <Eye className="h-4 w-4" />
                )}
              </Button>
              <Button
                variant="ghost"
                size="icon"
                onClick={handleCopy}
                type="button"
              >
                {copied ? (
                  <Check className="h-4 w-4" data-testid="copied-token" />
                ) : (
                  <Copy className="h-4 w-4" data-testid="copy-token" />
                )}
              </Button>
            </div>
          </div>
        </div>

        <DialogFooter className="sm:justify-start">
          <Button type="button" variant="secondary" onClick={onClose}>
            Done
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
