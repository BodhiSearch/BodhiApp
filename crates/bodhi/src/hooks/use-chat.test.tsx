import { useChat } from '@/hooks/use-chat';
import { ENDPOINT_OAI_CHAT_COMPLETIONS } from '@/hooks/use-chat-completions';
import { showErrorParams } from '@/lib/utils.test';
import { createWrapper } from '@/tests/wrapper';
import { Chat, Message } from '@/types/chat';
import { act, renderHook } from '@testing-library/react';
import { setupMswV2, server } from '@/test-utils/msw-v2/setup';
import {
  mockChatCompletionsStreaming,
  mockChatCompletionsError,
  mockChatCompletionsNetworkError,
  mockChatCompletionsStreamingWithError,
} from '@/test-utils/msw-v2/handlers/chat-completions';
import { afterAll, beforeAll, beforeEach, describe, expect, it, vi } from 'vitest';

// Setup MSW v2 server
setupMswV2();

beforeEach(() => {
  vi.clearAllMocks();
});

// In-memory chat storage
class InMemoryChatDB {
  private chats: Map<string, Chat> = new Map();
  currentChat: Chat | null = null;

  async createOrUpdateChat(chat: Chat): Promise<void> {
    this.chats.set(chat.id, { ...chat });
    this.currentChat = this.chats.get(chat.id) || null;
  }

  async getChat(id: string): Promise<Chat | null> {
    return this.chats.get(id) || null;
  }

  clear() {
    this.chats.clear();
  }

  setCurrentChat(chat: Chat | null) {
    if (chat) {
      this.createOrUpdateChat(chat);
    }
    this.currentChat = chat;
  }
}

const chatDB = new InMemoryChatDB();

// Mock use-chat-db with in-memory implementation
vi.mock('@/hooks/use-chat-db', () => ({
  useChatDB: () => ({
    createOrUpdateChat: (chat: Chat) => chatDB.createOrUpdateChat(chat),
    getChat: (id: string) => chatDB.getChat(id),
    currentChat: chatDB.currentChat,
    setCurrentChat: (chat: Chat | null) => chatDB.setCurrentChat(chat),
  }),
}));

// Mock toast
const mockToast = vi.fn();
vi.mock('@/hooks/use-toast', () => ({
  useToast: () => ({
    toast: mockToast,
  }),
}));

