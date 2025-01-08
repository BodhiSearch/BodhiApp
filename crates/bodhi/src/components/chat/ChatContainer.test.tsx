import { render, screen, act, waitFor } from '@testing-library/react';
import { ChatContainer } from './ChatContainer';
import { useSearchParams, useRouter } from 'next/navigation';
import { useChatDB } from '@/hooks/use-chat-db';
import { nanoid } from '@/lib/utils';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { createWrapper } from '@/tests/wrapper';
import { ChatSettingsProvider } from '@/hooks/use-chat-settings';
import { useLocalStorage } from '@/hooks/useLocalStorage';
import { Chat } from '@/types/chat';
import { useToast } from '@/hooks/use-toast';

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

vi.mock('@/hooks/use-toast', () => ({
  useToast: vi.fn(),
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
  const mockToast = vi.fn();
  const Wrapper = createTestWrapper();

  beforeEach(() => {
    vi.clearAllMocks();

    // Mock scrollIntoView
    Element.prototype.scrollIntoView = vi.fn();

    // Mock matchMedia
    Object.defineProperty(window, 'matchMedia', {
      writable: true,
      value: vi.fn().mockImplementation(query => ({
        matches: false,
        media: query,
        onchange: null,
        addListener: vi.fn(), // Deprecated
        removeListener: vi.fn(), // Deprecated
        addEventListener: vi.fn(),
        removeEventListener: vi.fn(),
        dispatchEvent: vi.fn(),
      })),
    });

    // Setup default mocks
    vi.mocked(useRouter).mockReturnValue(mockRouter as any);
    vi.mocked(useChatDB).mockImplementation(() => ({
      getChat: mockGetChat,
      createOrUpdateChat: vi.fn(),
      deleteChat: vi.fn(),
      clearChats: vi.fn(),
      listChats: vi.fn()
    }));
    vi.mocked(nanoid).mockImplementation(() => 'mock-id');
    vi.mocked(useToast).mockReturnValue({ toast: mockToast } as any);
  });

  afterEach(() => {
    // Clean up all mocks
    vi.restoreAllMocks();
  });

  describe('New Chat Initialization', () => {
    it('should create a new chat when no id is provided and no current chat exists', async () => {
      // Mock empty search params and no current chat
      vi.mocked(useSearchParams).mockImplementation(() => ({
        get: () => null
      } as any));

      await act(async () => {
        render(<ChatContainer />, { wrapper: Wrapper });
      });

      // Should not redirect since this is a new chat
      expect(mockRouter.replace).not.toHaveBeenCalled();
      expect(mockRouter.push).not.toHaveBeenCalled();

      // Should render chat UI after initialization
      await waitFor(() => {
        expect(screen.getByTestId('chat-ui')).toBeInTheDocument();
      });
    });

    it('should redirect to existing chat URL if current chat has messages', async () => {
      // Mock empty search params but existing chat with messages
      (useSearchParams as any).mockImplementation(() => ({
        get: () => null
      }));

      // Mock existing chat in localStorage
      const existingChat: Chat = {
        id: 'existing-id',
        title: 'Existing Chat',
        messages: [{ role: 'user', content: 'test message' }],
        createdAt: Date.now(),
        updatedAt: Date.now(),
      };

      vi.mocked(useLocalStorage).mockReturnValue([existingChat, vi.fn()]);

      await act(async () => {
        render(<ChatContainer />, { wrapper: Wrapper });
      });

      // Should redirect to existing chat URL
      expect(mockRouter.replace).toHaveBeenCalledWith('/ui/chat/?id=existing-id');
    });
  });

  describe('Existing Chat Loading', () => {
    it('should load existing chat when valid id is provided', async () => {
      // Mock search params with existing chat ID
      (useSearchParams as any).mockImplementation(() => ({
        get: () => 'existing-chat-id'
      }));

      const mockChat: Chat = {
        id: 'existing-chat-id',
        title: 'Existing Chat',
        messages: [],
        createdAt: Date.now(),
        updatedAt: Date.now(),
      };

      mockGetChat.mockResolvedValue({
        status: 200,
        data: mockChat
      });

      await act(async () => {
        render(<ChatContainer />, { wrapper: Wrapper });
      });

      expect(mockGetChat).toHaveBeenCalledWith('existing-chat-id');

      await waitFor(() => {
        expect(screen.getByTestId('chat-ui')).toBeInTheDocument();
      });

      // Should not redirect
      expect(mockRouter.replace).not.toHaveBeenCalled();
      expect(mockRouter.push).not.toHaveBeenCalled();
    });

    it('should redirect to chat page and show toast when chat is not found', async () => {
      vi.mocked(useSearchParams).mockImplementation(() => ({
        get: () => 'non-existent-id'
      } as any));

      vi.mocked(useChatDB).mockImplementation(() => ({
        getChat: vi.fn().mockResolvedValue({ status: 404 }),
        createOrUpdateChat: vi.fn(),
        deleteChat: vi.fn(),
        clearChats: vi.fn(),
        listChats: vi.fn()
      }));

      render(<ChatContainer />, { wrapper: Wrapper });

      await waitFor(() => {
        expect(mockToast).toHaveBeenCalledWith({
          variant: "destructive",
          title: "Chat not found",
          description: "The requested chat could not be found.",
        });
        expect(mockRouter.push).toHaveBeenCalledWith('/ui/chat');
      });
    });

    it('should handle chat loading errors gracefully', async () => {
      const mockError = new Error('Failed to load');

      vi.mocked(useSearchParams).mockImplementation(() => ({
        get: () => 'error-id'
      } as any));

      vi.mocked(useChatDB).mockImplementation(() => ({
        getChat: vi.fn().mockRejectedValue(mockError),
        createOrUpdateChat: vi.fn(),
        deleteChat: vi.fn(),
        clearChats: vi.fn(),
        listChats: vi.fn()
      }));

      render(<ChatContainer />, { wrapper: Wrapper });

      await waitFor(() => {
        expect(mockToast).toHaveBeenCalledWith({
          variant: "destructive",
          title: "Error loading chat",
          description: "Failed to load the requested chat. Please try again.",
        });
        expect(mockRouter.push).toHaveBeenCalledWith('/ui/chat');
      });
    });
  });
});