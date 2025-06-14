'use client';

import { Button } from '@/components/ui/button';
import { useLogoutHandler } from '@/hooks/useLogoutHandler';
import { useOAuthInitiate } from '@/hooks/useOAuth';
import { useUser } from '@/hooks/useQuery';
import { useToastMessages } from '@/hooks/use-toast-messages';
import { ROUTE_DEFAULT } from '@/lib/constants';
import { useState } from 'react';

export function LoginMenu() {
  const { data: userInfo, isLoading: userLoading } = useUser();
  const { showError } = useToastMessages();
  const [error, setError] = useState<string | null>(null);

  const { logout, isLoading: isLoggingOut } = useLogoutHandler({
    onSuccess: (response) => {
      const redirectUrl = response.headers?.location || ROUTE_DEFAULT;
      window.location.href = redirectUrl;
    },
    onError: (message) => {
      // Reset local storage and cookies on logout failure
      localStorage.clear();
      sessionStorage.clear();
      // Clear all cookies by setting them to expire
      document.cookie.split(';').forEach((c) => {
        const eqPos = c.indexOf('=');
        const name = eqPos > -1 ? c.substr(0, eqPos) : c;
        document.cookie = name + '=;expires=Thu, 01 Jan 1970 00:00:00 GMT;path=/';
      });
      showError('Logout failed', `Message: ${message}. Redirecting to login page.`);
      // Redirect to login page
      window.location.href = '/ui/login';
    },
  });

  const oauthInitiate = useOAuthInitiate({
    onSuccess: (response) => {
      // Handle redirect based on backend response
      // 303 response: Location header (OAuth URL or already authenticated)
      if (response.headers?.location) {
        window.location.href = response.headers.location;
      } else {
        setError('Auth URL not found in response. Please try again.');
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
