# Plan for Browser Extension Setup Page with Browser Detection

## Overview
Implement a new setup page (Step 5 of 6) to detect the user's browser and guide them through installing the bodhi-browser extension. This page will detect the browser type, show appropriate extension availability information, and provide browser-specific installation guidance.

## Key Features
- **Browser Detection**: Automatically detect browser using ua-parser-js (same as bodhi-js)
- **Browser Selection Dropdown**: Allow manual browser selection with logos
- **Extension Support Status**: Show availability per browser (Chrome/Edge supported, Firefox/Safari coming soon)
- **Extension Detection**: Detect if extension is already installed
- **Browser-Specific URLs**: Chrome Web Store links for supported browsers

## Incremental Phase-wise Implementation Plan

### Phase 1: Browser Detection Hook ✅ PENDING (2 tests)
- ✅ Create `crates/bodhi/src/hooks/use-browser-detection.ts`
- ✅ Add ua-parser-js dependency to package.json
- ✅ Implement browser detection using UAParser (following bodhi-js pattern)
- ✅ Support browser types: chrome, edge, firefox, safari, unknown
- ✅ Return browser info with name and type
- ✅ **Test after completion**: 2 tests for browser detection logic

### Phase 2: Extension Detection Hook ✅ PENDING (2 tests)
- ✅ Create `crates/bodhi/src/hooks/use-extension-detection.ts`
- ✅ Implement detection logic for `window.bodhiext` object
- ✅ Handle `bodhiext:initialized` event listening
- ✅ Provide refresh mechanism for post-installation detection
- ✅ Return detection status: 'detecting', 'installed', 'not-installed'
- ✅ Extract extension ID when available
- ✅ **Test after completion**: 2 tests for extension detection states

### Phase 3: Browser Selector Component ✅ PENDING (3 tests)
- ✅ Create `crates/bodhi/src/app/ui/setup/browser-extension/BrowserSelector.tsx`
- ✅ Create browser type definitions and utils in `crates/bodhi/src/lib/browser-utils.ts`
- ✅ Implement dropdown with browser logos (Chrome, Edge, Firefox, Safari, Unknown)
- ✅ Show browser-specific information below dropdown
- ✅ Handle browser selection changes
- ✅ **Test after completion**: 3 tests for browser selector functionality

### Phase 4: Main Setup Page - Basic Structure ✅ PENDING (3 tests)
- ✅ Create new `crates/bodhi/src/app/ui/setup/browser-extension/page.tsx`
- ✅ Setup progress header showing "Extension" (step 5 of 6)
- ✅ Integrate browser detection and extension detection hooks
- ✅ Basic page structure with AppInitializer
- ✅ **Test after completion**: 3 tests for page authentication and initial render

### Phase 5: Main Setup Page - Full UI Implementation ✅ PENDING (3 tests)
- ✅ Implement browser-specific UI states:
  - **Chrome/Edge**: Show extension detection + Chrome store link
  - **Firefox/Safari**: Show "Coming soon" message
  - **Unknown**: Show "Not supported" message
- ✅ Integrate BrowserSelector component
- ✅ Use consistent motion animations with other setup pages
- ✅ **Test after completion**: 3 tests for UI states and browser-specific content

### Phase 6: Navigation Flow Updates ✅ PENDING (2 tests)
- ✅ Update `crates/bodhi/src/app/ui/setup/api-models/page.tsx`
- ✅ Change navigation from `ROUTE_SETUP_COMPLETE` to `ROUTE_SETUP_BROWSER_EXTENSION`
- ✅ Update both success and skip button navigation
- ✅ Verify build success after navigation changes
- ✅ **Test after completion**: 2 tests for navigation flow changes

### Phase 7: Integration Testing ✅ PENDING (3 tests)
- ✅ Create comprehensive integration tests in `page.test.tsx`
- ✅ Test complete browser + extension detection workflow
- ✅ Test all browser types with different extension states
- ✅ Test navigation flows and user interactions
- ✅ **Test after completion**: 3 tests for end-to-end integration

## Detailed Implementation Specifications

### Browser Detection Implementation

Using ua-parser-js (same library as bodhi-js) to detect browser type:

