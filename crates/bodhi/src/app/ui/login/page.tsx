'use client';

import AppInitializer from '@/components/AppInitializer';
import { AuthCard } from '@/components/AuthCard';
import { useLogoutHandler } from '@/hooks/useLogoutHandler';
import { useOAuthInitiate } from '@/hooks/useOAuth';
import { useUser } from '@/hooks/useQuery';
import { useToastMessages } from '@/hooks/use-toast-messages';
import { ROUTE_DEFAULT, ROUTE_LOGIN } from '@/lib/constants';
import { handleSmartRedirect } from '@/lib/utils';
import { useState } from 'react';
import { useRouter, redirect } from 'next/navigation';

export function LoginContent() {
  const { data: userInfo, isLoading: userLoading } = useUser();
  const { showError } = useToastMessages();
  const [error, setError] = useState<string | null>(null);
  const [redirecting, setRedirecting] = useState(false);
  const router = useRouter();

  const { logout, isLoading: isLoggingOut } = useLogoutHandler({
    onSuccess: (response) => {
      const redirectUrl = response.data?.location || ROUTE_DEFAULT;
      if (redirectUrl.startsWith('http://') || redirectUrl.startsWith('https://')) {
        redirect(redirectUrl);
      } else {
        router.push(redirectUrl);
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
      router.push(ROUTE_LOGIN);
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
    return <AuthCard title="Loading..." isLoading={true} />;
  }

  if (userInfo?.auth_status === 'logged_in') {
    return (
      <AuthCard
        title="Welcome"
        description={`You are logged in as ${userInfo.username}`}
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
          {error && <p className="text-destructive text-sm">{error}</p>}
        </div>
      }
      actions={[
        {
          label: isLoading ? 'Initiating...' : redirecting ? 'Redirecting...' : 'Login',
          onClick: handleOAuthInitiate,
          disabled: isLoginButtonDisabled,
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
