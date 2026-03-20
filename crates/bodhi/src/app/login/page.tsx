'use client';

import { useEffect, useRef, useState } from 'react';

import { AxiosResponse } from 'axios';
import { AppStatus, RedirectResponse, TenantListItem } from '@bodhiapp/ts-client';
import { useRouter, useSearchParams } from 'next/navigation';

import AppInitializer from '@/components/AppInitializer';
import { AuthCard } from '@/components/AuthCard';
import { useToastMessages } from '@/hooks/use-toast-messages';
import { useLogoutHandler, useOAuthInitiate, useDashboardOAuthInitiate } from '@/hooks/useAuth';
import { useAppInfo } from '@/hooks/useInfo';
import { useTenants, useTenantActivate } from '@/hooks/useTenants';
import { useUser } from '@/hooks/useUsers';
import { ROUTE_DEFAULT, ROUTE_LOGIN, ROUTE_REQUEST_ACCESS, ROUTE_SETUP_TENANTS } from '@/lib/constants';
import { handleSmartRedirect } from '@/lib/utils';

function MultiTenantLoginContent() {
  const { data: appInfo } = useAppInfo();
  const { data: userInfo, isLoading: userLoading } = useUser();
  const { showError, showSuccess } = useToastMessages();
  const [error, setError] = useState<string | null>(null);
  const [redirecting, setRedirecting] = useState(false);
  const [selectedTenantId, setSelectedTenantId] = useState<string | null>(null);
  const [autoLoginFailed, setAutoLoginFailed] = useState(false);
  const router = useRouter();
  const searchParams = useSearchParams();

  // 5a. Read invite parameter on mount and store in sessionStorage
  const hasInviteProcessed = useRef(false);
  useEffect(() => {
    if (hasInviteProcessed.current) return;
    const inviteClientId = searchParams?.get('invite');
    if (inviteClientId) {
      hasInviteProcessed.current = true;
      sessionStorage.setItem('login_to_tenant', inviteClientId);
      router.replace('/login/');
    }
  }, [searchParams, router]);

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
      setAutoLoginFailed(true);
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
      setAutoLoginFailed(true);
    },
  });

  // Fetch tenants when user has dashboard session
  const needsTenantSelection = !!userInfo?.dashboard && !appInfo?.client_id;
  const { data: tenantsData, isLoading: tenantsLoading } = useTenants({
    enabled: needsTenantSelection,
  });

  // Logout
  const { logout, isLoading: isLoggingOut } = useLogoutHandler({
    onSuccess: (response) => {
      const redirectUrl = response.data?.location || ROUTE_DEFAULT;
      handleSmartRedirect(redirectUrl, router);
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
      handleSmartRedirect(ROUTE_LOGIN, router);
    },
  });

  // 5b/5c. Process invite flow (takes priority over auto-login)
  const hasInviteFlowTriggered = useRef(false);
  useEffect(() => {
    if (hasInviteFlowTriggered.current) return;
    const loginToTenant = sessionStorage.getItem('login_to_tenant');
    if (!loginToTenant) return;

    // No dashboard session — keep in sessionStorage (survive OAuth redirect), trigger dashboard login
    if (!userInfo?.dashboard) {
      if (!userLoading) {
        hasInviteFlowTriggered.current = true;
        initiateDashboardOAuth();
      }
      return;
    }

    // Dashboard session exists — wait for tenants to finish loading (if query is active)
    if (needsTenantSelection && (tenantsLoading || !tenantsData)) return;

    hasInviteFlowTriggered.current = true;
    sessionStorage.removeItem('login_to_tenant');

    // Check if target tenant is in the tenants list
    const targetTenant = tenantsData?.tenants?.find((t: TenantListItem) => t.client_id === loginToTenant);

    if (targetTenant?.logged_in) {
      // Already a member and logged in — activate and show toast
      activateTenant({ client_id: targetTenant.client_id });
      showSuccess('Workspace', 'Already a member of this workspace');
    } else {
      // Not logged in to target tenant or tenant not in list — initiate OAuth
      sessionStorage.setItem('bodhi-return-url', '/login/');
      initiateOAuth({ client_id: loginToTenant });
    }
  }, [
    userInfo,
    userLoading,
    needsTenantSelection,
    tenantsData,
    tenantsLoading,
    activateTenant,
    initiateOAuth,
    initiateDashboardOAuth,
    showSuccess,
  ]);

  // Auto-login if only one tenant (useRef guard prevents double-firing in StrictMode)
  // Suppressed when invite flow is active
  const hasAutoLoginTriggered = useRef(false);
  useEffect(() => {
    if (hasAutoLoginTriggered.current) return;
    // Suppress auto-login when invite flow is active
    if (sessionStorage.getItem('login_to_tenant')) return;
    if (needsTenantSelection && tenantsData?.tenants && tenantsData.tenants.length === 1) {
      hasAutoLoginTriggered.current = true;
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
  }, [needsTenantSelection, tenantsData, activateTenant, initiateOAuth]);

  if (userLoading || tenantsLoading) {
    return <AuthCard title="Loading..." isLoading={true} />;
  }

  // If status is 'setup' and no invite flow active, redirect to tenant creation
  if (appInfo?.status === 'setup' && !sessionStorage.getItem('login_to_tenant')) {
    router.push(ROUTE_SETUP_TENANTS);
    return null;
  }

  // 5d. Role: None/Guest/Anonymous guard — redirect to request-access if logged in without an assignable role
  if (
    userInfo?.auth_status === 'logged_in' &&
    appInfo?.client_id &&
    (!userInfo.role || userInfo.role === 'resource_guest' || userInfo.role === 'resource_anonymous')
  ) {
    router.push(ROUTE_REQUEST_ACCESS);
    return null;
  }

  // State C: Fully authenticated with a tenant
  if (userInfo?.auth_status === 'logged_in' && appInfo?.client_id) {
    const activeTenant = tenantsData?.tenants?.find((t: TenantListItem) => t.client_id === appInfo.client_id);
    const otherTenants = tenantsData?.tenants?.filter((t: TenantListItem) => t.client_id !== appInfo.client_id) || [];

    return (
      <AuthCard
        data-test-state="welcome"
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

  // State B1: Single tenant, auto-login failed - show manual connect
  if (needsTenantSelection && autoLoginFailed && tenantsData?.tenants && tenantsData.tenants.length === 1) {
    const tenant = tenantsData.tenants[0];
    return (
      <AuthCard
        data-test-state="connect"
        title="Connect to Workspace"
        description={
          <div className="space-y-4">
            <p>Auto-login failed. Connect manually to continue.</p>
            {error && <p className="text-destructive text-sm">{error}</p>}
          </div>
        }
        actions={[
          {
            label: isOAuthLoading || isActivating ? 'Connecting...' : `Connect to ${tenant.name}`,
            onClick: () => {
              setError(null);
              setSelectedTenantId(tenant.client_id);
              if (tenant.logged_in) {
                activateTenant({ client_id: tenant.client_id });
              } else {
                initiateOAuth({ client_id: tenant.client_id });
              }
            },
            disabled: isOAuthLoading || isActivating || redirecting,
          },
        ]}
      />
    );
  }

  // State B2: Dashboard session, not resource-authenticated - show tenant selector
  if (needsTenantSelection && tenantsData?.tenants && tenantsData.tenants.length > 1) {
    return (
      <AuthCard
        data-test-state="select"
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
      data-test-state="login"
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
      handleSmartRedirect(redirectUrl, router);
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
      handleSmartRedirect(ROUTE_LOGIN, router);
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
    setError(null);
    if (!appInfo?.client_id) {
      setError('Client ID is not set. Please check your configuration.');
      return;
    }
    initiateOAuth({ client_id: appInfo.client_id });
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

  // Allow 'setup' status when multi-tenant invite flow is active,
  // so the login page can process the invite before redirecting
  const hasInviteFlow = typeof window !== 'undefined' && sessionStorage.getItem('login_to_tenant');
  const allowedStatuses: AppStatus[] = isMultiTenant && hasInviteFlow ? ['ready', 'setup'] : ['ready'];

  return (
    <AppInitializer allowedStatus={allowedStatuses} authenticated={false}>
      <div className="pt-12 sm:pt-16" data-testid="login-page">
        {isMultiTenant ? <MultiTenantLoginContent /> : <LoginContent />}
      </div>
    </AppInitializer>
  );
}