```typescript
// crates/bodhi/src/lib/browser-utils.ts
import { UAParser } from 'ua-parser-js';

export type BrowserType = 'chrome' | 'edge' | 'firefox' | 'safari' | 'unknown';

export interface BrowserInfo {
  name: string;
  type: BrowserType;
  supported: boolean;
  extensionUrl: string | null;
  statusMessage: string;
}

export const BROWSER_CONFIG: Record<BrowserType, Omit<BrowserInfo, 'name'>> = {
  chrome: {
    type: 'chrome',
    supported: true,
    extensionUrl: 'https://chrome.google.com/webstore/detail/bodhi-browser/[EXTENSION_ID]',
    statusMessage: 'Extension available in Chrome Web Store'
  },
  edge: {
    type: 'edge',
    supported: true,
    extensionUrl: 'https://chrome.google.com/webstore/detail/bodhi-browser/[EXTENSION_ID]',
    statusMessage: 'Extension available in Chrome Web Store (Edge uses Chrome extensions)'
  },
  firefox: {
    type: 'firefox',
    supported: false,
    extensionUrl: null,
    statusMessage: 'Firefox extension coming soon'
  },
  safari: {
    type: 'safari',
    supported: false,
    extensionUrl: null,
    statusMessage: 'Safari extension coming soon'
  },
  unknown: {
    type: 'unknown',
    supported: false,
    extensionUrl: null,
    statusMessage: 'Extension not available for this browser'
  }
};

export function detectBrowser(): BrowserInfo {
  const parser = new UAParser();
  const browser = parser.getBrowser();
  const browserName = browser.name?.toLowerCase() || '';

  let type: BrowserType = 'unknown';
  let name = browser.name || 'Unknown Browser';

  if (browserName.includes('chrome')) {
    type = 'chrome';
    name = 'Google Chrome';
  } else if (browserName.includes('edge')) {
    type = 'edge';
    name = 'Microsoft Edge';
  } else if (browserName.includes('firefox')) {
    type = 'firefox';
    name = 'Mozilla Firefox';
  } else if (browserName.includes('safari')) {
    type = 'safari';
    name = 'Safari';
  }

  return {
    name,
    ...BROWSER_CONFIG[type]
  };
}
```

### Browser Detection Hook

```typescript
// crates/bodhi/src/hooks/use-browser-detection.ts
import { useState, useEffect } from 'react';
import { detectBrowser, type BrowserInfo } from '@/lib/browser-utils';

export function useBrowserDetection() {
  const [detectedBrowser, setDetectedBrowser] = useState<BrowserInfo | null>(null);
  const [selectedBrowser, setSelectedBrowser] = useState<BrowserInfo | null>(null);

  useEffect(() => {
    const browser = detectBrowser();
    setDetectedBrowser(browser);
    setSelectedBrowser(browser); // Initially select the detected browser
  }, []);

  return {
    detectedBrowser,
    selectedBrowser,
    setSelectedBrowser
  };
}
```

### Extension Detection Mechanism

The bodhi-browser extension injects a `window.bodhiext` object when loaded:

```typescript
// Extension creates this on window
window.bodhiext = {
  getExtensionId(): Promise<string>
  ping(): Promise<any>
  serverState(): Promise<ServerStateInfo>
  chat: {
    completions: {
      create: (params: any) => Promise<any> | AsyncIterable<any>
    }
  }
  // ... other methods
}
```

**Key Detection Points:**
1. **Object Presence**: Check if `window.bodhiext` exists
2. **Initialization Event**: Listen for `bodhiext:initialized` custom event
3. **Extension ID**: Call `window.bodhiext.getExtensionId()` for verification
4. **Page Refresh Required**: Extension injects on page load, so refresh needed after install

### Extension Detection Hook Implementation

