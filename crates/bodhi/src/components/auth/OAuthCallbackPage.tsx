import { AuthCard } from '@/components/AuthCard';
import { useOAuthCallback, oauthUtils } from '@/hooks/useOAuth';
import { useRouter } from '@/lib/navigation';
import { ROUTE_AUTH_CALLBACK, ROUTE_CHAT, ROUTE_LOGIN } from '@/lib/constants';
import { useEffect, useState, useRef } from 'react';

export function OAuthCallbackPage() {
  const router = useRouter();
  const [error, setError] = useState<string | null>(null);
  const [processed, setProcessed] = useState(false);
  const processingRef = useRef(false);

  const oauthCallback = useOAuthCallback({
    onSuccess: (location?: string) => {
      setProcessed(true);
      // Handle redirect based on backend response or fallback
      if (location) {
        window.location.href = location;
      } else {
        // Fallback redirect to default chat page
        window.location.href = ROUTE_CHAT;
      }
    },
    onError: (message: string) => {
      setError(message);
      setProcessed(true);
    },
  });

  useEffect(() => {
    // Prevent multiple processing of the same callback
    if (processed || processingRef.current) return;

    const handleCallback = () => {
      try {
        // Mark as processing immediately
        processingRef.current = true;

        const currentUrl = window.location.href;

        // Only process if this looks like a callback URL with parameters
        if (
          !currentUrl.includes(ROUTE_AUTH_CALLBACK) ||
          !window.location.search
        ) {
          setError('Invalid callback URL');
          setProcessed(true);
          processingRef.current = false;
          return;
        }

        const params = oauthUtils.extractOAuthParams(currentUrl);

        // Only process if we have OAuth parameters (code or error)
        if (params.code || params.error) {
          oauthCallback.mutate(params);
        } else {
          setError('Invalid callback URL');
          setProcessed(true);
          processingRef.current = false;
        }
      } catch (err) {
        console.error('URL parsing error:', err);
        setError('Invalid callback URL');
        setProcessed(true);
        processingRef.current = false;
      }
    };

    // Handle errors synchronously
    handleCallback();
  }, [processed, oauthCallback]);

  const handleTryAgain = () => {
    router.push(ROUTE_LOGIN);
  };

  if (error) {
    return (
      <div className="pt-12 sm:pt-16" data-testid="oauth-callback-error">
        <AuthCard
          title="Authentication Failed"
          description={error}
          actions={[
            {
              label: 'Try Login Again',
              onClick: handleTryAgain,
              variant: 'default',
            },
          ]}
        />
      </div>
    );
  }

  return (
    <div className="pt-12 sm:pt-16" data-testid="oauth-callback-loading">
      <AuthCard title="Completing Authentication" isLoading={true} />
    </div>
  );
}

export default OAuthCallbackPage;
