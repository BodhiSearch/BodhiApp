import { createWrapper } from '@/tests/wrapper';
import { render, screen, waitFor } from '@testing-library/react';
import { rest } from 'msw';
import { setupServer } from 'msw/node';
import {
  afterAll,
  beforeAll,
  beforeEach,
  describe,
  expect,
  it,
  vi,
} from 'vitest';
import ChatPage from '@/app/ui/chat/page';

// Mock the components
vi.mock('@/components/layout/MainLayout', () => ({
  MainLayout: ({ children }: { children: React.ReactNode }) => (
    <div data-testid="main-layout">{children}</div>
  ),
}));

vi.mock('@/components/chat/ChatContainer', () => ({
  ChatContainer: () => <div data-testid="chat-container">Chat Content</div>
}));

const pushMock = vi.fn();
vi.mock('next/navigation', () => ({
  useRouter: () => ({
    push: pushMock,
  }),
}));

const server = setupServer();

beforeAll(() => server.listen());
afterAll(() => server.close());
beforeEach(() => {
  server.resetHandlers();
  pushMock.mockClear();
});

describe('ChatPage', () => {
  it('redirects to /ui/setup if status is setup', async () => {
    server.use(
      rest.get('*/app/info', (_, res, ctx) => {
        return res(ctx.json({ status: 'setup' }));
      })
    );
    server.use(
      rest.get('*/api/ui/user', (_, res, ctx) => {
        return res(ctx.json({ logged_in: true, email: 'test@example.com' }));
      })
    );

    render(<ChatPage />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(pushMock).toHaveBeenCalledWith('/ui/setup');
    });
  });

  it('redirects to /ui/login if user is not logged in', async () => {
    server.use(
      rest.get('*/app/info', (_, res, ctx) => {
        return res(ctx.json({ status: 'ready', authz: true }));
      }),
      rest.get('*/api/ui/user', (_, res, ctx) => {
        return res(ctx.json({ logged_in: false }));
      })
    );

    render(<ChatPage />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(pushMock).toHaveBeenCalledWith('/ui/login');
    });
  });

  it('renders chat page when app is ready and user is logged in', async () => {
    server.use(
      rest.get('*/app/info', (_, res, ctx) => {
        return res(ctx.json({ status: 'ready', authz: true }));
      }),
      rest.get('*/api/ui/user', (_, res, ctx) => {
        return res(ctx.json({ logged_in: true, email: 'test@example.com' }));
      })
    );

    render(<ChatPage />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(screen.getByTestId('chat-container')).toBeInTheDocument();
    });
  });
});