```typescript
// crates/bodhi/src/hooks/use-extension-detection.ts
import { useState, useEffect } from 'react';

type ExtensionStatus = 'detecting' | 'installed' | 'not-installed';

interface ExtensionDetection {
  status: ExtensionStatus;
  extensionId: string | null;
  refresh: () => void;
  redetect: () => void;
}

export function useExtensionDetection(): ExtensionDetection {
  const [status, setStatus] = useState<ExtensionStatus>('detecting');
  const [extensionId, setExtensionId] = useState<string | null>(null);

  const checkExtension = async () => {
    try {
      if (typeof window !== 'undefined' && (window as any).bodhiext) {
        const id = await (window as any).bodhiext.getExtensionId();
        setExtensionId(id);
        setStatus('installed');
      } else {
        setStatus('not-installed');
      }
    } catch (error) {
      console.log('Extension detection error:', error);
      setStatus('not-installed');
    }
  };

  useEffect(() => {
    // Initial check
    const timer = setTimeout(checkExtension, 500); // Small delay for extension loading

    // Listen for extension initialization
    const handleInitialized = (event: CustomEvent) => {
      setExtensionId(event.detail.extensionId);
      setStatus('installed');
    };

    window.addEventListener('bodhiext:initialized', handleInitialized as EventListener);

    return () => {
      clearTimeout(timer);
      window.removeEventListener('bodhiext:initialized', handleInitialized as EventListener);
    };
  }, []);

  const refresh = () => {
    window.location.reload();
  };

  const redetect = () => {
    setStatus('detecting');
    setTimeout(checkExtension, 100);
  };

  return { status, extensionId, refresh, redetect };
}
```

### Browser Selector Component

```typescript
// crates/bodhi/src/app/ui/setup/browser-extension/BrowserSelector.tsx
'use client';

import React from 'react';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Card, CardContent, CardDescription } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Chrome, ExternalLink } from 'lucide-react';
import type { BrowserInfo, BrowserType } from '@/lib/browser-utils';

// Browser Icons (using Lucide icons + custom icons)
const BrowserIcons = {
  chrome: Chrome,
  edge: Chrome, // Edge uses Chrome-like icon
  firefox: Chrome, // Placeholder - would use Firefox icon
  safari: Chrome, // Placeholder - would use Safari icon
  unknown: Chrome // Generic browser icon
};

interface BrowserSelectorProps {
  detectedBrowser: BrowserInfo | null;
  selectedBrowser: BrowserInfo | null;
  onBrowserSelect: (browser: BrowserInfo) => void;
  availableBrowsers: BrowserInfo[];
}

export function BrowserSelector({
  detectedBrowser,
  selectedBrowser,
  onBrowserSelect,
  availableBrowsers
}: BrowserSelectorProps) {
  const handleBrowserChange = (browserType: string) => {
    const browser = availableBrowsers.find(b => b.type === browserType);
    if (browser) {
      onBrowserSelect(browser);
    }
  };

  const handleInstallExtension = () => {
    if (selectedBrowser?.extensionUrl) {
      window.open(selectedBrowser.extensionUrl, '_blank');
    }
  };

  return (
    <div className="space-y-4">
      <div>
        <label className="text-sm font-medium mb-2 block">
          Select your browser:
        </label>
        <Select
          value={selectedBrowser?.type || ''}
          onValueChange={handleBrowserChange}
          data-testid="browser-selector"
        >
          <SelectTrigger>
            <SelectValue placeholder="Choose your browser" />
          </SelectTrigger>
          <SelectContent>
            {availableBrowsers.map((browser) => {
              const Icon = BrowserIcons[browser.type];
              return (
                <SelectItem key={browser.type} value={browser.type}>
                  <div className="flex items-center gap-2">
                    <Icon className="h-4 w-4" />
                    <span>{browser.name}</span>
                    {detectedBrowser?.type === browser.type && (
                      <span className="text-xs text-muted-foreground">(detected)</span>
                    )}
                  </div>
                </SelectItem>
              );
            })}
          </SelectContent>
        </Select>
      </div>

      {selectedBrowser && (
        <Card data-testid="browser-info-card">
          <CardContent className="pt-4">
            <div className="text-center space-y-3">
              <CardDescription className="text-base">
                {selectedBrowser.statusMessage}
              </CardDescription>

              {selectedBrowser.supported && selectedBrowser.extensionUrl && (
                <Button
                  onClick={handleInstallExtension}
                  size="lg"
                  data-testid="install-extension-button"
                >
                  <ExternalLink className="mr-2 h-4 w-4" />
                  Install Extension
                </Button>
              )}

              {!selectedBrowser.supported && (
                <p className="text-sm text-muted-foreground">
                  We're working on bringing the extension to {selectedBrowser.name}.
                </p>
              )}
            </div>
          </CardContent>
        </Card>
      )}
    </div>
  );
}
```

