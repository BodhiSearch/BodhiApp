import { ChatDBProvider, useChatDB } from '@/hooks/use-chat-db';
import { Chat } from '@/types/chat';
import { act, renderHook } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';

// Mock nanoid to have predictable IDs
vi.mock('@/lib/utils', () => ({
  nanoid: () => 'test-id',
}));

describe('useChatDB', () => {
  beforeEach(() => {
    localStorage.clear();
    vi.clearAllMocks();
  });

  const wrapper = ({ children }: { children: React.ReactNode }) => (
    <ChatDBProvider>{children}</ChatDBProvider>
  );

  describe('initialization', () => {
    it('should initialize with empty state', () => {
      const { result } = renderHook(() => useChatDB(), { wrapper });

      expect(result.current.chats).toHaveLength(0);
      expect(result.current.currentChat).toBeNull();
      expect(result.current.currentChatId).toBeNull();
    });

    it('should initialize current chat when requested', async () => {
      const { result } = renderHook(() => useChatDB(), { wrapper });

      await act(async () => {
        await result.current.initializeCurrentChatId();
      });

      expect(result.current.currentChatId).toBe('test-id');
      expect(result.current.currentChat).toEqual({
        id: 'test-id',
        title: 'New Chat',
        messages: [],
        createdAt: expect.any(Number),
        updatedAt: expect.any(Number),
      });
    });
  });

  describe('chat management', () => {
    it('should create and update chats', async () => {
      const { result } = renderHook(() => useChatDB(), { wrapper });

      const chat: Chat = {
        id: '1',
        title: 'Test Chat',
        messages: [],
        createdAt: Date.now(),
      };

      await act(async () => {
        await result.current.createOrUpdateChat(chat);
      });

      expect(result.current.chats[0]).toEqual({
        ...chat,
        updatedAt: expect.any(Number),
      });
    });

    it('should handle creating new chat', async () => {
      const { result } = renderHook(() => useChatDB(), { wrapper });

      // First create a chat with messages
      const chat: Chat = {
        id: '1',
        title: 'Test Chat',
        messages: [{ role: 'user', content: 'test' }],
        createdAt: Date.now(),
      };

      await act(async () => {
        await result.current.createOrUpdateChat(chat);
        result.current.setCurrentChatId('1');
      });

      // Create new chat
      await act(async () => {
        await result.current.createNewChat();
      });

      expect(result.current.currentChatId).toBe('test-id');
      expect(result.current.currentChat?.messages).toHaveLength(0);
    });

    it('should not create new chat if current chat is empty', async () => {
      const { result } = renderHook(() => useChatDB(), { wrapper });

      const emptyChat: Chat = {
        id: '1',
        title: 'Empty Chat',
        messages: [],
        createdAt: Date.now(),
      };

      await act(async () => {
        await result.current.createOrUpdateChat(emptyChat);
        result.current.setCurrentChatId('1');
        await result.current.createNewChat();
      });

      expect(result.current.currentChatId).toBe('1');
      expect(result.current.chats).toHaveLength(1);
    });

    it('should handle chat deletion', async () => {
      const { result } = renderHook(() => useChatDB(), { wrapper });

      // Create two chats
      const chats: Chat[] = [
        {
          id: '1',
          title: 'Chat 1',
          messages: [{ role: 'user', content: 'test' }],
          createdAt: Date.now(),
        },
        {
          id: '2',
          title: 'Chat 2',
          messages: [],
          createdAt: Date.now(),
        },
      ];

      await act(async () => {
        for (const chat of chats) {
          await result.current.createOrUpdateChat(chat);
        }
        result.current.setCurrentChatId('1');
      });

      // Delete current chat
      await act(async () => {
        await result.current.deleteChat('1');
      });

      // Should switch to empty chat
      expect(result.current.currentChatId).toBe('2');
      expect(result.current.chats).toHaveLength(1);
    });

    it('should clear all chats', async () => {
      const { result } = renderHook(() => useChatDB(), { wrapper });

      const chat: Chat = {
        id: '1',
        title: 'Test Chat',
        messages: [],
        createdAt: Date.now(),
      };

      await act(async () => {
        await result.current.createOrUpdateChat(chat);
        result.current.setCurrentChatId('1');
        await result.current.clearChats();
      });

      expect(result.current.chats).toHaveLength(0);
      expect(result.current.currentChatId).toBeNull();
      expect(result.current.currentChat).toBeNull();
    });
  });
});