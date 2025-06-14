'use client';

import { Button } from '@/components/ui/button';
import { useLogoutHandler } from '@/hooks/useLogoutHandler';
import { useOAuthInitiate } from '@/hooks/useOAuth';
import { useUser } from '@/hooks/useQuery';
import { useState } from 'react';

export function LoginMenu() {
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
    return null;
  }

  if (userInfo?.logged_in) {
    return (
      <div className="p-2 space-y-1.5 text-center" data-testid="login-menu-logged-in">
        <p className="text-xs text-muted-foreground">Logged in as {userInfo.email}</p>
        <Button
          variant="destructive"
          className="w-full border border-destructive"
          onClick={() => logout()}
          disabled={isLoggingOut}
        >
          {isLoggingOut ? 'Logging out...' : 'Log Out'}
        </Button>
      </div>
    );
  }

  return (
    <div className="p-2" data-testid="login-menu-default">
      {error && <p className="text-destructive text-xs mb-2 text-center">{error}</p>}
      <Button
        variant="default"
        className="w-full border border-primary"
        onClick={() => oauthInitiate.mutate()}
        disabled={oauthInitiate.isLoading}
      >
        {oauthInitiate.isLoading ? 'Redirecting...' : 'Login'}
      </Button>
    </div>
  );
}
