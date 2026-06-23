import { CheckCircle2, Loader2, Unplug } from 'lucide-react';

import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import type { McpAuthConfigResponse } from '@/hooks/mcps';

import { safeOrigin } from './authUtils';

export function OAuthConnectedCard({
  config,
  onDisconnect,
  isDisconnecting,
}: {
  config: McpAuthConfigResponse | null;
  onDisconnect: () => void;
  isDisconnecting: boolean;
}) {
  return (
    <div
      className="rounded-lg border border-green-200 dark:border-green-800 bg-green-50 dark:bg-green-950/30 p-4 space-y-3"
      data-testid="oauth-connected-card"
    >
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2">
          <CheckCircle2 className="h-4 w-4 text-green-600 dark:text-green-400" />
          <Badge
            variant="outline"
            className="bg-green-100 dark:bg-green-900 text-green-700 dark:text-green-300 border-green-300 dark:border-green-700"
            data-testid="oauth-connected-badge"
          >
            Connected
          </Badge>
        </div>
        <Button
          type="button"
          variant="outline"
          size="sm"
          onClick={onDisconnect}
          disabled={isDisconnecting}
          data-testid="oauth-disconnect-button"
        >
          {isDisconnecting ? <Loader2 className="mr-2 h-4 w-4 animate-spin" /> : <Unplug className="mr-2 h-4 w-4" />}
          Disconnect
        </Button>
      </div>
      {config && config.type !== 'header' && (
        <div className="text-sm text-muted-foreground space-y-1" data-testid="oauth-connected-info">
          <p>
            <span className="font-medium">Client ID:</span> {config.client_id}
          </p>
          <p>
            <span className="font-medium">Auth Server:</span> {safeOrigin(config.authorization_endpoint)}
          </p>
          {config.scopes && (
            <p>
              <span className="font-medium">Scopes:</span> {config.scopes}
            </p>
          )}
        </div>
      )}
    </div>
  );
}

export default OAuthConnectedCard;
