'use client';

import { AuthCard } from '@/components/AuthCard';
import { extractOAuthParams, useOAuthCallback } from '@/hooks/useOAuth';
import { ROUTE_LOGIN } from '@/lib/constants';
import { useEffect, useState } from 'react';

export function OAuthCallbackContent() {
  const [error, setError] = useState<string | null>(null);
  const [isProcessing, setIsProcessing] = useState(true);

  const oauthCallback = useOAuthCallback({
    onSuccess: (response) => {
      // Handle redirect based on backend response
      const location = response.headers.location;
      if (location) {
        window.location.href = location;
      } else {
        setIsProcessing(false);
      }
    },
    onError: (message) => {
      setError(message);
      setIsProcessing(false);
    },
  });

  useEffect(() => {
    // Extract ALL query parameters from current URL - let backend decide what's valid
    const currentUrl = window.location.href;
    const oauthParams = extractOAuthParams(currentUrl);

    // Send all parameters to backend without any frontend validation
    oauthCallback.mutate(oauthParams);
  }, [oauthCallback]);

  if (error) {
    return (
      <AuthCard
        title="Authentication Failed"
        description={
          <div className="space-y-4">
            <p className="text-destructive">{error}</p>
            <p className="text-sm text-muted-foreground">
              Please try logging in again. If the problem persists, contact support.
            </p>
          </div>
        }
        actions={[
          {
            label: 'Try Again',
            href: ROUTE_LOGIN,
            variant: 'default',
          },
        ]}
      />
    );
  }

  return (
    <AuthCard
      title="Completing Authentication"
      description="Please wait while we complete your authentication..."
      isLoading={isProcessing}
    />
  );
}

export default function OAuthCallbackPage() {
  return (
    <div className="pt-12 sm:pt-16" data-testid="oauth-callback-page">
      <OAuthCallbackContent />
    </div>
  );
}