import { render, screen } from '@testing-library/react';
import { vi, describe, it, expect, beforeEach } from 'vitest';
import BrowserExtensionSetupPage from './page';
import { useBrowserDetection } from '@/hooks/use-browser-detection';
import { useExtensionDetection } from '@/hooks/use-extension-detection';

// Mock Next.js router
const mockPush = vi.fn();
vi.mock('next/navigation', () => ({
  useRouter: () => ({
    push: mockPush,
  }),
}));

// Mock framer-motion
vi.mock('framer-motion', () => ({
  motion: {
    div: ({ children, ...props }: any) => <div {...props}>{children}</div>,
  },
}));

// Mock AppInitializer
vi.mock('@/components/AppInitializer', () => ({
  default: ({ children }: any) => <div data-testid="app-initializer">{children}</div>,
}));

// Mock setup components
vi.mock('@/app/ui/setup/SetupProgress', () => ({
  SetupProgress: ({ currentStep, totalSteps, stepLabels }: any) => (
    <div data-testid="setup-progress">
      Step {currentStep} of {totalSteps}: {stepLabels[currentStep - 1]}
    </div>
  ),
}));

vi.mock('@/app/ui/setup/BodhiLogo', () => ({
  BodhiLogo: () => <div data-testid="bodhi-logo">BodhiApp Logo</div>,
}));

// Mock BrowserSelector
vi.mock('@/components/setup/BrowserSelector', () => ({
  BrowserSelector: ({ detectedBrowser, selectedBrowser, onBrowserSelect }: any) => (
    <div data-testid="browser-selector">Browser Selector - Detected: {detectedBrowser?.name || 'None'}</div>
  ),
}));

// Mock browser detection hook
vi.mock('@/hooks/use-browser-detection', () => ({
  useBrowserDetection: vi.fn(),
}));

// Mock extension detection hook
vi.mock('@/hooks/use-extension-detection', () => ({
  useExtensionDetection: vi.fn(),
}));

// Mock Lucide React icons
vi.mock('lucide-react', () => ({
  Monitor: ({ className }: any) => <div data-testid="monitor-icon" className={className}></div>,
  Check: ({ className }: any) => <div data-testid="check-icon" className={className}></div>,
  Download: ({ className }: any) => <div data-testid="download-icon" className={className}></div>,
  RefreshCw: ({ className }: any) => <div data-testid="refresh-icon" className={className}></div>,
}));

// Mock Shadcn UI components
vi.mock('@/components/ui/card', () => ({
  Card: ({ children, className }: any) => (
    <div data-testid="card" className={className}>
      {children}
    </div>
  ),
  CardContent: ({ children }: any) => <div data-testid="card-content">{children}</div>,
  CardDescription: ({ children }: any) => <div data-testid="card-description">{children}</div>,
  CardHeader: ({ children }: any) => <div data-testid="card-header">{children}</div>,
  CardTitle: ({ children }: any) => <div data-testid="card-title">{children}</div>,
}));

vi.mock('@/components/ui/button', () => ({
  Button: ({ children, onClick, ...props }: any) => (
    <button onClick={onClick} {...props}>
      {children}
    </button>
  ),
}));

