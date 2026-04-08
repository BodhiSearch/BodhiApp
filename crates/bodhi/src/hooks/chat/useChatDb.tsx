import { createContext, useContext, useCallback, useState, useEffect } from 'react';

import { chatDb, ChatRecord, MessageRecord } from '@/lib/chatDb';
import { nanoid } from '@/lib/utils';
import { Chat, Message } from '@/types/chat';

const MAX_CHATS = 1000;

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

function chatRecordToChat(record: ChatRecord, messages: Message[]): Chat {
  return {
    id: record.id,
    title: record.title,
    messages,
    createdAt: record.createdAt,
    updatedAt: record.updatedAt,
    model: record.model,
    enabledMcpTools: record.enabledMcpTools,
  };
}

function chatToChatRecord(chat: Chat, userId: string): ChatRecord {
  return {
    id: chat.id,
    userId,
    title: chat.title,
    model: chat.model,
    createdAt: chat.createdAt,
    updatedAt: chat.updatedAt,
    enabledMcpTools: chat.enabledMcpTools,
  };
}

function messagesToRecords(chatId: string, messages: Message[]): MessageRecord[] {
  const now = Date.now();
  return messages.map((message, index) => ({
    chatId,
    message,
    createdAt: now + index,
  }));
}

export function ChatDBProvider({ children, userId = 'default' }: { children: React.ReactNode; userId?: string }) {
  const storageKey = `current-chat-id:${userId}`;
  const [chats, setChats] = useState<Chat[]>([]);
  const [currentChatId, setCurrentChatIdState] = useState<string | null>(() => {
    if (typeof window !== 'undefined') {
      const saved = localStorage.getItem(storageKey);
      if (saved) {
        try {
          return JSON.parse(saved);
        } catch {
          return null;
        }
      }
    }
    return null;
  });

  const setCurrentChatId = useCallback((id: string | null) => {
    setCurrentChatIdState(id);
    if (typeof window !== 'undefined') {
      if (id === null) {
        localStorage.removeItem(storageKey);
      } else {
        localStorage.setItem(storageKey, JSON.stringify(id));
      }
    }
  }, [storageKey]);

  const loadChats = useCallback(async () => {
    const records = await chatDb.chats.where('userId').equals(userId).reverse().sortBy('updatedAt');

    const chatList: Chat[] = [];
    for (const record of records.slice(0, MAX_CHATS)) {
      const msgRecords = await chatDb.messages.where('chatId').equals(record.id).sortBy('createdAt');
      chatList.push(
        chatRecordToChat(
          record,
          msgRecords.map((r) => r.message)
        )
      );
    }
    setChats(chatList);
  }, [userId]);

  useEffect(() => {
    loadChats();
  }, [loadChats]);

  const currentChat = currentChatId ? (chats.find((chat) => chat.id === currentChatId) ?? null) : null;

  const getChat = useCallback(
    async (id: string) => {
      const record = await chatDb.chats.get(id);
      if (!record || record.userId !== userId) {
        return { data: {} as Chat, status: 404 };
      }
      const msgRecords = await chatDb.messages.where('chatId').equals(id).sortBy('createdAt');
      return {
        data: chatRecordToChat(
          record,
          msgRecords.map((r) => r.message)
        ),
        status: 200,
      };
    },
    [userId]
  );

  const createOrUpdateChat = useCallback(
    async (chat: Chat) => {
      const updatedChat = { ...chat, updatedAt: Date.now() };
      const record = chatToChatRecord(updatedChat, userId);

      await chatDb.transaction('rw', chatDb.chats, chatDb.messages, async () => {
        await chatDb.chats.put(record);
        await chatDb.messages.where('chatId').equals(chat.id).delete();
        if (updatedChat.messages.length > 0) {
          await chatDb.messages.bulkAdd(messagesToRecords(chat.id, updatedChat.messages));
        }
      });

      const count = await chatDb.chats.where('userId').equals(userId).count();
      if (count > MAX_CHATS) {
        const oldest = await chatDb.chats.where('userId').equals(userId).sortBy('updatedAt');
        const toDelete = oldest.slice(0, count - MAX_CHATS);
        for (const old of toDelete) {
          await chatDb.messages.where('chatId').equals(old.id).delete();
          await chatDb.chats.delete(old.id);
        }
      }

      await loadChats();
      return chat.id;
    },
    [userId, loadChats]
  );

  const deleteChat = useCallback(
    async (id: string) => {
      if (currentChatId !== id) {
        await chatDb.transaction('rw', chatDb.chats, chatDb.messages, async () => {
          await chatDb.messages.where('chatId').equals(id).delete();
          await chatDb.chats.delete(id);
        });
        await loadChats();
        return;
      }

      const emptyChat = chats.find((chat) => chat.id !== id && chat.messages.length === 0);

      if (emptyChat) {
        setCurrentChatId(emptyChat.id);
        await chatDb.transaction('rw', chatDb.chats, chatDb.messages, async () => {
          await chatDb.messages.where('chatId').equals(id).delete();
          await chatDb.chats.delete(id);
        });
        await loadChats();
        return;
      }

      await chatDb.transaction('rw', chatDb.chats, chatDb.messages, async () => {
        await chatDb.messages.where('chatId').equals(id).delete();
        await chatDb.chats.update(id, {
          title: 'New Chat',
          updatedAt: Date.now(),
        });
      });
      await loadChats();
    },
    [currentChatId, chats, setCurrentChatId, loadChats]
  );

  const clearChats = useCallback(async () => {
    await chatDb.transaction('rw', chatDb.chats, chatDb.messages, async () => {
      const userChats = await chatDb.chats.where('userId').equals(userId).toArray();
      for (const chat of userChats) {
        await chatDb.messages.where('chatId').equals(chat.id).delete();
      }
      await chatDb.chats.where('userId').equals(userId).delete();
    });
    setCurrentChatId(null);
    setChats([]);
  }, [userId, setCurrentChatId]);

  const createNewChat = useCallback(async () => {
    if (!currentChat || currentChat.messages.length === 0) {
      return;
    }

    const emptyChat = chats.find((chat) => chat.messages.length === 0);

    if (emptyChat) {
      const updatedChat: Chat = {
        ...emptyChat,
        updatedAt: Date.now(),
      };
      await createOrUpdateChat(updatedChat);
      setCurrentChatId(emptyChat.id);
      return;
    }

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
