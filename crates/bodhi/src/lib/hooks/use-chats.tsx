'use client';

import { createContext, useContext, useCallback, useState, useEffect } from 'react';
import { Chat } from '@/types/chat';

const CHATS_STORAGE_KEY = 'chats';
const MAX_CHATS = 100;

interface ChatsContextType {
  getChat: (id: string) => Promise<{ data: Chat; status: number }>;
  createOrUpdateChat: (chat: Chat) => Promise<string>;
  deleteChat: (id: string) => Promise<void>;
  clearChats: () => Promise<void>;
  listChats: () => Promise<Chat[]>;
}

const ChatsContext = createContext<ChatsContextType | undefined>(undefined);

export function ChatsProvider({ children }: { children: React.ReactNode }) {
  const [chats, setChats] = useState<Chat[]>(() => {
    if (typeof window !== 'undefined') {
      const saved = localStorage.getItem(CHATS_STORAGE_KEY);
      if (saved) {
        try {
          const savedChats = JSON.parse(saved);
          // Ensure we only load up to MAX_CHATS
          return savedChats.slice(0, MAX_CHATS);
        } catch (e) {
          console.warn('Failed to parse chats from localStorage:', e);
          return [];
        }
      }
      return [];
    }
    return [];
  });

  useEffect(() => {
    localStorage.setItem(CHATS_STORAGE_KEY, JSON.stringify(chats));
  }, [chats]);

  const getChat = useCallback(async (id: string) => {
    const chat = chats.find(c => c.id === id);
    if (!chat) {
      return { data: {} as Chat, status: 404 };
    }
    return { data: chat, status: 200 };
  }, [chats]);

  const createOrUpdateChat = useCallback(async (chat: Chat) => {
    setChats(prev => {
      const index = prev.findIndex(c => c.id === chat.id);
      const updatedChat = {
        ...chat,
        updatedAt: Date.now()
      };

      let newChats: Chat[];
      if (index === -1) {
        // Create new chat at the beginning
        newChats = [updatedChat, ...prev];
        // If we exceed MAX_CHATS, remove the oldest chat
        if (newChats.length > MAX_CHATS) {
          newChats = newChats.slice(0, MAX_CHATS);
        }
      } else {
        // Update existing chat and move to front
        newChats = [
          updatedChat,
          ...prev.slice(0, index),
          ...prev.slice(index + 1)
        ];
      }
      return newChats;
    });

    return chat.id;
  }, []);

  const deleteChat = useCallback(async (id: string) => {
    setChats(prev => prev.filter(c => c.id !== id));
  }, []);

  const clearChats = useCallback(async () => {
    setChats([]);
  }, []);

  const listChats = useCallback(async () => {
    return chats;
  }, [chats]);

  return (
    <ChatsContext.Provider
      value={{
        getChat,
        createOrUpdateChat,
        deleteChat,
        clearChats,
        listChats
      }}
    >
      {children}
    </ChatsContext.Provider>
  );
}

export function useChats() {
  const context = useContext(ChatsContext);
  if (context === undefined) {
    throw new Error('useChats must be used within a ChatsProvider');
  }
  return context;
}