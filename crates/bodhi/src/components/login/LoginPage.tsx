import AppInitializer from '@/components/AppInitializer';
import { AuthCard } from '@/components/AuthCard';
import { useOAuthInitiate } from '@/hooks/useOAuth';
import { useState } from 'react';

export function LoginContent() {
  const [error, setError] = useState<string | null>(null);

  const oauthInitiate = useOAuthInitiate({
    onSuccess: (response) => {
      // Handle redirect to OAuth provider
      window.location.href = response.auth_url;
    },
    onError: (message: string) => {
      setError(message);
    },
  });

  const handleLogin = () => {
    setError(null); // Clear any previous errors
    oauthInitiate.mutate();
  };

  if (error) {
    return (
      <AuthCard
        title="Authentication Error"
        description={error}
        actions={[
          {
            label: 'Try Again',
            onClick: handleLogin,
            variant: 'default',
            disabled: oauthInitiate.isLoading,
          },
        ]}
      />
    );
  }

  return (
    <AuthCard
      title="Welcome to Bodhi"
      description="Sign in to access your AI assistant"
      actions={[
        {
          label: 'Sign In',
          onClick: handleLogin,
          variant: 'default',
          disabled: oauthInitiate.isLoading,
        },
      ]}
      isLoading={oauthInitiate.isLoading}
    />
  );
}

export default function LoginPage() {
  return (
    <AppInitializer allowedStatus="ready" authenticated={false}>
      <div className="pt-12 sm:pt-16" data-testid="login-page">
        <LoginContent />
      </div>
    </AppInitializer>
  );
}
