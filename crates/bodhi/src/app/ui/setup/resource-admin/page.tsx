'use client';

import { SETUP_STEPS, SETUP_STEP_LABELS, SETUP_TOTAL_STEPS } from '@/app/ui/setup/constants';
import { SetupProgress } from '@/app/ui/setup/SetupProgress';
import AppInitializer from '@/components/AppInitializer';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardFooter, CardHeader, CardTitle } from '@/components/ui/card';
import { useOAuthInitiate } from '@/hooks/useOAuth';
import { motion } from 'framer-motion';
import { BodhiLogo } from '@/app/ui/setup/BodhiLogo';
import { handleSmartRedirect } from '@/lib/utils';
import { useState } from 'react';
import { useRouter } from 'next/navigation';

// Animation variants
const containerVariants = {
  hidden: { opacity: 0 },
  visible: {
    opacity: 1,
    transition: {
      staggerChildren: 0.1,
    },
  },
};

const itemVariants = {
  hidden: { y: 20, opacity: 0 },
  visible: {
    y: 0,
    opacity: 1,
  },
};

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
    <main className="min-h-screen bg-background p-4 md:p-8" data-testid="resource-admin-page">
      <motion.div
        className="mx-auto max-w-4xl space-y-8 p-4 md:p-8"
        variants={containerVariants}
        initial="hidden"
        animate="visible"
      >
        <SetupProgress
          currentStep={SETUP_STEPS.RESOURCE_ADMIN}
          totalSteps={SETUP_TOTAL_STEPS}
          stepLabels={SETUP_STEP_LABELS}
        />
        <BodhiLogo />

        <motion.div variants={itemVariants}>
          <Card>
            <CardHeader>
              <CardTitle className="text-center">Admin Setup</CardTitle>
            </CardHeader>
            <CardContent className="space-y-6">
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
            </CardContent>
            <CardFooter className="flex flex-col gap-4">
              <Button className="w-full" size="lg" onClick={handleOAuthInitiate} disabled={isButtonDisabled}>
                {isLoading ? 'Initiating...' : redirecting ? 'Redirecting...' : 'Continue with Login →'}
              </Button>
              <p className="text-sm text-muted-foreground text-center">Login with a valid email address to continue</p>
            </CardFooter>
          </Card>
        </motion.div>
      </motion.div>
    </main>
  );
}

export default function ResourceAdminPage() {
  return (
    <AppInitializer allowedStatus="resource-admin" authenticated={false}>
      <ResourceAdminContent />
    </AppInitializer>
  );
}
