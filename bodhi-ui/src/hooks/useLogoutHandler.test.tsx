import React from 'react';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, it, expect, beforeEach, vi, beforeAll, afterAll } from 'vitest';
import { useLogoutHandler } from './useLogoutHandler';
import { QueryClient, QueryClientProvider } from 'react-query';
import { setupServer } from 'msw/node';
import { rest } from 'msw';
import { Button } from '@/components/ui/button';

// Mock useToast hook
const toastMock = vi.fn();
vi.mock('@/hooks/use-toast', () => ({
  useToast: () => ({ toast: toastMock }),
}));

const server = setupServer();
const pushMock = vi.fn();

vi.mock('next/navigation', () => ({
  useRouter: () => ({
    push: pushMock,
  }),
}));

beforeAll(() => server.listen());
afterAll(() => server.close());
beforeEach(() => {
  server.resetHandlers();
  pushMock.mockClear();
  toastMock.mockClear();
});

const createWrapper = () => {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: {
        retry: false,
      },
    },
  });
  return ({ children }: { children: React.ReactNode }) => (
    <QueryClientProvider client={queryClient} > {children} </QueryClientProvider>
  );
};

// Simple component that uses the useLogoutHandler hook
const LogoutButton: React.FC = () => {
  const { logout, isLoading: isLoggingOut } = useLogoutHandler();
  return (
    <Button onClick={() => logout()} disabled={isLoggingOut} >
      {isLoggingOut ? 'Logging out...' : 'Log Out'}
    </Button>
  );
};

describe('useLogoutHandler', () => {
  beforeEach(() => {
    vi.resetAllMocks();
    pushMock.mockClear();
    toastMock.mockClear();
  });

  it('renders logout button and handles successful logout', async () => {
    server.use(
      rest.post('*/api/ui/logout', (_, res, ctx) => {
        return res(ctx.delay(100), ctx.status(200), ctx.set('Location', 'http://localhost:1135/ui/test/login'), ctx.json({}));
      })
    );

    render(<LogoutButton />, { wrapper: createWrapper() });

    const logoutButton = screen.getByRole('button', { name: 'Log Out' });
    expect(logoutButton).toBeInTheDocument();

    await userEvent.click(logoutButton);

    expect(screen.getByRole('button', { name: 'Logging out...' })).toBeInTheDocument();
    expect(logoutButton).toBeDisabled();
    await waitFor(() => {
      expect(pushMock).toHaveBeenCalledWith('http://localhost:1135/ui/test/login');
      expect(screen.getByRole('button', { name: 'Log Out' })).toBeInTheDocument();
    });
    expect(logoutButton).not.toBeDisabled();
  });

  it('handles logout API error and shows toast message', async () => {
    server.use(
      rest.post('*/api/ui/logout', (req, res, ctx) => {
        return res(ctx.status(500), ctx.json({ message: 'Internal Server Error' }));
      })
    );

    render(<LogoutButton />, { wrapper: createWrapper() });

    const logoutButton = screen.getByRole('button', { name: 'Log Out' });
    await userEvent.click(logoutButton);

    await waitFor(() => {
      expect(screen.getByRole('button', { name: 'Log Out' })).toBeInTheDocument();
    });

    expect(logoutButton).not.toBeDisabled();
    expect(pushMock).not.toHaveBeenCalled();

    // Check if toast was called with the correct error message
    expect(toastMock).toHaveBeenCalledWith({
      variant: 'destructive',
      title: 'Logout failed',
      description: 'Message: Internal Server Error. Try again later.',
    });
  });
});
