'use client';

import React, { useState } from 'react';

import { motion } from 'framer-motion';
import { useRouter } from 'next/navigation';

import { SetupContainer, SetupFooter } from '@/app/ui/setup/components';
import { itemVariants } from '@/app/ui/setup/types';
import AppInitializer from '@/components/AppInitializer';
import { BrowserExtensionCard } from '@/components/setup/BrowserExtensionCard';
import { useBrowserDetection } from '@/hooks/use-browser-detection';
import { useExtensionDetection } from '@/hooks/use-extension-detection';
import type { BrowserInfo } from '@/lib/browser-utils';
import { ROUTE_SETUP_COMPLETE } from '@/lib/constants';

function BrowserExtensionSetupContent() {
  const router = useRouter();
  const { detectedBrowser } = useBrowserDetection();
  const { status: extensionStatus, refresh } = useExtensionDetection();
  const [selectedBrowser, setSelectedBrowser] = useState<BrowserInfo | null>(null);

  const handleNext = () => {
    router.push(ROUTE_SETUP_COMPLETE);
  };

  return (
    <SetupContainer>
      <div data-testid="browser-extension-setup-page" data-page-state={extensionStatus}>
        {/* Main Card */}
        <motion.div variants={itemVariants} className="mb-8">
          <BrowserExtensionCard
            detectedBrowser={detectedBrowser}
            selectedBrowser={selectedBrowser}
            onBrowserSelect={setSelectedBrowser}
            extensionStatus={extensionStatus}
            onRefresh={refresh}
          />
        </motion.div>

        {/* Standard Footer */}
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
