import { useEffect, useState, useRef } from 'react';

import { AuthCallbackRequest } from '@bodhiapp/ts-client';
import { useNavigate, useSearch } from '@tanstack/react-router';

import { BodhiLogoImage } from '@/app/setup/BodhiLogo';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { useOAuthCallback } from '@/hooks/auth';
import { handleSmartRedirect } from '@/lib/utils';

function AuthCallbackContent() {
  const [error, setError] = useState<string | null>(null);
  const [processing, setProcessing] = useState(true);
  const [retryCount, setRetryCount] = useState(0);
  const hasProcessedRef = useRef(false);
  const navigate = useNavigate();
  const search = useSearch({ strict: false }) as Record<string, string | undefined>;

  const { mutate: submitCallback, isPending: isLoading } = useOAuthCallback({
    onSuccess: (response) => {
      // Check for stored return URL (e.g., from access request review page)
      const returnUrl = sessionStorage.getItem('bodhi-return-url');
      if (returnUrl) {
        sessionStorage.removeItem('bodhi-return-url');
        handleSmartRedirect(returnUrl, navigate);
        return;
      }

      // Handle redirect based on backend response
      const location = response.data?.location;
      if (!location) {
        setError('Redirect URL not found in response. Please try again.');
        setProcessing(false);
        return;
      }

      // Handle redirect using smart URL detection
      handleSmartRedirect(location, navigate);
    },
    onError: (message) => {
      hasProcessedRef.current = false;
      setError(message);
      setProcessing(false);
    },
  });

  useEffect(() => {
    // Prevent duplicate processing
    if (hasProcessedRef.current) {
      return;
    }

    hasProcessedRef.current = true;

    // Extract all OAuth query parameters
    const params: AuthCallbackRequest = {};
    Object.entries(search).forEach(([key, value]) => {
      params[key] = value as string;
    });

    // Submit all OAuth callback parameters to backend
    submitCallback(params);
  }, [submitCallback, search, retryCount]);

  const handleRetry = () => {
    setError(null);
    setProcessing(true);
    setRetryCount((c) => c + 1);
  };

  if (processing || isLoading) {
    return (
      <main className="min-h-screen bg-background p-4 md:p-8" data-testid="oauth-callback-page">
        <div className="mx-auto max-w-4xl space-y-8 p-4 md:p-8">
          <BodhiLogoImage />
          <Card>
            <CardHeader>
              <CardTitle className="text-center">Processing Login...</CardTitle>
            </CardHeader>
            <CardContent className="text-center space-y-4">
              <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-primary mx-auto"></div>
              <p className="text-muted-foreground">Please wait while we complete your login...</p>
            </CardContent>
          </Card>
        </div>
      </main>
    );
  }

  return (
    <main className="min-h-screen bg-background p-4 md:p-8" data-testid="oauth-callback-page">
      <div className="mx-auto max-w-4xl space-y-8 p-4 md:p-8">
        <BodhiLogoImage />
        <Card>
          <CardHeader>
            <CardTitle className="text-center">Login Error</CardTitle>
          </CardHeader>
          <CardContent className="text-center space-y-4">
            <p className="text-destructive">{error}</p>
            <Button onClick={handleRetry} variant="outline" disabled={isLoading}>
              Try Again
            </Button>
          </CardContent>
        </Card>
      </div>
    </main>
  );
}

export default function AuthCallbackPage() {
  return <AuthCallbackContent />;
}