### Main Page Component with Browser Detection

```typescript
// crates/bodhi/src/app/ui/setup/browser-extension/page.tsx
'use client';

import React from 'react';
import { motion } from 'framer-motion';
import { useRouter } from 'next/navigation';
import AppInitializer from '@/components/AppInitializer';
import { useBrowserDetection } from '@/hooks/use-browser-detection';
import { useExtensionDetection } from '@/hooks/use-extension-detection';
import { BrowserSelector } from './BrowserSelector';
import { SETUP_STEPS, SETUP_STEP_LABELS, SETUP_TOTAL_STEPS } from '@/app/ui/setup/constants';
import { SetupProgress } from '@/app/ui/setup/SetupProgress';
import { BodhiLogo } from '@/app/ui/setup/BodhiLogo';
import { containerVariants, itemVariants } from '@/app/ui/setup/types';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { ROUTE_SETUP_COMPLETE } from '@/lib/constants';
import { RefreshCw, Check, Download, Monitor } from 'lucide-react';
import { BROWSER_CONFIG, type BrowserInfo } from '@/lib/browser-utils';

function BrowserExtensionSetupContent() {
  const router = useRouter();
  const { detectedBrowser, selectedBrowser, setSelectedBrowser } = useBrowserDetection();
  const { status: extensionStatus, extensionId, refresh } = useExtensionDetection();

  // Create available browsers list
  const availableBrowsers: BrowserInfo[] = Object.entries(BROWSER_CONFIG).map(([type, config]) => ({
    name: type === 'chrome' ? 'Google Chrome' :
          type === 'edge' ? 'Microsoft Edge' :
          type === 'firefox' ? 'Mozilla Firefox' :
          type === 'safari' ? 'Safari' :
          'Unknown Browser',
    ...config
  }));

  const handleNext = () => {
    router.push(ROUTE_SETUP_COMPLETE);
  };

  const renderExtensionContent = () => {
    // Only show extension detection for supported browsers
    if (!selectedBrowser?.supported) {
      return null;
    }

    switch (extensionStatus) {
      case 'detecting':
        return (
          <Card>
            <CardHeader className="text-center">
              <CardTitle className="flex items-center justify-center gap-3 text-2xl">
                <RefreshCw className="h-8 w-8 animate-spin" />
                Checking Extension
              </CardTitle>
              <CardDescription className="text-lg">
                Looking for the Bodhi Browser extension...
              </CardDescription>
            </CardHeader>
          </Card>
        );

      case 'installed':
        return (
          <Card className="border-green-200 bg-green-50 dark:border-green-800 dark:bg-green-950">
            <CardHeader className="text-center">
              <CardTitle className="flex items-center justify-center gap-3 text-2xl text-green-700 dark:text-green-300">
                <Check className="h-8 w-8" />
                Extension Found!
              </CardTitle>
              <CardDescription className="text-lg">
                Perfect! The Bodhi Browser extension is installed and ready.
                {extensionId && (
                  <><br />Extension ID: <code className="text-sm">{extensionId}</code></>
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
          <Card>
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
            availableBrowsers={availableBrowsers}
          />
        </motion.div>

        {/* Extension Detection (only for supported browsers) */}
        {selectedBrowser?.supported && (
          <motion.div variants={itemVariants}>
            {renderExtensionContent()}
          </motion.div>
        )}

        {/* Skip button for unsupported browsers */}
        {selectedBrowser && !selectedBrowser.supported && (
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
    <AppInitializer requireAuth requireAppStatus>
      <BrowserExtensionSetupContent />
    </AppInitializer>
  );
}
```

## Testing Implementation Plan (18 Tests Total)

### Phase 1 Tests: Browser Detection Hook ✅ PENDING (2 tests)
**Test 1.1**: Browser detection with different user agents ✅
```typescript
describe('useBrowserDetection hook')
it('detects Chrome browser correctly')
it('detects Firefox browser and marks as unsupported')
```
- ✅ Mock UAParser to return different browser types
- ✅ Test Chrome detection returns supported: true
- ✅ Test Firefox detection returns supported: false
- ✅ **Run tests after Phase 1 completion**

