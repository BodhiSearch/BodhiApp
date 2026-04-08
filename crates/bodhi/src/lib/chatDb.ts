import Dexie, { type Table } from 'dexie';

import { Message } from '@/types/chat';

export interface ChatRecord {
  id: string;
  userId: string;
  title: string;
  model?: string;
  createdAt: number;
  updatedAt?: number;
  enabledMcpTools?: Record<string, string[]>;
}

export interface MessageRecord {
  id?: number;
  chatId: string;
  message: Message;
  createdAt: number;
}

class BodhiChatDB extends Dexie {
  chats!: Table<ChatRecord, string>;
  messages!: Table<MessageRecord, number>;

  constructor() {
    super('bodhi-chat');
    this.version(1).stores({
      chats: 'id, userId, createdAt, updatedAt',
      messages: '++id, chatId, [chatId+createdAt]',
    });
  }
}

export const chatDb = new BodhiChatDB();
