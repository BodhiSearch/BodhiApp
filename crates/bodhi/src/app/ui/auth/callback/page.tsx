'use client';

import { useEffect, useState, useRef, Suspense } from 'react';
import { useOAuthCallback } from '@/hooks/useAuth';
import { AuthCallbackRequest } from '@bodhiapp/ts-client';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { BodhiLogoImage } from '@/app/ui/setup/BodhiLogo';
import { handleSmartRedirect } from '@/lib/utils';
import { useRouter, useSearchParams } from 'next/navigation';

function AuthCallbackContent() {
  const [error, setError] = useState<string | null>(null);
  const [processing, setProcessing] = useState(true);
  const hasProcessedRef = useRef(false);
  const router = useRouter();
  const searchParams = useSearchParams();

  const { mutate: submitCallback, isLoading } = useOAuthCallback({
    onSuccess: (response) => {
      // Handle redirect based on backend response
      const location = response.data?.location;
      if (!location) {
        setError('Redirect URL not found in response. Please try again.');
        setProcessing(false);
        return;
      }

      // Handle redirect using smart URL detection
      handleSmartRedirect(location, router);
    },
    onError: (message) => {
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

    // Extract all OAuth query parameters using Next.js useSearchParams
    const params: AuthCallbackRequest = {};
    searchParams?.forEach((value, key) => {
      // All parameters are flattened in the generated type
      params[key] = value;
    });

    // Submit all OAuth callback parameters to backend
    submitCallback(params);
  }, [submitCallback, searchParams]);

  const handleRetry = () => {
    hasProcessedRef.current = false;
    setError(null);
    setProcessing(true);
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
  return (
    <Suspense
      fallback={
        <main className="min-h-screen bg-background p-4 md:p-8" data-testid="oauth-callback-page">
          <div className="mx-auto max-w-4xl space-y-8 p-4 md:p-8">
            <BodhiLogoImage />
            <Card>
              <CardHeader>
                <CardTitle className="text-center">Loading...</CardTitle>
              </CardHeader>
              <CardContent className="text-center space-y-4">
                <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-primary mx-auto"></div>
                <p className="text-muted-foreground">Initializing authentication...</p>
              </CardContent>
            </Card>
          </div>
        </main>
      }
    >
      <AuthCallbackContent />
    </Suspense>
  );
}
