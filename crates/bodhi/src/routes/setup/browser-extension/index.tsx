import React, { useState } from 'react';

import { useNavigate } from '@tanstack/react-router';
import { createFileRoute } from '@tanstack/react-router';
import { motion } from 'framer-motion';

import AppInitializer from '@/components/AppInitializer';
import { BrowserExtensionCard } from '@/components/setup/BrowserExtensionCard';
import { useBrowserDetection } from '@/hooks/use-browser-detection';
import { useExtensionDetection } from '@/hooks/use-extension-detection';
import type { BrowserInfo } from '@/lib/browser-utils';
import { ROUTE_SETUP_COMPLETE } from '@/lib/constants';
import { SetupContainer, SetupFooter } from '@/routes/setup/-components';
import { itemVariants } from '@/routes/setup/-shared/types';

export const Route = createFileRoute('/setup/browser-extension/')({
  component: BrowserExtensionSetupPage,
});

function BrowserExtensionSetupContent() {
  const navigate = useNavigate();
  const { detectedBrowser } = useBrowserDetection();
  const { status: extensionStatus, refresh } = useExtensionDetection();
  const [selectedBrowser, setSelectedBrowser] = useState<BrowserInfo | null>(null);

  const handleNext = () => {
    navigate({ to: ROUTE_SETUP_COMPLETE });
  };

  return (
    <SetupContainer>
      <div data-testid="browser-extension-setup-page" data-page-state={extensionStatus}>
        <motion.div variants={itemVariants} className="mb-8">
          <BrowserExtensionCard
            detectedBrowser={detectedBrowser}
            selectedBrowser={selectedBrowser}
            onBrowserSelect={setSelectedBrowser}
            extensionStatus={extensionStatus}
            onRefresh={refresh}
          />
        </motion.div>

        <SetupFooter
          clarificationText="Need help? The extension enables AI features directly in your browser tabs."
          subText="You can always install the extension later from the settings page."
          onContinue={handleNext}
          buttonLabel={extensionStatus === 'installed' ? 'Continue' : 'Skip for Now'}
          buttonTestId="browser-extension-continue"
        />
      </div>
    </SetupContainer>
  );
}

export default function BrowserExtensionSetupPage() {
  return (
    <AppInitializer allowedStatus="ready" authenticated={true}>
      <BrowserExtensionSetupContent />
    </AppInitializer>
  );
}
