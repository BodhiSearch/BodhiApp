import Setup from '@/routes/setup/index';
import { SetupProvider } from '@/routes/setup/-components';
import { server } from '@/test-utils/msw-v2/setup';
import { mockAppInfoSetup } from '@/test-utils/msw-v2/handlers/info';
import { mockUserLoggedOut } from '@/test-utils/msw-v2/handlers/user';
import {
  mockSetupSuccess,
  mockSetupResourceAdmin,
  mockSetupError,
  mockSetupSuccessWithDelay,
} from '@/test-utils/msw-v2/handlers/setup';
import { showErrorParams } from '@/lib/utils.test';
import { createWrapper } from '@/tests/wrapper';
import { act, render, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterAll, afterEach, beforeAll, beforeEach, describe, expect, it, vi } from 'vitest';

const mockToast = vi.fn();
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
    useLocation: () => ({ pathname: '/setup' }),
  };
});

vi.mock('@/hooks/use-toast', () => ({
  useToast: () => ({ toast: mockToast }),
}));

const renderWithSetupProvider = (component: React.ReactElement) => {
  return render(<SetupProvider>{component}</SetupProvider>, { wrapper: createWrapper() });
};

beforeAll(() => server.listen());
afterAll(() => server.close());
afterEach(() => {
  server.resetHandlers();
  vi.clearAllMocks();
  navigateMock.mockClear();
  mockToast.mockClear();
});

