import { render, screen } from '@testing-library/react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { act } from 'react';
import SetupCompletePage from './page';
import { createWrapper } from '@/tests/wrapper';
import { SetupProvider } from '@/app/ui/setup/components';

// Mock navigation
const mockPush = vi.fn();
vi.mock('next/navigation', () => ({
  useRouter: () => ({ push: mockPush }),
  usePathname: () => '/ui/setup/complete',
}));

// Mock AppInitializer to prevent status checks
vi.mock('@/components/AppInitializer', () => ({
  default: ({ children }: { children: React.ReactNode }) => <div>{children}</div>,
}));

// Mock Card components
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

// Helper to render with SetupProvider
const renderWithSetupProvider = (component: React.ReactElement) => {
  return render(<SetupProvider>{component}</SetupProvider>, { wrapper: createWrapper() });
};

describe('SetupCompletePage', () => {
  beforeEach(() => {
    mockPush.mockClear();
  });

  it('should render completion message', async () => {
    await act(async () => {
      renderWithSetupProvider(<SetupCompletePage />);
    });

    expect(screen.getByText(/Setup Complete!/i)).toBeInTheDocument();
    expect(screen.getByText(/Your Bodhi App is ready to use/i)).toBeInTheDocument();
  });

  it('should render community section with social links', async () => {
    await act(async () => {
      renderWithSetupProvider(<SetupCompletePage />);
    });

    expect(screen.getByText('Join Our Community')).toBeInTheDocument();
    expect(screen.getByText('Star on GitHub')).toBeInTheDocument();
    expect(screen.getByText('Join Discord')).toBeInTheDocument();
    expect(screen.getByText('Follow on X')).toBeInTheDocument();
    expect(screen.getByText('Watch Tutorials')).toBeInTheDocument();
  });

  it('should render resources section', async () => {
    await act(async () => {
      renderWithSetupProvider(<SetupCompletePage />);
    });

    expect(screen.getByText('Quick Resources')).toBeInTheDocument();
    expect(screen.getByText('Getting Started Guide')).toBeInTheDocument();
  });

  it('should render start button', async () => {
    await act(async () => {
      renderWithSetupProvider(<SetupCompletePage />);
    });

    const startButton = screen.getByText(/Start Using Bodhi App/i);
    expect(startButton).toBeInTheDocument();
  });

  it('should navigate to chat when start button is clicked', async () => {
    await act(async () => {
      renderWithSetupProvider(<SetupCompletePage />);
    });

    const startButton = screen.getByText(/Start Using Bodhi App/i);
    await act(async () => {
      startButton.click();
    });

    expect(mockPush).toHaveBeenCalledWith('/ui/chat');
  });

  it('should have external links with correct attributes', async () => {
    await act(async () => {
      renderWithSetupProvider(<SetupCompletePage />);
    });

    const githubLink = screen.getByText('Star on GitHub').closest('a');
    expect(githubLink).toHaveAttribute('href', 'https://github.com/BodhiSearch/BodhiApp');
    expect(githubLink).toHaveAttribute('target', '_blank');
    expect(githubLink).toHaveAttribute('rel', 'noopener noreferrer');

    const discordLink = screen.getByText('Join Discord').closest('a');
    expect(discordLink).toHaveAttribute('href', 'https://discord.gg/3vur28nz82');
    expect(discordLink).toHaveAttribute('target', '_blank');
    expect(discordLink).toHaveAttribute('rel', 'noopener noreferrer');
  });
});
