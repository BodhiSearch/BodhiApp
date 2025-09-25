'use client';

import { motion } from 'framer-motion';
import { useRouter } from 'next/navigation';
import { SETUP_STEPS, SETUP_STEP_LABELS, SETUP_TOTAL_STEPS } from '@/app/ui/setup/constants';
import { SetupProgress } from '@/app/ui/setup/SetupProgress';
import { BenefitCard } from '@/app/ui/setup/BenefitCard';
import { BodhiLogo } from '@/app/ui/setup/BodhiLogo';
import { containerVariants, itemVariants } from '@/app/ui/setup/types';
import AppInitializer from '@/components/AppInitializer';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { ApiModelSetupForm } from './ApiModelSetupForm';
import { PROVIDER_BENEFITS } from '@/components/api-models/providers/constants';
import { ROUTE_SETUP_COMPLETE } from '@/lib/constants';

function ApiModelsSetupContent() {
  const router = useRouter();

  const handleComplete = () => {
    router.push(ROUTE_SETUP_COMPLETE);
  };

  const handleSkip = () => {
    router.push(ROUTE_SETUP_COMPLETE);
  };

  return (
    <main className="min-h-screen bg-background">
      <motion.div
        className="mx-auto max-w-4xl space-y-8 p-4 md:p-8"
        variants={containerVariants}
        initial="hidden"
        animate="visible"
        data-testid="api-models-setup-page"
      >
        {/* Progress Header */}
        <SetupProgress
          currentStep={SETUP_STEPS.API_MODELS}
          totalSteps={SETUP_TOTAL_STEPS}
          stepLabels={SETUP_STEP_LABELS}
        />

        {/* Logo */}
        <BodhiLogo />

        {/* Welcome Section */}
        <motion.div variants={itemVariants}>
          <Card>
            <CardHeader className="text-center">
              <CardTitle className="flex items-center justify-center gap-3 text-2xl">
                <span className="text-3xl">☁️</span>
                Add Cloud AI Models
              </CardTitle>
              <CardDescription className="text-lg">
                Unlock the power of cloud-based AI models like GPT-4, Claude, and more.
                <br />
                Keep your local models and add cloud AI for the best of both worlds.
              </CardDescription>
            </CardHeader>
          </Card>
        </motion.div>

        {/* Benefits Grid */}
        <motion.div variants={itemVariants}>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            {PROVIDER_BENEFITS.map((benefit) => (
              <BenefitCard key={benefit.title} {...benefit} />
            ))}
          </div>
        </motion.div>

        {/* Main Setup Form */}
        <motion.div variants={itemVariants}>
          <ApiModelSetupForm onComplete={handleComplete} onSkip={handleSkip} />
        </motion.div>

        {/* Help Section */}
        <motion.div variants={itemVariants}>
          <Card className="bg-muted/30">
            <CardContent className="py-6">
              <div className="text-center space-y-2">
                <p className="text-sm text-muted-foreground">
                  <strong>Don't have an API key?</strong> You can skip this step and add cloud models later.
                </p>
                <p className="text-xs text-muted-foreground">
                  Cloud AI models complement your local models - you can always add them from the Models page.
                </p>
              </div>
            </CardContent>
          </Card>
        </motion.div>
      </motion.div>
    </main>
  );
}

export default function ApiModelsSetupPage() {
  return (
    <AppInitializer allowedStatus="ready" authenticated={true}>
      <ApiModelsSetupContent />
    </AppInitializer>
  );
}