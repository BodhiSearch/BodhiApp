import { useState } from 'react';

import { useNavigate } from '@tanstack/react-router';
import { createFileRoute } from '@tanstack/react-router';
import { motion } from 'framer-motion';
import { ArrowRight, Check, ShieldCheck } from 'lucide-react';

import AppInitializer from '@/components/AppInitializer';
import { Button } from '@/components/ui/button';
import { useOAuthInitiate } from '@/hooks/auth';
import { useGetAppInfo } from '@/hooks/info';
import { ROUTE_SETUP_DOWNLOAD_MODELS } from '@/lib/constants';
import { handleSmartRedirect } from '@/lib/utils';
import { SetupContainer, SetupCard, SetupCardIcon } from '@/routes/setup/-components';
import { itemVariants } from '@/routes/setup/-shared/types';

export const Route = createFileRoute('/setup/resource-admin/')({
  component: ResourceAdminPage,
});

function ResourceAdminContent() {
  const { data: appInfo } = useGetAppInfo();
  const [error, setError] = useState<string | null>(null);
  const [redirecting, setRedirecting] = useState(false);
  const navigate = useNavigate();

  const { mutate: initiateOAuth, isPending: isLoading } = useOAuthInitiate({
    onSuccess: (response) => {
      setError(null);
      setRedirecting(true);

      const location = response.data?.location;
      if (!location) {
        setError('Auth URL not found in response. Please try again.');
        setRedirecting(false);
        return;
      }

      handleSmartRedirect(location, navigate);
    },
    onError: (message) => {
      setError(message);
      setRedirecting(false);
    },
  });

  const handleOAuthInitiate = () => {
    setError(null);
    sessionStorage.setItem('bodhi-return-url', ROUTE_SETUP_DOWNLOAD_MODELS);
    if (!appInfo?.client_id) {
      setError('Client ID is not set. Please check your configuration.');
      return;
    }
    initiateOAuth({ client_id: appInfo.client_id });
  };

  const isButtonDisabled = isLoading || redirecting;

  return (
    <SetupContainer>
      <div data-testid="resource-admin-setup-page">
        <motion.div variants={itemVariants}>
          <SetupCard
            title={
              <>
                <SetupCardIcon icon={ShieldCheck} />
                <h2 className="text-[26px] font-bold leading-tight tracking-tight">Admin Setup</h2>
              </>
            }
            description="You're setting up Bodhi in authenticated mode. The email address you log in with will be granted the admin role for this instance."
            footer={
              <div className="flex flex-col gap-3 w-full">
                <Button
                  className="w-full gap-2"
                  size="lg"
                  onClick={handleOAuthInitiate}
                  disabled={isButtonDisabled}
                  data-testid="continue-login-button"
                >
                  {isLoading ? (
                    'Initiating...'
                  ) : redirecting ? (
                    'Redirecting...'
                  ) : (
                    <>
                      Continue with Login
                      <ArrowRight className="h-4 w-4" />
                    </>
                  )}
                </Button>
                <p className="text-sm text-muted-foreground text-center">
                  Log in with a valid email address to continue.
                </p>
              </div>
            }
          >
            <div className="space-y-2">
              {error && (
                <div className="rounded-[var(--radius-lg)] border border-destructive/50 bg-destructive/10 p-4">
                  <p className="text-destructive text-sm text-center">{error}</p>
                </div>
              )}

              <div className="rounded-[var(--radius-lg)] border border-border bg-muted/50 px-7 py-6">
                <h3 className="mb-4 text-lg font-bold tracking-tight">As an admin, you can</h3>
                <ul className="flex flex-col gap-4">
                  {['Manage user access and permissions', 'Unrestricted access to system-wide settings'].map((cap) => (
                    <li key={cap} className="flex items-start gap-3.5 text-[15px]">
                      <span className="mt-px flex h-[26px] w-[26px] flex-none items-center justify-center rounded-full bg-primary/[0.16] text-[hsl(var(--primary-hover))]">
                        <Check className="h-[15px] w-[15px]" strokeWidth={3} />
                      </span>
                      <span>{cap}</span>
                    </li>
                  ))}
                </ul>
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
    <AppInitializer allowedStatus="resource_admin" authenticated={false}>
      <ResourceAdminContent />
    </AppInitializer>
  );
}
