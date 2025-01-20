import { useChat } from '@/hooks/use-chat';
import { ENDPOINT_OAI_CHAT_COMPLETIONS } from '@/hooks/useQuery';
import { createWrapper } from '@/tests/wrapper';
import { Chat, Message } from '@/types/chat';
import { act, renderHook } from '@testing-library/react';
import { rest } from 'msw';
import { setupServer } from 'msw/node';
import { afterAll, beforeAll, beforeEach, describe, expect, it, vi } from 'vitest';

// Setup MSW server
const server = setupServer();

beforeAll(() => server.listen());
afterAll(() => server.close());
beforeEach(() => {
  server.resetHandlers();
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
        '[DONE]'
      ];

      server.use(
        rest.post(`*${ENDPOINT_OAI_CHAT_COMPLETIONS}`, (req, res, ctx) => {
          return res(
            ctx.status(200),
            ctx.set('Content-Type', 'text/event-stream'),
            ctx.body(chunks.map(chunk => `data: ${chunk}\n\n`).join(''))
          );
        })
      );

      const { result } = renderHook(() => useChat(), {
        wrapper: createWrapper()
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
        rest.post(`*${ENDPOINT_OAI_CHAT_COMPLETIONS}`, async (req, res, ctx) => {
          capturedRequest = await req.json();
          return res(
            ctx.status(200),
            ctx.set('Content-Type', 'text/event-stream'),
            ctx.body('data: {"choices":[{"delta":{"content":"Response"}}]}\n\ndata: [DONE]\n\n')
          );
        })
      );

      const { result } = renderHook(() => useChat(), {
        wrapper: createWrapper()
      });

      await act(async () => {
        await result.current.append({
          id: '1',
          role: 'user',
          content: 'Hello',
        });
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
        rest.post(`*${ENDPOINT_OAI_CHAT_COMPLETIONS}`, (req, res, ctx) => {
          return res(
            ctx.status(500),
            ctx.json({
              error: {
                message: 'Invalid API key provided',
                type: 'invalid_request_error',
              }
            })
          );
        })
      );

      const { result } = renderHook(() => useChat(), {
        wrapper: createWrapper()
      });

      await act(async () => {
        try {
          await result.current.append('Hello');
        } catch (error) {
        }
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
      server.use(
        rest.post(`*${ENDPOINT_OAI_CHAT_COMPLETIONS}`, (req, res) => {
          return res.networkError('Failed to connect');
        })
      );

      const { result } = renderHook(() => useChat(), {
        wrapper: createWrapper()
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
  });
});