describe('Setup Page', () => {
  beforeEach(() => {
    Object.defineProperty(window, 'localStorage', {
      value: {
        getItem: vi.fn(() => null),
        setItem: vi.fn(),
        removeItem: vi.fn(),
        clear: vi.fn(),
      },
      writable: true,
    });

    server.use(...mockAppInfoSetup(), ...mockUserLoggedOut(), ...mockSetupSuccess());
  });

  it('should render setup form and handle successful submission with redirect to download models', async () => {
    const user = userEvent.setup();

    await act(async () => {
      renderWithSetupProvider(<Setup />);
    });

    expect(screen.getByTestId('setup-form')).toBeInTheDocument();
    expect(screen.getByLabelText(/server name/i)).toBeInTheDocument();
    expect(screen.getByLabelText(/description/i)).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /setup bodhi server/i })).toBeInTheDocument();

    await user.type(screen.getByLabelText(/server name/i), 'My Test Server Instance');
    await user.type(screen.getByLabelText(/description/i), 'Test description for my server');

    await user.click(screen.getByRole('button', { name: /setup bodhi server/i }));

    await waitFor(() => {
      expect(navigateMock).toHaveBeenCalledWith({ to: '/setup/download-models/' });
    });
  });

  it('should redirect to resource admin when setup returns resource-admin status', async () => {
    server.use(...mockSetupResourceAdmin());

    const user = userEvent.setup();

    await act(async () => {
      renderWithSetupProvider(<Setup />);
    });

    await user.type(screen.getByLabelText(/server name/i), 'My Test Server Instance');

    await user.click(screen.getByRole('button', { name: /setup bodhi server/i }));

    await waitFor(() => {
      expect(navigateMock).toHaveBeenCalledWith({ to: '/setup/resource-admin/' });
    });
  });

  it('should show error toast when setup fails', async () => {
    server.use(...mockSetupError({ message: 'Setup failed', type: 'invalid_request_error' }));

    const user = userEvent.setup();

    await act(async () => {
      renderWithSetupProvider(<Setup />);
    });

    await user.type(screen.getByLabelText(/server name/i), 'My Test Server Instance');

    await user.click(screen.getByRole('button', { name: /setup bodhi server/i }));

    await waitFor(() => {
      expect(mockToast).toHaveBeenCalledWith(showErrorParams('Error', 'Setup failed'));
    });
  });

  it('should show validation error for server name shorter than 10 characters', async () => {
    const user = userEvent.setup();

    await act(async () => {
      renderWithSetupProvider(<Setup />);
    });

    await user.type(screen.getByLabelText(/server name/i), 'Short');

    await user.click(screen.getByRole('button', { name: /setup bodhi server/i }));

    await waitFor(() => {
      expect(screen.getByText('Server name must be at least 10 characters long')).toBeInTheDocument();
    });

    expect(navigateMock).not.toHaveBeenCalled();
  });

  it('should show validation error for server name longer than 100 characters', async () => {
    const user = userEvent.setup();

    await act(async () => {
      renderWithSetupProvider(<Setup />);
    });

    const longName = 'a'.repeat(101);
    await user.type(screen.getByLabelText(/server name/i), longName);

    await user.click(screen.getByRole('button', { name: /setup bodhi server/i }));

    await waitFor(() => {
      expect(screen.getByText('Server name must be less than 100 characters')).toBeInTheDocument();
    });

    expect(navigateMock).not.toHaveBeenCalled();
  });

  it('should show validation error for description longer than 500 characters', async () => {
    const user = userEvent.setup();

    await act(async () => {
      renderWithSetupProvider(<Setup />);
    });

    await user.type(screen.getByLabelText(/server name/i), 'My Test Server Instance');

    // Paste instead of typing 501 chars for performance.
    const longDescription = 'a'.repeat(501);
    const descriptionField = screen.getByLabelText(/description/i);
    await user.click(descriptionField);
    await user.keyboard(`{Control>}a{/Control}`);
    await user.paste(longDescription);

    await user.click(screen.getByRole('button', { name: /setup bodhi server/i }));

    await waitFor(() => {
      expect(screen.getByText('Description must be less than 500 characters')).toBeInTheDocument();
    });

    expect(navigateMock).not.toHaveBeenCalled();
  });

  it('should render page content with updated benefits', async () => {
    await act(async () => {
      renderWithSetupProvider(<Setup />);
    });

    expect(screen.getByText('Complete Privacy')).toBeInTheDocument();
    expect(screen.getByText('Cost Freedom')).toBeInTheDocument();
    expect(screen.getByText('Browser AI Revolution')).toBeInTheDocument();
    expect(screen.getByText('Multi-User Ready')).toBeInTheDocument();
    expect(screen.getByText('Hybrid Flexibility')).toBeInTheDocument();
    expect(screen.getByText('Open Ecosystem')).toBeInTheDocument();

    expect(screen.getByText('Welcome to Bodhi App')).toBeInTheDocument();
    expect(screen.getByText(/Your Personal AI Hub/i)).toBeInTheDocument();

    expect(screen.getByText('Step 1 of 6')).toBeInTheDocument();
  });

  it('should display NEW badges on new features', async () => {
    await act(async () => {
      renderWithSetupProvider(<Setup />);
    });

    const browserAICard = screen.getByTestId('benefit-card-browser-ai-revolution');
    expect(browserAICard).toBeInTheDocument();
    expect(within(browserAICard).getByText('NEW')).toBeInTheDocument();

    const multiUserCard = screen.getByTestId('benefit-card-multi-user-ready');
    expect(multiUserCard).toBeInTheDocument();
    expect(within(multiUserCard).getByText('NEW')).toBeInTheDocument();
  });

  it('should disable form fields and button when loading', async () => {
    // Delayed response keeps the form in its loading state long enough to assert on.
    server.use(...mockSetupSuccessWithDelay(1000));

    const user = userEvent.setup();
    await act(async () => {
      renderWithSetupProvider(<Setup />);
    });
    await user.type(screen.getByLabelText(/server name/i), 'My Test Server Instance');
    await user.click(screen.getByRole('button', { name: /setup bodhi server/i }));
    await waitFor(() => {
      expect(screen.getByLabelText(/server name/i)).toBeDisabled();
      expect(screen.getByLabelText(/description/i)).toBeDisabled();
      expect(screen.getByRole('button', { name: /setting up/i })).toBeDisabled();
    });
  });

  it('should submit form with only required fields', async () => {
    const user = userEvent.setup();
    await act(async () => {
      renderWithSetupProvider(<Setup />);
    });
    await user.type(screen.getByLabelText(/server name/i), 'My Test Server Instance');
    await user.click(screen.getByRole('button', { name: /setup bodhi server/i }));
    await waitFor(() => {
      expect(navigateMock).toHaveBeenCalledWith({ to: '/setup/download-models/' });
    });
  });
});
