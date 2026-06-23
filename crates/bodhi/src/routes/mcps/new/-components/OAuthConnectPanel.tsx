import { ExternalLink, Loader2 } from 'lucide-react';

import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { authConfigTypeLabel } from '@/lib/mcpUtils';

import { safeOrigin, type AuthConfigOption } from './authUtils';

export function OAuthConnectPanel({
  option,
  onConnect,
  isConnecting,
}: {
  option: AuthConfigOption;
  onConnect: () => void;
  isConnecting: boolean;
}) {
  return (
    <div className="space-y-3">
      <div className="rounded-lg border p-3 text-sm space-y-1 bg-muted/50">
        <p>
          <span className="font-medium">Config:</span> {option.name}
        </p>
        <p>
          <span className="font-medium">Type:</span>{' '}
          <Badge variant="secondary">{authConfigTypeLabel(option.type)}</Badge>
        </p>
        <p>
          <span className="font-medium">Auth Server:</span>{' '}
          {option.config.type !== 'header' && safeOrigin(option.config.authorization_endpoint)}
        </p>
      </div>
      <Button
        type="button"
        variant="outline"
        size="sm"
        onClick={onConnect}
        disabled={isConnecting}
        data-testid="auth-config-oauth-connect"
      >
        {isConnecting ? <Loader2 className="mr-2 h-4 w-4 animate-spin" /> : <ExternalLink className="mr-2 h-4 w-4" />}
        Connect
      </Button>
    </div>
  );
}

export default OAuthConnectPanel;