**Test 1.2**: Browser selection functionality ✅
```typescript
it('allows manual browser selection override')
```
- ✅ Test initial selected browser matches detected
- ✅ Test setSelectedBrowser changes selection
- ✅ **Run tests after Phase 1 completion**

### Phase 2 Tests: Extension Detection Hook ✅ PENDING (2 tests)
**Test 2.1**: Extension detection states ✅
```typescript
describe('useExtensionDetection hook')
it('detects installed extension')
it('handles extension not installed')
```
- ✅ Mock window.bodhiext for installed state
- ✅ Test uninstalled state detection
- ✅ **Run tests after Phase 2 completion**

**Test 2.2**: Extension initialization event ✅
```typescript
it('listens for bodhiext:initialized event')
```
- ✅ Test custom event handling
- ✅ Verify extension ID extraction
- ✅ **Run tests after Phase 2 completion**

### Phase 3 Tests: Browser Selector Component ✅ PENDING (3 tests)
**Test 3.1**: Browser dropdown rendering ✅
```typescript
describe('BrowserSelector component')
it('renders browser dropdown with all options')
```
- ✅ Test dropdown shows all browser types
- ✅ Verify detected browser marked as "(detected)"
- ✅ **Run tests after Phase 3 completion**

**Test 3.2**: Browser selection interaction ✅
```typescript
it('handles browser selection changes')
```
- ✅ Test dropdown selection triggers callback
- ✅ Verify browser info card updates
- ✅ **Run tests after Phase 3 completion**

**Test 3.3**: Browser-specific information display ✅
```typescript
it('shows correct information for each browser type')
```
- ✅ Test Chrome shows install button
- ✅ Test Firefox shows "coming soon" message
- ✅ Test Safari shows "coming soon" message
- ✅ **Run tests after Phase 3 completion**

### Phase 4 Tests: Main Page Basic Structure ✅ PENDING (3 tests)
**Test 4.1**: Page authentication and initialization ✅
```typescript
describe('BrowserExtensionSetupPage')
it('renders page with correct authentication requirements')
```
- ✅ Verify AppInitializer wrapper
- ✅ Test page container data-testid
- ✅ **Run tests after Phase 4 completion**

**Test 4.2**: Setup progress and navigation ✅
```typescript
it('displays correct setup progress')
```
- ✅ Verify SetupProgress shows step 5 of 6
- ✅ Test "Extension" label display
- ✅ **Run tests after Phase 4 completion**

**Test 4.3**: Basic page structure ✅
```typescript
it('renders welcome section and logo')
```
- ✅ Test BodhiLogo component rendering
- ✅ Verify welcome card with Monitor icon
- ✅ **Run tests after Phase 4 completion**

### Phase 5 Tests: Full UI Implementation ✅ PENDING (3 tests)
**Test 5.1**: Browser-specific UI states ✅
```typescript
describe('Browser-specific UI behavior')
it('shows extension detection for supported browsers')
it('shows coming soon message for unsupported browsers')
```
- ✅ Test Chrome/Edge shows extension detection
- ✅ Test Firefox/Safari shows continue button only
- ✅ **Run tests after Phase 5 completion**

**Test 5.2**: Extension detection integration ✅
```typescript
it('integrates browser and extension detection correctly')
```
- ✅ Test supported browser + extension installed
- ✅ Test supported browser + extension not installed
- ✅ **Run tests after Phase 5 completion**

**Test 5.3**: Navigation button behavior ✅
```typescript
it('shows correct navigation buttons based on state')
```
- ✅ Test "Continue Setup" for extension found
- ✅ Test "Skip for Now" for extension not found
- ✅ Test "Continue Setup" for unsupported browsers
- ✅ **Run tests after Phase 5 completion**

### Phase 6 Tests: Navigation Flow Updates ✅ PENDING (2 tests)
**Test 6.1**: API models navigation update ✅
```typescript
describe('Navigation flow changes')
it('api-models page navigates to browser-extension')
```
- ✅ Test ApiModelForm success route updated
- ✅ Test skip button navigation updated
- ✅ **Run tests after Phase 6 completion**

