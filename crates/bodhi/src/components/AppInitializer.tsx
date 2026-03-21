import { ReactNode, useEffect } from 'react';

import { AppStatus, OpenAiApiError } from '@bodhiapp/ts-client';
import { useNavigate } from '@tanstack/react-router';

import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert';
import { Loading } from '@/components/ui/Loading';
import { useGetAppInfo } from '@/hooks/info';
import { useGetUser } from '@/hooks/users';
import {
  ROUTE_DEFAULT,
  ROUTE_RESOURCE_ADMIN,
  ROUTE_SETUP,
  ROUTE_SETUP_TENANTS,
  ROUTE_REQUEST_ACCESS,
  ROUTE_LOGIN,
} from '@/lib/constants';
import { Role, meetsMinRole } from '@/lib/roles';

// For backward compatibility, allow passing clean role names (admin, manager, etc.)
type MinRole = 'user' | 'power_user' | 'manager' | 'admin';

interface AppInitializerProps {
  children?: ReactNode;
  allowedStatus?: AppStatus | AppStatus[];
  authenticated?: boolean;
  minRole?: MinRole;
}

export default function AppInitializer({
  children,
  allowedStatus,
  authenticated = false,
  minRole,
}: AppInitializerProps) {
  const navigate = useNavigate();

  const { data: appInfo, error: appError, isLoading: appLoading } = useGetAppInfo();
  const {
    data: userInfo,
    error: userError,
    isLoading: userLoading,
  } = useGetUser({
    enabled: authenticated,
  });

  useEffect(() => {
    if (!appLoading && appInfo) {
      const { status } = appInfo;
      const statusAllowed = Array.isArray(allowedStatus) ? allowedStatus.includes(status) : status === allowedStatus;
      if (!allowedStatus || !statusAllowed) {
        switch (status) {
          case 'setup':
            if (appInfo.deployment === 'multi_tenant') {
              navigate({ to: ROUTE_SETUP_TENANTS });
            } else {
              navigate({ to: ROUTE_SETUP });
            }
            break;
          case 'ready':
            if (appInfo.deployment === 'multi_tenant' && !appInfo.client_id) {
              navigate({ to: ROUTE_LOGIN });
            } else {
              navigate({ to: ROUTE_DEFAULT });
            }
            break;
          case 'resource_admin':
            navigate({ to: ROUTE_RESOURCE_ADMIN });
            break;
        }
      }
    }
  }, [appInfo, appLoading, allowedStatus, navigate]);

  useEffect(() => {
    if (appLoading || userLoading || appError || userError) return;

    if (authenticated && userInfo?.auth_status !== 'logged_in') {
      // Store current URL so we can return after login
      if (typeof window !== 'undefined') {
        sessionStorage.setItem('bodhi-return-url', window.location.href);
      }
      navigate({ to: ROUTE_LOGIN });
      return;
    }

    // Check for authenticated users
    if (authenticated && userInfo?.auth_status === 'logged_in') {
      // Check if user has no assignable role - redirect to request access
      if (!userInfo.role || userInfo.role === 'resource_guest' || userInfo.role === 'resource_anonymous') {
        navigate({ to: ROUTE_REQUEST_ACCESS });
        return;
      }

      // Check minimum role requirement
      if (minRole) {
        const userRoleValue = typeof userInfo.role === 'string' ? userInfo.role : null;
        const requiredRole = `resource_${minRole}` as Role; // Convert to resource_ format

        if (!userRoleValue || !meetsMinRole(userRoleValue, requiredRole)) {
          navigate({ to: ROUTE_LOGIN, search: { error: 'insufficient-role' } });
          return;
        }
      }
    }
  }, [authenticated, userInfo, minRole, navigate, appLoading, userLoading, appError, userError]);

  if (appLoading || userLoading) {
    return <Loading message="Initializing app..." />;
  }

  if (appError) {
    return (
      <Alert variant="destructive">
        <AlertTitle>Error</AlertTitle>
        <AlertDescription>
          {(appError.response?.data as OpenAiApiError)?.error?.message || appError.message}
        </AlertDescription>
      </Alert>
    );
  }

  if (userError) {
    return (
      <Alert variant="destructive">
        <AlertTitle>Error</AlertTitle>
        <AlertDescription>
          {(userError.response?.data as OpenAiApiError)?.error?.message || userError.message}
        </AlertDescription>
      </Alert>
    );
  }

  if (appInfo?.status) {
    if (!['setup', 'ready', 'resource_admin'].includes(appInfo.status)) {
      return (
        <Alert variant="destructive">
          <AlertTitle>Error</AlertTitle>
          <AlertDescription>{`unexpected status from /app/info endpoint - '${appInfo.status}'`}</AlertDescription>
        </Alert>
      );
    }

    const isStatusAllowed = Array.isArray(allowedStatus)
      ? allowedStatus.includes(appInfo.status)
      : appInfo.status === allowedStatus;
    if (!allowedStatus || !isStatusAllowed) {
      return <Loading message="Redirecting..." />;
    }
  }

  if (authenticated && userInfo?.auth_status !== 'logged_in') {
    return <Loading message="Redirecting to login..." />;
  }

  return <>{children}</>;
}
