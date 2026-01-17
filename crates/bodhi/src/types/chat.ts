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

export interface ToolCallFunction {
  name: string;
  arguments: string;
}

export interface ToolCall {
  id: string;
  type: 'function';
  function: ToolCallFunction;
}

export interface Message {
  id?: string;
  content: string;
  role: 'system' | 'user' | 'assistant' | 'tool';
  tool_calls?: ToolCall[];
  tool_call_id?: string;
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
  enabledTools?: Record<string, string[]>;
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
