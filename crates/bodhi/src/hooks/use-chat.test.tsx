import { renderHook, act } from '@testing-library/react';
import { ChatProvider, useChat } from './use-chat';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { Chat, Message } from '@/types/chat';

// Mock use-chat-settings
vi.mock('@/hooks/use-chat-settings', () => ({
  useChatSettings: () => ({
    getRequestSettings: () => ({
      model: 'test-model',
      temperature: 0.7,
    }),
  }),
}));

// Mock use-chat-completions
const mockAppend = vi.fn();
vi.mock('@/hooks/use-chat-completions', () => ({
  useChatCompletion: () => ({
    append: mockAppend,
    isLoading: false,
    error: null,
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
    vi.clearAllMocks();
    chatDB.clear();
  });

  const createWrapper = (chat: Chat) => ({ children }: { children: React.ReactNode }) => (
    <ChatProvider chat={chat}>{children}</ChatProvider>
  );

  describe('message handling', () => {
    it('should handle streaming response', async () => {
      const wrapper = createWrapper(initialChat);
      let streamCallback: (chunk: string) => void = () => { };
      let messageCallback: (message: Message) => void = () => { };

      mockAppend.mockImplementation(({ onDelta, onMessage }) => {
        onDelta('Hello');
        onDelta(' world');
        onMessage({
          role: 'assistant',
          content: 'Hello world',
        });
        return Promise.resolve();
      });

      const { result } = renderHook(() => useChat(), { wrapper });

      const userMessage: Message = {
        id: '1',
        role: 'user',
        content: 'Hi there',
      };

      await act(async () => {
        await result.current.append(userMessage);
      });

      // Wait for state updates to complete
      await act(async () => {
        await new Promise(resolve => setTimeout(resolve, 0));
      });

      expect(result.current.messages).toHaveLength(2);
      expect(result.current.messages[0]).toEqual(userMessage);
      expect(result.current.messages[1]).toEqual({
        role: 'assistant',
        content: 'Hello world',
      });

      // Verify chat was persisted
      const savedChat = await chatDB.getChat('1');
      expect(savedChat?.messages).toEqual(result.current.messages);
    });

    it('should handle reload with streaming', async () => {
      // First set up some initial messages
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

      const wrapper = createWrapper(chatWithMessages);
      const { result } = renderHook(() => useChat(), { wrapper });

      let capturedMessages: Message[] = [];

      // Mock the reload response with streaming
      mockAppend.mockImplementation(({ request, onDelta, onMessage }) => {
        capturedMessages = request.messages;
        onDelta('Reloaded');
        onDelta(' response');
        onMessage({
          role: 'assistant',
          content: 'Reloaded response',
        });
        return Promise.resolve();
      });

      // Verify initial state
      expect(result.current.messages).toEqual(messages);

      await act(async () => {
        await result.current.reload();
      });

      const messagesToKeep = messages.slice(0, 3);
      const expectedMessages = [
        ...messagesToKeep,
        { role: 'assistant', content: 'Reloaded response' },
      ];

      // Verify the messages sent in the request
      expect(capturedMessages).toEqual(messagesToKeep);

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

      await chatDB.createOrUpdateChat(chatWithMessages);

      const wrapper = createWrapper(chatWithMessages);
      const { result } = renderHook(() => useChat(), { wrapper });

      // Mock error
      mockAppend.mockRejectedValue(new Error('Reload failed'));
      const consoleSpy = vi.spyOn(console, 'error').mockImplementation(() => { });

      try {
        await act(async () => {
          await result.current.reload();
        });
      } catch (error) {
        // Error is expected
      }

      // Verify error was logged
      expect(consoleSpy).toHaveBeenCalledWith('Chat completion error:', expect.any(Error));

      consoleSpy.mockRestore();
    });
  });

  describe('error handling', () => {
    it('should handle API errors gracefully', async () => {
      const wrapper = createWrapper(initialChat);
      const { result } = renderHook(() => useChat(), { wrapper });
      const consoleSpy = vi.spyOn(console, 'error').mockImplementation(() => { });

      const userMessage: Message = {
        id: '1',
        role: 'user',
        content: 'Hi there',
      };

      // Mock the error
      mockAppend.mockRejectedValue(new Error('API Error'));

      try {
        await act(async () => {
          await result.current.append(userMessage);
        });
      } catch (error) {
        // Error is expected
      }

      expect(consoleSpy).toHaveBeenCalled();
      consoleSpy.mockRestore();
    });
  });
});