import { ResourceAdminContent } from '@/components/setup/resource-admin/ResourceAdminPage';
import { createWrapper } from '@/tests/wrapper';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { rest } from 'msw';
import { setupServer } from 'msw/node';
import {
  afterAll,
  afterEach,
  beforeAll,
  beforeEach,
  describe,
  expect,
  it,
  vi,
} from 'vitest';
import { ENDPOINT_AUTH_INITIATE } from '@/hooks/useQuery';

// Mock window.location
const mockLocation = {
  href: '',
  origin: 'http://localhost:3000',
};
Object.defineProperty(window, 'location', {
  value: mockLocation,
  writable: true,
});

const server = setupServer();

beforeAll(() => server.listen());
beforeEach(() => {
  vi.clearAllMocks();
  mockLocation.href = '';
});
afterEach(() => server.resetHandlers());
afterAll(() => server.close());

describe('ResourceAdminContent', () => {
  it('renders admin setup page with all required content and functionality', () => {
    render(<ResourceAdminContent />, { wrapper: createWrapper() });

    // Main content and layout
    expect(screen.getByTestId('resource-admin-page')).toBeInTheDocument();
    expect(screen.getByText('Admin Setup')).toBeInTheDocument();
    expect(screen.getByText('Step 2 of 4')).toBeInTheDocument();

    // Admin setup description
    expect(
      screen.getByText(
        'You are setting up Bodhi App in authenticated mode. The email address you log in with will be granted admin role for this app instance.'
      )
    ).toBeInTheDocument();

    // Admin privileges information
    expect(screen.getByText('As an Admin, you can:')).toBeInTheDocument();
    expect(screen.getByText('Manage user access and permissions')).toBeInTheDocument();
    expect(screen.getByText('Unrestricted access to system-wide settings')).toBeInTheDocument();

    // Action button and helper text
    const continueButton = screen.getByRole('button', { name: 'Continue with Login →' });
    expect(continueButton).toBeInTheDocument();
    expect(continueButton).toBeEnabled();
    expect(continueButton).toHaveClass('w-full');
    expect(screen.getByText('Login with a valid email address to continue')).toBeInTheDocument();
  });

  it('handles successful OAuth flow and redirects to auth URL', async () => {
    const authUrl = 'https://auth.example.com/oauth/authorize?client_id=test';

    server.use(
      rest.post(`*${ENDPOINT_AUTH_INITIATE}`, (_req, res, ctx) => {
        return res(
          ctx.status(401),
          ctx.set('WWW-Authenticate', 'Bearer realm="OAuth"'),
          ctx.json({ auth_url: authUrl })
        );
      })
    );

    render(<ResourceAdminContent />, { wrapper: createWrapper() });

    const continueButton = screen.getByRole('button', { name: 'Continue with Login →' });
    await userEvent.click(continueButton);

    await waitFor(() => {
      expect(mockLocation.href).toBe(authUrl);
    });
  });

  it('handles OAuth errors and allows retry', async () => {
    const authUrl = 'https://auth.example.com/oauth/authorize?client_id=test';
    let callCount = 0;

    server.use(
      rest.post(`*${ENDPOINT_AUTH_INITIATE}`, (_req, res, ctx) => {
        callCount++;
        if (callCount === 1) {
          return res(
            ctx.status(500),
            ctx.json({
              error: {
                message: 'OAuth service unavailable',
                type: 'server_error',
              },
            })
          );
        }
        return res(
          ctx.status(401),
          ctx.json({ auth_url: authUrl })
        );
      })
    );

    render(<ResourceAdminContent />, { wrapper: createWrapper() });

    const continueButton = screen.getByRole('button', { name: 'Continue with Login →' });
    await userEvent.click(continueButton);

    // Should show error state
    await waitFor(() => {
      expect(screen.getByText('OAuth service unavailable')).toBeInTheDocument();
    });

    const tryAgainButton = screen.getByRole('button', { name: 'Try Again' });
    expect(tryAgainButton).toBeInTheDocument();
    expect(tryAgainButton).toHaveClass('w-full');

    // Test retry functionality
    await userEvent.click(tryAgainButton);

    await waitFor(() => {
      expect(mockLocation.href).toBe(authUrl);
    });
  });

  it('handles network errors with fallback message', async () => {
    server.use(
      rest.post(`*${ENDPOINT_AUTH_INITIATE}`, (_req, res, _ctx) => {
        return res.networkError('Network error');
      })
    );

    render(<ResourceAdminContent />, { wrapper: createWrapper() });
    const continueButton = screen.getByRole('button', { name: 'Continue with Login →' });
    await userEvent.click(continueButton);

    await waitFor(() => {
      expect(screen.getByText('Failed to initiate OAuth flow')).toBeInTheDocument();
    });

    const tryAgainButton = screen.getByRole('button', { name: 'Try Again' });
    expect(tryAgainButton).toBeInTheDocument();
    expect(tryAgainButton).toBeEnabled();
  });

  it('displays error message with proper styling', async () => {
    server.use(
      rest.post(`*${ENDPOINT_AUTH_INITIATE}`, (_req, res, ctx) => {
        return res(
          ctx.status(500),
          ctx.json({
            error: {
              message: 'Test error message',
              type: 'server_error',
            },
          })
        );
      })
    );

    render(<ResourceAdminContent />, { wrapper: createWrapper() });
    const continueButton = screen.getByRole('button', { name: 'Continue with Login →' });
    await userEvent.click(continueButton);

    await waitFor(() => {
      expect(screen.getByText('Test error message')).toBeInTheDocument();
    });

    // Verify error styling and Try Again button
    const errorElement = screen.getByText('Test error message');
    expect(errorElement).toHaveClass('text-red-600', 'text-center');

    const tryAgainButton = screen.getByRole('button', { name: 'Try Again' });
    expect(tryAgainButton).toHaveClass('w-full');
    expect(tryAgainButton).toBeEnabled();
  });
});
