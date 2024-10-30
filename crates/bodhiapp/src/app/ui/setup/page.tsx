'use client';

import { useState } from 'react';
import { useRouter, useSearchParams } from 'next/navigation';
import Image from 'next/image';
import {
  Card,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert';
import { Loader2 } from 'lucide-react';
import AppInitializer from '@/components/AppInitializer';
import { useSetupApp } from '@/hooks/useQuery';
import { AxiosError } from 'axios';

function SetupContent() {
  const router = useRouter();
  const searchParams = useSearchParams();
  const [error, setError] = useState<string | null>(searchParams.get('error'));
  const { mutateAsync: setup, isLoading: isSettingUp } = useSetupApp();

  const handleSetup = async (authz: boolean) => {
    setError(null);
    try {
      const { data: appInfo } = await setup({ authz });
      if (appInfo.status === 'resource-admin') {
        router.push('/ui/setup/resource-admin');
      } else if (appInfo.status === 'ready') {
        router.push('/ui/home');
      } else {
        setError(`Unexpected setup status: ${appInfo.status}`);
      }
    } catch (err) {
      if (err instanceof AxiosError) {
        setError(
          `Error while setting up app: ${err.response?.data?.message || err?.message}`
        );
      } else {
        setError(`An unexpected error occurred during setup: ${err}`);
      }
    }
  };

  return (
    <div className="flex flex-col items-center justify-center min-h-screen p-4 bg-gray-100">
      <Image
        src="/bodhi-logo.png"
        alt="Bodhi App Logo"
        width={150}
        height={150}
        className="mb-8"
      />
      <Card className="w-full max-w-md">
        <CardHeader>
          <CardTitle>Bodhi App Setup</CardTitle>
          <CardDescription>Choose your setup mode</CardDescription>
        </CardHeader>
        <CardContent>
          {error && (
            <Alert variant="destructive" className="mb-4">
              <AlertTitle>Error</AlertTitle>
              <AlertDescription>{error}</AlertDescription>
            </Alert>
          )}
          <div className="space-y-4">
            <Button
              className="w-full"
              onClick={() => handleSetup(true)}
              disabled={isSettingUp}
            >
              {isSettingUp ? (
                <>
                  <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                  Setting up...
                </>
              ) : (
                'Setup Authenticated Instance →'
              )}
            </Button>
            <Alert className="mb-4">
              <AlertTitle>Warning</AlertTitle>
              <AlertDescription>
                Setting up in non-authenticated mode may compromise system
                resources. We recommend choosing the authenticated mode for
                better security.
              </AlertDescription>
            </Alert>
            <Button
              variant="outline"
              className="w-full"
              onClick={() => handleSetup(false)}
              disabled={isSettingUp}
            >
              {isSettingUp ? (
                <>
                  <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                  Setting up...
                </>
              ) : (
                'Setup Unauthenticated Instance →'
              )}
            </Button>
          </div>
        </CardContent>
        <CardFooter className="justify-center">
          <p className="text-sm text-gray-500">
            For more information, visit our documentation.
          </p>
        </CardFooter>
      </Card>
    </div>
  );
}

export default function SetupPage() {
  return (
    <AppInitializer allowedStatus="setup" authenticated={false}>
      <SetupContent />
    </AppInitializer>
  );
}
