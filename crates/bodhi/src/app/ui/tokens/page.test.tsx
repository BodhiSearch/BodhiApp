import TokenPage, { TokenPageContent } from '@/app/ui/tokens/page';
import { CREATE_TOKEN_ENDPOINT } from '@/hooks/useCreateToken';
import { ENDPOINT_APP_INFO, ENDPOINT_USER_INFO } from '@/hooks/useQuery';
import { createWrapper } from '@/tests/wrapper';
import { act, render, screen } from '@testing-library/react';
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

const pushMock = vi.fn();
vi.mock('next/navigation', () => ({
  useRouter: () => ({
    push: pushMock,
  }),
}));

const mockTokenResponse = {
  offline_token: 'test-token-123',
  name: 'Test Token',
  status: 'active',
  created_at: '2024-01-01T00:00:00Z',
  updated_at: '2024-01-01T00:00:00Z',
};

const server = setupServer();

beforeAll(() => server.listen());
afterAll(() => server.close());
afterEach(() => server.resetHandlers());
beforeEach(() => {
  vi.resetAllMocks();
  pushMock.mockClear();
});

describe('TokenPageContent', () => {
  it('shows loading skeleton initially', async () => {
    server.use(
      rest.get('*/app/info', (_, res, ctx) => {
        return res(ctx.delay(100), ctx.json({ status: 'ready', authz: true }));
      })
    );

    render(<TokenPageContent />, { wrapper: createWrapper() });

    expect(screen.getByTestId('token-page-loading')).toBeInTheDocument();
  });
});

describe('TokenPageContent', () => {
  beforeEach(() => {
    server.use(
      rest.get(`*${ENDPOINT_APP_INFO}`, (_, res, ctx) => {
        return res(ctx.json({ status: 'ready', authz: true }));
      }),
      rest.get(`*${ENDPOINT_USER_INFO}`, (_, res, ctx) => {
        return res(ctx.json({ logged_in: true, email: 'test@example.com' }));
      })
    );
  });

  it('renders authenticated view with form and security warning', async () => {
    await act(async () => {
      render(<TokenPage />, { wrapper: createWrapper() });
    });

    // Check title and description
    expect(screen.getByText(/API Tokens/)).toBeInTheDocument();
    expect(screen.getByText(/Generate and manage API tokens/)).toBeInTheDocument();

    // Check security warning
    expect(screen.getByText(/API tokens provide full access to the API/)).toBeInTheDocument();
    expect(screen.getByText(/Keep them secure and never share them/)).toBeInTheDocument();
    expect(screen.getByText(/Tokens cannot be viewed again/)).toBeInTheDocument();

    // Check form is rendered
    expect(screen.getByLabelText('Token Name (Optional)')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Generate Token' })).toBeInTheDocument();
  });

  it('handles complete token creation flow', async () => {
    const user = userEvent.setup();
    server.use(
      rest.post(`*${CREATE_TOKEN_ENDPOINT}`, (_, res, ctx) => {
        return res(ctx.status(201), ctx.json(mockTokenResponse));
      })
    );

    await act(async () => {
      render(<TokenPage />, { wrapper: createWrapper() });
    });

    // Fill and submit form
    await user.type(screen.getByLabelText('Token Name (Optional)'), 'Test Token');
    await user.click(screen.getByRole('button', { name: 'Generate Token' }));

    // Check token dialog appears
    expect(await screen.findByText('API Token Generated')).toBeInTheDocument();

    // Close dialog
    await user.click(screen.getByRole('button', { name: 'Done' }));
    expect(screen.queryByText('API Token Generated')).not.toBeInTheDocument();
  });

  it('shows non-authenticated setup message in card layout', async () => {
    server.use(
      rest.get('*/app/info', (_, res, ctx) => {
        return res(ctx.json({ status: 'ready', authz: false }));
      })
    );

    await act(async () => {
      render(<TokenPage />, { wrapper: createWrapper() });
    });

    // Check for card title and icon
    expect(screen.getByText(/API Tokens Not Supported/)).toBeInTheDocument();

    // Check for description message
    const description = screen.getByText((content) => {
      return content.includes("Non-authenticated setup doesn't need API Tokens") &&
        content.includes("Either ignore the Auth header or pass an empty/random Bearer token") &&
        content.includes("They are not validated");
    });
    expect(description).toBeInTheDocument();
  });
});

describe('TokenPage', () => {
  it('redirects to login when not authenticated', async () => {
    server.use(
      rest.get(`*${ENDPOINT_APP_INFO}`, (_, res, ctx) => {
        return res(ctx.json({ status: 'ready', authz: true }));
      }),
      rest.get(`*${ENDPOINT_USER_INFO}`, (_, res, ctx) => {
        return res(ctx.json({ logged_in: false }));
      })
    );

    await act(async () => {
      render(<TokenPage />, { wrapper: createWrapper() });
    });

    expect(pushMock).toHaveBeenCalledWith('/ui/login');
  });
});
