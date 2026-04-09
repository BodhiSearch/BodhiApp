import Dexie, { type Table } from 'dexie';

import { ApiFormat } from '@bodhiapp/ts-client';

import { Message } from '@/types/chat';

export type ApiFormatSetting = Exclude<ApiFormat, 'placeholder'>;

export interface PersistedChatSettings {
  model: string;
  apiFormat: ApiFormatSetting;
  stream?: boolean;
  stream_enabled: boolean;
  seed?: number;
  seed_enabled: boolean;
  systemPrompt?: string;
  systemPrompt_enabled: boolean;
  stop?: string[];
  stop_enabled: boolean;
  max_tokens?: number;
  max_tokens_enabled: boolean;
  n?: number;
  n_enabled: boolean;
  temperature?: number;
  temperature_enabled: boolean;
  top_p?: number;
  top_p_enabled: boolean;
  presence_penalty?: number;
  presence_penalty_enabled: boolean;
  frequency_penalty?: number;
  frequency_penalty_enabled: boolean;
  logit_bias?: Record<string, number>;
  logit_bias_enabled: boolean;
  response_format?: {
    type: 'text' | 'json_object';
    schema?: object;
  };
  response_format_enabled: boolean;
  maxToolIterations?: number;
  maxToolIterations_enabled: boolean;
}

export interface ChatRecord {
  id: string;
  userId: string;
  title: string;
  model?: string;
  createdAt: number;
  updatedAt?: number;
  enabledMcpTools?: Record<string, string[]>;
  settings?: PersistedChatSettings;
  messageCount?: number;
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
