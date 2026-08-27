import React, { useState } from 'react';
import { Card, CardHeader, CardTitle, CardContent, Button, Input, Label } from '@/components/ui';
import { useAuth } from '@/context/AuthContext';
import { loadConfig, loadToken, saveConfig } from '@/lib/storage';
import { buildAuthUrl, decodeJwtPayload, generatePkce } from '@/lib/oauth';

export function ReauthorizeSection() {
  const { token, config: contextConfig } = useAuth();
  const [savedConfig] = useState(() => contextConfig || loadConfig());
  const [scope, setScope] = useState(savedConfig?.scope ?? '');
  const [error, setError] = useState<string | null>(null);

  const handleReauthorize = async () => {
    setError(null);
    const config = loadConfig();
    const accessToken = token || loadToken();
    if (!config || !accessToken) {
      setError('Missing saved OAuth config or access token');
      return;
    }
    const claims = decodeJwtPayload(accessToken);
    const accessRequestId = claims?.['access_request_id'];
    if (typeof accessRequestId !== 'string' || !accessRequestId) {
      setError('access_request_id claim missing from access token');
      return;
    }
    const { codeVerifier, codeChallenge, state } = await generatePkce();
    const updated = { ...config, scope, codeVerifier, state };
    saveConfig(updated);
    window.location.href = buildAuthUrl(updated, codeChallenge, state, accessRequestId);
  };

  return (
    <Card data-testid="section-reauthorize" data-test-state={error ? 'error' : 'idle'}>
      <CardHeader>
        <CardTitle>Reauthorize</CardTitle>
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="space-y-1">
          <Label htmlFor="reauthorize-scope">Scope</Label>
          <Input
            id="reauthorize-scope"
            data-testid="input-reauthorize-scope"
            value={scope}
            onChange={(e) => setScope(e.target.value)}
            placeholder="openid profile email roles scope_user_user"
          />
        </div>

        <Button data-testid="btn-reauthorize" onClick={handleReauthorize} size="sm">
          Reauthorize
        </Button>

        {error && (
          <div data-testid="reauthorize-error" className="rounded-md border border-destructive/30 bg-destructive/5 p-4">
            <h3 className="font-semibold text-destructive mb-1">Error</h3>
            <p className="text-sm text-destructive">{error}</p>
          </div>
        )}
      </CardContent>
    </Card>
  );
}
