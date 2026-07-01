import React, { useEffect, useRef } from 'react';
import { useNavigate, useSearchParams } from 'react-router-dom';
import { Card, CardContent } from '@/components/ui';
import { useAuth } from '@/context/AuthContext';
import { loadConfig } from '@/lib/storage';
import { exchangeCodeForToken } from '@/lib/oauth';

export const OAUTH_RESULT_KEY = 'bodhi_oauth_result';

export type OAuthResult =
  | { type: 'success'; code: string; state: string }
  | { type: 'error'; error: string; errorSource: string | null; errorDescription: string | null };

function isPopup(): boolean {
  return !!window.opener && window.opener !== window;
}

export function OAuthCallbackPage() {
  const navigate = useNavigate();
  const [searchParams] = useSearchParams();
  const { setToken } = useAuth();

  const processedRef = useRef(false);

  useEffect(() => {
    if (processedRef.current) return;
    processedRef.current = true;

    const error = searchParams.get('error');
    const code = searchParams.get('code');
    const state = searchParams.get('state');

    const result: OAuthResult = error
      ? {
          type: 'error',
          error,
          errorSource: searchParams.get('error_source'),
          errorDescription: searchParams.get('error_description'),
        }
      : { type: 'success', code: code ?? '', state: state ?? '' };

    // Popup flow: hand the raw result to the opener (which owns the PKCE verifier) and close.
    if (isPopup()) {
      window.opener.postMessage({ [OAUTH_RESULT_KEY]: result }, window.location.origin);
      window.close();
      return;
    }

    // Redirect flow: this tab owns the verifier — finish here.
    void finishRedirect(result);

    async function finishRedirect(res: OAuthResult) {
      if (res.type === 'error') {
        const params = new URLSearchParams({ error: res.error });
        if (res.errorSource) params.set('error_source', res.errorSource);
        if (res.errorDescription) params.set('error_description', res.errorDescription);
        navigate(`/?${params.toString()}`, { replace: true });
        return;
      }
      const config = loadConfig();
      if (!config) {
        navigate('/?error=no_config&error_description=OAuth+configuration+not+found', { replace: true });
        return;
      }
      if (!res.code || res.state !== config.state) {
        navigate('/?error=invalid_state&error_description=State+mismatch+or+missing+code', { replace: true });
        return;
      }
      try {
        const tokenData = await exchangeCodeForToken(res.code, config);
        setToken(tokenData.access_token);
        window.history.replaceState({}, document.title, window.location.pathname);
        navigate('/rest', { replace: true });
      } catch (err) {
        const message = err instanceof Error ? err.message : String(err);
        navigate(`/?error=token_exchange_failed&error_description=${encodeURIComponent(message)}`, { replace: true });
      }
    }
  }, [navigate, searchParams, setToken]);

  return (
    <div className="w-full max-w-2xl py-8 px-4">
      <Card>
        <CardContent className="pt-6">
          <div className="flex items-center justify-center gap-2 py-4 text-muted-foreground">
            <div className="h-4 w-4 border-2 border-current border-t-transparent rounded-full animate-spin" />
            <span className="text-sm italic">Completing sign-in...</span>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
