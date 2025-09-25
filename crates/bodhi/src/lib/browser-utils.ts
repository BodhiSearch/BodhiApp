/**
 * Browser detection utilities using UAParser.js
 * Based on implementation from bodhi-js project
 */

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
    statusMessage: 'Extension available in Chrome Web Store',
  },
  edge: {
    type: 'edge',
    supported: true,
    extensionUrl: 'https://chrome.google.com/webstore/detail/bodhi-browser/[EXTENSION_ID]',
    statusMessage: 'Extension available in Chrome Web Store (Edge uses Chrome extensions)',
  },
  firefox: {
    type: 'firefox',
    supported: false,
    extensionUrl: null,
    statusMessage: 'Firefox extension coming soon',
  },
  safari: {
    type: 'safari',
    supported: false,
    extensionUrl: null,
    statusMessage: 'Safari extension coming soon',
  },
  unknown: {
    type: 'unknown',
    supported: false,
    extensionUrl: null,
    statusMessage: 'Extension not available for this browser',
  },
};

/**
 * Detects the current browser using UAParser.js
 * Maps UAParser results to our supported browsers
 */
export function detectBrowser(): BrowserInfo {
  const parser = new UAParser();
  const browser = parser.getBrowser();
  const browserName = browser.name?.toLowerCase() || '';
  const actualBrowserName = browser.name || 'Unknown Browser';

  let type: BrowserType = 'unknown';
  let name = actualBrowserName;

  // Map UAParser results to our supported browsers (following bodhi-js pattern)
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
  } else {
    // Clean up browser name for unknown browsers (remove common suffixes)
    const cleanName =
      actualBrowserName
        .replace(/\s+Browser$/i, '')
        .replace(/\s+browser$/i, '')
        .trim() || 'Unknown Browser';
    name = cleanName;
  }

  return {
    name,
    ...BROWSER_CONFIG[type],
  };
}
