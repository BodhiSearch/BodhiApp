'use client';

import { useEffect, useState } from 'react';

import { AxiosResponse } from 'axios';
import { RedirectResponse, TenantListItem } from '@bodhiapp/ts-client';
import { useRouter, redirect } from 'next/navigation';

import AppInitializer from '@/components/AppInitializer';
import { AuthCard } from '@/components/AuthCard';
import { useToastMessages } from '@/hooks/use-toast-messages';
import { useLogoutHandler, useOAuthInitiate, useDashboardOAuthInitiate } from '@/hooks/useAuth';
import { useAppInfo } from '@/hooks/useInfo';
import { useTenants, useTenantActivate } from '@/hooks/useTenants';
import { useUser } from '@/hooks/useUsers';
import { ROUTE_DEFAULT, ROUTE_LOGIN } from '@/lib/constants';
import { handleSmartRedirect } from '@/lib/utils';

function MultiTenantLoginContent() {
  const { data: appInfo } = useAppInfo();
  const { data: userInfo, isLoading: userLoading } = useUser();
  const { showError } = useToastMessages();
  const [error, setError] = useState<string | null>(null);
  const [redirecting, setRedirecting] = useState(false);
  const [selectedTenantId, setSelectedTenantId] = useState<string | null>(null);
  const router = useRouter();

  // Dashboard OAuth (platform login)
  const { mutate: initiateDashboardOAuth, isLoading: isDashboardLoading } = useDashboardOAuthInitiate({
    onSuccess: (response: AxiosResponse<RedirectResponse>) => {
      setError(null);
      setRedirecting(true);
      const location = response.data?.location;
      if (!location) {
        setError('Auth URL not found in response. Please try again.');
        setRedirecting(false);
        return;
      }
      handleSmartRedirect(location, router);
    },
    onError: (message: string) => {
      setError(message);
      setRedirecting(false);
    },
  });

  // Resource OAuth (tenant login)
  const { mutate: initiateOAuth, isLoading: isOAuthLoading } = useOAuthInitiate({
    onSuccess: (response) => {
      setError(null);
      setRedirecting(true);
      const location = response.data?.location;
      if (!location) {
        setError('Auth URL not found in response. Please try again.');
        setRedirecting(false);
        return;
      }
      handleSmartRedirect(location, router);
    },
    onError: (message) => {
      setError(message);
      setRedirecting(false);
    },
  });

  // Tenant activation
  const { mutate: activateTenant, isLoading: isActivating } = useTenantActivate({
    onSuccess: () => {
      // After activation, trigger resource OAuth for the activated tenant
      if (selectedTenantId) {
        initiateOAuth({ client_id: selectedTenantId });
      }
    },
    onError: (message: string) => {
      setError(message);
    },
  });

  // Fetch tenants when user has dashboard session
  const isDashboardLoggedIn = !!userInfo?.has_dashboard_session && !appInfo?.client_id;
  const { data: tenantsData, isLoading: tenantsLoading } = useTenants({
    enabled: isDashboardLoggedIn,
  });

  // Logout
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
      localStorage.clear();
      sessionStorage.clear();
      document.cookie.split(';').forEach((c) => {
        const eqPos = c.indexOf('=');
        const name = eqPos > -1 ? c.substr(0, eqPos) : c;
        document.cookie = name + '=;expires=Thu, 01 Jan 1970 00:00:00 GMT;path=/';
      });
      showError('Logout failed', `Message: ${message}. Redirecting to login page.`);
      router.push(ROUTE_LOGIN);
    },
  });

  // Auto-login if only one tenant
  useEffect(() => {
    if (isDashboardLoggedIn && tenantsData?.tenants && tenantsData.tenants.length === 1) {
      const tenant = tenantsData.tenants[0];
      setSelectedTenantId(tenant.client_id);
      if (tenant.logged_in) {
        // Already logged in to this tenant, activate it
        activateTenant({ client_id: tenant.client_id });
      } else {
        // Need to OAuth into this tenant
        initiateOAuth({ client_id: tenant.client_id });
      }
    }
  }, [isDashboardLoggedIn, tenantsData, activateTenant, initiateOAuth]);

  if (userLoading || tenantsLoading) {
    return <AuthCard title="Loading..." isLoading={true} />;
  }

  // State C: Fully authenticated with a tenant
  if (userInfo?.auth_status === 'logged_in' && appInfo?.client_id) {
    const activeTenant = tenantsData?.tenants?.find((t: TenantListItem) => t.client_id === appInfo.client_id);
    const otherTenants = tenantsData?.tenants?.filter((t: TenantListItem) => t.client_id !== appInfo.client_id) || [];

    return (
      <AuthCard
        title="Welcome"
        description={
          <div className="space-y-2">
            <p>You are logged in as {userInfo.username}</p>
            {activeTenant && (
              <p className="text-sm">
                Active workspace: <strong>{activeTenant.name}</strong>
              </p>
            )}
          </div>
        }
        actions={[
          {
            label: 'Go to Home',
            href: ROUTE_DEFAULT,
            variant: 'secondary',
          },
          ...(otherTenants.length > 0
            ? otherTenants.map((tenant: TenantListItem) => ({
                label: `Switch to ${tenant.name}`,
                onClick: () => {
                  setSelectedTenantId(tenant.client_id);
                  if (tenant.logged_in) {
                    activateTenant({ client_id: tenant.client_id });
                  } else {
                    initiateOAuth({ client_id: tenant.client_id });
                  }
                },
                disabled: isActivating || isOAuthLoading,
                variant: 'secondary' as const,
              }))
            : []),
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

  // State B: Dashboard session, not resource-authenticated - show tenant selector
  if (isDashboardLoggedIn && tenantsData?.tenants && tenantsData.tenants.length > 1) {
    return (
      <AuthCard
        title="Select Workspace"
        description={
          <div className="space-y-4">
            <p>Choose a workspace to continue</p>
            {error && <p className="text-destructive text-sm">{error}</p>}
          </div>
        }
        actions={tenantsData.tenants.map((tenant: TenantListItem) => ({
          label: isActivating && selectedTenantId === tenant.client_id ? 'Connecting...' : tenant.name,
          onClick: () => {
            setError(null);
            setSelectedTenantId(tenant.client_id);
            if (tenant.logged_in) {
              activateTenant({ client_id: tenant.client_id });
            } else {
              initiateOAuth({ client_id: tenant.client_id });
            }
          },
          disabled: isActivating || isOAuthLoading || redirecting,
        }))}
      />
    );
  }

  // State A: No dashboard session - show platform login button
  return (
    <AuthCard
      title="Login"
      description={
        <div className="space-y-4">
          <p>Login to the Bodhi Platform</p>
          {error && <p className="text-destructive text-sm">{error}</p>}
        </div>
      }
      actions={[
        {
          label: isDashboardLoading ? 'Initiating...' : redirecting ? 'Redirecting...' : 'Login to Bodhi Platform',
          onClick: () => {
            setError(null);
            initiateDashboardOAuth();
          },
          disabled: isDashboardLoading || redirecting,
        },
      ]}
    />
  );
}

export function LoginContent() {
  const { data: appInfo } = useAppInfo();
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
    if (appInfo?.client_id) {
      initiateOAuth({ client_id: appInfo.client_id });
    }
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
  const { data: appInfo } = useAppInfo();
  const isMultiTenant = appInfo?.deployment === 'multi_tenant';

  return (
    <AppInitializer allowedStatus={['ready', 'tenant_selection']} authenticated={false}>
      <div className="pt-12 sm:pt-16" data-testid="login-page">
        {isMultiTenant ? <MultiTenantLoginContent /> : <LoginContent />}
      </div>
    </AppInitializer>
  );
}
