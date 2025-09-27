import ChatPage from '@/app/ui/chat/page';
import { createWrapper } from '@/tests/wrapper';
import { act, render, screen, waitFor } from '@testing-library/react';
import { setupMswV2, server } from '@/test-utils/msw-v2/setup';
import { mockAppInfoSetup, mockAppInfoReady } from '@/test-utils/msw-v2/handlers/info';
import { mockUserLoggedIn, mockUserLoggedOut } from '@/test-utils/msw-v2/handlers/user';
import userEvent from '@testing-library/user-event';
import { afterAll, beforeAll, beforeEach, describe, expect, it, vi } from 'vitest';

// Mock the components
vi.mock('@/components/chat/ChatContainer', () => ({
  ChatContainer: () => <div data-testid="chat-container">Chat Content</div>,
}));

// Mock use-mobile hook
vi.mock('@/hooks/use-mobile', () => ({
  useMobile: () => ({ isMobile: false }),
}));

const pushMock = vi.fn();
vi.mock('next/navigation', () => ({
  useRouter: () => ({
    push: pushMock,
  }),
}));

const toastMock = vi.fn();
vi.mock('@/hooks/use-toast', () => ({
  useToast: () => ({
    toast: toastMock,
  }),
}));

setupMswV2();

beforeEach(() => {
  pushMock.mockClear();
  toastMock.mockClear();
});
afterEach(() => {
  vi.resetAllMocks();
});

describe('ChatPage', () => {
  it('redirects to /ui/setup if status is setup', async () => {
    server.use(...mockAppInfoSetup());
    server.use(...mockUserLoggedIn());

    await act(async () => {
      render(<ChatPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(pushMock).toHaveBeenCalledWith('/ui/setup');
    });
  });

  it('redirects to /ui/login if user is not logged in', async () => {
    server.use(...mockAppInfoReady(), ...mockUserLoggedOut());

    render(<ChatPage />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(pushMock).toHaveBeenCalledWith('/ui/login');
    });
  });
});
