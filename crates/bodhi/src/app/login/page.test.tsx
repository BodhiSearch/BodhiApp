import LoginPage, { LoginContent } from '@/app/login/page';
import {
  mockAuthInitiate,
  mockAuthInitiateConfigError,
  mockAuthInitiateError,
  mockLogout,
} from '@/test-utils/msw-v2/handlers/auth';
import { mockAppInfo, mockAppInfoSetup } from '@/test-utils/msw-v2/handlers/info';
import { mockTenantsList } from '@/test-utils/msw-v2/handlers/tenants';
import { mockUserLoggedIn, mockUserLoggedOut } from '@/test-utils/msw-v2/handlers/user';
import { server } from '@/test-utils/msw-v2/setup';
import { createWrapper, mockWindowLocation } from '@/tests/wrapper';
import { act, render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterAll, beforeAll, beforeEach, describe, expect, it, vi } from 'vitest';

// Mock the hooks
const pushMock = vi.fn();
const replaceMock = vi.fn();
vi.mock('next/navigation', () => ({
  useRouter: () => ({
    push: pushMock,
    replace: replaceMock,
  }),
  useSearchParams: () => new URLSearchParams(),
}));

beforeAll(() => {
  server.listen();
});

afterAll(() => server.close());

beforeEach(() => {
  mockWindowLocation('http://localhost:3000/ui/login');
  server.resetHandlers();
  pushMock.mockClear();
  replaceMock.mockClear();
  sessionStorage.clear();
  vi.clearAllMocks();
});

describe('LoginContent loading states', () => {
  beforeEach(() => {
    vi.resetAllMocks();
    pushMock.mockClear();
    server.use(...mockUserLoggedOut(), ...mockAppInfo({ status: 'ready' }));
  });

  it('shows loading indicator while fetching data', async () => {
    render(<LoginContent />, { wrapper: createWrapper() });
    expect(screen.getByText('Loading...')).toBeInTheDocument();
    await waitFor(() => expect(screen.getByRole('button', { name: 'Login' })).toBeInTheDocument());
  });
});

