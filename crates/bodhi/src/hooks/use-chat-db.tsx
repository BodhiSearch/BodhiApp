'use client';

import { createContext, useContext, useCallback, useState, useEffect } from 'react';
import { Chat } from '@/types/chat';
import { useLocalStorage } from '@/hooks/useLocalStorage';
import { nanoid } from '@/lib/utils';

const CHATS_STORAGE_KEY = 'chats';
const CURRENT_CHAT_ID_KEY = 'current-chat-id';
const MAX_CHATS = 100;

interface ChatDBContextType {
  chats: Chat[];
  currentChat: Chat | null;
  currentChatId: string | null;
  setCurrentChatId: (id: string | null) => void;
  createNewChat: () => Promise<void>;
  getChat: (id: string) => Promise<{ data: Chat; status: number }>;
  createOrUpdateChat: (chat: Chat) => Promise<string>;
  deleteChat: (id: string) => Promise<void>;
  clearChats: () => Promise<void>;
}

const ChatDBContext = createContext<ChatDBContextType | undefined>(undefined);

export function ChatDBProvider({ children }: { children: React.ReactNode }) {
  const [chats, setChats] = useState<Chat[]>(() => {
    if (typeof window !== 'undefined') {
      const saved = localStorage.getItem(CHATS_STORAGE_KEY);
      if (saved) {
        try {
          const savedChats = JSON.parse(saved);
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

  const [currentChatId, setCurrentChatId] = useLocalStorage<string | null>(CURRENT_CHAT_ID_KEY, null);

  const currentChat = currentChatId ? (chats.find((chat) => chat.id === currentChatId) ?? null) : null;

  useEffect(() => {
    localStorage.setItem(CHATS_STORAGE_KEY, JSON.stringify(chats));
  }, [chats]);

  const getChat = useCallback(
    async (id: string) => {
      const chat = chats.find((c) => c.id === id);
      if (!chat) {
        return { data: {} as Chat, status: 404 };
      }
      return { data: chat, status: 200 };
    },
    [chats]
  );

  const createOrUpdateChat = useCallback(async (chat: Chat) => {
    setChats((prev) => {
      const index = prev.findIndex((c) => c.id === chat.id);
      const updatedChat = {
        ...chat,
        updatedAt: Date.now(),
      };

      let newChats: Chat[];
      if (index === -1) {
        newChats = [updatedChat, ...prev];
        if (newChats.length > MAX_CHATS) {
          newChats = newChats.slice(0, MAX_CHATS);
        }
      } else {
        newChats = [updatedChat, ...prev.slice(0, index), ...prev.slice(index + 1)];
      }
      return newChats;
    });

    return chat.id;
  }, []);

  const deleteChat = useCallback(
    async (id: string) => {
      if (currentChatId !== id) {
        // If not current chat, just delete it
        setChats((prev) => prev.filter((c) => c.id !== id));
        return;
      }

      // If it's current chat, try to find an empty chat first
      setChats((prev) => {
        const emptyChat = prev.find((chat) => chat.id !== id && chat.messages.length === 0);

        if (emptyChat) {
          // Found empty chat - delete current and switch to empty
          setCurrentChatId(emptyChat.id);
          return prev.filter((c) => c.id !== id);
        }

        // No empty chat found - reset current chat instead of deleting
        return prev.map((chat) => {
          if (chat.id === id) {
            return {
              ...chat,
              title: 'New Chat',
              messages: [],
              updatedAt: Date.now(),
            };
          }
          return chat;
        });
      });
    },
    [currentChatId, setCurrentChatId]
  );

  const clearChats = useCallback(async () => {
    setChats([]);
    setCurrentChatId(null);
  }, [setCurrentChatId]);

  const createNewChat = useCallback(async () => {
    // Don't create new if current chat is empty
    if (!currentChat || currentChat.messages.length === 0) {
      return;
    }

    // Try to find an empty chat
    const emptyChat = chats.find((chat) => chat.messages.length === 0);

    if (emptyChat) {
      // Update the empty chat's timestamp
      const updatedChat: Chat = {
        ...emptyChat,
        updatedAt: Date.now(),
      };

      await createOrUpdateChat(updatedChat);
      setCurrentChatId(emptyChat.id);
      return;
    }

    // Create new chat if no empty chat found
    const newChat: Chat = {
      id: nanoid(),
      title: 'New Chat',
      messages: [],
      createdAt: Date.now(),
      updatedAt: Date.now(),
    };

    await createOrUpdateChat(newChat);
    setCurrentChatId(newChat.id);
  }, [currentChat, chats, createOrUpdateChat, setCurrentChatId]);

  return (
    <ChatDBContext.Provider
      value={{
        chats,
        currentChat,
        currentChatId,
        setCurrentChatId,
        createNewChat,
        getChat,
        createOrUpdateChat,
        deleteChat,
        clearChats,
      }}
    >
      {children}
    </ChatDBContext.Provider>
  );
}

export function useChatDB() {
  const context = useContext(ChatDBContext);
  if (context === undefined) {
    throw new Error('useChatDB must be used within a ChatDBProvider');
  }
  return context;
}
