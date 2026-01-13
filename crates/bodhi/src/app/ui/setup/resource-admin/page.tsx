'use client';

import { useState } from 'react';

import { motion } from 'framer-motion';
import { useRouter } from 'next/navigation';

import { SetupContainer, SetupCard } from '@/app/ui/setup/components';
import { itemVariants } from '@/app/ui/setup/types';
import AppInitializer from '@/components/AppInitializer';
import { Button } from '@/components/ui/button';
import { useOAuthInitiate } from '@/hooks/useAuth';
import { handleSmartRedirect } from '@/lib/utils';

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
      <div data-testid="resource-admin-setup-page">
        <motion.div variants={itemVariants}>
          <SetupCard
            title="Admin Setup"
            description={
              <>
                <p className="py-2">You are setting up Bodhi App in authenticated mode.</p>
                <p className="py-2">
                  The email address you log in with will be granted admin role for this app instance.
                </p>
              </>
            }
            footer={
              <div className="flex flex-col gap-3 w-full">
                <Button
                  className="w-full"
                  size="lg"
                  onClick={handleOAuthInitiate}
                  disabled={isButtonDisabled}
                  data-testid="continue-login-button"
                >
                  {isLoading ? 'Initiating...' : redirecting ? 'Redirecting...' : 'Continue with Login →'}
                </Button>
                <p className="text-sm text-muted-foreground text-center">
                  Login with a valid email address to continue
                </p>
              </div>
            }
          >
            <div className="space-y-2">
              {error && (
                <div className="rounded-lg border border-destructive/50 bg-destructive/10 p-4">
                  <p className="text-destructive text-sm text-center">{error}</p>
                </div>
              )}

              {/* Admin Capabilities */}
              <div className="rounded-lg bg-muted/30 p-6 space-y-4">
                <h4 className="font-semibold text-base">As an Admin, you can:</h4>
                <div className="space-y-3">
                  <div className="flex items-start">
                    <span className="text-primary mr-3 mt-0.5">✓</span>
                    <span className="text-sm text-muted-foreground">Manage user access and permissions</span>
                  </div>
                  <div className="flex items-start">
                    <span className="text-primary mr-3 mt-0.5">✓</span>
                    <span className="text-sm text-muted-foreground">Unrestricted access to system-wide settings</span>
                  </div>
                </div>
              </div>
            </div>
          </SetupCard>
        </motion.div>
      </div>
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
