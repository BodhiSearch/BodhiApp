import { render, screen } from '@testing-library/react';
import { vi, describe, it, expect, beforeEach } from 'vitest';
import BrowserExtensionSetupPage from './index';
import { useBrowserDetection } from '@/hooks/use-browser-detection';
import { useExtensionDetection } from '@/hooks/use-extension-detection';
import { SetupProvider } from '@/routes/setup/-components';

const navigateMock = vi.fn();
vi.mock('@tanstack/react-router', async () => {
  const actual = await vi.importActual('@tanstack/react-router');
  return {
    ...actual,
    Link: ({ to, children, ...rest }: any) => (
      <a href={to} {...rest}>
        {children}
      </a>
    ),
    useNavigate: () => navigateMock,
    useLocation: () => ({ pathname: '/setup/browser-extension' }),
  };
});

vi.mock('framer-motion', () => ({
  motion: {
    div: ({ children, ...props }: any) => <div {...props}>{children}</div>,
  },
}));

vi.mock('@/components/AppInitializer', () => ({
  default: ({ children }: any) => <div data-testid="app-initializer">{children}</div>,
}));

vi.mock('@/routes/setup/-components/SetupProgress', () => ({
  SetupProgress: ({ currentStep, totalSteps, stepLabels }: any) => (
    <div data-testid="setup-progress">
      Step {currentStep} of {totalSteps}: {stepLabels[currentStep - 1]}
    </div>
  ),
}));

vi.mock('@/routes/setup/-components/BodhiLogo', () => ({
  BodhiLogo: () => <div data-testid="bodhi-logo">BodhiApp Logo</div>,
}));

vi.mock('@/components/setup/BrowserSelector', () => ({
  BrowserSelector: ({ detectedBrowser, selectedBrowser, onBrowserSelect }: any) => (
    <div data-testid="browser-selector">Browser Selector - Detected: {detectedBrowser?.name || 'None'}</div>
  ),
}));

vi.mock('@/hooks/use-browser-detection', () => ({
  useBrowserDetection: vi.fn(),
}));

vi.mock('@/hooks/use-extension-detection', () => ({
  useExtensionDetection: vi.fn(),
}));

vi.mock('lucide-react', () => ({
  Monitor: ({ className }: any) => <div data-testid="monitor-icon" className={className}></div>,
  Check: ({ className }: any) => <div data-testid="check-icon" className={className}></div>,
  Download: ({ className }: any) => <div data-testid="download-icon" className={className}></div>,
  RefreshCw: ({ className }: any) => <div data-testid="refresh-icon" className={className}></div>,
  Puzzle: ({ className }: any) => <div data-testid="puzzle-icon" className={className}></div>,
  ArrowRight: ({ className }: any) => <div data-testid="arrow-right-icon" className={className}></div>,
  Moon: ({ className }: any) => <div data-testid="moon-icon" className={className}></div>,
  Sun: ({ className }: any) => <div data-testid="sun-icon" className={className}></div>,
}));

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
  CardFooter: ({ children }: any) => <div data-testid="card-footer">{children}</div>,
}));

vi.mock('@/components/ui/button', () => ({
  Button: ({ children, onClick, ...props }: any) => (
    <button onClick={onClick} {...props}>
      {children}
    </button>
  ),
}));

const renderWithSetupProvider = (component: React.ReactElement) => {
  return render(<SetupProvider>{component}</SetupProvider>);
};

