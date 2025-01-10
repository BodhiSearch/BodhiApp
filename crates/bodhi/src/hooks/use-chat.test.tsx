import { renderHook, act } from '@testing-library/react';
import { rest } from 'msw';
import { setupServer } from 'msw/node';
import { useChat } from '@/hooks/use-chat';
import { beforeAll, afterAll, beforeEach, describe, expect, it, vi } from 'vitest';
import { Chat, Message } from '@/types/chat';
import { createWrapper } from '@/tests/wrapper';

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
        rest.post('*/v1/chat/completions', (req, res, ctx) => {
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

      const userMessage: Message = {
        id: '1',
        role: 'user',
        content: 'Hi there',
      };

      await act(async () => {
        await result.current.append(userMessage);
      });

      const savedChat = await chatDB.getChat('1');
      expect(savedChat?.messages).toHaveLength(2);
      expect(savedChat?.messages).toEqual([
        userMessage,
        { role: 'assistant', content: ' Hello world' },
      ]);
      expect(savedChat?.messages[1]).toEqual({
        role: 'assistant',
        content: ' Hello world',
      });
    });

    it('should handle reload functionality', async () => {
      const messages: Message[] = [
        { id: '1', role: 'user', content: 'First message' },
        { id: '2', role: 'assistant', content: 'First response' },
        { id: '3', role: 'user', content: 'Second message' },
        { id: '4', role: 'assistant', content: 'Second response' },
      ];

      const initialChat: Chat = {
        id: '1',
        title: 'Test Chat',
        messages,
        createdAt: Date.now(),
      };

      chatDB.setCurrentChat(initialChat);

      server.use(
        rest.post('*/v1/chat/completions', (req, res, ctx) => {
          return res(
            ctx.status(200),
            ctx.set('Content-Type', 'text/event-stream'),
            ctx.body('data: {"choices":[{"delta":{"content":"New response"}}]}\n\ndata: [DONE]\n\n')
          );
        })
      );

      const { result } = renderHook(() => useChat(), {
        wrapper: createWrapper()
      });

      await act(async () => {
        await result.current.reload();
      });

      const savedChat = await chatDB.getChat('1');
      expect(savedChat?.messages).toEqual([
        ...messages.slice(0, 3),
        { role: 'assistant', content: 'New response' },
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
        rest.post('*/v1/chat/completions', async (req, res, ctx) => {
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
});