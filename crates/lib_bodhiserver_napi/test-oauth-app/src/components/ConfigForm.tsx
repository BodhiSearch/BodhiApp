import React, { useState } from 'react';
import { Button, Input, Label, Checkbox, Textarea } from '@/components/ui';
import { ScopeDisplay } from '@/components/ScopeDisplay';
import type { OAuthConfig } from '@/context/AuthContext';
import { requestAccess } from '@/lib/api';
import { saveConfig, loadConfig } from '@/lib/storage';
import {
  generateCodeVerifier,
  generateCodeChallenge,
  generateState,
  buildAuthUrl,
} from '@/lib/oauth';

interface ConfigFormProps {
  initialError?: string | null;
}

export function ConfigForm({ initialError }: ConfigFormProps) {
  const [bodhiServerUrl, setBodhiServerUrl] = useState('http://localhost:1135');
  const [authServerUrl, setAuthServerUrl] = useState(import.meta.env.INTEG_TEST_MAIN_AUTH_URL || 'https://main-id.getbodhi.app');
  const [realm, setRealm] = useState(import.meta.env.INTEG_TEST_AUTH_REALM || 'bodhi');
  const [clientId, setClientId] = useState(import.meta.env.INTEG_TEST_APP_CLIENT_ID || 'client-bodhi-dev-console');
  const [isConfidential, setIsConfidential] = useState(false);
  const [clientSecret, setClientSecret] = useState(import.meta.env.INTEG_TEST_DEV_CONSOLE_CLIENT_SECRET || 'change-me');
  const [redirectUri, setRedirectUri] = useState(`${window.location.origin}/callback`);
  const [scope, setScope] = useState('openid profile email roles');
  const [requestedToolsets, setRequestedToolsets] = useState('[{"toolset_type":"builtin-exa-search"}]');

  const [buttonState, setButtonState] = useState<'request-access' | 'login'>('request-access');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(initialError || null);

  const [resourceScope, setResourceScope] = useState<string | null>(null);
  const [accessRequestScope, setAccessRequestScope] = useState<string | null>(null);
  const [showScopes, setShowScopes] = useState(false);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);

    if (buttonState === 'login') {
      await handleLogin();
    } else {
      await handleRequestAccess();
    }
  };

  const handleRequestAccess = async () => {
    if (!bodhiServerUrl || !authServerUrl || !realm || !clientId || !redirectUri) {
      setError('Please fill in all required fields');
      return;
    }

    // Parse requested toolsets if provided
    let requested: { toolset_types: unknown[] } | undefined;
    if (requestedToolsets.trim()) {
      try {
        const parsed = JSON.parse(requestedToolsets.trim());
        requested = { toolset_types: parsed };
      } catch (err) {
        setError('Invalid JSON in Requested Toolsets: ' + (err instanceof Error ? err.message : String(err)));
        return;
      }
    }

    // Save config to sessionStorage
    const config: OAuthConfig = {
      bodhiServerUrl,
      authServerUrl,
      realm,
      clientId,
      isConfidential,
      clientSecret,
      redirectUri,
      scope,
      requestedToolsets,
    };
    saveConfig(config);

    setLoading(true);

    try {
      const body: {
        app_client_id: string;
        flow_type: string;
        redirect_url: string;
        requested?: { toolset_types: unknown[] };
      } = {
        app_client_id: clientId,
        flow_type: 'redirect',
        redirect_url: window.location.origin + '/access-callback',
      };
      if (requested) {
        body.requested = requested;
      }

      const data = await requestAccess(bodhiServerUrl, body);

      if (data.status === 'approved') {
        // Auto-approved: append resource_scope and show ready-to-login
        let updatedScope = scope;
        if (data.resource_scope) {
          updatedScope = scope + ' ' + data.resource_scope;
        }

        // Update config with approved info
        const storedConfig = loadConfig();
        if (storedConfig) {
          storedConfig.approvedScopes = [data.resource_scope].filter(Boolean);
          storedConfig.accessRequestId = data.id;
          saveConfig(storedConfig);
        }

        setScope(updatedScope);
        setResourceScope(data.resource_scope || null);
        setAccessRequestScope(null);
        setShowScopes(true);
        setButtonState('login');
      } else if (data.status === 'draft') {
        // Needs user review: store id and redirect to review URL
        const storedConfig = loadConfig();
        if (storedConfig) {
          storedConfig.accessRequestId = data.id;
          saveConfig(storedConfig);
        }

        window.location.href = data.review_url;
      } else {
        throw new Error('Unexpected response status: ' + data.status);
      }
    } catch (err) {
      console.error('Access request error:', err);
      setError('Access request failed: ' + (err instanceof Error ? err.message : String(err)));
    } finally {
      setLoading(false);
    }
  };

  const handleLogin = async () => {
    const storedConfig = loadConfig();
    if (!storedConfig) {
      setError('OAuth configuration not found. Please start the flow again.');
      return;
    }

    // Read scope from input (user may have modified it)
    const finalScope = scope;

    // Generate PKCE parameters
    const codeVerifier = generateCodeVerifier();
    const state = generateState();
    const codeChallenge = await generateCodeChallenge(codeVerifier);

    // Update config with PKCE params and final scope
    storedConfig.codeVerifier = codeVerifier;
    storedConfig.state = state;
    storedConfig.scope = finalScope;
    saveConfig(storedConfig);

    // Build and redirect to auth URL
    const authUrl = buildAuthUrl(storedConfig, codeChallenge, state);
    window.location.href = authUrl;
  };

  const formState = loading ? 'loading' : error ? 'error' : buttonState;

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
          <Label htmlFor="requested-toolsets">Requested Toolsets (JSON array)</Label>
          <Textarea
            id="requested-toolsets"
            data-testid="input-requested-toolsets"
            value={requestedToolsets}
            onChange={(e) => setRequestedToolsets(e.target.value)}
            placeholder='[{"toolset_type":"builtin-exa-search"}]'
            rows={3}
          />
        </div>

        {showScopes && (
          <ScopeDisplay
            resourceScope={resourceScope}
            accessRequestScope={accessRequestScope}
          />
        )}

        <div className="pt-2">
          <Button
            type="submit"
            data-testid="btn-request-access"
            data-test-state={buttonState}
            disabled={loading}
          >
            {buttonState === 'login' ? 'Login' : 'Request Access & Login'}
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
