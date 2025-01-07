import { render, screen, act } from '@testing-library/react';
import { ChatContainer } from './ChatContainer';
import { useSearchParams, useRouter } from 'next/navigation';
import { useChatDB } from '@/hooks/use-chat-db';
import { nanoid } from '@/lib/utils';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { createWrapper } from '@/tests/wrapper';
import { ChatSettingsProvider } from '@/hooks/use-chat-settings';

// Mock dependencies
vi.mock('next/navigation', () => ({
  useSearchParams: vi.fn(),
  useRouter: vi.fn(() => ({
    push: vi.fn(),
    replace: vi.fn()
  }))
}));

vi.mock('@/hooks/use-chat-db', () => ({
  useChatDB: vi.fn()
}));

// Mock utils with cn function
vi.mock('@/lib/utils', () => ({
  nanoid: vi.fn(),
  cn: (...inputs: any) => inputs.filter(Boolean).join(' ')
}));

vi.mock('@/hooks/useLocalStorage', () => ({
  useLocalStorage: vi.fn(() => [true, vi.fn()])
}));

// Create a wrapper that combines QueryClient and ChatSettings providers
function createTestWrapper() {
  const QueryWrapper = createWrapper();

  return ({ children }: { children: React.ReactNode }) => (
    <QueryWrapper>
      <ChatSettingsProvider>
        {children}
      </ChatSettingsProvider>
    </QueryWrapper>
  );
}

describe('ChatContainer', () => {
  const mockRouter = {
    push: vi.fn(),
    replace: vi.fn()
  };

  const mockGetChat = vi.fn();
  const Wrapper = createTestWrapper();

  beforeEach(() => {
    vi.clearAllMocks();

    // Setup default mocks
    (useRouter as any).mockImplementation(() => mockRouter);
    (useChatDB as any).mockImplementation(() => ({
      getChat: mockGetChat
    }));
    (nanoid as any).mockImplementation(() => 'mock-id');
  });

  describe('New Chat Initialization', () => {
    it('should create a new chat when no id is provided', async () => {
      // Mock empty search params
      (useSearchParams as any).mockImplementation(() => ({
        get: () => null
      }));

      let container;
      await act(async () => {
        container = render(<ChatContainer />, { wrapper: Wrapper });
      });

      // Should redirect to new chat URL using replace
      expect(mockRouter.replace).toHaveBeenCalledWith('/ui/chat/?id=mock-id');
      expect(mockRouter.push).not.toHaveBeenCalled();

      // Should render chat UI after initialization
      expect(screen.getByTestId('chat-ui')).toBeInTheDocument();
    });
  });

  describe('Existing Chat Loading', () => {
    it('should load existing chat when valid id is provided', async () => {
      // Mock search params with existing chat ID
      (useSearchParams as any).mockImplementation(() => ({
        get: () => 'existing-chat-id'
      }));

      // Mock successful chat fetch
      const mockChat = {
        id: 'existing-chat-id',
        title: 'Existing Chat',
        messages: [],
        createdAt: Date.now(),
        updatedAt: Date.now()
      };

      mockGetChat.mockResolvedValue({
        status: 200,
        data: mockChat
      });

      await act(async () => {
        render(<ChatContainer />, { wrapper: Wrapper });
      });

      // Should attempt to load the chat
      expect(mockGetChat).toHaveBeenCalledWith('existing-chat-id');

      // Should render chat UI after initialization
      expect(screen.getByTestId('chat-ui')).toBeInTheDocument();

      // Should not redirect
      expect(mockRouter.replace).not.toHaveBeenCalled();
      expect(mockRouter.push).not.toHaveBeenCalled();
    });

    it('should create new chat in memory when chat fetch fails', async () => {
      // Mock search params with non-existent chat ID
      (useSearchParams as any).mockImplementation(() => ({
        get: () => 'non-existent-id'
      }));

      // Mock failed chat fetch
      mockGetChat.mockResolvedValue({
        status: 404
      });

      await act(async () => {
        render(<ChatContainer />, { wrapper: Wrapper });
      });

      // Should attempt to load the chat
      expect(mockGetChat).toHaveBeenCalledWith('non-existent-id');

      // Should render chat UI with new chat after initialization
      expect(screen.getByTestId('chat-ui')).toBeInTheDocument();

      // Should not redirect
      expect(mockRouter.replace).not.toHaveBeenCalled();
      expect(mockRouter.push).not.toHaveBeenCalled();
    });

    it('should handle chat loading errors gracefully', async () => {
      // Mock search params with chat ID
      (useSearchParams as any).mockImplementation(() => ({
        get: () => 'error-chat-id'
      }));

      // Mock error during chat fetch
      mockGetChat.mockRejectedValue(new Error('Failed to load chat'));

      // Spy on console.error
      const consoleSpy = vi.spyOn(console, 'error');

      await act(async () => {
        render(<ChatContainer />, { wrapper: Wrapper });
      });

      // Should log error
      expect(consoleSpy).toHaveBeenCalledWith(
        'Failed to load chat:',
        expect.any(Error)
      );

      // Should render chat UI after initialization
      expect(screen.getByTestId('chat-ui')).toBeInTheDocument();

      // Should not redirect
      expect(mockRouter.replace).not.toHaveBeenCalled();
      expect(mockRouter.push).not.toHaveBeenCalled();
    });
  });

  describe('Settings Sidebar', () => {
    it('should render settings sidebar with correct initial state', async () => {
      (useSearchParams as any).mockImplementation(() => ({
        get: () => null
      }));

      await act(async () => {
        render(<ChatContainer />, { wrapper: Wrapper });
      });

      // Should render settings toggle button
      expect(screen.getByRole('button', { name: /toggle settings/i })).toBeInTheDocument();

      // Should render settings sidebar
      expect(screen.getByTestId('settings-sidebar')).toBeInTheDocument();
    });
  });
});