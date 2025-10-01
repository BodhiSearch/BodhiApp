'use client';

import { SetupContainer, SetupCard } from '@/app/ui/setup/components';
import AppInitializer from '@/components/AppInitializer';
import { Button } from '@/components/ui/button';
import { useOAuthInitiate } from '@/hooks/useAuth';
import { handleSmartRedirect } from '@/lib/utils';
import { useState } from 'react';
import { useRouter } from 'next/navigation';

function ResourceAdminContent() {
  const [error, setError] = useState<string | null>(null);
  const [redirecting, setRedirecting] = useState(false);
  const router = useRouter();

  const { mutate: initiateOAuth, isLoading } = useOAuthInitiate({
    onSuccess: (response) => {
      // Clear any previous errors and set redirecting state
      setError(null);
      setRedirecting(true);

      // Handle redirect based on backend response
      const location = response.data?.location;
      if (!location) {
        setError('Auth URL not found in response. Please try again.');
        setRedirecting(false);
        return;
      }

      // Handle redirect using smart URL detection
      handleSmartRedirect(location, router);
    },
    onError: (message) => {
      setError(message);
      setRedirecting(false);
    },
  });

  const handleOAuthInitiate = () => {
    setError(null); // Clear any previous errors
    initiateOAuth();
  };

  const isButtonDisabled = isLoading || redirecting;

  return (
    <SetupContainer>
      <SetupCard
        title="Admin Setup"
        footer={
          <div className="flex flex-col gap-4 w-full">
            <Button className="w-full" size="lg" onClick={handleOAuthInitiate} disabled={isButtonDisabled}>
              {isLoading ? 'Initiating...' : redirecting ? 'Redirecting...' : 'Continue with Login →'}
            </Button>
            <p className="text-sm text-muted-foreground text-center">Login with a valid email address to continue</p>
          </div>
        }
      >
        <div className="space-y-6" data-testid="resource-admin-page">
          <div className="prose dark:prose-invert mx-auto">
            <p className="text-center text-muted-foreground">
              You are setting up Bodhi App in authenticated mode. The email address you log in with will be granted
              admin role for this app instance.
            </p>
            {error && <p className="text-destructive text-sm text-center">{error}</p>}
          </div>

          <div className="space-y-4 text-sm">
            <div className="space-y-2">
              <h3 className="font-semibold">As an Admin, you can:</h3>
              <ul className="list-disc pl-5 space-y-1 text-muted-foreground">
                <li>Manage user access and permissions</li>
                <li>Unrestricted access to system-wide settings</li>
              </ul>
            </div>
          </div>
        </div>
      </SetupCard>
    </SetupContainer>
  );
}

export default function ResourceAdminPage() {
  return (
    <AppInitializer allowedStatus="resource-admin" authenticated={false}>
      <ResourceAdminContent />
    </AppInitializer>
  );
}
