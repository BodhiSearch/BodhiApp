import { LoginMenu } from '@/components/LoginMenu';
import { createWrapper, mockWindowLocation } from '@/tests/wrapper';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { setupMswV2, server } from '@/test-utils/msw-v2/setup';
import {
  mockAuthInitiate,
  mockAuthInitiateConfigError,
  mockLogout,
  mockLogoutSessionError,
} from '@/test-utils/msw-v2/handlers/auth';
import { mockUserLoggedOut, mockUserLoggedIn } from '@/test-utils/msw-v2/handlers/user';
import { mockAppInfo } from '@/test-utils/msw-v2/handlers/info';
import { afterAll, afterEach, beforeAll, beforeEach, describe, expect, it, vi } from 'vitest';
import { redirect } from 'next/navigation';

const mockPush = vi.fn();
vi.mock('next/navigation', () => ({
  useRouter: () => ({
    push: mockPush,
    refresh: vi.fn(),
  }),
  usePathname: () => '/',
  redirect: vi.fn(),
}));

const mockToast = vi.fn();
vi.mock('@/hooks/use-toast', () => ({
  useToast: () => ({ toast: mockToast }),
}));

// MSW v2 setup with default handlers
setupMswV2();

// Set up default handlers using MSW v2 patterns
beforeEach(() => {
  server.use(
    ...mockUserLoggedOut(),
    ...mockAppInfo({ status: 'ready' }),
    ...mockAuthInitiate({ location: 'https://oauth.example.com/auth?client_id=test' })
  );
});

beforeEach(() => {
  mockWindowLocation('http://localhost:3000/ui/login');
  vi.clearAllMocks();
});

describe('LoginMenu Component', () => {
  it('shows login button when logged out', async () => {
    render(<LoginMenu />, { wrapper: createWrapper() });

    await waitFor(() => {
      const loginButton = screen.getByRole('button', { name: /login/i });
      expect(loginButton).toBeInTheDocument();
      expect(loginButton).toHaveClass('border-primary');
    });
  });

  it('shows logout button and email when logged in', async () => {
    server.use(...mockUserLoggedIn({ role: 'resource_user', username: 'test@example.com' }));

    render(<LoginMenu />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(screen.getByRole('button', { name: /log out/i })).toBeInTheDocument();
      expect(screen.getByText('Logged in as test@example.com')).toBeInTheDocument();
    });
  });

  it('handles OAuth initiation on login button click', async () => {
    render(<LoginMenu />, { wrapper: createWrapper() });

    const loginButton = await screen.findByRole('button', { name: /login/i });
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
    server.use(...mockAuthInitiate({ location: 'http://localhost:3000/ui/chat' }));

    render(<LoginMenu />, { wrapper: createWrapper() });

    const loginButton = await screen.findByRole('button', { name: /login/i });
    await userEvent.click(loginButton);

    await waitFor(() => {
      expect(mockPush).toHaveBeenCalledWith('/ui/chat');
    });
  });

  it('shows initiating and redirecting states during OAuth initiation', async () => {
    // Use mock handler with delay for testing loading states
    server.use(...mockAuthInitiate({ location: 'https://oauth.example.com/auth?client_id=test' }, 100));

    render(<LoginMenu />, { wrapper: createWrapper() });

    const loginButton = await screen.findByRole('button', { name: /login/i });
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

  it('shows error message when OAuth initiation fails and re-enables button', async () => {
    server.use(...mockAuthInitiateConfigError());

    render(<LoginMenu />, { wrapper: createWrapper() });

    const loginButton = await screen.findByRole('button', { name: /login/i });
    await userEvent.click(loginButton);

    await waitFor(() => {
      expect(screen.getByText('OAuth configuration error')).toBeInTheDocument();
    });

    // Verify login button is re-enabled after error
    await waitFor(() => {
      expect(screen.getByRole('button', { name: /login/i })).toBeInTheDocument();
      expect(screen.getByRole('button', { name: /login/i })).not.toBeDisabled();
    });
  });

  it('shows default auth initiate response', async () => {
    server.use(...mockAuthInitiate());

    render(<LoginMenu />, { wrapper: createWrapper() });

    const loginButton = await screen.findByRole('button', { name: /login/i });
    await userEvent.click(loginButton);

    // Should redirect to default OAuth URL
    await waitFor(() => {
      expect(window.location.href).toBe('https://oauth.example.com/auth?client_id=test');
    });
  });

  it('handles custom URL in response by treating as external and keeping button disabled', async () => {
    server.use(...mockAuthInitiate({ location: 'invalid-url-format' }));

    render(<LoginMenu />, { wrapper: createWrapper() });

    const loginButton = await screen.findByRole('button', { name: /login/i });
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

  it('handles logout action with external redirect URL', async () => {
    server.use(
      ...mockUserLoggedIn({ role: 'resource_user' }),
      // Use mock handler with delay for testing loading states
      ...mockLogout({ location: 'http://localhost:1135/ui/login' }, 100)
    );

    render(<LoginMenu />, { wrapper: createWrapper() });

    const logoutButton = await screen.findByRole('button', { name: /log out/i });
    await userEvent.click(logoutButton);

    expect(screen.getByRole('button', { name: /logging out/i })).toBeInTheDocument();

    await waitFor(() => {
      expect(redirect).toHaveBeenCalledWith('http://localhost:1135/ui/login');
    });
  });

  it('handles logout action with internal redirect URL', async () => {
    server.use(...mockUserLoggedIn({ role: 'resource_user' }), ...mockLogout({ location: '/ui/login' }));

    render(<LoginMenu />, { wrapper: createWrapper() });

    const logoutButton = await screen.findByRole('button', { name: /log out/i });
    await userEvent.click(logoutButton);

    // Wait for logout to complete and navigation to occur
    await waitFor(() => {
      expect(mockPush).toHaveBeenCalledWith('/ui/login');
    });
  });

  it('handles logout error', async () => {
    server.use(...mockUserLoggedIn({ role: 'resource_user' }), ...mockLogoutSessionError());

    render(<LoginMenu />, { wrapper: createWrapper() });

    const logoutButton = await screen.findByRole('button', { name: /log out/i });
    await userEvent.click(logoutButton);

    await waitFor(() => {
      expect(screen.getByRole('button', { name: /log out/i })).toBeInTheDocument();
    });

    expect(logoutButton).not.toBeDisabled();

    // Should redirect to login page on error
    await waitFor(() => {
      expect(redirect).toHaveBeenCalledWith('/ui/login');
    });
  });

  it('handles logout with default location', async () => {
    server.use(...mockUserLoggedIn({ role: 'resource_user' }), ...mockLogout());

    render(<LoginMenu />, { wrapper: createWrapper() });

    const logoutButton = await screen.findByRole('button', { name: /log out/i });
    await userEvent.click(logoutButton);

    await waitFor(() => {
      expect(redirect).toHaveBeenCalledWith('http://localhost:1135/ui/login');
    });
  });

  it('shows nothing during loading', async () => {
    // This test relies on user handler which still has delayMs support for now
    server.use(...mockUserLoggedIn({ role: 'resource_user' }));

    const { container } = render(<LoginMenu />, { wrapper: createWrapper() });
    expect(container).toBeEmptyDOMElement();
  });
});
