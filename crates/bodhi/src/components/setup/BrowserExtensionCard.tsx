import { Check, Puzzle, RefreshCw } from 'lucide-react';

import { Button } from '@/components/ui/button';
import type { BrowserInfo } from '@/lib/browser-utils';
import { SetupCardIcon } from '@/routes/setup/-components/SetupCardIcon';

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
      <div
        className="rounded-[var(--radius-lg)] border border-border bg-muted/40 p-6"
        data-testid="extension-detecting"
      >
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
        className="space-y-4 rounded-[var(--radius-lg)] border border-border bg-muted/40 p-6 text-center"
        data-testid="extension-not-found"
      >
        <div>
          <p className="font-semibold">Extension Not Found</p>
          <p className="mt-1 text-sm text-muted-foreground">Install the extension, then verify the connection below.</p>
        </div>
        <Button variant="outline" onClick={onRefresh} data-testid="refresh-button" className="gap-2">
          <RefreshCw className="h-4 w-4" />
          Check Again
        </Button>
      </div>
    );
  }

  if (status === 'installed') {
    return (
      <div
        className="rounded-[var(--radius-lg)] border border-[hsl(var(--success)/0.35)] bg-[hsl(var(--success)/0.1)] p-6 text-center"
        data-testid="extension-found"
      >
        <div className="mx-auto mb-3 flex h-[38px] w-[38px] items-center justify-center rounded-full bg-[hsl(var(--success)/0.16)] text-[hsl(var(--success))]">
          <Check className="h-5 w-5" strokeWidth={3} data-testid="check-icon" />
        </div>
        <p className="font-semibold text-[hsl(var(--success))]">Extension Connected</p>
        <p className="mt-1 text-sm text-muted-foreground">Bodhi is now active in your browser. You&apos;re all set.</p>
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
    <section className="overflow-hidden rounded-[var(--radius-xl)] border border-border bg-card shadow-sm">
      <div className="px-9 py-8">
        <header className="mb-6 text-center">
          <SetupCardIcon icon={Puzzle} />
          <h2 className="text-[26px] font-bold leading-tight tracking-tight">Browser Extension Setup</h2>
          <p className="mx-auto mt-2 max-w-[50ch] text-[14.5px] leading-relaxed text-muted-foreground">
            Install the Bodhi extension to unlock AI features on any website you visit.
          </p>
        </header>

        <div className="space-y-6">
          <BrowserSelector
            detectedBrowser={detectedBrowser}
            selectedBrowser={selectedBrowser}
            onBrowserSelect={onBrowserSelect}
          />

          {showExtensionStatus && <ExtensionStatusDisplay status={extensionStatus} onRefresh={onRefresh} />}
        </div>
      </div>
    </section>
  );
}
