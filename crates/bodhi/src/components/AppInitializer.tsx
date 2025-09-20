'use client';

import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert';
import { Loading } from '@/components/ui/Loading';
import { useLocalStorage } from '@/hooks/useLocalStorage';
import { useAppInfo, useUser } from '@/hooks/useQuery';
import {
  FLAG_MODELS_DOWNLOAD_PAGE_DISPLAYED,
  ROUTE_DEFAULT,
  ROUTE_RESOURCE_ADMIN,
  ROUTE_SETUP,
  ROUTE_SETUP_DOWNLOAD_MODELS,
  ROUTE_REQUEST_ACCESS,
  ROUTE_LOGIN,
} from '@/lib/constants';
import { AppStatus, OpenAiApiError } from '@bodhiapp/ts-client';
import { useRouter } from 'next/navigation';
import { ReactNode, useEffect } from 'react';
import { Role, meetsMinRole, getCleanRoleName } from '@/lib/roles';

// For backward compatibility, allow passing clean role names (admin, manager, etc.)
type MinRole = 'user' | 'power_user' | 'manager' | 'admin';

interface AppInitializerProps {
  children?: ReactNode;
  allowedStatus?: AppStatus;
  authenticated?: boolean;
  minRole?: MinRole;
}

export default function AppInitializer({
  children,
  allowedStatus,
  authenticated = false,
  minRole,
}: AppInitializerProps) {
  const router = useRouter();
  const [hasShownModelsPage] = useLocalStorage(FLAG_MODELS_DOWNLOAD_PAGE_DISPLAYED, false);

  const { data: appInfo, error: appError, isLoading: appLoading } = useAppInfo();
  const {
    data: userInfo,
    error: userError,
    isLoading: userLoading,
  } = useUser({
    enabled: authenticated,
  });

  useEffect(() => {
    if (!appLoading && appInfo) {
      const { status } = appInfo;
      if (!allowedStatus || status !== allowedStatus) {
        switch (status) {
          case 'setup':
            router.push(ROUTE_SETUP);
            break;
          case 'ready':
            if (!hasShownModelsPage) {
              router.push(ROUTE_SETUP_DOWNLOAD_MODELS);
            } else {
              router.push(ROUTE_DEFAULT);
            }
            break;
          case 'resource-admin':
            router.push(ROUTE_RESOURCE_ADMIN);
            break;
        }
      }
    }
  }, [appInfo, appLoading, allowedStatus, router, hasShownModelsPage]);

  useEffect(() => {
    if (appLoading || userLoading || appError || userError) return;

    if (authenticated && userInfo?.auth_status !== 'logged_in') {
      router.push(ROUTE_LOGIN);
      return;
    }

    // Check for authenticated users
    if (authenticated && userInfo?.auth_status === 'logged_in') {
      // Check if user has no role - redirect to request access
      if (!userInfo.role) {
        router.push(ROUTE_REQUEST_ACCESS);
        return;
      }

      // Check minimum role requirement
      if (minRole) {
        const userRoleValue = typeof userInfo.role === 'string' ? userInfo.role : null;
        const requiredRole = `resource_${minRole}` as Role; // Convert to resource_ format

        if (!userRoleValue || !meetsMinRole(userRoleValue, requiredRole)) {
          router.push(ROUTE_LOGIN + '?error=insufficient-role');
          return;
        }
      }
    }
  }, [authenticated, userInfo, minRole, router, appLoading, userLoading, appError, userError]);

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
    if (!['setup', 'ready', 'resource-admin'].includes(appInfo.status)) {
      return (
        <Alert variant="destructive">
          <AlertTitle>Error</AlertTitle>
          <AlertDescription>{`unexpected status from /app/info endpoint - '${appInfo.status}'`}</AlertDescription>
        </Alert>
      );
    }

    if (!allowedStatus || appInfo.status !== allowedStatus) {
      return <Loading message="Redirecting..." />;
    }
  }

  if (authenticated && userInfo?.auth_status !== 'logged_in') {
    return <Loading message="Redirecting to login..." />;
  }

  return <>{children}</>;
}
