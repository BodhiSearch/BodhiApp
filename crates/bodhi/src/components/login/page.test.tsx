import LoginPage, { LoginContent } from '@/components/login/page';
import { ENDPOINT_APP_INFO, ENDPOINT_LOGOUT, ENDPOINT_USER_INFO } from '@/hooks/useQuery';
import { createWrapper } from '@/tests/wrapper';
import {
  act,
  render,
  screen,
  waitFor
} from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { rest } from 'msw';
import { setupServer } from 'msw/node';
import {
  afterAll,
  beforeAll,
  beforeEach,
  describe,
  expect,
  it,
  vi
} from 'vitest';

// Mock the hooks
const server = setupServer();
const pushMock = vi.fn();
vi.mock('@/lib/navigation', () => ({
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

describe('LoginContent loading states', () => {
  beforeEach(() => {
    vi.resetAllMocks();
    pushMock.mockClear();
    server.use(
      rest.get(`*${ENDPOINT_USER_INFO}`, (req, res, ctx) => {
        return res(
          ctx.delay(100),
          ctx.status(200),
          ctx.json({ logged_in: false })
        );
      }),
      rest.get(`*${ENDPOINT_APP_INFO}`, (req, res, ctx) => {
        return res(
          ctx.delay(100),
          ctx.status(200),
          ctx.json({ status: 'ready', authz: true })
        );
      })
    );
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
      rest.get(`*${ENDPOINT_USER_INFO}`, (req, res, ctx) => {
        return res(ctx.status(200), ctx.json({ logged_in: false }));
      }),
      rest.get(`*${ENDPOINT_APP_INFO}`, (req, res, ctx) => {
        return res(ctx.status(200), ctx.json({ status: 'ready', authz: true }));
      })
    );
  });

  it('renders login button when user is not logged in', async () => {
    await act(async () => {
      render(<LoginContent />, { wrapper: createWrapper() });
    });
    const loginButton = screen.getByRole('button', { name: 'Login' });
    expect(loginButton).toBeDefined();
    expect(
      screen.getByText('Login to use the Bodhi App')
    ).toBeInTheDocument();
  });

  it('renders login button with correct styling', async () => {
    await act(async () => {
      render(<LoginContent />, { wrapper: createWrapper() });
    });
    const loginButton = screen.getByRole('button', { name: 'Login' });
    expect(loginButton).toHaveClass('w-full');
    expect(loginButton).not.toBeDisabled();
  });
});

describe('LoginContent with user Logged In', () => {
  beforeEach(() => {
    vi.resetAllMocks();
    pushMock.mockClear();
    server.use(
      rest.get(`*${ENDPOINT_USER_INFO}`, (req, res, ctx) => {
        return res(
          ctx.status(200),
          ctx.json({ logged_in: true, email: 'test@example.com' })
        );
      }),
      rest.get(`*${ENDPOINT_APP_INFO}`, (req, res, ctx) => {
        return res(ctx.status(200), ctx.json({ status: 'ready' }));
      })
    );
  });

  it('renders welcome message and logout button when user is logged in', async () => {
    await act(async () => {
      render(<LoginContent />, { wrapper: createWrapper() });
    });
    expect(
      screen.getByText('You are logged in as test@example.com')
    ).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Log Out' })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Go to Home' })).toBeInTheDocument();
  });

  it('calls logout function when logout button is clicked and pushes the route in location', async () => {
    server.use(
      rest.post(`*${ENDPOINT_LOGOUT}`, (req, res, ctx) => {
        return res(
          ctx.status(200),
          ctx.set('Location', 'http://localhost:1135/ui/test/login'),
          ctx.json({})
        );
      })
    );
    await act(async () => {
      render(<LoginContent />, { wrapper: createWrapper() });
    });
    const logoutButton = screen.getByRole('button', { name: 'Log Out' });
    await userEvent.click(logoutButton);
    expect(pushMock).toHaveBeenCalledWith(
      'http://localhost:1135/ui/test/login'
    );
  });

  it('disables logout button and shows loading text when logging out', async () => {
    server.use(
      rest.post(`*${ENDPOINT_LOGOUT}`, (_, res, ctx) => {
        return res(
          ctx.delay(100),
          ctx.status(200),
          ctx.set('Location', 'http://localhost:1135/ui/test/login'),
          ctx.json({})
        );
      })
    );

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

describe('LoginContent with non-authz mode', () => {
  beforeEach(() => {
    vi.resetAllMocks();
    pushMock.mockClear();
    server.use(
      rest.get(`*${ENDPOINT_USER_INFO}`, (req, res, ctx) => {
        return res(ctx.status(200), ctx.json({ logged_in: false }));
      }),
      rest.get(`*${ENDPOINT_APP_INFO}`, (req, res, ctx) => {
        return res(ctx.status(200), ctx.json({ status: 'ready', authz: false }));
      })
    );
  });

  it('should display non-authz warning and disable login button', async () => {
    await act(async () => {
      render(<LoginContent />, { wrapper: createWrapper() });
    });
    expect(screen.getByText('This app is setup in non-authenticated mode.User login is not available.')).toBeInTheDocument();
    const loginButton = screen.getByRole('button', { name: /login/i });
    expect(loginButton).toBeDisabled();
  });

  it('applies correct styling to disabled state', async () => {
    await act(async () => {
      render(<LoginContent />, { wrapper: createWrapper() });
    });
    const container = screen.getByRole('button', { name: /login/i }).closest('div');
    expect(container).toHaveClass('opacity-50 pointer-events-none');
  });
});

describe('LoginContent access control', () => {
  it('redirects to setup when app is not setup', async () => {
    server.use(
      rest.get(`*${ENDPOINT_APP_INFO}`, (_, res, ctx) => {
        return res(ctx.status(200), ctx.json({ status: 'setup' }));
      })
    );
    await act(async () => {
      render(<LoginPage />, { wrapper: createWrapper() });
    });
    await waitFor(() => expect(pushMock).toHaveBeenCalledWith('/ui/setup'));
  });
});
