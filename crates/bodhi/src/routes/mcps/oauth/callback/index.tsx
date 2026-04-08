import { createFileRoute, useNavigate, useSearch } from '@tanstack/react-router';
import { useEffect, useState } from 'react';

import { Loader2 } from 'lucide-react';
import { z } from 'zod';

import AppInitializer from '@/components/AppInitializer';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { useOAuthTokenExchange } from '@/hooks/mcps';
import { OAUTH_FORM_STORAGE_KEY } from '@/stores/mcpFormStore';

export const Route = createFileRoute('/mcps/oauth/callback/')({
  validateSearch: z.object({
    code: z.string().optional(),
    state: z.string().optional(),
    error: z.string().optional(),
    error_description: z.string().optional(),
  }),
  component: McpOAuthCallbackPage,
});

function OAuthCallbackContent() {
  const navigate = useNavigate();
  const search = useSearch({ from: '/mcps/oauth/callback/' });
  const code = search.code;
  const state = search.state;
  const error = search.error;
  const errorDescription = search.error_description;

  const [status, setStatus] = useState<'loading' | 'success' | 'error'>('loading');
  const [errorMessage, setErrorMessage] = useState('');
  const [exchanged, setExchanged] = useState(false);

  const tokenExchangeMutation = useOAuthTokenExchange();

  useEffect(() => {
    if (exchanged) return;

    if (error) {
      setStatus('error');
      setErrorMessage(errorDescription || error);
      return;
    }

    if (!code) {
      setStatus('error');
      setErrorMessage('No authorization code received');
      return;
    }

    if (!state) {
      setStatus('error');
      setErrorMessage('Missing state parameter. Please start the OAuth flow again.');
      return;
    }

    const saved = sessionStorage.getItem(OAUTH_FORM_STORAGE_KEY);
    if (!saved) {
      setStatus('error');
      setErrorMessage('Session expired. Please start the OAuth flow again.');
      return;
    }

    let formState;
    try {
      formState = JSON.parse(saved);
    } catch {
      setStatus('error');
      setErrorMessage('Corrupt session data. Please start the OAuth flow again.');
      return;
    }

    const configId = formState.selected_auth_config_id;
    if (!configId) {
      setStatus('error');
      setErrorMessage('Missing OAuth config. Please start the OAuth flow again.');
      return;
    }

    const mcpId = formState.mcp_id || undefined;

    setExchanged(true);
    const redirectUri = `${window.location.origin}/ui/mcps/oauth/callback`;

    tokenExchangeMutation.mutate(
      { id: configId, mcp_id: mcpId, code, redirect_uri: redirectUri, state },
      {
        onSuccess: (response) => {
          const tokenId = response.data.id;
          formState.oauth_token_id = tokenId;
          sessionStorage.setItem(OAUTH_FORM_STORAGE_KEY, JSON.stringify(formState));
          setStatus('success');
          const returnUrl = formState.return_url || '/mcps/new/';
          navigate({ to: returnUrl });
        },
        onError: (err) => {
          sessionStorage.removeItem(OAUTH_FORM_STORAGE_KEY);
          setStatus('error');
          setErrorMessage(err?.response?.data?.error?.message || 'Failed to exchange authorization code');
        },
      }
    );
  }, [code, state, error, errorDescription, exchanged, navigate]); // eslint-disable-line react-hooks/exhaustive-deps

  return (
    <div className="container mx-auto p-4 max-w-md" data-testid="oauth-callback-page">
      <Card>
        <CardHeader>
          <CardTitle>OAuth Authorization</CardTitle>
          <CardDescription>
            {status === 'loading' && 'Completing OAuth authorization...'}
            {status === 'success' && 'Authorization successful'}
            {status === 'error' && 'Authorization failed'}
          </CardDescription>
        </CardHeader>
        <CardContent>
          {status === 'loading' && (
            <div className="flex items-center justify-center py-8" data-testid="oauth-callback-loading">
              <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
            </div>
          )}

          {status === 'success' && (
            <div className="text-center py-4" data-testid="oauth-callback-success">
              <p className="text-green-600 dark:text-green-400 mb-2">OAuth authorization completed successfully.</p>
              <p className="text-sm text-muted-foreground">Redirecting back to form...</p>
            </div>
          )}

          {status === 'error' && (
            <div className="space-y-4" data-testid="oauth-callback-error">
              <p className="text-destructive">{errorMessage}</p>
              <Button
                variant="outline"
                onClick={() => {
                  let returnUrl = '/mcps/new/';
                  try {
                    const saved = sessionStorage.getItem(OAUTH_FORM_STORAGE_KEY);
                    if (saved) {
                      const parsed = JSON.parse(saved);
                      if (parsed.return_url) returnUrl = parsed.return_url;
                    }
                  } catch {
                    /* ignore parse errors */
                  }
                  navigate({ to: returnUrl });
                }}
                data-testid="oauth-callback-back"
              >
                Back to form
              </Button>
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  );
}

export default function McpOAuthCallbackPage() {
  return (
    <AppInitializer authenticated={true} allowedStatus="ready">
      <OAuthCallbackContent />
    </AppInitializer>
  );
}
