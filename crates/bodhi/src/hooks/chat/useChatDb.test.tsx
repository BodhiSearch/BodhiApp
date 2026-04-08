import { ChatDBProvider, useChatDB } from '@/hooks/chat';
import { chatDb } from '@/lib/chatDb';
import { Chat } from '@/types/chat';
import { act, renderHook, waitFor } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';

vi.mock('@/lib/utils', () => ({
  nanoid: () => 'test-id',
}));

describe('useChatDB', () => {
  beforeEach(async () => {
    localStorage.clear();
    vi.clearAllMocks();
    await chatDb.chats.clear();
    await chatDb.messages.clear();
  });

  const wrapper = ({ children }: { children: React.ReactNode }) => (
    <ChatDBProvider userId="user-1">{children}</ChatDBProvider>
  );

  describe('initialization', () => {
    it('should initialize with empty state', () => {
      const { result } = renderHook(() => useChatDB(), { wrapper });

      expect(result.current.chats).toHaveLength(0);
      expect(result.current.currentChat).toBeNull();
      expect(result.current.currentChatId).toBeNull();
    });
  });

  describe('chat management', () => {
    it('should create and update chats', async () => {
      const { result } = renderHook(() => useChatDB(), { wrapper });

      const chat: Chat = {
        id: '1',
        title: 'Test Chat',
        messages: [],
        messageCount: 0,
        createdAt: Date.now(),
      };

      await act(async () => {
        await result.current.createOrUpdateChat(chat);
      });

      await waitFor(() => {
        expect(result.current.chats).toHaveLength(1);
      });

      expect(result.current.chats[0]).toEqual(
        expect.objectContaining({
          id: '1',
          title: 'Test Chat',
          messages: [],
          createdAt: chat.createdAt,
          updatedAt: expect.any(Number),
        })
      );
    });

    it('should handle creating new chat', async () => {
      const { result } = renderHook(() => useChatDB(), { wrapper });

      const chat: Chat = {
        id: '1',
        title: 'Test Chat',
        messages: [{ role: 'user', content: 'test' }],
        messageCount: 1,
        createdAt: Date.now(),
      };

      await act(async () => {
        await result.current.createOrUpdateChat(chat);
      });

      await waitFor(() => {
        expect(result.current.chats).toHaveLength(1);
      });

      await act(async () => {
        result.current.setCurrentChatId('1');
      });

      await act(async () => {
        await result.current.createNewChat();
      });

      await waitFor(() => {
        expect(result.current.currentChatId).toBe('test-id');
      });

      expect(result.current.currentChat?.messages).toHaveLength(0);
    });

    it('should not create new chat if current chat is empty', async () => {
      const { result } = renderHook(() => useChatDB(), { wrapper });

      const emptyChat: Chat = {
        id: '1',
        title: 'Empty Chat',
        messages: [],
        messageCount: 0,
        createdAt: Date.now(),
      };

      await act(async () => {
        await result.current.createOrUpdateChat(emptyChat);
      });

      await waitFor(() => {
        expect(result.current.chats).toHaveLength(1);
      });

      await act(async () => {
        result.current.setCurrentChatId('1');
      });

      await act(async () => {
        await result.current.createNewChat();
      });

      expect(result.current.currentChatId).toBe('1');
      expect(result.current.chats).toHaveLength(1);
    });

    it('should handle chat deletion', async () => {
      const { result } = renderHook(() => useChatDB(), { wrapper });

      const chats: Chat[] = [
        {
          id: '1',
          title: 'Chat 1',
          messages: [{ role: 'user', content: 'test' }],
          messageCount: 1,
          createdAt: Date.now(),
        },
        {
          id: '2',
          title: 'Chat 2',
          messages: [],
          messageCount: 0,
          createdAt: Date.now(),
        },
      ];

      await act(async () => {
        for (const chat of chats) {
          await result.current.createOrUpdateChat(chat);
        }
      });

      await waitFor(() => {
        expect(result.current.chats).toHaveLength(2);
      });

      await act(async () => {
        result.current.setCurrentChatId('1');
      });

      await act(async () => {
        await result.current.deleteChat('1');
      });

      await waitFor(() => {
        expect(result.current.currentChatId).toBe('2');
        expect(result.current.chats).toHaveLength(1);
      });
    });

    it('should clear all chats', async () => {
      const { result } = renderHook(() => useChatDB(), { wrapper });

      const chat: Chat = {
        id: '1',
        title: 'Test Chat',
        messages: [],
        messageCount: 0,
        createdAt: Date.now(),
      };

      await act(async () => {
        await result.current.createOrUpdateChat(chat);
      });

      await waitFor(() => {
        expect(result.current.chats).toHaveLength(1);
      });

      await act(async () => {
        result.current.setCurrentChatId('1');
      });

      await act(async () => {
        await result.current.clearChats();
      });

      await waitFor(() => {
        expect(result.current.chats).toHaveLength(0);
      });

      expect(result.current.currentChatId).toBeNull();
      expect(result.current.currentChat).toBeNull();
    });

    it('should isolate chats by userId', async () => {
      const user1Wrapper = ({ children }: { children: React.ReactNode }) => (
        <ChatDBProvider userId="user-1">{children}</ChatDBProvider>
      );
      const user2Wrapper = ({ children }: { children: React.ReactNode }) => (
        <ChatDBProvider userId="user-2">{children}</ChatDBProvider>
      );

      const { result: result1 } = renderHook(() => useChatDB(), { wrapper: user1Wrapper });

      await act(async () => {
        await result1.current.createOrUpdateChat({
          id: 'chat-u1',
          title: 'User 1 Chat',
          messages: [{ role: 'user', content: 'hello from user 1' }],
          messageCount: 1,
          createdAt: Date.now(),
        });
      });

      await waitFor(() => {
        expect(result1.current.chats).toHaveLength(1);
      });

      const { result: result2 } = renderHook(() => useChatDB(), { wrapper: user2Wrapper });

      await waitFor(() => {
        expect(result2.current.chats).toHaveLength(0);
      });
    });

    it('should persist messages with chat', async () => {
      const { result } = renderHook(() => useChatDB(), { wrapper });

      const chat: Chat = {
        id: 'msg-test',
        title: 'Message Test',
        messages: [
          { role: 'user', content: 'hello' },
          { role: 'assistant', content: 'hi there' },
        ],
        messageCount: 2,
        createdAt: Date.now(),
      };

      await act(async () => {
        await result.current.createOrUpdateChat(chat);
      });

      await waitFor(() => {
        expect(result.current.chats).toHaveLength(1);
      });

      expect(result.current.chats[0].messages).toHaveLength(2);
      expect(result.current.chats[0].messages[0]).toEqual({ role: 'user', content: 'hello' });
      expect(result.current.chats[0].messages[1]).toEqual({ role: 'assistant', content: 'hi there' });

      const getResult = await result.current.getChat('msg-test');
      expect(getResult.status).toBe(200);
      expect(getResult.data.messages).toHaveLength(2);
    });
  });
});
