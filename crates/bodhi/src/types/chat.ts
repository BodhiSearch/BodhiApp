export interface MessageMetadata {
  model?: string;
  usage?: {
    completion_tokens?: number;
    prompt_tokens?: number;
    total_tokens?: number;
  };
  timings?: {
    prompt_per_second?: number;
    predicted_per_second?: number;
  };
}

export interface Message {
  id?: string;
  content: string;
  role: 'system' | 'user' | 'assistant';
  metadata?: MessageMetadata;
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
