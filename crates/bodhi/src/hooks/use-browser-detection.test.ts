/**
 * Tests for browser detection hook
 */

import { renderHook } from '@testing-library/react';
import { vi, describe, it, expect, beforeEach } from 'vitest';

import * as browserUtils from '@/lib/browser-utils';

import { useBrowserDetection } from './use-browser-detection';

// Mock UAParser
vi.mock('ua-parser-js', () => {
  return {
    UAParser: vi.fn(),
  };
});

// Mock browser-utils module
vi.mock('@/lib/browser-utils', () => ({
  detectBrowser: vi.fn(),
  BROWSER_CONFIG: {
    chrome: {
      type: 'chrome',
      supported: true,
      extensionUrl: 'https://chrome.google.com/webstore/detail/bodhi-browser/test-id',
      statusMessage: 'Extension available in Chrome Web Store',
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
    edge: {
      type: 'edge',
      supported: true,
      extensionUrl: 'https://chrome.google.com/webstore/detail/bodhi-browser/test-id',
      statusMessage: 'Extension available in Chrome Web Store (Edge uses Chrome extensions)',
    },
    unknown: {
      type: 'unknown',
      supported: false,
      extensionUrl: null,
      statusMessage: 'Extension not available for this browser',
    },
  },
}));

describe('useBrowserDetection hook', () => {
  const mockDetectBrowser = vi.mocked(browserUtils.detectBrowser);

  beforeEach(() => {
    vi.clearAllMocks();
  });

  const supportedBrowsers = [
    [
      'Chrome',
      {
        name: 'Google Chrome',
        type: 'chrome' as const,
        supported: true,
        extensionUrl: 'https://chrome.google.com/webstore/detail/bodhi-browser/test-id',
        statusMessage: 'Extension available in Chrome Web Store',
      },
    ],
    [
      'Edge',
      {
        name: 'Microsoft Edge',
        type: 'edge' as const,
        supported: true,
        extensionUrl: 'https://chrome.google.com/webstore/detail/bodhi-browser/test-id',
        statusMessage: 'Extension available in Chrome Web Store (Edge uses Chrome extensions)',
      },
    ],
  ] as const;

  const unsupportedBrowsers = [
    [
      'Firefox',
      {
        name: 'Mozilla Firefox',
        type: 'firefox' as const,
        supported: false,
        extensionUrl: null,
        statusMessage: 'Firefox extension coming soon',
      },
    ],
    [
      'Safari',
      {
        name: 'Safari',
        type: 'safari' as const,
        supported: false,
        extensionUrl: null,
        statusMessage: 'Safari extension coming soon',
      },
    ],
    [
      'Unknown',
      {
        name: 'Unknown Browser',
        type: 'unknown' as const,
        supported: false,
        extensionUrl: null,
        statusMessage: 'Extension not available for this browser',
      },
    ],
  ] as const;

  it.each(supportedBrowsers)('detects %s as supported with extension URL', (_browserName, browserInfo) => {
    mockDetectBrowser.mockReturnValue(browserInfo);

    const { result } = renderHook(() => useBrowserDetection());

    expect(result.current.detectedBrowser).toEqual(browserInfo);
    expect(result.current.detectedBrowser?.supported).toBe(true);
    expect(result.current.detectedBrowser?.extensionUrl).toContain('chrome.google.com/webstore');
    expect(result.current.detectedBrowser?.extensionUrl).not.toBeNull();
  });

  it.each(unsupportedBrowsers)('detects %s as unsupported with appropriate message', (browserName, browserInfo) => {
    mockDetectBrowser.mockReturnValue(browserInfo);

    const { result } = renderHook(() => useBrowserDetection());

    expect(result.current.detectedBrowser).toEqual(browserInfo);
    expect(result.current.detectedBrowser?.supported).toBe(false);
    expect(result.current.detectedBrowser?.extensionUrl).toBeNull();
    expect(result.current.detectedBrowser?.statusMessage).toBeTruthy();

    if (browserName === 'Firefox') {
      expect(result.current.detectedBrowser?.statusMessage).toBe('Firefox extension coming soon');
    } else if (browserName === 'Safari') {
      expect(result.current.detectedBrowser?.statusMessage).toBe('Safari extension coming soon');
    } else if (browserName === 'Unknown') {
      expect(result.current.detectedBrowser?.statusMessage).toBe('Extension not available for this browser');
    }
  });
});