**Test 6.2**: Complete navigation flow ✅
```typescript
it('browser-extension page navigates to complete')
```
- ✅ Test all navigation buttons go to setup/complete
- ✅ Verify router.push calls
- ✅ **Run tests after Phase 6 completion**

### Phase 7 Tests: Integration Testing ✅ PENDING (3 tests)
**Test 7.1**: Complete Chrome workflow ✅
```typescript
describe('End-to-end integration testing')
it('handles complete Chrome extension installation flow')
```
- ✅ Test Chrome detection → extension detection → installation → success
- ✅ Verify all UI states and transitions
- ✅ **Run tests after Phase 7 completion**

**Test 7.2**: Unsupported browser workflow ✅
```typescript
it('handles Firefox browser with no extension available')
```
- ✅ Test Firefox detection → coming soon message → continue
- ✅ Verify proper messaging and navigation
- ✅ **Run tests after Phase 7 completion**

**Test 7.3**: Manual browser selection ✅
```typescript
it('handles manual browser override scenarios')
```
- ✅ Test detection override → different browser selection
- ✅ Verify UI updates correctly
- ✅ **Run tests after Phase 7 completion**

### Testing Strategy Summary

**Total: 18 Tests across 7 phases**
1. **Phase 1**: Browser detection hook (2 tests)
2. **Phase 2**: Extension detection hook (2 tests)
3. **Phase 3**: Browser selector component (3 tests)
4. **Phase 4**: Main page basic structure (3 tests)
5. **Phase 5**: Full UI implementation (3 tests)
6. **Phase 6**: Navigation flow updates (2 tests)
7. **Phase 7**: Integration testing (3 tests)

**Incremental Testing Approach:**
- ✅ After each phase implementation, run that phase's tests
- ✅ Fix any issues before proceeding to next phase
- ✅ Build confidence through incremental validation
- ✅ Ensure robust implementation with comprehensive coverage

**Key Testing Areas:**
- **Browser Detection**: All browser types properly detected and categorized
- **Extension Detection**: Extension installation status correctly identified
- **UI States**: All combinations of browser + extension states render correctly
- **Navigation**: Proper routing throughout setup flow
- **User Interactions**: Dropdowns, buttons, and links work as expected
- **Integration**: End-to-end workflows function seamlessly

## Navigation Flow Updates

### Current Flow:
1. Welcome (Step 1) → Resource Admin (Step 2) → Download Models (Step 3) → API Models (Step 4) → **Complete (Step 6)**

### Updated Flow:
1. Welcome (Step 1) → Resource Admin (Step 2) → Download Models (Step 3) → API Models (Step 4) → **Browser Extension (Step 5)** → Complete (Step 6)

### Files to Update:
1. **`api-models/page.tsx`**: Change navigation from `ROUTE_SETUP_COMPLETE` to `ROUTE_SETUP_BROWSER_EXTENSION`
   - Update `onSuccessRoute` prop in ApiModelForm
   - Update `onCancelRoute` prop in ApiModelForm
   - Update `handleSkip` function

## Implementation Notes

### Chrome Extension Store URL
- **Placeholder URL**: `https://chrome.google.com/webstore/detail/bodhi-browser/[EXTENSION_ID]`
- **Update Required**: Once extension is published, update with actual Chrome Web Store URL
- **New Tab**: Always open in new tab using `window.open(url, '_blank')`

### Extension Detection Timing
- **Initial Delay**: Small 500ms delay for extension loading
- **Event Listening**: Listen for `bodhiext:initialized` event for dynamic detection
- **Refresh Required**: Page refresh needed after extension installation
- **Error Handling**: Gracefully handle detection failures

### Accessibility Considerations
- **Loading State**: Clear loading indicator during detection
- **Success State**: Green styling and checkmark for successful detection
- **Action State**: Clear call-to-action buttons for installation
- **Help Text**: Explanatory text for each state

