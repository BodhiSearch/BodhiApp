'use client';

import React from 'react';
import { motion } from 'framer-motion';
import { useRouter } from 'next/navigation';
import AppInitializer from '@/components/AppInitializer';
import ApiModelForm from '@/components/api-models/ApiModelForm';
import { SETUP_STEPS, SETUP_STEP_LABELS, SETUP_TOTAL_STEPS } from '@/app/ui/setup/constants';
import { SetupProgress } from '@/app/ui/setup/SetupProgress';
import { BodhiLogo } from '@/app/ui/setup/BodhiLogo';
import { containerVariants, itemVariants } from '@/app/ui/setup/types';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { ROUTE_SETUP_COMPLETE } from '@/lib/constants';

function ApiModelsSetupContent() {
  const router = useRouter();

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

        {/* Welcome Section - Simple without benefit cards */}
        <motion.div variants={itemVariants}>
          <Card>
            <CardHeader className="text-center">
              <CardTitle className="flex items-center justify-center gap-3 text-2xl">
                <span className="text-3xl">☁️</span>
                Setup API Models
              </CardTitle>
              <CardDescription className="text-lg">
                Connect to cloud-based AI models like GPT-4, Claude, and more.
                <br />
                You can always add more models later from the Models page.
              </CardDescription>
            </CardHeader>
          </Card>
        </motion.div>

        {/* Main API Model Form */}
        <motion.div variants={itemVariants}>
          <ApiModelForm mode="setup" onSuccessRoute={ROUTE_SETUP_COMPLETE} onCancelRoute={ROUTE_SETUP_COMPLETE} />
        </motion.div>

        {/* Skip Button */}
        <motion.div variants={itemVariants} className="flex justify-center">
          <Button variant="outline" onClick={handleSkip} data-testid="skip-api-setup">
            Skip for Now
          </Button>
        </motion.div>

        {/* Help Section */}
        <motion.div variants={itemVariants}>
          <Card className="bg-muted/30">
            <CardContent className="py-6">
              <div className="text-center space-y-2">
                <p className="text-sm text-muted-foreground">
                  <strong>Don't have an API key?</strong> You can skip this step and add API models later.
                </p>
                <p className="text-xs text-muted-foreground">
                  API models complement your local models - you can always configure them from the Models page.
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
