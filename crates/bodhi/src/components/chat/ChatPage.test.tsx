import ChatPage from '@/components/chat/ChatPage';
import { ENDPOINT_APP_INFO, ENDPOINT_USER_INFO, ENDPOINT_MODELS } from '@/hooks/useQuery';
import { createWrapper } from '@/tests/wrapper';
import { act, render, screen, waitFor } from '@testing-library/react';
import { rest } from 'msw';
import { setupServer } from 'msw/node';
import userEvent from '@testing-library/user-event';
import { MemoryRouter } from 'react-router-dom';
import {
  afterAll,
  beforeAll,
  beforeEach,
  describe,
  expect,
  it,
  vi,
} from 'vitest';

// Mock the components
vi.mock('@/components/chat/ChatContainer', () => ({
  ChatContainer: () => <div data-testid="chat-container">Chat Content</div>
}));

// Mock use-mobile hook
vi.mock('@/hooks/use-mobile', () => ({
  useMobile: () => ({ isMobile: false }),
  useIsMobile: () => false,
}));

const pushMock = vi.fn();
const mockSearchParams = {
  get: vi.fn(),
  set: vi.fn(),
  delete: vi.fn(),
};

vi.mock('@/lib/navigation', () => ({
  useRouter: () => ({
    push: pushMock,
  }),
  useSearchParams: () => mockSearchParams,
}));

const toastMock = vi.fn();
vi.mock('@/hooks/use-toast', () => ({
  useToast: () => ({
    toast: toastMock,
  }),
}));

const server = setupServer();

beforeAll(() => server.listen());
afterAll(() => server.close());
beforeEach(() => {
  server.resetHandlers();
  pushMock.mockClear();
  toastMock.mockClear();
  mockSearchParams.get.mockClear();
  mockSearchParams.set.mockClear();
  mockSearchParams.delete.mockClear();
});
afterEach(() => {
  vi.resetAllMocks();
});

describe('ChatPage', () => {
  it('redirects to /ui/setup if status is setup', async () => {
    server.use(
      rest.get(`*${ENDPOINT_APP_INFO}`, (_, res, ctx) => {
        return res(ctx.json({ status: 'setup' }));
      })
    );
    server.use(
      rest.get(`*${ENDPOINT_USER_INFO}`, (_, res, ctx) => {
        return res(ctx.json({ logged_in: true, email: 'test@example.com' }));
      })
    );

    await act(async () => {
      render(<ChatPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(pushMock).toHaveBeenCalledWith('/ui/setup');
    });
  });

  it('redirects to /ui/login if user is not logged in', async () => {
    server.use(
      rest.get(`*${ENDPOINT_APP_INFO}`, (_, res, ctx) => {
        return res(ctx.json({ status: 'ready' }));
      }),
      rest.get(`*${ENDPOINT_USER_INFO}`, (_, res, ctx) => {
        return res(ctx.json({ logged_in: false }));
      })
    );

    render(<ChatPage />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(pushMock).toHaveBeenCalledWith('/ui/login');
    });
  });

  describe('URL Synchronization', () => {
    beforeEach(() => {
      // Mock successful app status and user login for URL sync tests
      server.use(
        rest.get(`*${ENDPOINT_APP_INFO}`, (_, res, ctx) => {
          return res(ctx.json({ status: 'ready' }));
        }),
        rest.get(`*${ENDPOINT_USER_INFO}`, (_, res, ctx) => {
          return res(ctx.json({ logged_in: true, email: 'test@example.com' }));
        }),
        rest.get(`*${ENDPOINT_MODELS}`, (_, res, ctx) => {
          return res(ctx.json([]));
        })
      );
    });

    it('reads chat ID from URL parameter', async () => {
      mockSearchParams.get.mockImplementation((key) => {
        if (key === 'id') return 'test-chat-123';
        return null;
      });

      await act(async () => {
        render(<ChatPage />, { wrapper: createWrapper() });
      });

      expect(mockSearchParams.get).toHaveBeenCalledWith('id');
    });

    it('reads alias parameter from URL', async () => {
      mockSearchParams.get.mockImplementation((key) => {
        if (key === 'alias') return 'llama3';
        return null;
      });

      await act(async () => {
        render(<ChatPage />, { wrapper: createWrapper() });
      });

      expect(mockSearchParams.get).toHaveBeenCalledWith('alias');
    });

    it('handles URL without parameters', async () => {
      mockSearchParams.get.mockReturnValue(null);

      await act(async () => {
        render(<ChatPage />, { wrapper: createWrapper() });
      });

      // Should not throw errors and render successfully
      expect(mockSearchParams.get).toHaveBeenCalled();
    });
  });
});
