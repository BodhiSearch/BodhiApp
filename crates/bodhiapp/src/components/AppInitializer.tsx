'use client';

import { useAppInfo, useUser } from '@/hooks/useQuery';
import { useRouter } from 'next/navigation';
import { ReactNode } from 'react';
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert';
import { Loader2 } from 'lucide-react';
import { ApiError } from '@/types/models';

interface AppInitializerProps {
  children?: ReactNode;
  allowedStatus?: 'setup' | 'ready' | 'resource-admin';
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
  } = authenticated // eslint-disable-next-line react-hooks/rules-of-hooks
    ? useUser()
    : { data: { logged_in: false }, error: null, isLoading: false };

  if (appLoading || (authenticated && userLoading)) {
    return (
      <div className="flex flex-col items-center justify-center">
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

  if (authenticated && userError) {
    return (
      <Alert variant="destructive">
        <AlertTitle>Error</AlertTitle>
        <AlertDescription>
          {(userError.response?.data as ApiError)?.message || userError.message}
        </AlertDescription>
      </Alert>
    );
  }

  if (appInfo) {
    const { status } = appInfo;
    if (!allowedStatus || status !== allowedStatus) {
      switch (status) {
        case 'setup':
          router.push('/ui/setup');
          return null;
        case 'ready':
          router.push('/ui/home');
          return null;
        case 'resource-admin':
          router.push('/ui/setup/resource-admin');
          return null;
        default:
          return (
            <Alert variant="destructive">
              <h2>Error</h2>
              <p>{`unexpected status from /app/info endpoint - '${status}'`}</p>
            </Alert>
          );
      }
    }
  }

  if (authenticated && !userInfo?.logged_in) {
    router.push('/ui/login');
    return null;
  }

  return <>{children}</>;
}
