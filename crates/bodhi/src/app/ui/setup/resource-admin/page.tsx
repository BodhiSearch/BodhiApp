'use client';

import { motion } from 'framer-motion';
// import AppInitializer from '@/components/AppInitializer';
import { ENDPOINT_APP_LOGIN } from '@/hooks/useQuery';
import { SetupProgress } from '@/components/setup/SetupProgress';
import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
  CardFooter,
} from '@/components/ui/card';
import { Button } from '@/components/ui/button';

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
  return (
    <motion.div
      className="mx-auto max-w-4xl space-y-8 p-4 md:p-8"
      variants={containerVariants}
      initial="hidden"
      animate="visible"
    >
      {/* Progress Indicator */}
      <SetupProgress currentStep={2} totalSteps={5} />

      {/* Admin Role Info with Login */}
      <motion.div variants={itemVariants}>
        <Card>
          <CardHeader>
            <CardTitle className="text-center">Admin Setup</CardTitle>
          </CardHeader>
          <CardContent className="space-y-6">
            <div className="prose dark:prose-invert mx-auto">
              <p className="text-center text-muted-foreground">
                You&apos;re setting up Bodhi App in authenticated mode. The
                account you log in with will be granted admin role.
              </p>
            </div>

            <div className="space-y-4 text-sm">
              <div className="space-y-2">
                <h3 className="font-semibold">As an Admin, you can:</h3>
                <ul className="list-disc pl-5 space-y-1 text-muted-foreground">
                  <li>Manage user access and permissions</li>
                  <li>Configure system-wide settings</li>
                  <li>Monitor resource usage</li>
                  <li>Manage API tokens</li>
                </ul>
              </div>
            </div>
          </CardContent>
          <CardFooter className="flex flex-col gap-4">
            <Button
              className="w-full"
              size="lg"
              onClick={() => (window.location.href = ENDPOINT_APP_LOGIN)}
            >
              Continue with Login →
            </Button>
            <p className="text-sm text-muted-foreground text-center">
              Login with a valid email address to continue
            </p>
          </CardFooter>
        </Card>
      </motion.div>
    </motion.div>
  );
}

export default function ResourceAdminPage() {
  return (
    // <AppInitializer allowedStatus="resource-admin" authenticated={false}>
    <main
      className="min-h-screen bg-background"
      data-testid="resource-admin-page"
    >
      <ResourceAdminContent />
    </main>
    // </AppInitializer>
  );
}
