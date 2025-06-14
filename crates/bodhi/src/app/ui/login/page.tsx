'use client';

import AppInitializer from '@/components/AppInitializer';
import { AuthCard } from '@/components/AuthCard';
import { useLogoutHandler } from '@/hooks/useLogoutHandler';
import { useOAuthInitiate } from '@/hooks/useOAuth';
import { useUser } from '@/hooks/useQuery';
import { useToastMessages } from '@/hooks/use-toast-messages';
import { ROUTE_DEFAULT, ROUTE_LOGIN } from '@/lib/constants';
import { useState } from 'react';
import { useRouter, redirect } from 'next/navigation';

export function LoginContent() {
  const { data: userInfo, isLoading: userLoading } = useUser();
  const { showError } = useToastMessages();
  const [error, setError] = useState<string | null>(null);
  const router = useRouter();

  const { logout, isLoading: isLoggingOut } = useLogoutHandler({
    onSuccess: (response) => {
      const redirectUrl = response.headers?.location || ROUTE_DEFAULT;
      // For internal URLs, use router.push; for external URLs, use window.location.href
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

  const oauthInitiate = useOAuthInitiate({
    onSuccess: (response) => {
      // Handle redirect based on backend response
      // 303 response: Location header (OAuth URL or already authenticated)
      if (response.headers?.location) {
        const redirectUrl = response.headers.location;
        // For external URLs (OAuth), use window.location.href; for internal routes, use router.push
        if (redirectUrl.startsWith('http://') || redirectUrl.startsWith('https://')) {
          redirect(redirectUrl);
        } else {
          router.push(redirectUrl);
        }
      } else {
        setError('Auth URL not found in response. Please try again.');
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
          {error && <p className="text-destructive text-sm">{error}</p>}
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
