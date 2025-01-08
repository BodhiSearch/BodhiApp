import { render, screen, act, waitFor } from '@testing-library/react';
import { ChatContainer } from './ChatContainer';
import { useSearchParams, useRouter } from 'next/navigation';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { createWrapper } from '@/tests/wrapper';
import { Chat } from '@/types/chat';
import { useLocalStorage } from '@/hooks/useLocalStorage';

// Consolidate all mocks
const mockRouter = { push: vi.fn(), replace: vi.fn() };
const mockGetChat = vi.fn();
const mockToast = vi.fn();
const mockSetLocalStorage = vi.fn();

// Setup all mocks
vi.mock('next/navigation', () => ({
  useSearchParams: vi.fn(),
  useRouter: vi.fn(() => mockRouter)
}));

vi.mock('@/hooks/use-chat-db', () => ({
  useChatDB: vi.fn(() => ({
    getChat: mockGetChat,
    createOrUpdateChat: vi.fn(),
    deleteChat: vi.fn(),
    clearChats: vi.fn(),
    chats: []
  })),
  ChatDBProvider: ({ children }: { children: React.ReactNode }) => children
}));

vi.mock('@/lib/utils', () => ({
  nanoid: vi.fn(() => 'mock-id'),
  cn: (...inputs: any) => inputs.filter(Boolean).join(' ')
}));

vi.mock('@/hooks/useLocalStorage', () => ({
  useLocalStorage: vi.fn(() => [null, mockSetLocalStorage])
}));

vi.mock('@/hooks/use-toast', () => ({
  useToast: vi.fn(() => ({ toast: mockToast }))
}));

vi.mock('@/components/layout/MainLayout', () => ({
  MainLayout: ({ children }: { children: React.ReactNode }) => children
}));

// Simplified wrapper
const Wrapper = createWrapper();

describe('ChatContainer', () => {
  beforeEach(() => {
    vi.clearAllMocks();

    // Mock scrollIntoView
    Element.prototype.scrollIntoView = vi.fn();

    // Mock matchMedia (often needed with scroll behavior)
    Object.defineProperty(window, 'matchMedia', {
      writable: true,
      value: vi.fn().mockImplementation(query => ({
        matches: false,
        media: query,
        onchange: null,
        addEventListener: vi.fn(),
        removeEventListener: vi.fn(),
        dispatchEvent: vi.fn(),
      })),
    });
  });

  describe('New Chat Initialization', () => {
    it('creates new chat when no id or current chat exists', async () => {
      vi.mocked(useSearchParams).mockImplementation(() => ({
        get: () => null
      } as any));

      await act(async () => {
        render(<ChatContainer />, { wrapper: Wrapper });
      });

      expect(mockRouter.push).not.toHaveBeenCalled();
      expect(screen.getByText('Welcome to Chat')).toBeInTheDocument();
    });

    it('redirects to existing chat URL if current chat has messages', async () => {
      vi.mocked(useSearchParams).mockImplementation(() => ({
        get: () => null
      } as any));

      const existingChat: Chat = {
        id: 'existing-id',
        title: 'Existing Chat',
        messages: [{ role: 'user', content: 'test message' }],
        createdAt: Date.now(),
        updatedAt: Date.now(),
      };

      vi.mocked(useLocalStorage).mockReturnValue([existingChat, mockSetLocalStorage]);

      await act(async () => {
        render(<ChatContainer />, { wrapper: Wrapper });
      });

      expect(mockRouter.replace).toHaveBeenCalledWith('/ui/chat/?id=existing-id');
    });
  });

  describe('Existing Chat Loading', () => {
    it('loads existing chat when valid id is provided', async () => {
      vi.mocked(useSearchParams).mockImplementation(() => ({
        get: () => 'existing-chat-id'
      } as any));

      mockGetChat.mockResolvedValue({
        status: 200,
        data: {
          id: 'existing-chat-id',
          title: 'Existing Chat',
          messages: [],
          createdAt: Date.now(),
          updatedAt: Date.now(),
        }
      });

      await act(async () => {
        render(<ChatContainer />, { wrapper: Wrapper });
      });

      expect(mockGetChat).toHaveBeenCalledWith('existing-chat-id');
    });

    it('shows error toast and redirects when chat not found', async () => {
      vi.mocked(useSearchParams).mockImplementation(() => ({
        get: () => 'non-existent-id'
      } as any));

      mockGetChat.mockResolvedValue({ status: 404 });

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

    it('handles loading errors gracefully', async () => {
      vi.mocked(useSearchParams).mockImplementation(() => ({
        get: () => 'error-id'
      } as any));

      mockGetChat.mockRejectedValue(new Error('Failed to load'));

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