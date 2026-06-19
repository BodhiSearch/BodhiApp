import React from 'react';

import { useNavigate } from '@tanstack/react-router';
import { createFileRoute } from '@tanstack/react-router';
import { motion } from 'framer-motion';

import ApiModelForm from '@/components/api-models/ApiModelForm';
import AppInitializer from '@/components/AppInitializer';
import { ROUTE_SETUP_BROWSER_EXTENSION } from '@/lib/constants';

import { SetupContainer, SetupFooter } from '../-components';
import { itemVariants } from '../-shared/types';

export const Route = createFileRoute('/setup/api-models/')({
  component: ApiModelsSetupPage,
});

function ApiModelsSetupContent() {
  const navigate = useNavigate();

  const handleSkip = () => {
    navigate({ to: ROUTE_SETUP_BROWSER_EXTENSION });
  };

  return (
    <SetupContainer>
      <div data-testid="api-models-setup-page">
        <motion.div variants={itemVariants} className="mb-6 text-center">
          <h2 className="text-[26px] font-bold leading-tight tracking-tight">Set up API Models</h2>
          <p className="mx-auto mt-2 max-w-[50ch] text-[14.5px] leading-relaxed text-muted-foreground">
            Configure cloud-based AI models to complement your local ones. You can connect a provider now, or skip and
            add models later.
          </p>
        </motion.div>

        <motion.div variants={itemVariants}>
          <ApiModelForm
            mode="setup"
            onSuccessRoute={ROUTE_SETUP_BROWSER_EXTENSION}
            onCancelRoute={ROUTE_SETUP_BROWSER_EXTENSION}
          />
        </motion.div>

        <SetupFooter
          clarificationText="Don't have an API key? You can skip this step and add API models later — they always complement your local models from the Models page."
          onContinue={handleSkip}
          buttonLabel="Continue"
          buttonTestId="skip-api-setup"
        />
      </div>
    </SetupContainer>
  );
}

export default function ApiModelsSetupPage() {
  return (
    <AppInitializer allowedStatus="ready" authenticated={true}>
      <ApiModelsSetupContent />
    </AppInitializer>
  );
}
