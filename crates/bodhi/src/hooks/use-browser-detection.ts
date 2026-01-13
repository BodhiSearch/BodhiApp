/**
 * Browser detection hook for identifying user's browser
 */

import { useState, useEffect } from 'react';

import { detectBrowser, type BrowserInfo } from '@/lib/browser-utils';

export function useBrowserDetection() {
  const [detectedBrowser, setDetectedBrowser] = useState<BrowserInfo | null>(null);

  useEffect(() => {
    const browser = detectBrowser();
    setDetectedBrowser(browser);
  }, []);

  return { detectedBrowser };
}
