'use client';

import { Button } from '@/components/ui/button';
import { useLogoutHandler, useOAuthInitiate } from '@/hooks/useAuth';
import { useUser } from '@/hooks/useUsers';
import { useToastMessages } from '@/hooks/use-toast-messages';
import { ROUTE_DEFAULT, ROUTE_LOGIN } from '@/lib/constants';
import { handleSmartRedirect } from '@/lib/utils';
import { useState } from 'react';
import { redirect, useRouter } from 'next/navigation';

export function LoginMenu() {
  const { data: userInfo, isLoading: userLoading } = useUser();
  const { showError } = useToastMessages();
  const [error, setError] = useState<string | null>(null);
  const [redirecting, setRedirecting] = useState(false);
  const router = useRouter();

  const { logout, isLoading: isLoggingOut } = useLogoutHandler({
    onSuccess: (response) => {
      const redirectUrl = response.data?.location || ROUTE_DEFAULT;

      // Check if URL is internal (starts with '/') or same origin
      if (redirectUrl.startsWith('/')) {
        router.push(redirectUrl);
      } else {
        // For external URLs, use redirect
        redirect(redirectUrl);
      }
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
      redirect(ROUTE_LOGIN);
    },
  });

  const { mutate: initiateOAuth, isLoading } = useOAuthInitiate({
    onSuccess: (response) => {
      // Clear any previous errors and set redirecting state
      setError(null);
      setRedirecting(true);

      // Handle redirect based on backend response
      const location = response.data?.location;
      if (!location) {
        setError('Auth URL not found in response. Please try again.');
        setRedirecting(false);
        return;
      }

      // Handle redirect using smart URL detection
      handleSmartRedirect(location, router);
    },
    onError: (message) => {
      setError(message);
      setRedirecting(false);
    },
  });

  const handleOAuthInitiate = () => {
    setError(null); // Clear any previous errors
    initiateOAuth();
  };

  const isLoginButtonDisabled = isLoading || redirecting;

  if (userLoading) {
    return null;
  }

  if (userInfo?.auth_status === 'logged_in') {
    return (
      <div className="p-2 space-y-1.5 text-center" data-testid="login-menu-logged-in">
        <p className="text-xs text-muted-foreground">Logged in as {userInfo.username}</p>
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
        onClick={handleOAuthInitiate}
        disabled={isLoginButtonDisabled}
      >
        {isLoading ? 'Initiating...' : redirecting ? 'Redirecting...' : 'Login'}
      </Button>
    </div>
  );
}
