import ResourceAdminPage from '@/app/ui/setup/resource-admin/page';
import { ROUTE_DEFAULT, ROUTE_SETUP_DOWNLOAD_MODELS } from '@/lib/constants';
import {
  mockAuthInitiate,
  mockAuthInitiateError,
  mockAuthInitiateUnauthenticated,
  mockAuthInitiateAlreadyAuthenticated,
} from '@/test-utils/msw-v2/handlers/auth';
import { mockAppInfoReady, mockAppInfoResourceAdmin, mockAppInfoSetup } from '@/test-utils/msw-v2/handlers/info';
import { server, setupMswV2 } from '@/test-utils/msw-v2/setup';
import { createWrapper, mockWindowLocation } from '@/tests/wrapper';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { SetupProvider } from '@/app/ui/setup/components';

const pushMock = vi.fn();
vi.mock('next/navigation', () => ({
  useRouter: () => ({
    push: pushMock,
  }),
  useSearchParams: () => ({
    get: () => null,
  }),
  redirect: vi.fn(),
  usePathname: () => '/ui/setup/resource-admin',
}));

vi.mock('next/image', () => ({
  default: () => <img alt="mocked image" />,
}));

setupMswV2();

const renderWithSetupProvider = (component: React.ReactElement) => {
  return render(<SetupProvider>{component}</SetupProvider>, { wrapper: createWrapper() });
};

beforeEach(() => {
  mockWindowLocation('http://localhost:3000/ui/setup/resource-admin');
  server.resetHandlers();
  pushMock.mockClear();
  vi.clearAllMocks();
});

