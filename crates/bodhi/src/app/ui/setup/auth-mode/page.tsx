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

function SetupContent() {
  const router = useRouter();
  const searchParams = useSearchParams();
  const [error, setError] = useState<string | null>(searchParams.get('error'));

  const { mutate: setup, isLoading: isSettingUp } = useSetupApp({
    onSuccess: (appInfo) => {
      if (appInfo.status === 'resource-admin') {
        router.push('/ui/setup/resource-admin');
      } else if (appInfo.status === 'ready') {
        router.push('/ui/home');
      } else {
        setError(`Unexpected setup status: ${appInfo.status}`);
      }
    },
    onError: (message) => {
      setError(`Error while setting up app: ${message}`);
    },
  });

  const handleSetup = (authz: boolean) => {
    setError(null);
    setup({ authz });
  };

  return (
    <main className="min-h-screen bg-background p-4 pt-16">
      <div className="mx-auto w-full max-w-md space-y-8">
        <Image
          src="/bodhi-logo/bodhi-logo-480.svg"
          alt="Bodhi App Logo"
          width={150}
          height={150}
          className="mx-auto"
          priority
        />
        <Card>
          <CardHeader>
            <CardTitle>Bodhi App Setup</CardTitle>
            <CardDescription>Choose your setup mode</CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            {error && (
              <Alert variant="destructive">
                <AlertTitle>Error</AlertTitle>
                <AlertDescription>{error}</AlertDescription>
              </Alert>
            )}
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
            <Alert>
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
          </CardContent>
          <CardFooter>
            <p className="mx-auto text-sm text-gray-500">
              For more information, visit our documentation.
            </p>
          </CardFooter>
        </Card>
      </div>
    </main>
  );
}

export default function SetupPage() {
  return (
    <AppInitializer allowedStatus="setup" authenticated={false}>
      <SetupContent />
    </AppInitializer>
  );
}
