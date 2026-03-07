'use client';

import { Suspense, useEffect, useRef, useState } from 'react';
import { useSearchParams, useRouter } from 'next/navigation';

import { AxiosResponse } from 'axios';
import { RedirectResponse } from '@bodhiapp/ts-client';

import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert';
import { Button } from '@/components/ui/button';
import { Loading } from '@/components/ui/Loading';
import { useDashboardOAuthCallback } from '@/hooks/useAuth';

function DashboardCallbackContent() {
  const searchParams = useSearchParams();
  const router = useRouter();
  const [error, setError] = useState<string | null>(null);
  const hasSubmitted = useRef(false);

  const { mutate: dashboardCallback } = useDashboardOAuthCallback({
    onSuccess: (response: AxiosResponse<RedirectResponse>) => {
      const location = response.data.location;
      if (location.startsWith('http')) {
        window.location.href = location;
      } else {
        router.push(location);
      }
    },
    onError: (message: string) => {
      setError(message);
    },
  });

  useEffect(() => {
    if (hasSubmitted.current) return;
    hasSubmitted.current = true;

    const code = searchParams.get('code');
    const state = searchParams.get('state');

    if (code && state) {
      dashboardCallback({ code, state });
    } else {
      setError('Missing authorization code or state parameter');
    }
  }, [searchParams, dashboardCallback]);

  if (error) {
    return (
      <div className="flex min-h-screen items-center justify-center">
        <div className="w-full max-w-md space-y-4 p-4">
          <Alert variant="destructive">
            <AlertTitle>Authentication Error</AlertTitle>
            <AlertDescription>{error}</AlertDescription>
          </Alert>
          <Button onClick={() => router.push('/ui/login')} className="w-full">
            Try Again
          </Button>
        </div>
      </div>
    );
  }

  return <Loading message="Processing Login..." />;
}

export default function DashboardCallbackPage() {
  return (
    <Suspense fallback={<Loading message="Loading..." />}>
      <DashboardCallbackContent />
    </Suspense>
  );
}