describe('ResourceAdminPage', () => {
  it('renders the resource admin page when status is resource-admin', async () => {
    server.use(...mockAppInfoResourceAdmin());

    renderWithSetupProvider(<ResourceAdminPage />);

    await waitFor(() => {
      expect(screen.getByText('Admin Setup')).toBeInTheDocument();
      expect(screen.getByText('Continue with Login →')).toBeInTheDocument();
    });
  });

  it('renders admin capabilities section with proper styling', async () => {
    server.use(...mockAppInfoResourceAdmin());

    renderWithSetupProvider(<ResourceAdminPage />);

    await waitFor(() => {
      expect(screen.getByText('As an Admin, you can:')).toBeInTheDocument();
      expect(screen.getByText('Manage user access and permissions')).toBeInTheDocument();
      expect(screen.getByText('Unrestricted access to system-wide settings')).toBeInTheDocument();
    });
  });

  it('redirects to /ui/setup when status is setup', async () => {
    server.use(...mockAppInfoSetup());

    renderWithSetupProvider(<ResourceAdminPage />);

    await waitFor(() => {
      expect(pushMock).toHaveBeenCalledWith('/ui/setup');
    });
  });

  it(`redirects to ${ROUTE_DEFAULT} when status is ready`, async () => {
    server.use(...mockAppInfoReady());

    renderWithSetupProvider(<ResourceAdminPage />);

    await waitFor(() => {
      expect(pushMock).toHaveBeenCalledWith(ROUTE_DEFAULT);
    });
  });

  it('sets sessionStorage return URL before OAuth initiation', async () => {
    const setItemSpy = vi.spyOn(Storage.prototype, 'setItem');
    server.use(
      ...mockAppInfoResourceAdmin(),
      ...mockAuthInitiate({ location: 'https://oauth.example.com/auth?client_id=test' })
    );

    renderWithSetupProvider(<ResourceAdminPage />);

    const loginButton = await screen.findByRole('button', { name: 'Continue with Login →' });
    await userEvent.click(loginButton);

    expect(setItemSpy).toHaveBeenCalledWith('bodhi-return-url', ROUTE_SETUP_DOWNLOAD_MODELS);
    setItemSpy.mockRestore();
  });

  it('handles OAuth initiation with external OAuth provider URL', async () => {
    server.use(
      ...mockAppInfoResourceAdmin(),
      ...mockAuthInitiateUnauthenticated({ location: 'https://oauth.example.com/auth?client_id=test' })
    );

    renderWithSetupProvider(<ResourceAdminPage />);

    const loginButton = await screen.findByRole('button', { name: 'Continue with Login →' });
    await userEvent.click(loginButton);

    // Should show "Redirecting..." after successful response and remain disabled
    await waitFor(() => {
      expect(screen.getByRole('button', { name: /redirecting/i })).toBeInTheDocument();
      expect(screen.getByRole('button', { name: /redirecting/i })).toBeDisabled();
    });

    await waitFor(() => {
      expect(window.location.href).toBe('https://oauth.example.com/auth?client_id=test');
    });
  });

  it('handles OAuth initiation with same-origin redirect URL', async () => {
    server.use(
      ...mockAppInfoResourceAdmin(),
      ...mockAuthInitiateAlreadyAuthenticated({ location: 'http://localhost:3000/ui/chat' })
    );

    renderWithSetupProvider(<ResourceAdminPage />);

    const loginButton = await screen.findByRole('button', { name: 'Continue with Login →' });
    await userEvent.click(loginButton);

    // Should show "Redirecting..." after successful response and remain disabled
    await waitFor(() => {
      expect(screen.getByRole('button', { name: /redirecting/i })).toBeInTheDocument();
      expect(screen.getByRole('button', { name: /redirecting/i })).toBeDisabled();
    });

    await waitFor(() => {
      expect(pushMock).toHaveBeenCalledWith('/ui/chat');
    });
  });

  it('handles OAuth initiation successfully', async () => {
    server.use(
      ...mockAppInfoResourceAdmin(),
      ...mockAuthInitiate({ location: 'https://oauth.example.com/auth?client_id=test' })
    );

    renderWithSetupProvider(<ResourceAdminPage />);

    const loginButton = await screen.findByRole('button', { name: 'Continue with Login →' });
    await userEvent.click(loginButton);

    await waitFor(() => {
      expect(window.location.href).toBe('https://oauth.example.com/auth?client_id=test');
    });
  });

  it('displays error message when OAuth initiation fails and re-enables button', async () => {
    server.use(
      ...mockAppInfoResourceAdmin(),
      ...mockAuthInitiateError({ status: 500, code: 'oauth_config_error', message: 'OAuth configuration error' })
    );

    renderWithSetupProvider(<ResourceAdminPage />);

    const loginButton = await screen.findByRole('button', { name: 'Continue with Login →' });
    await userEvent.click(loginButton);

    await waitFor(() => {
      expect(screen.getByText('OAuth configuration error')).toBeInTheDocument();
    });

    // Verify login button is re-enabled after error
    await waitFor(() => {
      expect(screen.getByRole('button', { name: 'Continue with Login →' })).toBeInTheDocument();
      expect(screen.getByRole('button', { name: 'Continue with Login →' })).not.toBeDisabled();
    });
  });

  it('displays generic error message when OAuth initiation fails without specific message', async () => {
    server.use(...mockAppInfoResourceAdmin(), ...mockAuthInitiateError());

    renderWithSetupProvider(<ResourceAdminPage />);

    const loginButton = await screen.findByRole('button', { name: 'Continue with Login →' });
    await userEvent.click(loginButton);

    await waitFor(() => {
      expect(screen.getByText('Internal server error')).toBeInTheDocument();
    });
  });

  it('handles default auth initiate response', async () => {
    server.use(...mockAppInfoResourceAdmin(), ...mockAuthInitiate());

    renderWithSetupProvider(<ResourceAdminPage />);

    const loginButton = await screen.findByRole('button', { name: 'Continue with Login →' });
    await userEvent.click(loginButton);

    await waitFor(() => {
      expect(window.location.href).toBe('https://oauth.example.com/auth?client_id=test');
    });
  });

  it('handles custom URL in response by treating as external and keeping button disabled', async () => {
    server.use(...mockAppInfoResourceAdmin(), ...mockAuthInitiate({ location: 'invalid-url-format' }));

    renderWithSetupProvider(<ResourceAdminPage />);

    const loginButton = await screen.findByRole('button', { name: 'Continue with Login →' });
    await userEvent.click(loginButton);

    // Should show "Redirecting..." and remain disabled even for invalid URLs
    await waitFor(() => {
      expect(screen.getByRole('button', { name: /redirecting/i })).toBeInTheDocument();
      expect(screen.getByRole('button', { name: /redirecting/i })).toBeDisabled();
    });

    await waitFor(() => {
      expect(window.location.href).toBe('invalid-url-format');
    });
  });
});