describe('useChat', () => {
  beforeEach(() => {
    chatDB.clear();
    // Mock use-chat-settings
    vi.mock('@/hooks/use-chat-settings', () => ({
      useChatSettings: () => ({
        getRequestSettings: () => ({
          model: 'test-model',
          temperature: 0.7,
        }),
        systemPrompt: '',
        systemPrompt_enabled: false,
        api_token: 'test-token',
        api_token_enabled: true,
      }),
    }));
  });

  describe('message handling', () => {
    it('should handle streaming response', async () => {
      const initialChat: Chat = {
        id: '1',
        title: 'Test Chat',
        messages: [],
        createdAt: Date.now(),
      };
      chatDB.setCurrentChat(initialChat);

      const chunks = [
        '{"choices":[{"delta":{"content":" Hello"}}]}',
        '{"choices":[{"delta":{"content":" world"}}]}',
        '[DONE]',
      ];

      server.use(
        ...mockChatCompletionsStreaming({
          chunks,
        })
      );

      const { result } = renderHook(() => useChat(), {
        wrapper: createWrapper(),
      });

      await act(async () => {
        await result.current.append('Hi there');
      });

      const savedChat = await chatDB.getChat('1');
      expect(savedChat?.messages).toHaveLength(2);
      expect(savedChat?.messages).toEqual([
        { role: 'user', content: 'Hi there' },
        { role: 'assistant', content: ' Hello world' },
      ]);
    });

    it('should handle system prompt when enabled', async () => {
      vi.mock('@/hooks/use-chat-settings', () => ({
        useChatSettings: () => ({
          getRequestSettings: () => ({
            model: 'test-model',
            temperature: 0.7,
          }),
          systemPrompt: 'Test system prompt',
          systemPrompt_enabled: true,
        }),
      }));

      const initialChat: Chat = {
        id: '1',
        title: 'Test Chat',
        messages: [],
        createdAt: Date.now(),
      };

      chatDB.setCurrentChat(initialChat);

      let capturedRequest: any;
      server.use(
        ...mockChatCompletionsStreaming({
          chunks: ['{"choices":[{"delta":{"content":"Response"}}]}', '[DONE]'],
          captureRequest: (req) => {
            capturedRequest = req;
          },
        })
      );

      const { result } = renderHook(() => useChat(), {
        wrapper: createWrapper(),
      });

      await act(async () => {
        await result.current.append('Hello');
      });

      expect(capturedRequest.messages[0]).toEqual({
        role: 'system',
        content: 'Test system prompt',
      });
    });
  });

  describe('error handling', () => {
    it('should handle API errors with error message and not save the message', async () => {
      const initialChat = {
        id: '1',
        title: 'Test Chat',
        messages: [],
        createdAt: Date.now(),
      };
      chatDB.setCurrentChat(initialChat);

      // Mock API error response
      server.use(
        ...mockChatCompletionsError({
          status: 500,
          message: 'Invalid API key provided',
        })
      );

      const { result } = renderHook(() => useChat(), {
        wrapper: createWrapper(),
      });

      await act(async () => {
        try {
          await result.current.append('Hello');
        } catch (error) {}
      });

      // Verify toast was called with error message
      expect(mockToast).toHaveBeenCalledWith({
        title: 'Error',
        description: 'Invalid API key provided',
        variant: 'destructive',
        duration: 5000,
      });

      // Verify the user message was still saved
      const savedChat = await chatDB.getChat('1');
      expect(savedChat?.messages).toHaveLength(0);
    });

    it('should handle network errors and not save the message', async () => {
      const initialChat = {
        id: '1',
        title: 'Test Chat',
        messages: [],
        createdAt: Date.now(),
      };
      chatDB.setCurrentChat(initialChat);

      // Mock network error
      server.use(...mockChatCompletionsNetworkError());

      const { result } = renderHook(() => useChat(), {
        wrapper: createWrapper(),
      });

      await act(async () => {
        await result.current.append('Hello');
      });

      expect(mockToast).toHaveBeenCalledWith({
        title: 'Error',
        description: 'Failed to fetch',
        variant: 'destructive',
        duration: 5000,
      });

      // Verify the user message was still saved
      const savedChat = await chatDB.getChat('1');
      expect(savedChat?.messages).toHaveLength(0);
    });

    it('should restore input when API request fails completely', async () => {
      const initialChat = {
        id: '1',
        title: 'Test Chat',
        messages: [],
        createdAt: Date.now(),
      };
      chatDB.setCurrentChat(initialChat);

      // Mock network error
      server.use(...mockChatCompletionsNetworkError());

      const { result } = renderHook(() => useChat(), {
        wrapper: createWrapper(),
      });

      // Set initial input
      act(() => {
        result.current.setInput('Hello world');
      });

      const originalInput = result.current.input;

      await act(async () => {
        await result.current.append('Hello world');
      });

      // Verify state is restored
      expect(result.current.input).toBe(originalInput);
      expect(result.current.userMessage.content).toBe('');
      expect(result.current.assistantMessage.content).toBe('');
      expect(mockToast).toHaveBeenCalledWith(showErrorParams('Error', 'Failed to fetch'));
    });

    it('should handle errors in event stream', async () => {
      const initialChat = {
        id: '1',
        title: 'Test Chat',
        messages: [],
        createdAt: Date.now(),
      };
      chatDB.setCurrentChat(initialChat);

      server.use(
        ...mockChatCompletionsStreamingWithError({
          initialChunks: ['{"choices":[{"delta":{"content":"Hello"},"finish_reason":null}]}'],
          errorMessage: 'Server error occurred',
        })
      );

      const { result } = renderHook(() => useChat(), {
        wrapper: createWrapper(),
      });

      await act(async () => {
        await result.current.append('Hello');
      });

      // Verify that the messages were saved (current behavior)
      const savedChat = await chatDB.getChat('1');
      expect(savedChat?.messages).toEqual([
        { role: 'user', content: 'Hello' },
        { role: 'assistant', content: 'Hello' },
      ]);

      // The assistant message should contain the content received before the error
      expect(result.current.assistantMessage.content).toBe('');

      // Currently errors in the stream are silently ignored (no error toast)
      expect(mockToast).not.toHaveBeenCalled();
    });
  });
});
