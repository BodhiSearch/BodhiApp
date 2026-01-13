import { FaChrome, FaFirefox, FaSafari, FaEdge } from 'react-icons/fa';

import { Badge } from '@/components/ui/badge';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { type BrowserInfo, type BrowserType, BROWSER_CONFIG } from '@/lib/browser-utils';

interface BrowserSelectorProps {
  detectedBrowser: BrowserInfo | null;
  selectedBrowser: BrowserInfo | null;
  onBrowserSelect: (browser: BrowserInfo) => void;
  className?: string;
}

const browserIcons: Record<BrowserType, React.ComponentType<{ className?: string }>> = {
  chrome: FaChrome,
  edge: FaEdge,
  firefox: FaFirefox,
  safari: FaSafari,
  unknown: FaChrome, // Fallback to chrome icon
};

const browserDisplayNames: Record<BrowserType, string> = {
  chrome: 'Google Chrome',
  edge: 'Microsoft Edge',
  firefox: 'Mozilla Firefox',
  safari: 'Safari',
  unknown: 'Unknown Browser',
};

export function BrowserSelector({
  detectedBrowser,
  selectedBrowser,
  onBrowserSelect,
  className,
}: BrowserSelectorProps) {
  const browserOptions: BrowserInfo[] = Object.entries(BROWSER_CONFIG).map(([type, config]) => ({
    name: browserDisplayNames[type as BrowserType],
    ...config,
  }));

  const handleValueChange = (value: string) => {
    const selected = browserOptions.find((browser) => browser.type === value);
    if (selected) {
      onBrowserSelect(selected);
    }
  };

  const renderBrowserOption = (browser: BrowserInfo, isDetected: boolean = false) => {
    const Icon = browserIcons[browser.type];
    return (
      <div className="flex items-center gap-2">
        <Icon className="h-4 w-4" />
        <span>{browser.name}</span>
        {isDetected && (
          <Badge variant="secondary" className="text-xs">
            detected
          </Badge>
        )}
      </div>
    );
  };

  const currentBrowser = selectedBrowser || detectedBrowser;

  return (
    <div className={className}>
      <div className="space-y-2">
        <label htmlFor="browser-selector" className="text-sm font-medium">
          Browser
        </label>
        <Select value={currentBrowser?.type || ''} onValueChange={handleValueChange}>
          <SelectTrigger id="browser-selector" data-testid="browser-selector" className="w-full">
            <SelectValue>
              {currentBrowser ? (
                renderBrowserOption(currentBrowser, detectedBrowser?.type === currentBrowser.type && !selectedBrowser)
              ) : (
                <span>Select a browser</span>
              )}
            </SelectValue>
          </SelectTrigger>
          <SelectContent>
            {browserOptions.map((browser) => (
              <SelectItem key={browser.type} value={browser.type}>
                {renderBrowserOption(browser, detectedBrowser?.type === browser.type)}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
      </div>

      {currentBrowser && (
        <div className="mt-4 p-3 border rounded-md bg-muted/50">
          <div className="space-y-2">
            <p className="text-sm text-muted-foreground">{currentBrowser.statusMessage}</p>
            {currentBrowser.supported && currentBrowser.extensionUrl && (
              <a
                href={currentBrowser.extensionUrl}
                target="_blank"
                rel="noopener noreferrer"
                className="inline-block text-sm text-primary hover:underline"
                data-testid="install-extension-link"
              >
                Install Bodhi Extension →
              </a>
            )}
          </div>
        </div>
      )}
    </div>
  );
}
