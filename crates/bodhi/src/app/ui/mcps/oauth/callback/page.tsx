'use client';

import { useEffect, useState } from 'react';

import { Loader2 } from 'lucide-react';
import { useRouter, useSearchParams } from 'next/navigation';

import AppInitializer from '@/components/AppInitializer';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { useOAuthTokenExchange } from '@/hooks/useMcps';

const OAUTH_FORM_STORAGE_KEY = 'mcp_oauth_form_state';

function OAuthCallbackContent() {
  const router = useRouter();
  const searchParams = useSearchParams();
  const code = searchParams.get('code');
  const state = searchParams.get('state');
  const error = searchParams.get('error');
  const errorDescription = searchParams.get('error_description');

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

    const configId = formState.oauth_config_id;
    const serverId = formState.mcp_server_id;
    if (!configId) {
      setStatus('error');
      setErrorMessage('Missing OAuth config. Please start the OAuth flow again.');
      return;
    }
    if (!serverId) {
      setStatus('error');
      setErrorMessage('Missing server ID. Please start the OAuth flow again.');
      return;
    }

    setExchanged(true);
    const redirectUri = `${window.location.origin}/ui/mcps/oauth/callback`;

    tokenExchangeMutation.mutate(
      { id: configId, serverId, code, redirect_uri: redirectUri, state },
      {
        onSuccess: (response) => {
          const tokenId = response.data.id;
          formState.oauth_token_id = tokenId;
          sessionStorage.setItem(OAUTH_FORM_STORAGE_KEY, JSON.stringify(formState));
          setStatus('success');
          router.push('/ui/mcps/new');
        },
        onError: (err) => {
          setStatus('error');
          setErrorMessage(err?.response?.data?.error?.message || 'Failed to exchange authorization code');
        },
      }
    );
  }, [code, state, error, errorDescription, exchanged, router]); // eslint-disable-line react-hooks/exhaustive-deps

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
              <Button variant="outline" onClick={() => router.push('/ui/mcps/new')} data-testid="oauth-callback-back">
                Back to form
              </Button>
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  );
}

export default function OAuthCallbackPage() {
  return (
    <AppInitializer authenticated={true} allowedStatus="ready">
      <OAuthCallbackContent />
    </AppInitializer>
  );
}
