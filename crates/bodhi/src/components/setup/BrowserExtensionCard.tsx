'use client';

import { Check, RefreshCw } from 'lucide-react';

import { Button } from '@/components/ui/button';
import { Card, CardHeader, CardTitle, CardDescription, CardContent } from '@/components/ui/card';
import type { BrowserInfo } from '@/lib/browser-utils';

import { BrowserSelector } from './BrowserSelector';

type ExtensionStatus = 'detecting' | 'installed' | 'not-installed';

interface BrowserExtensionCardProps {
  detectedBrowser: BrowserInfo | null;
  selectedBrowser: BrowserInfo | null;
  onBrowserSelect: (browser: BrowserInfo) => void;
  extensionStatus: ExtensionStatus;
  onRefresh: () => void;
}

function ExtensionStatusDisplay({ status, onRefresh }: { status: ExtensionStatus; onRefresh: () => void }) {
  if (status === 'detecting') {
    return (
      <div className="rounded-lg border bg-muted/30 p-6" data-testid="extension-detecting">
        <div className="flex items-center justify-center gap-2">
          <RefreshCw className="h-4 w-4 animate-spin" data-testid="refresh-icon" />
          <span className="text-sm">Checking for extension...</span>
        </div>
      </div>
    );
  }

  if (status === 'not-installed') {
    return (
      <div
        className="rounded-lg border bg-orange-50 dark:bg-orange-900/10 border-orange-200 dark:border-orange-800 p-6 space-y-4"
        data-testid="extension-not-found"
      >
        <div className="text-center">
          <p className="font-medium text-orange-900 dark:text-orange-200">Extension Not Found</p>
          <p className="text-sm text-muted-foreground mt-2">Install the extension and click below to verify</p>
        </div>
        <div className="flex justify-center">
          <Button variant="outline" onClick={onRefresh} data-testid="refresh-button">
            <RefreshCw className="mr-2 h-4 w-4" />
            Check Again
          </Button>
        </div>
      </div>
    );
  }

  if (status === 'installed') {
    return (
      <div
        className="rounded-lg border border-green-200 dark:border-green-800 bg-green-50 dark:bg-green-900/10 p-6"
        data-testid="extension-found"
      >
        <div className="flex items-center justify-center gap-2">
          <Check className="h-5 w-5 text-green-600 dark:text-green-400" data-testid="check-icon" />
          <p className="font-medium text-green-900 dark:text-green-200">Extension Ready</p>
        </div>
        <p className="text-center text-sm text-green-700 dark:text-green-300 mt-2">
          The Bodhi Browser extension is installed and ready to use
        </p>
      </div>
    );
  }

  return null;
}

export function BrowserExtensionCard({
  detectedBrowser,
  selectedBrowser,
  onBrowserSelect,
  extensionStatus,
  onRefresh,
}: BrowserExtensionCardProps) {
  const currentBrowser = selectedBrowser || detectedBrowser;
  const showExtensionStatus = currentBrowser?.supported;

  return (
    <Card>
      <CardHeader className="text-center">
        <CardTitle>Browser Extension Setup</CardTitle>
        <CardDescription>
          Choose your browser and install the Bodhi extension to unlock AI features on any website.
        </CardDescription>
      </CardHeader>

      <CardContent className="space-y-6">
        {/* Browser Selector Section */}
        <BrowserSelector
          detectedBrowser={detectedBrowser}
          selectedBrowser={selectedBrowser}
          onBrowserSelect={onBrowserSelect}
        />

        {/* Extension Status Section - Only show for supported browsers */}
        {showExtensionStatus && <ExtensionStatusDisplay status={extensionStatus} onRefresh={onRefresh} />}
      </CardContent>
    </Card>
  );
}
