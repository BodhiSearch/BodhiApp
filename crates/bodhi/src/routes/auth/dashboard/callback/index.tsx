import { useEffect, useRef, useState } from 'react';

import { RedirectResponse } from '@bodhiapp/ts-client';
import { createFileRoute, useNavigate, useSearch } from '@tanstack/react-router';
import { AxiosResponse } from 'axios';
import { z } from 'zod';

import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert';
import { Button } from '@/components/ui/button';
import { Loading } from '@/components/ui/Loading';
import { useDashboardOAuthCallback } from '@/hooks/auth';
import { handleSmartRedirect } from '@/lib/utils';

export const Route = createFileRoute('/auth/dashboard/callback/')({
  validateSearch: z.object({
    code: z.string().optional(),
    state: z.string().optional(),
  }),
  component: DashboardCallbackPage,
});

function DashboardCallbackContent() {
  const search = useSearch({ from: '/auth/dashboard/callback/' });
  const navigate = useNavigate();
  const [error, setError] = useState<string | null>(null);
  const hasSubmitted = useRef(false);

  const { mutate: dashboardCallback } = useDashboardOAuthCallback({
    onSuccess: (response: AxiosResponse<RedirectResponse>) => {
      const location = response.data.location;
      handleSmartRedirect(location, navigate);
    },
    onError: (message: string) => {
      setError(message);
    },
  });

  useEffect(() => {
    if (hasSubmitted.current) return;
    hasSubmitted.current = true;

    const code = search.code;
    const state = search.state;

    if (code && state) {
      dashboardCallback({ code, state });
    } else {
      setError('Missing authorization code or state parameter');
    }
  }, [search, dashboardCallback]);

  if (error) {
    return (
      <div className="flex min-h-screen items-center justify-center">
        <div className="w-full max-w-md space-y-4 p-4">
          <Alert variant="destructive">
            <AlertTitle>Authentication Error</AlertTitle>
            <AlertDescription>{error}</AlertDescription>
          </Alert>
          <Button onClick={() => navigate({ to: '/login/' })} className="w-full">
            Try Again
          </Button>
        </div>
      </div>
    );
  }

  return <Loading message="Processing Login..." />;
}

export default function DashboardCallbackPage() {
  return <DashboardCallbackContent />;
}
