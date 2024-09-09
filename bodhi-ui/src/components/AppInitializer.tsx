'use client';

import React, { useEffect, useState } from 'react';
import { useRouter } from 'next/navigation';
import { Alert, AlertTitle, AlertDescription } from '@/components/ui/alert';
import { Loader2 } from 'lucide-react';
import { useAppSetup } from '@/hooks/useAppSetup';

interface AppInitializerProps {
  allowedStatus?: 'setup' | 'ready' | 'resource-admin';
  children?: React.ReactNode;
}

const AppInitializer: React.FC<AppInitializerProps> = ({
  allowedStatus,
  children,
}) => {
  const router = useRouter();
  const { appInfo, isLoading, isError, error } = useAppSetup();
  const [errorMessage, setErrorMessage] = useState<string | null>(null);

  useEffect(() => {
    if (appInfo && (!allowedStatus || appInfo.status !== allowedStatus)) {
      switch (appInfo.status) {
        case 'setup':
          router.push('/ui/setup');
          break;
        case 'ready':
          router.push('/ui/home');
          break;
        case 'resource-admin':
          router.push('/ui/setup/resource-admin');
          break;
        default:
          setErrorMessage(`unexpected status from /app/info endpoint - '${appInfo.status}'`);
      }
    }
  }, [router, allowedStatus, appInfo, errorMessage]);

  if (isLoading) {
    return (
      <div className="flex flex-col items-center justify-center">
        <Loader2 className="h-12 w-12 animate-spin text-gray-500" />
        <p className="mt-4 text-gray-600">Initializing app...</p>
      </div>
    );
  }

  if (isError) {
    return (
      <Alert variant="destructive">
        <AlertTitle>Error</AlertTitle>
        <AlertDescription>
          {/* @ts-ignore */}
          {error?.response?.data?.message || 'An unexpected error occurred'}
        </AlertDescription>
      </Alert>
    );
  }

  if (errorMessage) {
    return (
      <Alert variant="destructive">
        <AlertTitle>Error</AlertTitle>
        <AlertDescription>{errorMessage}</AlertDescription>
      </Alert>
    );
  }

  if (appInfo && appInfo.status === allowedStatus) {
    return children ? <>{children}</> : null;
  }

  return null;
};

export default AppInitializer;
