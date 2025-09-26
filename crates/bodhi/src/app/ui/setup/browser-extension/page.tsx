'use client';

import React, { useState } from 'react';
import { motion } from 'framer-motion';
import { useRouter } from 'next/navigation';
import { Monitor, Check, Download, RefreshCw } from 'lucide-react';
import AppInitializer from '@/components/AppInitializer';
import { BrowserSelector } from '@/components/setup/BrowserSelector';
import { useBrowserDetection } from '@/hooks/use-browser-detection';
import { useExtensionDetection } from '@/hooks/use-extension-detection';
import { SETUP_STEPS, SETUP_STEP_LABELS, SETUP_TOTAL_STEPS } from '@/app/ui/setup/constants';
import { SetupProgress } from '@/app/ui/setup/SetupProgress';
import { BodhiLogo } from '@/app/ui/setup/BodhiLogo';
import { containerVariants, itemVariants } from '@/app/ui/setup/types';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { ROUTE_SETUP_COMPLETE } from '@/lib/constants';
import type { BrowserInfo } from '@/lib/browser-utils';

function BrowserExtensionSetupContent() {
  const router = useRouter();
  const { detectedBrowser } = useBrowserDetection();
  const { status: extensionStatus, extensionId, refresh } = useExtensionDetection();
  const [selectedBrowser, setSelectedBrowser] = useState<BrowserInfo | null>(null);

  // Current browser is either manually selected or auto-detected
  const currentBrowser = selectedBrowser || detectedBrowser;

  const handleNext = () => {
    router.push(ROUTE_SETUP_COMPLETE);
  };

  const renderExtensionStatus = () => {
    if (!currentBrowser?.supported) {
      return null;
    }

    switch (extensionStatus) {
      case 'detecting':
        return (
          <Card data-testid="extension-detecting">
            <CardHeader className="text-center">
              <CardTitle className="flex items-center justify-center gap-3 text-2xl">
                <RefreshCw className="h-8 w-8 animate-spin" />
                Checking for Extension
              </CardTitle>
              <CardDescription className="text-lg">
                Detecting if the Bodhi Browser extension is installed...
              </CardDescription>
            </CardHeader>
          </Card>
        );

      case 'installed':
        return (
          <Card className="border-green-200 dark:border-green-800" data-testid="extension-found">
            <CardHeader className="text-center">
              <CardTitle className="flex items-center justify-center gap-3 text-2xl text-green-700 dark:text-green-400">
                <Check className="h-8 w-8" />
                Extension Found!
              </CardTitle>
              <CardDescription className="text-lg">
                Perfect! The Bodhi Browser extension is installed and ready.
                {extensionId && (
                  <>
                    <br />
                    Extension ID: <code className="text-sm" data-testid="extension-id-display">{extensionId}</code>
                  </>
                )}
              </CardDescription>
            </CardHeader>
            <CardContent className="flex justify-center">
              <Button onClick={handleNext} size="lg" data-testid="next-button">
                Continue Setup
              </Button>
            </CardContent>
          </Card>
        );

      case 'not-installed':
        return (
          <Card data-testid="extension-not-found">
            <CardHeader className="text-center">
              <CardTitle className="flex items-center justify-center gap-3 text-2xl">
                <Download className="h-8 w-8" />
                Extension Not Found
              </CardTitle>
              <CardDescription className="text-lg">
                Install the extension to continue, then refresh this page.
              </CardDescription>
            </CardHeader>
            <CardContent className="flex justify-center space-x-4">
              <Button variant="outline" onClick={refresh} data-testid="refresh-button">
                <RefreshCw className="mr-2 h-4 w-4" />
                Check Again
              </Button>
              <Button onClick={handleNext} variant="outline" data-testid="skip-button">
                Skip for Now
              </Button>
            </CardContent>
          </Card>
        );

      default:
        return null;
    }
  };

  return (
    <main className="min-h-screen bg-background">
      <motion.div
        className="mx-auto max-w-4xl space-y-8 p-4 md:p-8"
        variants={containerVariants}
        initial="hidden"
        animate="visible"
        data-testid="browser-extension-setup-page"
        data-page-state={extensionStatus}
      >
        {/* Progress Header */}
        <SetupProgress
          currentStep={SETUP_STEPS.BROWSER_EXTENSION}
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
                <Monitor className="h-8 w-8" />
                Browser Extension Setup
              </CardTitle>
              <CardDescription className="text-lg">
                Choose your browser and install the Bodhi extension to unlock AI features on any website.
              </CardDescription>
            </CardHeader>
          </Card>
        </motion.div>

        {/* Browser Selector */}
        <motion.div variants={itemVariants}>
          <BrowserSelector
            detectedBrowser={detectedBrowser}
            selectedBrowser={selectedBrowser}
            onBrowserSelect={setSelectedBrowser}
          />
        </motion.div>

        {/* Extension Detection (only for supported browsers) */}
        {currentBrowser?.supported && <motion.div variants={itemVariants}>{renderExtensionStatus()}</motion.div>}

        {/* Continue button for unsupported browsers */}
        {currentBrowser && !currentBrowser.supported && (
          <motion.div variants={itemVariants} className="flex justify-center">
            <Button onClick={handleNext} data-testid="continue-button">
              Continue Setup
            </Button>
          </motion.div>
        )}

        {/* Help Section */}
        <motion.div variants={itemVariants}>
          <Card className="bg-muted/30">
            <CardContent className="py-6">
              <div className="text-center space-y-2">
                <p className="text-sm text-muted-foreground">
                  <strong>Need help?</strong> The extension enables AI features directly in your browser tabs.
                </p>
                <p className="text-xs text-muted-foreground">
                  You can always install the extension later from the settings page.
                </p>
              </div>
            </CardContent>
          </Card>
        </motion.div>
      </motion.div>
    </main>
  );
}

export default function BrowserExtensionSetupPage() {
  return (
    <AppInitializer allowedStatus="ready" authenticated={true}>
      <BrowserExtensionSetupContent />
    </AppInitializer>
  );
}
