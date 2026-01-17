'use client';

import { motion } from 'framer-motion';
import { useRouter } from 'next/navigation';

import { SetupContainer, SetupFooter } from '@/app/ui/setup/components';
import { itemVariants } from '@/app/ui/setup/types';
import AppInitializer from '@/components/AppInitializer';
import { ROUTE_SETUP_BROWSER_EXTENSION } from '@/lib/constants';

import { SetupToolsetForm } from './SetupToolsetForm';

function ToolsetsSetupContent() {
  const router = useRouter();

  const handleSuccess = () => {
    router.push(ROUTE_SETUP_BROWSER_EXTENSION);
  };

  const handleSkip = () => {
    router.push(ROUTE_SETUP_BROWSER_EXTENSION);
  };

  return (
    <SetupContainer>
      <div data-testid="toolsets-setup-page">
        {/* Main Toolset Config Form */}
        <motion.div variants={itemVariants}>
          <SetupToolsetForm toolsetId="builtin-exa-web-search" onSuccess={handleSuccess} />
        </motion.div>

        {/* Footer with clarification and Continue button - mt-4 for consistent padding */}
        <motion.div variants={itemVariants} className="mt-4">
          <SetupFooter
            clarificationText="Don't have an Exa API key? You can skip this step and configure later."
            subText="Web search enhances AI with real-time information from the internet."
            onContinue={handleSkip}
            buttonLabel="Continue"
            buttonTestId="skip-toolsets-setup"
          />
        </motion.div>
      </div>
    </SetupContainer>
  );
}

export default function ToolsetsSetupPage() {
  return (
    <AppInitializer allowedStatus="ready" authenticated={true}>
      <ToolsetsSetupContent />
    </AppInitializer>
  );
}
