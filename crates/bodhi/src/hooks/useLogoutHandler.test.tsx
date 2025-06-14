import { Button } from '@/components/ui/button';
import { useLogoutHandler } from '@/hooks/useLogoutHandler';
import { ENDPOINT_LOGOUT } from '@/hooks/useQuery';
import { createWrapper } from '@/tests/wrapper';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { rest } from 'msw';
import { setupServer } from 'msw/node';
import React from 'react';
import { afterAll, beforeAll, beforeEach, describe, expect, it, vi } from 'vitest';

const server = setupServer();

beforeAll(() => server.listen());
afterAll(() => server.close());
beforeEach(() => {
  server.resetHandlers();
});

// Simple component that uses the useLogoutHandler hook
const LogoutButton: React.FC<{ onSuccess?: (response: any) => void; onError?: (message: string) => void }> = ({
  onSuccess,
  onError,
}) => {
  const { logout, isLoading: isLoggingOut } = useLogoutHandler({ onSuccess, onError });
  return (
    <Button onClick={() => logout()} disabled={isLoggingOut}>
      {isLoggingOut ? 'Logging out...' : 'Log Out'}
    </Button>
  );
};

describe('useLogoutHandler', () => {
  it('calls onSuccess callback when logout succeeds', async () => {
    const mockOnSuccess = vi.fn();
    const mockOnError = vi.fn();

    server.use(
      rest.post(`*${ENDPOINT_LOGOUT}`, (_, res, ctx) => {
        return res(
          ctx.delay(100),
          ctx.status(201),
          ctx.set('Location', 'http://localhost:1135/ui/login'),
          ctx.set('Content-Length', '0')
        );
      })
    );

    render(<LogoutButton onSuccess={mockOnSuccess} onError={mockOnError} />, { wrapper: createWrapper() });

    const logoutButton = screen.getByRole('button', { name: 'Log Out' });
    expect(logoutButton).toBeInTheDocument();

    await userEvent.click(logoutButton);

    expect(screen.getByRole('button', { name: 'Logging out...' })).toBeInTheDocument();
    expect(logoutButton).toBeDisabled();

    await waitFor(() => {
      expect(mockOnSuccess).toHaveBeenCalledWith(
        expect.objectContaining({
          status: 201,
          headers: expect.objectContaining({ location: 'http://localhost:1135/ui/login' }),
        })
      );
      expect(screen.getByRole('button', { name: 'Log Out' })).toBeInTheDocument();
    });

    expect(logoutButton).not.toBeDisabled();
    expect(mockOnError).not.toHaveBeenCalled();
  });

  it('calls onError callback when logout fails', async () => {
    const mockOnSuccess = vi.fn();
    const mockOnError = vi.fn();

    server.use(
      rest.post(`*${ENDPOINT_LOGOUT}`, (_, res, ctx) => {
        return res(ctx.status(500), ctx.json({ error: { message: 'Internal Server Error' } }));
      })
    );

    render(<LogoutButton onSuccess={mockOnSuccess} onError={mockOnError} />, { wrapper: createWrapper() });

    const logoutButton = screen.getByRole('button', { name: 'Log Out' });
    await userEvent.click(logoutButton);

    await waitFor(() => {
      expect(screen.getByRole('button', { name: 'Log Out' })).toBeInTheDocument();
    });

    expect(logoutButton).not.toBeDisabled();
    expect(mockOnSuccess).not.toHaveBeenCalled();
    expect(mockOnError).toHaveBeenCalledWith('Internal Server Error');
  });

  it('handles logout without callbacks', async () => {
    server.use(
      rest.post(`*${ENDPOINT_LOGOUT}`, (_, res, ctx) => {
        return res(
          ctx.status(201),
          ctx.set('Location', 'http://localhost:1135/ui/login'),
          ctx.set('Content-Length', '0')
        );
      })
    );

    render(<LogoutButton />, { wrapper: createWrapper() });

    const logoutButton = screen.getByRole('button', { name: 'Log Out' });
    await userEvent.click(logoutButton);

    await waitFor(() => {
      expect(screen.getByRole('button', { name: 'Log Out' })).toBeInTheDocument();
    });

    expect(logoutButton).not.toBeDisabled();
  });
});
