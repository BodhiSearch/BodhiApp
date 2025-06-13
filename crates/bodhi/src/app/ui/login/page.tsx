'use client';

import AppInitializer from '@/components/AppInitializer';
import { AuthCard } from '@/components/AuthCard';
import { useLogoutHandler } from '@/hooks/useLogoutHandler';
import { useOAuthInitiate } from '@/hooks/useOAuth';
import { useUser } from '@/hooks/useQuery';
import { ROUTE_DEFAULT } from '@/lib/constants';
import { useState } from 'react';

export function LoginContent() {
  const { data: userInfo, isLoading: userLoading } = useUser();
  const { logout, isLoading: isLoggingOut } = useLogoutHandler();
  const [error, setError] = useState<string | null>(null);

  const oauthInitiate = useOAuthInitiate({
    onSuccess: (response) => {
      // Handle redirect based on backend response
      // 401 response: auth_url in body (login required)
      // 303 response: Location header (already authenticated)
      if (response.data?.auth_url) {
        window.location.href = response.data.auth_url;
      } else if (response.headers?.location) {
        window.location.href = response.headers.location;
      }
    },
    onError: (message) => {
      setError(message);
    },
  });

  if (userLoading) {
    return <AuthCard title="Loading..." isLoading={true} />;
  }

  if (userInfo?.logged_in) {
    return (
      <AuthCard
        title="Welcome"
        description={`You are logged in as ${userInfo.email}`}
        actions={[
          {
            label: 'Go to Home',
            href: ROUTE_DEFAULT,
            variant: 'secondary',
          },
          {
            label: isLoggingOut ? 'Logging out...' : 'Log Out',
            onClick: () => logout(),
            disabled: isLoggingOut,
            variant: 'destructive',
          },
        ]}
      />
    );
  }

  return (
    <AuthCard
      title="Login"
      description={
        <div className="space-y-4">
          <p>Login to use the Bodhi App</p>
          {error && (
            <p className="text-destructive text-sm">{error}</p>
          )}
        </div>
      }
      actions={[
        {
          label: oauthInitiate.isLoading ? 'Redirecting...' : 'Login',
          onClick: () => oauthInitiate.mutate(),
          disabled: oauthInitiate.isLoading,
        },
      ]}
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
