import { renderHook, act } from '@testing-library/react';
import { rest } from 'msw';
import { setupServer } from 'msw/node';
import { ChatProvider, useChat } from './use-chat';
import { beforeAll, afterAll, beforeEach, describe, expect, it, vi } from 'vitest';
import { Chat, Message } from '@/types/chat';
import { createWrapper as createQueryWrapper } from '@/tests/wrapper';

// Setup MSW server
const server = setupServer();

beforeAll(() => server.listen());
afterAll(() => server.close());
beforeEach(() => {
  server.resetHandlers();
});

// Mock use-chat-settings
vi.mock('@/hooks/use-chat-settings', () => ({
  useChatSettings: () => ({
    getRequestSettings: () => ({
      model: 'test-model',
      temperature: 0.7,
    }),
  }),
}));

// In-memory chat storage
class InMemoryChatDB {
  private chats: Map<string, Chat> = new Map();

  async createOrUpdateChat(chat: Chat): Promise<void> {
    this.chats.set(chat.id, { ...chat });
  }

  async getChat(id: string): Promise<Chat | null> {
    return this.chats.get(id) || null;
  }

  clear() {
    this.chats.clear();
  }
}

const chatDB = new InMemoryChatDB();

// Mock use-chat-db with in-memory implementation
vi.mock('@/hooks/use-chat-db', () => ({
  useChatDB: () => ({
    createOrUpdateChat: (chat: Chat) => chatDB.createOrUpdateChat(chat),
    getChat: (id: string) => chatDB.getChat(id),
  }),
}));

describe('useChat', () => {
  const initialChat: Chat = {
    id: '1',
    title: 'Test Chat',
    messages: [],
    createdAt: Date.now(),
  };

  beforeEach(() => {
    chatDB.clear();
  });

  const createWrapper = (chat: Chat = initialChat) => {
    const QueryWrapper = createQueryWrapper();
    return ({ children }: { children: React.ReactNode }) => (
      <QueryWrapper>
        <ChatProvider chat={chat}>{children}</ChatProvider>
      </QueryWrapper>
    );
  };

  describe('message handling', () => {
    it('should handle streaming response', async () => {
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

      const { result } = renderHook(() => useChat(), { wrapper: createWrapper() });

      const userMessage: Message = {
        id: '1',
        role: 'user',
        content: 'Hi there',
      };

      await act(async () => {
        await result.current.append(userMessage);
      });

      expect(result.current.messages).toHaveLength(2);
      expect(result.current.messages[0]).toEqual(userMessage);
      expect(result.current.messages[1]).toEqual({
        role: 'assistant',
        content: ' Hello world',
      });

      // Verify chat was persisted
      const savedChat = await chatDB.getChat('1');
      expect(savedChat?.messages).toEqual(result.current.messages);
    });

    it('should handle reload with streaming', async () => {
      const messages: Message[] = [
        { id: '1', role: 'user', content: 'First message' },
        { id: '2', role: 'assistant', content: 'First response' },
        { id: '3', role: 'user', content: 'Second message' },
        { id: '4', role: 'assistant', content: 'Second response' },
      ];

      const chatWithMessages = {
        ...initialChat,
        messages,
      };

      await chatDB.createOrUpdateChat(chatWithMessages);

      const chunks = [
        '{"choices":[{"delta":{"content":" Reloaded"}}]}',
        '{"choices":[{"delta":{"content":" response"}}]}',
        '[DONE]'
      ];

      server.use(
        rest.post('*/v1/chat/completions', async (req, res, ctx) => {
          // Parse request body properly
          const body = await req.json();

          // Verify the request contains the correct messages
          expect(body.messages).toEqual(messages.slice(0, 3));

          return res(
            ctx.status(200),
            ctx.set('Content-Type', 'text/event-stream'),
            ctx.body(chunks.map(chunk => `data: ${chunk}\n\n`).join(''))
          );
        })
      );

      const { result } = renderHook(() => useChat(), {
        wrapper: createWrapper(chatWithMessages)
      });

      // Verify initial state
      expect(result.current.messages).toEqual(messages);

      await act(async () => {
        await result.current.reload();
      });

      const expectedMessages = [
        ...messages.slice(0, 3),
        { role: 'assistant', content: ' Reloaded response' },
      ];

      // Verify final state
      expect(result.current.messages).toEqual(expectedMessages);

      // Verify persistence
      const savedChat = await chatDB.getChat('1');
      expect(savedChat?.messages).toEqual(expectedMessages);
    });

    it('should handle reload error', async () => {
      const messages: Message[] = [
        { id: '1', role: 'user', content: 'First message' },
        { id: '2', role: 'assistant', content: 'First response' },
      ];

      const chatWithMessages = {
        ...initialChat,
        messages,
      };

      server.use(
        rest.post('*/v1/chat/completions', (req, res, ctx) => {
          return res(ctx.status(500));
        })
      );

      const wrapper = createWrapper(chatWithMessages);
      const { result } = renderHook(() => useChat(), { wrapper });
      const consoleSpy = vi.spyOn(console, 'error').mockImplementation(() => { });

      try {
        await act(async () => {
          await result.current.reload();
        });
      } catch (error) {
        // Error is expected
      }

      expect(consoleSpy).toHaveBeenCalled();
      consoleSpy.mockRestore();
    });
  });
});