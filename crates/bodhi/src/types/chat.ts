export interface Message {
  id?: string;
  content: string;
  role: 'system' | 'user' | 'assistant';
}

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
