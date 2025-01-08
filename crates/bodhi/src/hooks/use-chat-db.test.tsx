import { renderHook, act } from '@testing-library/react';
import { useChatDB, ChatDBProvider } from './use-chat-db';
import { beforeEach, describe, expect, it } from 'vitest';
import { Chat } from '@/types/chat';

describe('useChatDB', () => {
  beforeEach(() => {
    localStorage.clear();
  });

  const wrapper = ({ children }: { children: React.ReactNode }) => (
    <ChatDBProvider>{children}</ChatDBProvider>
  );

  describe('initialization', () => {
    it('should initialize with empty chats', async () => {
      const { result } = renderHook(() => useChatDB(), { wrapper });

      const response = result.current.chats;
      expect(response).toHaveLength(0);
    });

    it('should load persisted chats from localStorage', async () => {
      const mockChat = {
        id: '123',
        title: 'Test Chat',
        messages: [],
        createdAt: Date.now()
      };

      localStorage.setItem('chats', JSON.stringify([mockChat]));

      const { result } = renderHook(() => useChatDB(), { wrapper });

      let response;
      await act(async () => {
        response = await result.current.getChat('123');
      });

      expect(response!.status).toBe(200);
      expect(response!.data).toEqual(mockChat);
    });
  });

  describe('chat operations', () => {
    it('should create a new chat at the beginning of the list', async () => {
      const { result } = renderHook(() => useChatDB(), { wrapper });

      const chat1: Chat = {
        id: crypto.randomUUID(),
        title: 'Chat 1',
        messages: [],
        createdAt: Date.now()
      };

      const chat2: Chat = {
        id: crypto.randomUUID(),
        title: 'Chat 2',
        messages: [],
        createdAt: Date.now()
      };

      await act(async () => {
        await result.current.createOrUpdateChat(chat1);
        await result.current.createOrUpdateChat(chat2);
      });

      const response = result.current.chats;
      expect(response[0].id).toBe(chat2.id);
      expect(response[1].id).toBe(chat1.id);
    });

    it('should update existing chat and move to beginning', async () => {
      const { result } = renderHook(() => useChatDB(), { wrapper });

      const chat1: Chat = {
        id: crypto.randomUUID(),
        title: 'Chat 1',
        messages: [],
        createdAt: Date.now()
      };

      const chat2: Chat = {
        id: crypto.randomUUID(),
        title: 'Chat 2',
        messages: [],
        createdAt: Date.now()
      };

      await act(async () => {
        await result.current.createOrUpdateChat(chat1);
        await result.current.createOrUpdateChat(chat2);
      });

      const updatedChat1 = {
        ...chat1,
        messages: [{ id: '1', content: 'Hello', role: 'user' }]
      };

      await act(async () => {
        await result.current.createOrUpdateChat(updatedChat1 as Chat);
      });

      const response = result.current.chats;
      expect(response[0].id).toBe(chat1.id);
      expect(response[1].id).toBe(chat2.id);
      expect(response[0].messages).toEqual(updatedChat1.messages);
      expect(response[0].updatedAt).toBeDefined();
    });

    it('should delete a chat', async () => {
      const { result } = renderHook(() => useChatDB(), { wrapper });

      const chat: Chat = {
        id: crypto.randomUUID(),
        title: 'Test Chat',
        messages: [],
        createdAt: Date.now()
      };

      await act(async () => {
        await result.current.createOrUpdateChat(chat);
        await result.current.deleteChat(chat.id);
      });

      const response = await result.current.getChat(chat.id);
      expect(response.status).toBe(404);
    });

    it('should clear all chats', async () => {
      const { result } = renderHook(() => useChatDB(), { wrapper });

      const chats: Chat[] = Array.from({ length: 2 }, (_, i) => ({
        id: crypto.randomUUID(),
        title: `Chat ${i + 1}`,
        messages: [],
        createdAt: Date.now()
      }));

      await act(async () => {
        for (const chat of chats) {
          await result.current.createOrUpdateChat(chat);
        }
        await result.current.clearChats();
      });

      const response = result.current.chats;
      expect(response).toHaveLength(0);
    });

    it('should maintain chat order with most recent first', async () => {
      const { result } = renderHook(() => useChatDB(), { wrapper });

      // Create three chats
      const chats: Chat[] = Array.from({ length: 3 }, (_, i) => ({
        id: crypto.randomUUID(),
        title: `Chat ${i + 1}`,
        messages: [],
        createdAt: Date.now()
      }));

      await act(async () => {
        for (const chat of chats) {
          await result.current.createOrUpdateChat(chat);
        }
      });

      // Update middle chat
      const updatedChat = {
        ...chats[1],
        messages: [{ id: '1', content: 'test', role: 'user' }]
      };

      await act(async () => {
        await result.current.createOrUpdateChat(updatedChat as Chat);
      });

      const response = result.current.chats;
      expect(response.map(chat => chat.id)).toEqual([
        chats[1].id, // Updated chat
        chats[2].id, // Most recently created
        chats[0].id  // Oldest
      ]);
    });

    it('should preserve order after localStorage reload', async () => {
      const { result } = renderHook(() => useChatDB(), { wrapper });

      const chats: Chat[] = Array.from({ length: 3 }, (_, i) => ({
        id: crypto.randomUUID(),
        title: `Chat ${i + 1}`,
        messages: [],
        createdAt: Date.now()
      }));

      await act(async () => {
        for (const chat of chats) {
          await result.current.createOrUpdateChat(chat);
        }
      });

      // Force reload from localStorage
      const { result: reloadedResult } = renderHook(() => useChatDB(), { wrapper });

      const response = reloadedResult.current.chats;
      expect(response.map(chat => chat.id)).toEqual([
        chats[2].id, // Most recent first
        chats[1].id,
        chats[0].id
      ]);
    });

    it('should maintain maximum of 100 chats', async () => {
      const { result } = renderHook(() => useChatDB(), { wrapper });

      // Create 105 chats
      const chats: Chat[] = Array.from({ length: 105 }, (_, i) => ({
        id: crypto.randomUUID(),
        title: `Chat ${i + 1}`,
        messages: [],
        createdAt: Date.now()
      }));

      await act(async () => {
        for (const chat of chats) {
          await result.current.createOrUpdateChat(chat);
        }
      });

      const response = result.current.chats;
      expect(response).toHaveLength(100);
      // Should have the most recent 100 chats (last 100 from the array)
      expect(response.map(c => c.id)).toEqual(
        chats.slice(5).reverse().map(c => c.id)
      );
    });

    it('should not remove existing chat when updating if at max capacity', async () => {
      const { result } = renderHook(() => useChatDB(), { wrapper });

      // Create 100 chats
      const chats: Chat[] = Array.from({ length: 100 }, (_, i) => ({
        id: crypto.randomUUID(),
        title: `Chat ${i + 1}`,
        messages: [],
        createdAt: Date.now()
      }));

      await act(async () => {
        for (const chat of chats) {
          await result.current.createOrUpdateChat(chat);
        }
      });

      // Update the oldest chat (last in the list)
      const updatedChat = {
        ...chats[0],
        messages: [{ id: '1', content: 'test', role: 'user' }]
      };

      await act(async () => {
        await result.current.createOrUpdateChat(updatedChat as Chat);
      });

      const response = result.current.chats;
      expect(response).toHaveLength(100);
      expect(response[0].id).toBe(chats[0].id); // Updated chat should be first
      expect(response[0].messages).toEqual(updatedChat.messages);
    });
  });

  describe('chat listing', () => {
    it('should list all chats', async () => {
      const { result } = renderHook(() => useChatDB(), { wrapper });

      const chats: Chat[] = Array.from({ length: 5 }, (_, i) => ({
        id: crypto.randomUUID(),
        title: `Chat ${i + 1}`,
        messages: [],
        createdAt: Date.now()
      }));

      await act(async () => {
        for (const chat of chats) {
          await result.current.createOrUpdateChat(chat);
        }
      });

      const response = result.current.chats;
      expect(response).toHaveLength(5);
      expect(response.map(c => c.id)).toEqual(
        chats.reverse().map(c => c.id)
      );
    });
  });
});