describe('BrowserExtensionSetupPage', () => {
  beforeEach(() => {
    vi.clearAllMocks();

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
    renderWithSetupProvider(<BrowserExtensionSetupPage />);

    expect(screen.getByTestId('app-initializer')).toBeInTheDocument();
    expect(screen.getByTestId('browser-extension-setup-page')).toBeInTheDocument();
  });

  it('displays correct setup progress', () => {
    renderWithSetupProvider(<BrowserExtensionSetupPage />);

    const setupProgress = screen.getByTestId('setup-progress');
    expect(setupProgress).toBeInTheDocument();
    expect(setupProgress).toHaveTextContent('Step 5 of 6: Extension');
  });

  it('renders welcome section and logo', () => {
    renderWithSetupProvider(<BrowserExtensionSetupPage />);

    expect(screen.getByTestId('bodhi-logo')).toBeInTheDocument();

    expect(screen.getByText('Browser Extension Setup')).toBeInTheDocument();
    expect(
      screen.getByText('Install the Bodhi extension to unlock AI features on any website you visit.')
    ).toBeInTheDocument();

    expect(screen.getByTestId('browser-selector')).toBeInTheDocument();

    expect(
      screen.getByText(/Need help\? The extension enables AI features directly in your browser tabs/)
    ).toBeInTheDocument();
    expect(screen.getByText('You can always install the extension later from the settings page.')).toBeInTheDocument();
  });

  describe('Browser-specific UI behavior', () => {
    it('shows extension detection for supported browsers', () => {
      vi.mocked(useBrowserDetection).mockReturnValue({
        detectedBrowser: {
          name: 'Google Chrome',
          type: 'chrome',
          supported: true,
          extensionUrl: 'https://chrome.google.com/webstore',
          statusMessage: 'Extension available',
        },
      });

      renderWithSetupProvider(<BrowserExtensionSetupPage />);

      expect(screen.getByText('Extension Not Found')).toBeInTheDocument();
      expect(screen.getByTestId('refresh-button')).toBeInTheDocument();
      expect(screen.getByTestId('browser-extension-continue')).toBeInTheDocument();
      expect(screen.getByText('Skip for Now')).toBeInTheDocument();
    });

    it('shows coming soon message for unsupported browsers', () => {
      vi.mocked(useBrowserDetection).mockReturnValue({
        detectedBrowser: {
          name: 'Mozilla Firefox',
          type: 'firefox',
          supported: false,
          extensionUrl: null,
          statusMessage: 'Firefox extension coming soon',
        },
      });

      renderWithSetupProvider(<BrowserExtensionSetupPage />);

      expect(screen.queryByText('Extension Not Found')).not.toBeInTheDocument();
      expect(screen.queryByTestId('refresh-button')).not.toBeInTheDocument();

      expect(screen.getByTestId('browser-extension-continue')).toBeInTheDocument();
      expect(screen.getByText('Skip for Now')).toBeInTheDocument();
    });
  });

  describe('Extension detection integration', () => {
    it('integrates browser and extension detection correctly', () => {
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

      renderWithSetupProvider(<BrowserExtensionSetupPage />);

      expect(screen.getByText('Extension Connected')).toBeInTheDocument();
      expect(screen.getByTestId('check-icon')).toBeInTheDocument();
      expect(screen.getByText(/Bodhi is now active in your browser/)).toBeInTheDocument();
      expect(screen.getByTestId('browser-extension-continue')).toBeInTheDocument();
      expect(screen.getByText('Continue')).toBeInTheDocument();
    });

    it('handles supported browser with extension not installed', () => {
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

      renderWithSetupProvider(<BrowserExtensionSetupPage />);

      expect(screen.getByText('Extension Not Found')).toBeInTheDocument();
      expect(screen.getByText(/Install the extension, then verify the connection below/)).toBeInTheDocument();
      expect(screen.getByTestId('refresh-button')).toBeInTheDocument();
      expect(screen.getByTestId('browser-extension-continue')).toBeInTheDocument();
      expect(screen.getByText('Skip for Now')).toBeInTheDocument();
    });
  });

  describe('Navigation button behavior', () => {
    it('continues setup when continue button is clicked (extension installed)', () => {
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

      renderWithSetupProvider(<BrowserExtensionSetupPage />);

      const continueButton = screen.getByTestId('browser-extension-continue');
      continueButton.click();

      expect(navigateMock).toHaveBeenCalledWith({ to: '/setup/complete/' });
    });

    it('continues setup when skip button is clicked (extension not installed)', () => {
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

      renderWithSetupProvider(<BrowserExtensionSetupPage />);

      const skipButton = screen.getByTestId('browser-extension-continue');
      skipButton.click();

      expect(navigateMock).toHaveBeenCalledWith({ to: '/setup/complete/' });
    });

    it('continues setup when skip button is clicked for unsupported browsers', () => {
      vi.mocked(useBrowserDetection).mockReturnValue({
        detectedBrowser: {
          name: 'Mozilla Firefox',
          type: 'firefox',
          supported: false,
          extensionUrl: null,
          statusMessage: 'Firefox extension coming soon',
        },
      });

      renderWithSetupProvider(<BrowserExtensionSetupPage />);

      const skipButton = screen.getByTestId('browser-extension-continue');
      skipButton.click();

      expect(navigateMock).toHaveBeenCalledWith({ to: '/setup/complete/' });
    });
  });
});
