'use client';

import { useAppInfo, useUser } from '@/hooks/useQuery';
import { useRouter } from 'next/navigation';
import { ReactNode, useEffect } from 'react';
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert';
import { Loader2 } from 'lucide-react';
import { ApiError, AppStatus } from '@/types/models';

interface AppInitializerProps {
  children?: ReactNode;
  allowedStatus?: AppStatus;
  authenticated?: boolean;
}

export default function AppInitializer({
  children,
  allowedStatus,
  authenticated = false,
}: AppInitializerProps) {
  const router = useRouter();
  const {
    data: appInfo,
    error: appError,
    isLoading: appLoading,
  } = useAppInfo();
  const {
    data: userInfo,
    error: userError,
    isLoading: userLoading,
  } = useUser({
    enabled: authenticated || !!appInfo?.authz,
  });

  useEffect(() => {
    if (!appLoading && appInfo) {
      const { status } = appInfo;
      if (!allowedStatus || status !== allowedStatus) {
        switch (status) {
          case 'setup':
            router.push('/ui/setup');
            break;
          case 'ready':
            router.push('/ui/home');
            break;
          case 'resource-admin':
            router.push('/ui/setup/resource-admin');
            break;
        }
      }
    }
  }, [appInfo, appLoading, allowedStatus, router]);

  useEffect(() => {
    if (appLoading || userLoading || appError || userError) return;
    if (authenticated && appInfo?.authz && !userInfo?.logged_in) {
      router.push('/ui/login');
    }
  }, [
    appInfo?.authz,
    authenticated,
    userInfo,
    router,
    appLoading,
    userLoading,
    appError,
    userError,
  ]);

  if (appLoading || userLoading) {
    return (
      <div className="flex flex-1 flex-col items-center justify-center w-full">
        <Loader2 className="h-12 w-12 animate-spin text-gray-500" />
        <p className="mt-4 text-gray-600">Initializing app...</p>
      </div>
    );
  }

  if (appError) {
    return (
      <Alert variant="destructive">
        <AlertTitle>Error</AlertTitle>
        <AlertDescription>
          {(appError.response?.data as ApiError)?.message || appError.message}
        </AlertDescription>
      </Alert>
    );
  }

  if (userError) {
    return (
      <Alert variant="destructive">
        <AlertTitle>Error</AlertTitle>
        <AlertDescription>
          {(userError.response?.data as ApiError)?.message || userError.message}
        </AlertDescription>
      </Alert>
    );
  }

  if (appInfo?.status) {
    if (!['setup', 'ready', 'resource-admin'].includes(appInfo.status)) {
      return (
        <Alert variant="destructive">
          <AlertTitle>Error</AlertTitle>
          <AlertDescription>
            {`unexpected status from /app/info endpoint - '${appInfo.status}'`}
          </AlertDescription>
        </Alert>
      );
    }

    if (!allowedStatus || appInfo.status !== allowedStatus) {
      return (
        <div className="flex flex-1 flex-col items-center justify-center w-full">
          <Loader2 className="h-12 w-12 animate-spin text-gray-500" />
          <p className="mt-4 text-gray-600">Redirecting...</p>
        </div>
      );
    }
  }

  if (authenticated && appInfo?.authz && !userInfo?.logged_in) {
    return (
      <div className="flex flex-1 flex-col items-center justify-center w-full">
        <Loader2 className="h-12 w-12 animate-spin text-gray-500" />
        <p className="mt-4 text-gray-600">Redirecting to login...</p>
      </div>
    );
  }

  return <>{children}</>;
}
