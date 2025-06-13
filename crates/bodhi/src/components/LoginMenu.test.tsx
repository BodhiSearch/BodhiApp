import { LoginMenu } from '@/components/LoginMenu';
import {
  ENDPOINT_APP_INFO,
  ENDPOINT_LOGOUT,
  ENDPOINT_USER_INFO
} from '@/hooks/useQuery';
import { createWrapper } from '@/tests/wrapper';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { rest } from 'msw';
import { setupServer } from 'msw/node';
import { afterAll, afterEach, beforeAll, describe, expect, it, vi } from 'vitest';

const mockPush = vi.fn();
vi.mock('next/navigation', () => ({
  useRouter: () => ({
    push: mockPush,
    refresh: vi.fn(),
  }),
  usePathname: () => '/',
}));

const mockToast = vi.fn();
vi.mock('@/hooks/use-toast', () => ({
  useToast: () => ({ toast: mockToast }),
}));

const mockLoggedInUser = {
  logged_in: true,
  email: 'test@example.com',
};

const mockLoggedOutUser = {
  logged_in: false,
};

const mockAppInfo = {
  status: 'ready',
  version: '0.1.0',
};

const server = setupServer(
  rest.get(`*${ENDPOINT_USER_INFO}`, (_, res, ctx) => {
    return res(ctx.json(mockLoggedOutUser));
  }),
  rest.get(`*${ENDPOINT_APP_INFO}`, (_, res, ctx) => {
    return res(ctx.json(mockAppInfo));
  }),
  rest.post(`*${ENDPOINT_LOGOUT}`, (_, res, ctx) => {
    return res(ctx.status(200));
  })
);

beforeAll(() => server.listen());
afterAll(() => server.close());
afterEach(() => {
  server.resetHandlers();
  mockToast.mockClear();
  mockPush.mockClear();
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
    server.use(
      rest.get(`*${ENDPOINT_USER_INFO}`, (_, res, ctx) => {
        return res(ctx.json(mockLoggedInUser));
      })
    );

    render(<LoginMenu />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(screen.getByRole('button', { name: /log out/i })).toBeInTheDocument();
      expect(screen.getByText(`Logged in as ${mockLoggedInUser.email}`)).toBeInTheDocument();
    });
  });



  it('handles logout action', async () => {
    server.use(
      rest.get(`*${ENDPOINT_USER_INFO}`, (_, res, ctx) => {
        return res(ctx.json(mockLoggedInUser));
      }),
      rest.post(`*${ENDPOINT_LOGOUT}`, (_, res, ctx) => {
        return res(ctx.delay(100), ctx.status(200));
      })
    );

    render(<LoginMenu />, { wrapper: createWrapper() });

    const logoutButton = await screen.findByRole('button', { name: /log out/i });
    await userEvent.click(logoutButton);

    expect(screen.getByRole('button', { name: /logging out/i })).toBeInTheDocument();
  });

  it('shows nothing during loading', async () => {
    server.use(
      rest.get(`*${ENDPOINT_USER_INFO}`, (_, res, ctx) => {
        return res(ctx.delay(100), ctx.json(mockLoggedInUser));
      })
    );

    const { container } = render(<LoginMenu />, { wrapper: createWrapper() });
    expect(container).toBeEmptyDOMElement();
  });
}); 