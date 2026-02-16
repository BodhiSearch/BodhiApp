import React, { useEffect, useState, useRef } from 'react';
import { useSearchParams } from 'react-router-dom';
import { Card, CardHeader, CardTitle, CardContent, Button, Input, Label } from '@/components/ui';
import { ScopeDisplay } from '@/components/ScopeDisplay';
import { getAccessRequestStatus } from '@/lib/api';
import { loadConfig, saveConfig } from '@/lib/storage';
import {
  generateCodeVerifier,
  generateCodeChallenge,
  generateState,
  buildAuthUrl,
} from '@/lib/oauth';

export function AccessCallbackPage() {
  const [searchParams] = useSearchParams();
  const id = searchParams.get('id');

  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [resourceScope, setResourceScope] = useState<string | null>(null);
  const [accessRequestScope, setAccessRequestScope] = useState<string | null>(null);
  const [scope, setScope] = useState('');
  const [ready, setReady] = useState(false);

  const processedRef = useRef(false);

  useEffect(() => {
    if (processedRef.current) return;
    processedRef.current = true;

    async function checkStatus() {
      if (!id) {
        setError('No access request ID found in URL');
        setLoading(false);
        return;
      }

      const config = loadConfig();
      if (!config) {
        setError('OAuth configuration not found. Please start the flow again.');
        setLoading(false);
        return;
      }

      try {
        const data = await getAccessRequestStatus(config.bodhiServerUrl, id, config.clientId);

        if (data.status === 'approved') {
          const approvedScopes: string[] = [];
          if (data.resource_scope) approvedScopes.push(data.resource_scope);
          if (data.access_request_scope) approvedScopes.push(data.access_request_scope);

          let updatedScope = config.scope;
          if (approvedScopes.length > 0) {
            updatedScope = config.scope + ' ' + approvedScopes.join(' ');
          }

          // Update config
          config.approvedScopes = approvedScopes;
          config.accessRequestId = id;
          saveConfig(config);

          setResourceScope(data.resource_scope || null);
          setAccessRequestScope(data.access_request_scope || null);
          setScope(updatedScope);
          setReady(true);

          // Clear URL params
          window.history.replaceState({}, document.title, window.location.pathname);
        } else if (data.status === 'denied') {
          setError('Access request was denied.');
        } else if (data.status === 'draft') {
          setError('Access request is still pending review.');
        } else {
          setError('Access request failed: ' + (data.message || 'Unknown status: ' + data.status));
        }
      } catch (err) {
        console.error('Access request status check error:', err);
        setError('Failed to check access request status: ' + (err instanceof Error ? err.message : String(err)));
      } finally {
        setLoading(false);
      }
    }

    checkStatus();
  }, []);

  const handleLogin = async () => {
    const config = loadConfig();
    if (!config) {
      setError('OAuth configuration not found. Please start the flow again.');
      return;
    }

    // Generate PKCE parameters
    const codeVerifier = generateCodeVerifier();
    const state = generateState();
    const codeChallenge = await generateCodeChallenge(codeVerifier);

    // Update config with PKCE params and final scope
    config.codeVerifier = codeVerifier;
    config.state = state;
    config.scope = scope;
    saveConfig(config);

    // Build and redirect to auth URL
    const authUrl = buildAuthUrl(config, codeChallenge, state);
    window.location.href = authUrl;
  };

  const testState = loading ? 'loading' : error ? 'error' : ready ? 'ready' : 'idle';

  return (
    <div data-testid="div-access-callback" data-test-state={testState} className="w-full max-w-2xl py-8 px-4">
      {loading && (
        <Card>
          <CardContent className="pt-6">
            <div data-testid="access-callback-loading" className="flex items-center justify-center gap-2 py-4 text-muted-foreground">
              <div className="h-4 w-4 border-2 border-current border-t-transparent rounded-full animate-spin" />
              <span className="text-sm italic">Checking access request status...</span>
            </div>
          </CardContent>
        </Card>
      )}

      {error && (
        <Card>
          <CardHeader>
            <CardTitle>Access Request</CardTitle>
          </CardHeader>
          <CardContent>
            <div data-testid="error-section" className="rounded-md border border-destructive/30 bg-destructive/5 p-4">
              <h3 className="font-semibold text-destructive mb-1">Error</h3>
              <p className="text-sm text-destructive">{error}</p>
            </div>
          </CardContent>
        </Card>
      )}

      {ready && (
        <Card>
          <CardHeader>
            <CardTitle>Access Request Approved</CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            <ScopeDisplay
              resourceScope={resourceScope}
              accessRequestScope={accessRequestScope}
            />

            <div className="space-y-2">
              <Label htmlFor="scope">Scope (editable)</Label>
              <Input
                id="scope"
                data-testid="input-scope"
                value={scope}
                onChange={(e) => setScope(e.target.value)}
              />
            </div>

            <Button
              data-testid="btn-login"
              data-test-state="login"
              onClick={handleLogin}
            >
              Login
            </Button>
          </CardContent>
        </Card>
      )}
    </div>
  );
}
