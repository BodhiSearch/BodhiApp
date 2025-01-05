import { type Message } from 'ai/react';

export interface Chat {
  id: string;
  title: string;
  messages: Message[];
  createdAt: number;
  updatedAt?: number;
  model?: string;
  settings?: ChatSettings;
}

export interface ChatSettings {
  model?: string;
  systemPrompt?: string;
  stopWords?: string[];
  maxTokens?: number;
  temperature?: number;
  topP?: number;
  frequencyPenalty?: number;
  presencePenalty?: number;
}

export interface ChatListResponse {
  data: Chat[];
  total: number;
  page: number;
  page_size: number;
}

export interface CreateChatRequest {
  title?: string;
  messages?: Message[];
  settings?: ChatSettings;
}

export interface UpdateChatRequest {
  id: string;
  title?: string;
  messages?: Message[];
  settings?: ChatSettings;
}

export interface ChatResponse {
  id: string;
  title: string;
  messages: Message[];
  settings?: ChatSettings;
  createdAt: number;
  updatedAt?: number;
}

// For streaming chat responses
export interface ChatStreamResponse {
  id: string;
  role: Role;
  content: string;
  createdAt: number;
}

// For error handling
export interface ChatError {
  message: string;
  code: string;
  status: number;
}