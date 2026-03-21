import { useState } from 'react';

import { useNavigate } from '@tanstack/react-router';

import { Button } from '@/components/ui/button';
import { useToastMessages } from '@/hooks/use-toast-messages';
import { useLogoutHandler, useOAuthInitiate } from '@/hooks/auth';
import { useGetAppInfo } from '@/hooks/info';
import { useGetUser } from '@/hooks/users';
import { ROUTE_DEFAULT, ROUTE_LOGIN } from '@/lib/constants';
import { handleSmartRedirect } from '@/lib/utils';

export function LoginMenu() {
  const { data: appInfo } = useGetAppInfo();
  const { data: userInfo, isLoading: userLoading } = useGetUser();
  const { showError } = useToastMessages();
  const [error, setError] = useState<string | null>(null);
  const [redirecting, setRedirecting] = useState(false);
  const navigate = useNavigate();

  const { logout, isLoading: isLoggingOut } = useLogoutHandler({
    onSuccess: (response) => {
      const redirectUrl = response.data?.location || ROUTE_DEFAULT;
      handleSmartRedirect(redirectUrl, navigate);
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
      handleSmartRedirect(ROUTE_LOGIN, navigate);
    },
  });

  const { mutate: initiateOAuth, isPending: isLoading } = useOAuthInitiate({
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
      handleSmartRedirect(location, navigate);
    },
    onError: (message) => {
      setError(message);
      setRedirecting(false);
    },
  });

  const handleOAuthInitiate = () => {
    setError(null);
    if (!appInfo?.client_id) {
      setError('Client ID is not set. Please check your configuration.');
      return;
    }
    initiateOAuth({ client_id: appInfo.client_id });
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