### Security Considerations
- **External Links**: Chrome Web Store links open in new tab
- **Extension Validation**: Verify extension by calling getExtensionId()
- **Error Boundaries**: Handle detection errors gracefully
- **No Sensitive Data**: No sensitive information exposed in extension detection

## Future Enhancements

### Extension Store Support
- **Multi-Browser**: Support for Edge, Firefox extension stores
- **Browser Detection**: Detect user's browser and show appropriate store link
- **Direct Installation**: Browser-specific installation APIs if available

### Enhanced Detection
- **Real-time Updates**: Use MutationObserver to detect extension injection
- **Health Checks**: Verify extension functionality beyond just presence
- **Version Detection**: Show extension version information when available

### User Experience
- **Installation Progress**: Show progress during extension installation
- **Onboarding**: Mini-tutorial for using extension features
- **Troubleshooting**: Help section for common installation issues

## Dependency Requirements

### Add ua-parser-js to package.json
```json
{
  "dependencies": {
    "ua-parser-js": "^1.0.40"
  }
}
```

The `ua-parser-js` library is already used in the bodhi-js project and provides reliable browser detection across all major browsers and platforms.

## Implementation Summary

### File Structure
```
crates/bodhi/src/
├── lib/
│   └── browser-utils.ts           # Browser type definitions and detection
├── hooks/
│   ├── use-browser-detection.ts   # Browser detection hook
│   └── use-extension-detection.ts # Extension detection hook
├── app/ui/setup/browser-extension/
│   ├── page.tsx                   # Main setup page with browser + extension detection
│   ├── page.test.tsx              # Comprehensive test suite (18 tests)
│   └── BrowserSelector.tsx        # Browser dropdown component
└── app/ui/setup/api-models/
    └── page.tsx                   # Updated to navigate to browser-extension
```

## Final Implementation Status: ✅ ALL PHASES PENDING

This specification provides a complete blueprint for implementing the browser extension setup page with advanced browser detection:

### Enhanced Key Features
- ✅ **Browser Detection**: Automatic browser detection using ua-parser-js (Chrome, Edge, Firefox, Safari, Unknown)
- ✅ **Browser Selection Dropdown**: Manual browser selection with logos and detected browser indication
- ✅ **Browser-Specific UI**: Different experiences for supported vs unsupported browsers
- ✅ **Extension Detection**: Robust detection using window.bodhiext object and events for supported browsers
- ✅ **Smart Navigation**: Browser-aware navigation and messaging
- ✅ **Chrome Store Integration**: Install button that opens Chrome Web Store for supported browsers
- ✅ **Coming Soon Messaging**: Informative messaging for Firefox and Safari users
- ✅ **Setup Flow Integration**: Proper navigation between API models and extension setup
- ✅ **Comprehensive Testing**: 18 tests across 7 phases with incremental validation
- ✅ **Accessibility**: Clear UI states with proper loading indicators and success messaging
- ✅ **Error Handling**: Graceful handling of detection failures and edge cases

### Technical Implementation
- ✅ **Dual Detection Hooks**: `useBrowserDetection` and `useExtensionDetection` hooks
- ✅ **Browser Utilities**: Centralized browser configuration and detection logic
- ✅ **Component Architecture**: Modular BrowserSelector component for reusability
- ✅ **TypeScript Safety**: Proper typing for all browser and extension objects
- ✅ **Motion Animations**: Consistent with other setup pages using framer-motion
- ✅ **Incremental Testing**: Phase-by-phase testing approach with 18 comprehensive tests
- ✅ **Navigation Updates**: Updated API models page to flow to extension setup

### Browser Support Matrix
| Browser | Extension Available | Detection | UI Experience |
|---------|-------------------|-----------|---------------|
| Chrome | ✅ Yes | ✅ Auto-detect | Extension detection + install link |
| Edge | ✅ Yes | ✅ Auto-detect | Extension detection + install link |
| Firefox | ❌ Coming Soon | ✅ Auto-detect | "Coming soon" message |
| Safari | ❌ Coming Soon | ✅ Auto-detect | "Coming soon" message |
| Unknown | ❌ Not Supported | ❌ Generic | "Not available" message |

This implementation will provide an intelligent, browser-aware user experience for the extension setup phase, with incremental development and comprehensive testing ensuring a robust final product.