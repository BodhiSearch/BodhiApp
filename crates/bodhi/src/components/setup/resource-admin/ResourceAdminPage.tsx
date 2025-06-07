import { SetupProgress } from '@/components/setup/SetupProgress';
import AppInitializer from '@/components/AppInitializer';
import { Button } from '@/components/ui/button';
import {
  Card,
  CardContent,
  CardFooter,
  CardHeader,
  CardTitle,
} from '@/components/ui/card';
import { ENDPOINT_APP_LOGIN } from '@/hooks/useQuery';
import { motion } from 'framer-motion';
import { BodhiLogo } from '@/components/setup/BodhiLogo';

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
    <main
      className="min-h-screen bg-background p-4 md:p-8"
      data-testid="resource-admin-page"
    >
      <motion.div
        className="mx-auto max-w-4xl space-y-8 p-4 md:p-8"
        variants={containerVariants}
        initial="hidden"
        animate="visible"
      >
        <SetupProgress currentStep={2} totalSteps={4} />
        <BodhiLogo />

        <motion.div variants={itemVariants}>
          <Card>
            <CardHeader>
              <CardTitle className="text-center">Admin Setup</CardTitle>
            </CardHeader>
            <CardContent className="space-y-6">
              <div className="prose dark:prose-invert mx-auto">
                <p className="text-center text-muted-foreground">
                  You are setting up Bodhi App in authenticated mode. The email
                  address you log in with will be granted admin role for this
                  app instance.
                </p>
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
