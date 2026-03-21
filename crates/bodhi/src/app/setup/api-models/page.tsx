import React from 'react';

import { motion } from 'framer-motion';
import { useNavigate } from '@tanstack/react-router';

import { SetupContainer, SetupFooter } from '@/app/setup/components';
import { itemVariants } from '@/app/setup/types';
import ApiModelForm from '@/components/api-models/ApiModelForm';
import AppInitializer from '@/components/AppInitializer';
import { ROUTE_SETUP_TOOLSETS } from '@/lib/constants';

function ApiModelsSetupContent() {
  const navigate = useNavigate();

  const handleSkip = () => {
    navigate({ to: ROUTE_SETUP_TOOLSETS });
  };

  return (
    <SetupContainer>
      <div data-testid="api-models-setup-page">
        {/* Main API Model Form */}
        <motion.div variants={itemVariants}>
          <ApiModelForm mode="setup" onSuccessRoute={ROUTE_SETUP_TOOLSETS} onCancelRoute={ROUTE_SETUP_TOOLSETS} />
        </motion.div>

        {/* Footer with clarification and Continue button */}
        <SetupFooter
          clarificationText="Don't have an API key? You can skip this step and add API models later."
          subText="API models complement your local models - you can always configure them from the Models page."
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
