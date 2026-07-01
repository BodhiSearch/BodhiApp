import React, { useEffect, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { Button, Input, Label, Checkbox, Textarea } from '@/components/ui';
import type { OAuthConfig } from '@/context/AuthContext';
import { useAuth } from '@/context/AuthContext';
import { requestAccess } from '@/lib/api';
import { saveConfig, loadConfig } from '@/lib/storage';
import {
  buildAuthUrl,
  buildReviewRedirect,
  exchangeCodeForToken,
  generateCodeChallenge,
  generateCodeVerifier,
  generateState,
} from '@/lib/oauth';
import type { OAuthResult } from '@/pages/OAuthCallbackPage';
import { OAUTH_RESULT_KEY } from '@/pages/OAuthCallbackPage';

interface ConfigFormProps {
  initialError?: string | null;
}

export function ConfigForm({ initialError }: ConfigFormProps) {
  const navigate = useNavigate();
  const { setToken } = useAuth();
  const [bodhiServerUrl, setBodhiServerUrl] = useState('http://localhost:1135');
  const [authServerUrl, setAuthServerUrl] = useState(import.meta.env.INTEG_TEST_MAIN_AUTH_URL || 'https://main-id.getbodhi.app');
  const [realm, setRealm] = useState(import.meta.env.INTEG_TEST_AUTH_REALM || 'bodhi');
  const [clientId, setClientId] = useState(import.meta.env.INTEG_TEST_APP_CLIENT_ID || 'client-bodhi-dev-console');
  const [isConfidential, setIsConfidential] = useState(false);
  const [clientSecret, setClientSecret] = useState(import.meta.env.INTEG_TEST_DEV_CONSOLE_CLIENT_SECRET || 'change-me');
  const [redirectUri, setRedirectUri] = useState(`${window.location.origin}/callback`);
  const [scope, setScope] = useState('openid profile email roles');
  const [requestedRole, setRequestedRole] = useState('scope_user_user');
  const [flowType, setFlowType] = useState<'redirect' | 'popup'>('redirect');
  const [requested, setRequested] = useState('{"version":"1","mcp_servers":[{"url":"https://mcp.example.com/mcp"}]}');

  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(initialError || null);

  // Popup flow: the popup's /callback posts the OAuth result (code/error) back here; the opener
  // owns the PKCE verifier and does the token exchange, so the verifier never leaves this window.
  useEffect(() => {
    const onMessage = async (event: MessageEvent) => {
      if (event.origin !== window.location.origin) return;
      const result = (event.data as { [OAUTH_RESULT_KEY]?: OAuthResult })?.[OAUTH_RESULT_KEY];
      if (!result) return;
      if (result.type === 'error') {
        setLoading(false);
        setError(`OAuth Error: ${result.error} (source: ${result.errorSource ?? 'keycloak'})`);
        return;
      }
      const config = loadConfig();
      if (!config || result.state !== config.state) {
        setLoading(false);
        setError('OAuth Error: state mismatch on popup result');
        return;
      }
      try {
        const tokenData = await exchangeCodeForToken(result.code, config);
        setToken(tokenData.access_token);
        navigate('/rest', { replace: true });
      } catch (err) {
        setLoading(false);
        setError('Token exchange failed: ' + (err instanceof Error ? err.message : String(err)));
      }
    };
    window.addEventListener('message', onMessage);
    return () => window.removeEventListener('message', onMessage);
  }, [navigate, setToken]);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);
    await handleRequestAccess();
  };

  const handleRequestAccess = async () => {
    if (!bodhiServerUrl || !authServerUrl || !realm || !clientId || !redirectUri) {
      setError('Please fill in all required fields');
      return;
    }

    let parsedRequested: Record<string, unknown> | undefined;
    if (requested.trim().length > 0) {
      try {
        parsedRequested = JSON.parse(requested.trim());
      } catch (err) {
        setError('Invalid JSON in Requested Resources: ' + (err instanceof Error ? err.message : String(err)));
        return;
      }
    }

    // Generate PKCE + state up front and build the full Keycloak authorize URL — Bodhi appends
    // the dynamic scope on approval and redirects straight to Keycloak (single-step flow).
    const codeVerifier = generateCodeVerifier();
    const state = generateState();
    const codeChallenge = await generateCodeChallenge(codeVerifier);

    const config: OAuthConfig = {
      bodhiServerUrl,
      authServerUrl,
      realm,
      clientId,
      isConfidential,
      clientSecret,
      redirectUri,
      scope,
      requested,
      codeVerifier,
      state,
    };
    saveConfig(config);

    setLoading(true);

    try {
      const data = await requestAccess(bodhiServerUrl, {
        app_client_id: clientId,
        requested_role: requestedRole,
        requested: parsedRequested ?? { version: '1' },
      });

      if (data.status !== 'draft') {
        throw new Error('Unexpected response status: ' + data.status);
      }

      const authUrl = buildAuthUrl(config, codeChallenge, state);
      const reviewTarget = buildReviewRedirect(data.review_url, authUrl, redirectUri);

      if (flowType === 'popup') {
        window.open(reviewTarget, '_bodhi_review', 'width=600,height=700');
      } else {
        window.location.href = reviewTarget;
      }
    } catch (err) {
      console.error('Access request error:', err);
      setError('Access request failed: ' + (err instanceof Error ? err.message : String(err)));
      setLoading(false);
    }
  };

  const formState = loading ? 'loading' : error ? 'error' : 'request-access';

  return (
    <div data-testid="div-config-form" data-test-state={formState} className="space-y-6">
      <form onSubmit={handleSubmit} className="space-y-4">
        <div className="space-y-2">
          <Label htmlFor="bodhi-server-url">BodhiApp Server URL</Label>
          <Input
            id="bodhi-server-url"
            data-testid="input-bodhi-server-url"
            value={bodhiServerUrl}
            onChange={(e) => setBodhiServerUrl(e.target.value)}
            placeholder="http://localhost:1135"
          />
        </div>

        <div className="space-y-2">
          <Label htmlFor="auth-server-url">Authorization Server URL</Label>
          <Input
            id="auth-server-url"
            data-testid="input-auth-server-url"
            value={authServerUrl}
            onChange={(e) => setAuthServerUrl(e.target.value)}
            placeholder="https://main-id.getbodhi.app"
          />
        </div>

        <div className="grid grid-cols-2 gap-4">
          <div className="space-y-2">
            <Label htmlFor="realm">Realm</Label>
            <Input
              id="realm"
              data-testid="input-realm"
              value={realm}
              onChange={(e) => setRealm(e.target.value)}
              placeholder="bodhi"
            />
          </div>

          <div className="space-y-2">
            <Label htmlFor="client-id">Client ID</Label>
            <Input
              id="client-id"
              data-testid="input-client-id"
              value={clientId}
              onChange={(e) => setClientId(e.target.value)}
              placeholder="client-bodhi-dev-console"
            />
          </div>
        </div>

        <div className="flex items-center space-x-2">
          <Checkbox
            id="confidential-client"
            data-testid="toggle-confidential"
            checked={isConfidential}
            onChange={(e) => setIsConfidential(e.target.checked)}
          />
          <Label htmlFor="confidential-client">Confidential Client (requires client secret)</Label>
        </div>

        <div className="space-y-2">
          <Label htmlFor="client-secret">Client Secret</Label>
          <Input
            id="client-secret"
            data-testid="input-client-secret"
            value={clientSecret}
            onChange={(e) => setClientSecret(e.target.value)}
            placeholder="change-me"
            disabled={!isConfidential}
          />
        </div>

        <div className="space-y-2">
          <Label htmlFor="redirect-uri">Redirect URI</Label>
          <Input
            id="redirect-uri"
            data-testid="input-redirect-uri"
            value={redirectUri}
            onChange={(e) => setRedirectUri(e.target.value)}
            placeholder={`${window.location.origin}/callback`}
          />
        </div>

        <div className="space-y-2">
          <Label htmlFor="scope">Scope</Label>
          <Input
            id="scope"
            data-testid="input-scope"
            value={scope}
            onChange={(e) => setScope(e.target.value)}
            placeholder="openid profile email roles"
          />
        </div>

        <div className="space-y-2">
          <Label htmlFor="requested-role">Requested Role</Label>
          <select
            id="requested-role"
            data-testid="select-requested-role"
            value={requestedRole}
            onChange={(e) => setRequestedRole(e.target.value)}
            className="w-full rounded-md border border-input bg-background px-3 py-2 text-sm"
          >
            <option value="scope_user_user">scope_user_user (User)</option>
            <option value="scope_user_power_user">scope_user_power_user (Power User)</option>
          </select>
        </div>

        <div className="space-y-2">
          <Label htmlFor="flow-type">Flow Type</Label>
          <select
            id="flow-type"
            data-testid="select-flow-type"
            value={flowType}
            onChange={(e) => setFlowType(e.target.value as 'redirect' | 'popup')}
            className="w-full rounded-md border border-input bg-background px-3 py-2 text-sm"
          >
            <option value="redirect">redirect (same tab)</option>
            <option value="popup">popup (new window)</option>
          </select>
        </div>

        <div className="space-y-2">
          <Label htmlFor="requested">Requested Resources (JSON)</Label>
          <Textarea
            id="requested"
            data-testid="input-requested"
            value={requested}
            onChange={(e) => setRequested(e.target.value)}
            placeholder='{"version":"1","mcp_servers":[{"url":"https://mcp.example.com/mcp"}]}'
            rows={3}
          />
        </div>

        <div className="pt-2">
          <Button
            type="submit"
            data-testid="btn-request-access"
            data-test-state="request-access"
            disabled={loading}
          >
            Request Access & Login
          </Button>
        </div>
      </form>

      {loading && (
        <div data-testid="access-request-loading" className="flex items-center justify-center gap-2 py-4 text-muted-foreground">
          <div className="h-4 w-4 border-2 border-current border-t-transparent rounded-full animate-spin" />
          <span className="text-sm italic">Requesting access...</span>
        </div>
      )}

      {error && (
        <div data-testid="error-section" className="rounded-md border border-destructive/30 bg-destructive/5 p-4">
          <h3 className="font-semibold text-destructive mb-1">Error</h3>
          <p className="text-sm text-destructive">{error}</p>
        </div>
      )}
    </div>
  );
}
