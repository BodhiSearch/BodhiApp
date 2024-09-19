import React from 'react';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, it, expect, beforeEach, vi, beforeAll, afterAll } from 'vitest';
import LoginContent from './page';
import { useUserContext } from '@/hooks/useUserContext';
import { useLogoutHandler } from '@/hooks/useLogoutHandler';
import { QueryClient, QueryClientProvider } from 'react-query';
import { setupServer } from 'msw/node';
import { rest } from 'msw';

// Mock the hooks
vi.mock('@/hooks/useUserContext');
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
    <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
  );
};

describe('LoginContent with user not Logged In', () => {
  beforeEach(() => {
    vi.resetAllMocks();
    pushMock.mockClear();
  });

  it('renders loading state', () => {
    vi.mocked(useUserContext).mockReturnValue({ isLoading: true } as any);
    render(<LoginContent />, { wrapper: createWrapper() });
    expect(screen.getByText('Loading...')).toBeDefined();
  });

  it('renders login button when user is not logged in', () => {
    vi.mocked(useUserContext).mockReturnValue({ userInfo: { logged_in: false }, isLoading: false } as any);
    render(<LoginContent />, { wrapper: createWrapper() });
    const loginButton = screen.getByRole('button', { name: 'Log In' });
    expect(loginButton).toBeDefined();
    expect(screen.getByText('You need to login to use the Bodhi App')).toBeDefined();
  });
});

describe('LoginContent with user Logged In', () => {
  beforeEach(() => {
    vi.resetAllMocks();
    pushMock.mockClear();
    vi.mocked(useUserContext).mockReturnValue({
      userInfo: { logged_in: true, email: 'test@example.com' },
      isLoading: false
    } as any);
  });

  it('renders welcome message and logout button when user is logged in', () => {
    render(<LoginContent />, { wrapper: createWrapper() });
    expect(screen.getByText('Welcome')).toBeDefined();
    expect(screen.getByText('You are logged in as test@example.com')).toBeDefined();
    expect(screen.getByRole('button', { name: 'Go to Home' })).toBeDefined();
    expect(screen.getByRole('button', { name: 'Log Out' })).toBeDefined();
  });

  it('calls logout function when logout button is clicked and pushes the route in location', async () => {
    server.use(
      rest.post('*/api/ui/logout', (req, res, ctx) => {
        return res(ctx.status(200), ctx.set('Location', 'http://localhost:1135/ui/test/login'), ctx.json({}));
      })
    );

    render(<LoginContent />, { wrapper: createWrapper() });

    const logoutButton = screen.getByRole('button', { name: 'Log Out' });
    await userEvent.click(logoutButton);
    expect(pushMock).toHaveBeenCalledWith('http://localhost:1135/ui/test/login');
  });

  it('disables logout button and shows loading text when logging out', async () => {
    server.use(
      rest.post('*/api/ui/logout', (req, res, ctx) => {
        return res(ctx.delay(100), ctx.status(200), ctx.set('Location', 'http://localhost:1135/ui/test/login'), ctx.json({}));
      })
    );

    render(<LoginContent />, { wrapper: createWrapper() });

    const logoutButton = screen.getByRole('button', { name: 'Log Out' });
    await userEvent.click(logoutButton);
    const loggingOut = screen.getByRole('button', { name: 'Logging out...' });
    expect(loggingOut).toBeInTheDocument();
    expect(loggingOut).toHaveAttribute('disabled');
  });
});
