import { create } from 'zustand';

import { chatDb, ChatRecord, MessageRecord, PersistedChatSettings } from '@/lib/chatDb';
import { nanoid } from '@/lib/utils';
import { Chat, Message } from '@/types/chat';

const MAX_CHATS = 1000;
const CURRENT_CHAT_ID_KEY = 'current-chat-id';

function chatRecordToChat(record: ChatRecord, messages: Message[] = [], messageCount?: number): Chat {
  return {
    id: record.id,
    title: record.title,
    messages,
    messageCount: messageCount ?? record.messageCount ?? messages.length,
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
    messageCount: chat.messages.length,
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

function loadCurrentChatId(userId: string): string | null {
  if (typeof window === 'undefined') return null;
  const key = `${CURRENT_CHAT_ID_KEY}:${userId}`;
  try {
    const saved = localStorage.getItem(key);
    return saved ? JSON.parse(saved) : null;
  } catch {
    return null;
  }
}

function persistCurrentChatId(userId: string, id: string | null): void {
  if (typeof window === 'undefined') return;
  const key = `${CURRENT_CHAT_ID_KEY}:${userId}`;
  if (id === null) {
    localStorage.removeItem(key);
  } else {
    localStorage.setItem(key, JSON.stringify(id));
  }
}

export interface ChatStoreState {
  chats: Chat[];
  currentChatId: string | null;
  isLoaded: boolean;
  userId: string;

  loadChats: (userId?: string) => Promise<void>;
  loadMessagesForChat: (chatId: string) => Promise<Message[]>;
  setCurrentChatId: (id: string | null) => void;
  createNewChat: () => Promise<void>;
  createOrUpdateChat: (chat: Chat) => Promise<string>;
  deleteChat: (id: string) => Promise<void>;
  clearChats: () => Promise<void>;
  getChat: (id: string) => Promise<{ data: Chat; status: number }>;
  saveChatSettings: (chatId: string, settings: PersistedChatSettings) => Promise<void>;
  getChatSettings: (chatId: string) => Promise<PersistedChatSettings | undefined>;
}

export const useChatStore = create<ChatStoreState>((set, get) => ({
  chats: [],
  currentChatId: loadCurrentChatId('default'),
  isLoaded: false,
  userId: 'default',

  loadChats: async (userId?: string) => {
    const uid = userId ?? get().userId;
    if (userId) {
      set({ userId: uid, currentChatId: loadCurrentChatId(uid) });
    }

    const records = await chatDb.chats.where('userId').equals(uid).reverse().sortBy('updatedAt');
    const chatList: Chat[] = [];
    for (const record of records.slice(0, MAX_CHATS)) {
      const count = record.messageCount ?? (await chatDb.messages.where('chatId').equals(record.id).count());
      chatList.push(chatRecordToChat(record, [], count));
    }
    set({ chats: chatList, isLoaded: true });
  },

  loadMessagesForChat: async (chatId: string) => {
    const chat = get().chats.find((c) => c.id === chatId);
    if (chat && chat.messages.length > 0) {
      return chat.messages;
    }
    const msgRecords = await chatDb.messages.where('chatId').equals(chatId).sortBy('createdAt');
    const messages = msgRecords.map((r) => r.message);
    if (chat) {
      set({
        chats: get().chats.map((c) => (c.id === chatId ? { ...c, messages, messageCount: messages.length } : c)),
      });
    }
    return messages;
  },

  setCurrentChatId: (id) => {
    const { userId } = get();
    set({ currentChatId: id });
    persistCurrentChatId(userId, id);
  },

  createNewChat: async () => {
    const { chats, currentChatId } = get();
    const currentChat = currentChatId ? (chats.find((c) => c.id === currentChatId) ?? null) : null;

    if (!currentChat || currentChat.messageCount === 0) {
      return;
    }

    const emptyChat = chats.find((chat) => chat.messageCount === 0);
    if (emptyChat) {
      const updatedChat: Chat = { ...emptyChat, updatedAt: Date.now() };
      await get().createOrUpdateChat(updatedChat);
      get().setCurrentChatId(emptyChat.id);
      return;
    }

    const newChat: Chat = {
      id: nanoid(),
      title: 'New Chat',
      messages: [],
      messageCount: 0,
      createdAt: Date.now(),
      updatedAt: Date.now(),
    };
    await get().createOrUpdateChat(newChat);
    get().setCurrentChatId(newChat.id);
  },

  createOrUpdateChat: async (chat) => {
    const { userId } = get();
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

    await get().loadChats();
    return chat.id;
  },

  deleteChat: async (id) => {
    const { currentChatId, chats } = get();

    if (currentChatId !== id) {
      await chatDb.transaction('rw', chatDb.chats, chatDb.messages, async () => {
        await chatDb.messages.where('chatId').equals(id).delete();
        await chatDb.chats.delete(id);
      });
      await get().loadChats();
      return;
    }

    const emptyChat = chats.find((chat) => chat.id !== id && chat.messageCount === 0);
    if (emptyChat) {
      get().setCurrentChatId(emptyChat.id);
      await chatDb.transaction('rw', chatDb.chats, chatDb.messages, async () => {
        await chatDb.messages.where('chatId').equals(id).delete();
        await chatDb.chats.delete(id);
      });
      await get().loadChats();
      return;
    }

    await chatDb.transaction('rw', chatDb.chats, chatDb.messages, async () => {
      await chatDb.messages.where('chatId').equals(id).delete();
      await chatDb.chats.update(id, {
        title: 'New Chat',
        updatedAt: Date.now(),
      });
    });
    await get().loadChats();
  },

  clearChats: async () => {
    const { userId } = get();
    await chatDb.transaction('rw', chatDb.chats, chatDb.messages, async () => {
      const userChats = await chatDb.chats.where('userId').equals(userId).toArray();
      for (const chat of userChats) {
        await chatDb.messages.where('chatId').equals(chat.id).delete();
      }
      await chatDb.chats.where('userId').equals(userId).delete();
    });
    get().setCurrentChatId(null);
    set({ chats: [] });
  },

  getChat: async (id) => {
    const { userId } = get();
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

  saveChatSettings: async (chatId, settings) => {
    await chatDb.chats.update(chatId, { settings });
  },

  getChatSettings: async (chatId) => {
    const record = await chatDb.chats.get(chatId);
    return record?.settings;
  },
}));
