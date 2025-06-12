import ChatPage from './page';
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
vi.mock('./ChatUI', () => ({
  ChatUI: () => <div data-testid="chat-ui">Chat UI</div>
}));

vi.mock('./ChatHistory', () => ({
  ChatHistory: () => <div data-testid="chat-history">Chat History</div>
}));

vi.mock('./NewChatButton', () => ({
  NewChatButton: () => <div data-testid="new-chat-button">New Chat</div>
}));

vi.mock('./settings/SettingsSidebar', () => ({
  SettingsSidebar: () => <div data-testid="settings-sidebar">Settings</div>
}));

// Mock use-mobile hook
vi.mock('@/hooks/use-mobile', () => ({
  useMobile: () => ({ isMobile: false }),
  useIsMobile: () => false,
}));

// Mock matchMedia for media query hook
Object.defineProperty(window, 'matchMedia', {
  writable: true,
  value: vi.fn().mockImplementation(query => ({
    matches: false,
    media: query,
    onchange: null,
    addListener: vi.fn(), // deprecated
    removeListener: vi.fn(), // deprecated
    addEventListener: vi.fn(),
    removeEventListener: vi.fn(),
    dispatchEvent: vi.fn(),
  })),
});

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

// Mock chat database hook
const mockChatDB = {
  currentChatId: null,
  setCurrentChatId: vi.fn(),
  chats: [],
};

vi.mock('@/hooks/use-chat-db', () => ({
  ChatDBProvider: ({ children }: { children: React.ReactNode }) => <div>{children}</div>,
  useChatDB: () => mockChatDB,
}));

// Mock chat settings hook
vi.mock('@/hooks/use-chat-settings', () => ({
  ChatSettingsProvider: ({ children }: { children: React.ReactNode }) => <div>{children}</div>,
}));

// Mock local storage hook
vi.mock('@/hooks/useLocalStorage', () => ({
  useLocalStorage: (key: string, defaultValue: any) => [defaultValue, vi.fn()],
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
  mockChatDB.setCurrentChatId.mockClear();
  mockChatDB.currentChatId = null;
  mockChatDB.chats = [];
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
