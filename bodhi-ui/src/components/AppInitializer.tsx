'use client';

import React, { useEffect, useMemo, useState } from 'react';
import { useRouter, useSearchParams } from 'next/navigation';
import { Alert, AlertTitle, AlertDescription } from '@/components/ui/alert';
import { Loader2 } from 'lucide-react';
import { BodhiBackend } from '@/services/BodhiBackend';

interface AppInitializerProps {
  allowedStatus?: 'setup' | 'ready' | 'resource-admin';
  children?: React.ReactNode;
}

const AppInitializer: React.FC<AppInitializerProps> = ({
  allowedStatus,
  children,
}) => {
  const router = useRouter();
  const searchParams = useSearchParams();
  const [error, setError] = useState<string | null>(searchParams.get('error'));
  const [isInitialized, setIsInitialized] = useState(false);
  const bodhi_url = process.env.NEXT_PUBLIC_BODHI_URL || '';
  const bodhiBackend = useMemo(() => {
    return new BodhiBackend(bodhi_url);
  }, [bodhi_url]);

  useEffect(() => {
    const initializeApp = async () => {
      try {
        const data = await bodhiBackend.getAppInfo();

        if (!allowedStatus || data.status !== allowedStatus) {
          switch (data.status) {
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
              setError(
                `unexpected /app/info status from server - ${data.status}`
              );
          }
        } else {
          setIsInitialized(true);
        }
      } catch (error) {
        setError(
          `Unable to connect to backend: '${bodhi_url}', error: ${error}`
        );
      }
    };

    initializeApp();
  }, [router, bodhi_url, allowedStatus, bodhiBackend]);

  if (error) {
    return (
      <Alert variant="destructive">
        <AlertTitle>Error</AlertTitle>
        <AlertDescription>{error}</AlertDescription>
      </Alert>
    );
  }

  if (!isInitialized) {
    return (
      <div className="flex flex-col items-center justify-center">
        <Loader2 className="h-12 w-12 animate-spin text-gray-500" />
        <p className="mt-4 text-gray-600">Initializing app...</p>
      </div>
    );
  }

  return children ? <>{children}</> : null;
};

export default AppInitializer;
