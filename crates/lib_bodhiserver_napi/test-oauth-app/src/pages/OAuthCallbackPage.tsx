import React, { useEffect, useRef, useState } from 'react';
import { useNavigate, useSearchParams } from 'react-router-dom';
import { Card, CardContent } from '@/components/ui';
import { useAuth } from '@/context/AuthContext';
import { loadConfig } from '@/lib/storage';
import { exchangeCodeForToken } from '@/lib/oauth';
import { saveToken } from '@/lib/storage';

export function OAuthCallbackPage() {
  const navigate = useNavigate();
  const [searchParams] = useSearchParams();
  const { setToken } = useAuth();
  const [status, setStatus] = useState('Exchanging code for token...');

  const callbackProcessedRef = useRef(false);

  useEffect(() => {
    if (callbackProcessedRef.current) return;
    callbackProcessedRef.current = true;

    async function processCallback() {
      const error = searchParams.get('error');
      const errorDescription = searchParams.get('error_description');

      if (error) {
        // Redirect to config page with error info
        const params = new URLSearchParams();
        params.set('error', error);
        if (errorDescription) {
          params.set('error_description', errorDescription);
        }
        navigate(`/?${params.toString()}`, { replace: true });
        return;
      }

      const code = searchParams.get('code');
      const state = searchParams.get('state');

      if (!code || !state) {
        navigate('/?error=missing_params&error_description=Missing+code+or+state+parameter', { replace: true });
        return;
      }

      const config = loadConfig();
      if (!config) {
        navigate('/?error=no_config&error_description=OAuth+configuration+not+found', { replace: true });
        return;
      }

      if (state !== config.state) {
        navigate('/?error=invalid_state&error_description=State+mismatch.+Possible+CSRF+attack.', { replace: true });
        return;
      }

      try {
        const tokenData = await exchangeCodeForToken(code, config);

        // Save token
        saveToken(tokenData.access_token);
        setToken(tokenData.access_token);

        // Clear URL parameters
        window.history.replaceState({}, document.title, window.location.pathname);

        // Navigate to REST page (default landing after login)
        navigate('/rest', { replace: true });
      } catch (err) {
        console.error('Token exchange error:', err);
        const message = err instanceof Error ? err.message : String(err);
        navigate(`/?error=token_exchange_failed&error_description=${encodeURIComponent(message)}`, { replace: true });
      }
    }

    processCallback();
  }, []);

  return (
    <div className="w-full max-w-2xl py-8 px-4">
      <Card>
        <CardContent className="pt-6">
          <div className="flex items-center justify-center gap-2 py-4 text-muted-foreground">
            <div className="h-4 w-4 border-2 border-current border-t-transparent rounded-full animate-spin" />
            <span className="text-sm italic">{status}</span>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