describe('BrowserExtensionSetupPage', () => {
  beforeEach(() => {
    vi.clearAllMocks();

    // Set default mock return values
    vi.mocked(useBrowserDetection).mockReturnValue({
      detectedBrowser: {
        name: 'Google Chrome',
        type: 'chrome',
        supported: true,
        extensionUrl: 'https://chrome.google.com/webstore',
        statusMessage: 'Extension available',
      },
    });

    vi.mocked(useExtensionDetection).mockReturnValue({
      status: 'not-installed',
      extensionId: null,
      refresh: vi.fn(),
      redetect: vi.fn(),
    });
  });

  it('renders page with correct authentication requirements', () => {
    render(<BrowserExtensionSetupPage />);

    expect(screen.getByTestId('app-initializer')).toBeInTheDocument();
    expect(screen.getByTestId('browser-extension-setup-page')).toBeInTheDocument();
  });

  it('displays correct setup progress', () => {
    render(<BrowserExtensionSetupPage />);

    const setupProgress = screen.getByTestId('setup-progress');
    expect(setupProgress).toBeInTheDocument();
    expect(setupProgress).toHaveTextContent('Step 5 of 6: Extension');
  });

  it('renders welcome section and logo', () => {
    render(<BrowserExtensionSetupPage />);

    // Check for logo
    expect(screen.getByTestId('bodhi-logo')).toBeInTheDocument();

    // Check for welcome section
    expect(screen.getByText('Browser Extension Setup')).toBeInTheDocument();
    expect(screen.getByTestId('monitor-icon')).toBeInTheDocument();
    expect(
      screen.getByText('Choose your browser and install the Bodhi extension to unlock AI features on any website.')
    ).toBeInTheDocument();

    // Check for browser selector
    expect(screen.getByTestId('browser-selector')).toBeInTheDocument();

    // Check for help section
    expect(screen.getByText('Need help?')).toBeInTheDocument();
    expect(screen.getByText('You can always install the extension later from the settings page.')).toBeInTheDocument();
  });

  describe('Browser-specific UI behavior', () => {
    it('shows extension detection for supported browsers', () => {
      // Mock Chrome browser (supported)
      vi.mocked(useBrowserDetection).mockReturnValue({
        detectedBrowser: {
          name: 'Google Chrome',
          type: 'chrome',
          supported: true,
          extensionUrl: 'https://chrome.google.com/webstore',
          statusMessage: 'Extension available',
        },
      });

      render(<BrowserExtensionSetupPage />);

      // Should show extension detection UI
      expect(screen.getByText('Extension Not Found')).toBeInTheDocument();
      expect(screen.getByTestId('download-icon')).toBeInTheDocument();
      expect(screen.getByTestId('refresh-button')).toBeInTheDocument();
      expect(screen.getByTestId('skip-button')).toBeInTheDocument();

      // Should not show continue button for unsupported browsers
      expect(screen.queryByTestId('continue-button')).not.toBeInTheDocument();
    });

    it('shows coming soon message for unsupported browsers', () => {
      // Mock Firefox browser (unsupported)
      vi.mocked(useBrowserDetection).mockReturnValue({
        detectedBrowser: {
          name: 'Mozilla Firefox',
          type: 'firefox',
          supported: false,
          extensionUrl: null,
          statusMessage: 'Firefox extension coming soon',
        },
      });

      render(<BrowserExtensionSetupPage />);

      // Should not show extension detection UI
      expect(screen.queryByText('Extension Not Found')).not.toBeInTheDocument();
      expect(screen.queryByTestId('download-icon')).not.toBeInTheDocument();
      expect(screen.queryByTestId('refresh-button')).not.toBeInTheDocument();
      expect(screen.queryByTestId('skip-button')).not.toBeInTheDocument();

      // Should show continue button for unsupported browsers
      expect(screen.getByTestId('continue-button')).toBeInTheDocument();
      expect(screen.getByText('Continue Setup')).toBeInTheDocument();
    });
  });

  describe('Extension detection integration', () => {
    it('integrates browser and extension detection correctly', () => {
      // Test supported browser + extension installed
      vi.mocked(useBrowserDetection).mockReturnValue({
        detectedBrowser: {
          name: 'Google Chrome',
          type: 'chrome',
          supported: true,
          extensionUrl: 'https://chrome.google.com/webstore',
          statusMessage: 'Extension available',
        },
      });

      vi.mocked(useExtensionDetection).mockReturnValue({
        status: 'installed',
        extensionId: 'test-extension-id-123',
        refresh: vi.fn(),
        redetect: vi.fn(),
      });

      render(<BrowserExtensionSetupPage />);

      // Should show extension found UI
      expect(screen.getByText('Extension Found!')).toBeInTheDocument();
      expect(screen.getByTestId('check-icon')).toBeInTheDocument();
      expect(screen.getByText(/Perfect! The Bodhi Browser extension is installed and ready/)).toBeInTheDocument();
      expect(screen.getByText(/Extension ID:/)).toBeInTheDocument();
      expect(screen.getByText('test-extension-id-123')).toBeInTheDocument();
      expect(screen.getByTestId('next-button')).toBeInTheDocument();
    });

    it('handles supported browser with extension not installed', () => {
      // Test supported browser + extension not installed
      vi.mocked(useBrowserDetection).mockReturnValue({
        detectedBrowser: {
          name: 'Microsoft Edge',
          type: 'edge',
          supported: true,
          extensionUrl: 'https://chrome.google.com/webstore',
          statusMessage: 'Extension available in Chrome Web Store (Edge uses Chrome extensions)',
        },
      });

      vi.mocked(useExtensionDetection).mockReturnValue({
        status: 'not-installed',
        extensionId: null,
        refresh: vi.fn(),
        redetect: vi.fn(),
      });

      render(<BrowserExtensionSetupPage />);

      // Should show extension not found UI
      expect(screen.getByText('Extension Not Found')).toBeInTheDocument();
      expect(screen.getByTestId('download-icon')).toBeInTheDocument();
      expect(screen.getByText('Install the extension to continue, then refresh this page.')).toBeInTheDocument();
      expect(screen.getByTestId('refresh-button')).toBeInTheDocument();
      expect(screen.getByTestId('skip-button')).toBeInTheDocument();
    });
  });

  describe('Navigation button behavior', () => {
    it('continues setup when next button is clicked', () => {
      // Mock Chrome browser with extension installed (shows next button)
      vi.mocked(useBrowserDetection).mockReturnValue({
        detectedBrowser: {
          name: 'Google Chrome',
          type: 'chrome',
          supported: true,
          extensionUrl: 'https://chrome.google.com/webstore',
          statusMessage: 'Extension available',
        },
      });

      vi.mocked(useExtensionDetection).mockReturnValue({
        status: 'installed',
        extensionId: 'test-extension-id-123',
        refresh: vi.fn(),
        redetect: vi.fn(),
      });

      render(<BrowserExtensionSetupPage />);

      const nextButton = screen.getByTestId('next-button');
      nextButton.click();

      expect(mockPush).toHaveBeenCalledWith('/ui/setup/complete');
    });

    it('continues setup when skip button is clicked', () => {
      // Mock Chrome browser with extension not installed (shows skip button)
      vi.mocked(useBrowserDetection).mockReturnValue({
        detectedBrowser: {
          name: 'Google Chrome',
          type: 'chrome',
          supported: true,
          extensionUrl: 'https://chrome.google.com/webstore',
          statusMessage: 'Extension available',
        },
      });

      vi.mocked(useExtensionDetection).mockReturnValue({
        status: 'not-installed',
        extensionId: null,
        refresh: vi.fn(),
        redetect: vi.fn(),
      });

      render(<BrowserExtensionSetupPage />);

      const skipButton = screen.getByTestId('skip-button');
      skipButton.click();

      expect(mockPush).toHaveBeenCalledWith('/ui/setup/complete');
    });

    it('continues setup when continue button is clicked for unsupported browsers', () => {
      // Mock Firefox browser (unsupported, shows continue button)
      vi.mocked(useBrowserDetection).mockReturnValue({
        detectedBrowser: {
          name: 'Mozilla Firefox',
          type: 'firefox',
          supported: false,
          extensionUrl: null,
          statusMessage: 'Firefox extension coming soon',
        },
      });

      render(<BrowserExtensionSetupPage />);

      const continueButton = screen.getByTestId('continue-button');
      continueButton.click();

      expect(mockPush).toHaveBeenCalledWith('/ui/setup/complete');
    });
  });
});
