'use client';

import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert';
import { useAppInfo, useUser } from '@/hooks/useQuery';
import { useLocalStorage } from '@/hooks/useLocalStorage';
import { AppStatus, ErrorResponse } from '@/types/models';
import { useRouter } from 'next/navigation';
import { ReactNode, useEffect } from 'react';
import { Loading } from '@/components/ui/Loading';
import { FLAG_MODELS_DOWNLOAD_PAGE_DISPLAYED } from '@/lib/constants';

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
  const [hasShownModelsPage, setHasShownModelsPage] = useLocalStorage(
    FLAG_MODELS_DOWNLOAD_PAGE_DISPLAYED,
    true
  );

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
            if (!hasShownModelsPage) {
              router.push('/ui/setup/download-models');
            } else {
              router.push('/ui/home');
            }
            break;
          case 'resource-admin':
            router.push('/ui/setup/resource-admin');
            break;
        }
      }
    }
  }, [
    appInfo,
    appLoading,
    allowedStatus,
    router,
    hasShownModelsPage,
    setHasShownModelsPage,
  ]);

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
    return <Loading message="Initializing app..." />;
  }

  if (appError) {
    return (
      <Alert variant="destructive">
        <AlertTitle>Error</AlertTitle>
        <AlertDescription>
          {(appError.response?.data as ErrorResponse)?.error?.message ||
            appError.message}
        </AlertDescription>
      </Alert>
    );
  }

  if (userError) {
    return (
      <Alert variant="destructive">
        <AlertTitle>Error</AlertTitle>
        <AlertDescription>
          {(userError.response?.data as ErrorResponse)?.error?.message ||
            userError.message}
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
      return <Loading message="Redirecting..." />;
    }
  }

  if (authenticated && appInfo?.authz && !userInfo?.logged_in) {
    return <Loading message="Redirecting to login..." />;
  }

  return <>{children}</>;
}
