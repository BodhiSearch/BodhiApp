import { render, screen } from '@testing-library/react';
import { vi, describe, it, expect, beforeEach } from 'vitest';
import { BrowserSelector } from './BrowserSelector';
import { type BrowserInfo } from '@/lib/browser-utils';

// Mock Shadcn UI components with simpler implementation
vi.mock('@/components/ui/select', () => ({
  Select: ({ children, value, onValueChange }: any) => (
    <div data-testid="browser-select" data-value={value}>
      {children}
    </div>
  ),
  SelectContent: ({ children }: any) => <div>{children}</div>,
  SelectItem: ({ children, value }: any) => <div data-testid={`select-item-${value}`}>{children}</div>,
  SelectTrigger: ({ children }: any) => <div data-testid="select-trigger">{children}</div>,
  SelectValue: ({ children }: any) => <div>{children}</div>,
}));

vi.mock('@/components/ui/badge', () => ({
  Badge: ({ children }: any) => <span data-testid="badge">{children}</span>,
}));

// Mock react-icons/fa
vi.mock('react-icons/fa', () => ({
  FaChrome: ({ className }: any) => <div data-testid="chrome-icon" className={className}></div>,
  FaFirefox: ({ className }: any) => <div data-testid="firefox-icon" className={className}></div>,
  FaSafari: ({ className }: any) => <div data-testid="safari-icon" className={className}></div>,
  FaEdge: ({ className }: any) => <div data-testid="edge-icon" className={className}></div>,
}));

describe('BrowserSelector component', () => {
  const mockOnBrowserSelect = vi.fn();

  const chromeInfo: BrowserInfo = {
    name: 'Google Chrome',
    type: 'chrome',
    supported: true,
    extensionUrl: 'https://chrome.google.com/webstore/detail/bodhi-browser/test-id',
    statusMessage: 'Extension available in Chrome Web Store',
  };

  const firefoxInfo: BrowserInfo = {
    name: 'Mozilla Firefox',
    type: 'firefox',
    supported: false,
    extensionUrl: null,
    statusMessage: 'Firefox extension coming soon',
  };

  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('displays detected browser with indicator', () => {
    render(
      <BrowserSelector detectedBrowser={chromeInfo} selectedBrowser={null} onBrowserSelect={mockOnBrowserSelect} />
    );

    expect(screen.getAllByText('Google Chrome')).toHaveLength(2); // Once in trigger, once in dropdown
    expect(screen.getAllByTestId('badge')).toHaveLength(2); // Once in trigger, once in dropdown
    expect(screen.getAllByText('detected')).toHaveLength(2);

    // Check for chrome icons (trigger + dropdown Chrome + dropdown Edge uses chrome fallback)
    const chromeIcons = screen.getAllByTestId('chrome-icon');
    expect(chromeIcons.length).toBeGreaterThan(0);

    expect(screen.getByText('Extension available in Chrome Web Store')).toBeInTheDocument();
    expect(screen.getByText('Install Bodhi Extension →')).toBeInTheDocument();
  });

  it('handles manual browser selection display', () => {
    render(
      <BrowserSelector
        detectedBrowser={chromeInfo}
        selectedBrowser={firefoxInfo}
        onBrowserSelect={mockOnBrowserSelect}
      />
    );

    const selectTrigger = screen.getByTestId('select-trigger');
    expect(selectTrigger).toContainHTML('Mozilla Firefox');
    expect(selectTrigger.querySelector('[data-testid="badge"]')).toBeNull();

    // Check for Firefox icon in trigger
    const firefoxIcon = selectTrigger.querySelector('[data-testid="firefox-icon"]');
    expect(firefoxIcon).toBeInTheDocument();
  });

  it('displays browser-specific information correctly', () => {
    const { rerender } = render(
      <BrowserSelector detectedBrowser={null} selectedBrowser={chromeInfo} onBrowserSelect={mockOnBrowserSelect} />
    );

    expect(screen.getByText('Extension available in Chrome Web Store')).toBeInTheDocument();
    expect(screen.getByText('Install Bodhi Extension →')).toBeInTheDocument();

    const extensionLink = screen.getByText('Install Bodhi Extension →');
    expect(extensionLink.closest('a')).toHaveAttribute('href', chromeInfo.extensionUrl);
    expect(extensionLink.closest('a')).toHaveAttribute('target', '_blank');
    expect(extensionLink.closest('a')).toHaveAttribute('rel', 'noopener noreferrer');

    rerender(
      <BrowserSelector detectedBrowser={null} selectedBrowser={firefoxInfo} onBrowserSelect={mockOnBrowserSelect} />
    );

    expect(screen.getByText('Firefox extension coming soon')).toBeInTheDocument();
    expect(screen.queryByText('Install Bodhi Extension →')).not.toBeInTheDocument();
  });
});
