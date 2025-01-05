import React from 'react';
import {
  render,
  screen,
  waitFor,
  waitForElementToBeRemoved,
} from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import {
  describe,
  it,
  expect,
  beforeEach,
  vi,
  beforeAll,
  afterAll,
  afterEach,
} from 'vitest';
import LoginContent from '@/app/ui/login/page';
import { createWrapper } from '@/tests/wrapper';
import { setupServer } from 'msw/node';
import { rest } from 'msw';

// Mock the hooks
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
  vi.clearAllMocks();
});

describe('LoginContent with user not Logged In', () => {
  beforeEach(() => {
    vi.resetAllMocks();
    pushMock.mockClear();
    server.use(
      rest.get('*/api/ui/user', (req, res, ctx) => {
        return res(ctx.status(200), ctx.json({ logged_in: false }));
      }),
      rest.get('*/app/info', (req, res, ctx) => {
        return res(ctx.status(200), ctx.json({ status: 'ready' }));
      })
    );
  });

  it('renders loading state', async () => {
    server.use(
      rest.get('*/api/ui/user', (req, res, ctx) => {
        return res(
          ctx.delay(100),
          ctx.status(200),
          ctx.json({ logged_in: false })
        );
      })
    );
    render(<LoginContent />, { wrapper: createWrapper() });
    await waitForElementToBeRemoved(() =>
      screen.getByText('Initializing app...')
    );
  });

  it('renders login button when user is not logged in', async () => {
    render(<LoginContent />, { wrapper: createWrapper() });
    await waitFor(() => expect(screen.getByRole('button', { name: 'Log In' })));
    const loginButton = screen.getByRole('button', { name: 'Log In' });
    expect(loginButton).toBeDefined();
    expect(
      screen.getByText('You need to login to use the Bodhi App')
    ).toBeDefined();
  });
});

describe('LoginContent with user Logged In', () => {
  beforeEach(() => {
    vi.resetAllMocks();
    pushMock.mockClear();
    server.use(
      rest.get('*/api/ui/user', (_, res, ctx) => {
        return res(
          ctx.status(200),
          ctx.json({ logged_in: true, email: 'test@example.com' })
        );
      }),
      rest.get('*/app/info', (_, res, ctx) => {
        return res(ctx.status(200), ctx.json({ status: 'ready' }));
      })
    );
  });

  it('renders welcome message and logout button when user is logged in', async () => {
    render(<LoginContent />, { wrapper: createWrapper() });
    await waitFor(() =>
      expect(screen.getByRole('button', { name: 'Log Out' }))
    );
    expect(screen.getByText('Welcome')).toBeDefined();
    expect(
      screen.getByText('You are logged in as test@example.com')
    ).toBeDefined();
    expect(screen.getByRole('button', { name: 'Go to Home' })).toBeDefined();
    expect(screen.getByRole('button', { name: 'Log Out' })).toBeDefined();
  });

  it('calls logout function when logout button is clicked and pushes the route in location', async () => {
    server.use(
      rest.post('*/api/ui/logout', (req, res, ctx) => {
        return res(
          ctx.status(200),
          ctx.set('Location', 'http://localhost:1135/ui/test/login'),
          ctx.json({})
        );
      })
    );
    render(<LoginContent />, { wrapper: createWrapper() });
    await waitFor(() =>
      expect(screen.getByRole('button', { name: 'Log Out' }))
    );
    const logoutButton = screen.getByRole('button', { name: 'Log Out' });
    await userEvent.click(logoutButton);
    expect(pushMock).toHaveBeenCalledWith(
      'http://localhost:1135/ui/test/login'
    );
  });

  it('disables logout button and shows loading text when logging out', async () => {
    server.use(
      rest.post('*/api/ui/logout', (_, res, ctx) => {
        return res(
          ctx.delay(100),
          ctx.status(200),
          ctx.set('Location', 'http://localhost:1135/ui/test/login'),
          ctx.json({})
        );
      })
    );

    render(<LoginContent />, { wrapper: createWrapper() });
    await waitFor(() =>
      expect(screen.getByRole('button', { name: 'Log Out' }))
    );
    const logoutButton = screen.getByRole('button', { name: 'Log Out' });
    await userEvent.click(logoutButton);
    const loggingOut = screen.getByRole('button', { name: 'Logging out...' });
    expect(loggingOut).toBeInTheDocument();
    expect(loggingOut).toHaveAttribute('disabled');
  });
});

describe('LoginContent access control', () => {
  it('redirects to setup when app is not setup', async () => {
    server.use(
      rest.get('*/app/info', (_, res, ctx) => {
        return res(ctx.status(200), ctx.json({ status: 'setup' }));
      })
    );
    render(<LoginContent />, { wrapper: createWrapper() });
    await waitForElementToBeRemoved(() =>
      screen.getByText('Initializing app...')
    );
    await waitFor(() => expect(pushMock).toHaveBeenCalledWith('/ui/setup'));
  });
});