describe('LoginContent with user not Logged In', () => {
  beforeEach(() => {
    vi.resetAllMocks();
    pushMock.mockClear();
    server.use(
      ...mockUserLoggedOut(),
      ...mockAppInfo({ status: 'ready', client_id: 'test_client_id' }),
      ...mockAuthInitiateConfigError(),
      ...mockLogout({ location: 'http://localhost:1135/ui/login' })
    );
  });

  it('renders login button when user is not logged in', async () => {
    await act(async () => {
      render(<LoginContent />, { wrapper: createWrapper() });
    });
    const loginButton = screen.getByRole('button', { name: 'Login' });
    expect(loginButton).toBeDefined();
    expect(screen.getByText('Login to use the Bodhi App')).toBeInTheDocument();
  });

  it('renders login button with correct styling', async () => {
    await act(async () => {
      render(<LoginContent />, { wrapper: createWrapper() });
    });
    const loginButton = screen.getByRole('button', { name: 'Login' });
    expect(loginButton).toHaveClass('w-full');
    expect(loginButton).not.toBeDisabled();
  });

  it('handles OAuth initiation when login required and redirects to auth URL', async () => {
    server.use(...mockAuthInitiate({ location: 'https://oauth.example.com/auth?client_id=test' }));

    await act(async () => {
      render(<LoginContent />, { wrapper: createWrapper() });
    });

    const loginButton = screen.getByRole('button', { name: 'Login' });
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

  it('shows initiating and redirecting states during OAuth initiation', async () => {
    server.use(...mockAuthInitiate({ location: 'https://oauth.example.com/auth?client_id=test' }, 100));

    await act(async () => {
      render(<LoginContent />, { wrapper: createWrapper() });
    });

    const loginButton = screen.getByRole('button', { name: 'Login' });
    await userEvent.click(loginButton);

    // Check for initiating state during request
    await waitFor(() => {
      expect(screen.getByRole('button', { name: /initiating/i })).toBeInTheDocument();
      expect(screen.getByRole('button', { name: /initiating/i })).toBeDisabled();
    });

    // Check for redirecting state after successful response
    await waitFor(() => {
      expect(screen.getByRole('button', { name: /redirecting/i })).toBeInTheDocument();
      expect(screen.getByRole('button', { name: /redirecting/i })).toBeDisabled();
    });
  });

  it('displays error message when OAuth initiation fails and re-enables button', async () => {
    server.use(...mockAuthInitiateConfigError());

    await act(async () => {
      render(<LoginContent />, { wrapper: createWrapper() });
    });

    const loginButton = screen.getByRole('button', { name: 'Login' });
    await userEvent.click(loginButton);

    await waitFor(() => {
      expect(screen.getByText('OAuth configuration error')).toBeInTheDocument();
    });

    // Verify login button is re-enabled after error
    await waitFor(() => {
      expect(screen.getByRole('button', { name: 'Login' })).toBeInTheDocument();
      expect(screen.getByRole('button', { name: 'Login' })).not.toBeDisabled();
    });
  });

  it('displays generic error message when OAuth initiation fails without specific message', async () => {
    server.use(...mockAuthInitiateError());

    await act(async () => {
      render(<LoginContent />, { wrapper: createWrapper() });
    });

    const loginButton = screen.getByRole('button', { name: 'Login' });
    await userEvent.click(loginButton);

    await waitFor(() => {
      expect(screen.getByText('Internal server error')).toBeInTheDocument();
    });
  });

  it('handles already authenticated user with external redirect URL', async () => {
    server.use(...mockAuthInitiate({ location: 'https://external.example.com/dashboard' }));

    await act(async () => {
      render(<LoginContent />, { wrapper: createWrapper() });
    });

    const loginButton = screen.getByRole('button', { name: 'Login' });
    await userEvent.click(loginButton);

    await waitFor(() => {
      expect(window.location.href).toBe('https://external.example.com/dashboard');
    });
  });

  it('handles auth initiate with default location', async () => {
    server.use(...mockAuthInitiate());

    await act(async () => {
      render(<LoginContent />, { wrapper: createWrapper() });
    });

    const loginButton = screen.getByRole('button', { name: 'Login' });
    await userEvent.click(loginButton);

    await waitFor(() => {
      expect(window.location.href).toBe('https://oauth.example.com/auth?client_id=test');
    });
  });

  it('handles custom location in auth initiate response', async () => {
    server.use(...mockAuthInitiate({ location: 'invalid-url-format' }));

    await act(async () => {
      render(<LoginContent />, { wrapper: createWrapper() });
    });

    const loginButton = screen.getByRole('button', { name: 'Login' });
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

  it('handles already authenticated user with same-origin redirect URL', async () => {
    server.use(...mockAuthInitiate({ location: 'http://localhost:3000/ui/chat' }));

    await act(async () => {
      render(<LoginContent />, { wrapper: createWrapper() });
    });

    const loginButton = screen.getByRole('button', { name: 'Login' });

    await act(async () => {
      await userEvent.click(loginButton);
    });

    // Should show "Redirecting..." after successful response
    await waitFor(() => {
      expect(screen.getByRole('button', { name: /redirecting/i })).toBeInTheDocument();
      expect(screen.getByRole('button', { name: /redirecting/i })).toBeDisabled();
    });

    // Should use router.push for same-origin URLs
    await waitFor(() => {
      expect(pushMock).toHaveBeenCalledWith('/chat');
    });
  });
});

describe('LoginContent with user Logged In', () => {
  beforeEach(() => {
    vi.resetAllMocks();
    pushMock.mockClear();
    server.use(
      ...mockUserLoggedIn(),
      ...mockAppInfo({ status: 'ready' }),
      ...mockLogout({ location: 'http://localhost:1135/ui/login' })
    );
  });

  it('renders welcome message and logout button when user is logged in', async () => {
    await act(async () => {
      render(<LoginContent />, { wrapper: createWrapper() });
    });
    expect(screen.getByText('You are logged in as test@example.com')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Log Out' })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Go to Home' })).toBeInTheDocument();
  });

  it('calls logout function when logout button is clicked and redirects to external location', async () => {
    server.use(...mockLogout({ location: 'http://localhost:1135/ui/test/login' }));
    await act(async () => {
      render(<LoginContent />, { wrapper: createWrapper() });
    });
    const logoutButton = screen.getByRole('button', { name: 'Log Out' });
    await userEvent.click(logoutButton);

    await waitFor(() => {
      expect(window.location.href).toBe('http://localhost:1135/ui/test/login');
    });
  });

  it('calls logout function when logout button is clicked and redirects to internal location', async () => {
    server.use(...mockLogout({ location: '/ui/login' }));
    await act(async () => {
      render(<LoginContent />, { wrapper: createWrapper() });
    });
    const logoutButton = screen.getByRole('button', { name: 'Log Out' });
    await userEvent.click(logoutButton);

    await waitFor(() => {
      expect(pushMock).toHaveBeenCalledWith('/login');
    });
  });

  it('disables logout button and shows loading text when logging out', async () => {
    server.use(...mockLogout({ location: 'http://localhost:1135/ui/test/login' }, 100));

    await act(async () => {
      render(<LoginContent />, { wrapper: createWrapper() });
    });
    const logoutButton = screen.getByRole('button', { name: 'Log Out' });
    await userEvent.click(logoutButton);
    const loggingOut = screen.getByRole('button', { name: 'Logging out...' });
    expect(loggingOut).toBeInTheDocument();
    expect(loggingOut).toHaveAttribute('disabled');
  });

  it('renders buttons with correct styling', async () => {
    await act(async () => {
      render(<LoginContent />, { wrapper: createWrapper() });
    });
    const logoutButton = screen.getByRole('button', { name: 'Log Out' });
    expect(logoutButton).toHaveClass('w-full');
  });
});

describe('LoginContent access control', () => {
  it('redirects to setup when app is not setup', async () => {
    server.use(...mockAppInfoSetup());
    await act(async () => {
      render(<LoginPage />, { wrapper: createWrapper() });
    });
    await waitFor(() => expect(pushMock).toHaveBeenCalledWith('/setup'));
  });
});

describe('MultiTenantLoginContent', () => {
  beforeEach(() => {
    vi.resetAllMocks();
    pushMock.mockClear();
    replaceMock.mockClear();
    sessionStorage.clear();
  });

  it('does not call /bodhi/v1/tenants when no dashboard session present', async () => {
    server.use(...mockUserLoggedOut(), ...mockAppInfo({ status: 'ready', deployment: 'multi_tenant' }));

    await act(async () => {
      render(<LoginPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByRole('button', { name: 'Login to Bodhi Platform' })).toBeInTheDocument();
    });

    // Verify no tenants call was made by ensuring login state is shown (not loading)
    expect(screen.getByTestId('login-page')).toBeInTheDocument();
    expect(screen.queryByText('Select Workspace')).not.toBeInTheDocument();
  });

  it('shows login button when no dashboard session (State A)', async () => {
    server.use(...mockUserLoggedOut(), ...mockAppInfo({ status: 'ready', deployment: 'multi_tenant' }));

    await act(async () => {
      render(<LoginPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByRole('button', { name: 'Login to Bodhi Platform' })).toBeInTheDocument();
    });

    const card = screen.getByTestId('login-page').querySelector('[data-test-state="login"]');
    expect(card).toBeInTheDocument();
  });

  it('shows tenant selection when dashboard session present without tenant login (State B)', async () => {
    const tenants = [
      { client_id: 'tenant-1', name: 'Workspace 1', status: 'ready' as const, is_active: false, logged_in: false },
      { client_id: 'tenant-2', name: 'Workspace 2', status: 'ready' as const, is_active: false, logged_in: false },
    ];

    server.use(
      ...mockUserLoggedOut({
        stub: true,
        dashboard: { user_id: 'test-id', username: 'test@example.com', first_name: null, last_name: null },
      }),
      ...mockAppInfo({ status: 'ready', deployment: 'multi_tenant' }),
      ...mockTenantsList(tenants, { stub: true })
    );

    await act(async () => {
      render(<LoginPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByText('Select Workspace')).toBeInTheDocument();
    });

    const card = screen.getByTestId('login-page').querySelector('[data-test-state="select"]');
    expect(card).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Workspace 1' })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Workspace 2' })).toBeInTheDocument();
  });

  it('shows welcome when fully authenticated with tenant (State C)', async () => {
    server.use(
      ...mockUserLoggedIn({
        role: 'resource_admin',
        dashboard: { user_id: 'test-id', username: 'test@example.com', first_name: null, last_name: null },
      }),
      ...mockAppInfo({ status: 'ready', deployment: 'multi_tenant', client_id: 'test-client' }),
      ...mockLogout({ location: '/ui/login' })
    );

    await act(async () => {
      render(<LoginPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByText('Welcome')).toBeInTheDocument();
    });

    const card = screen.getByTestId('login-page').querySelector('[data-test-state="welcome"]');
    expect(card).toBeInTheDocument();
    expect(screen.getByText(/test@example.com/)).toBeInTheDocument();
  });
});
