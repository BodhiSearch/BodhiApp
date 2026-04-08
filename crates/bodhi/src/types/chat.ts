import type {
  TextContent as PiTextContent,
  ThinkingContent as PiThinkingContent,
  AssistantMessage as PiAssistantMessage,
} from '@mariozechner/pi-ai';
import type { AgentMessage as PiAgentMessage } from '@mariozechner/pi-agent-core';

export type { PiTextContent, PiThinkingContent, PiAssistantMessage, PiAgentMessage };

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
  messageCount: number;
  createdAt: number;
  updatedAt?: number;
  model?: string;
  settings?: ChatSettings;
  enabledTools?: Record<string, string[]>;
  enabledMcpTools?: Record<string, string[]>;
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

export function extractTextFromAgentMessage(msg: PiAgentMessage): string {
  if ('role' in msg) {
    if (msg.role === 'user') {
      return typeof msg.content === 'string'
        ? msg.content
        : (msg.content as PiTextContent[])
            .filter((c): c is PiTextContent => c.type === 'text')
            .map((c) => c.text)
            .join('');
    }
    if (msg.role === 'assistant') {
      const assistantMsg = msg as PiAssistantMessage;
      return assistantMsg.content
        .filter((c): c is PiTextContent => c.type === 'text')
        .map((c) => c.text)
        .join('');
    }
  }
  return '';
}

export function extractThinkingFromAgentMessage(msg: PiAgentMessage): string {
  if ('role' in msg && msg.role === 'assistant') {
    const assistantMsg = msg as PiAssistantMessage;
    return assistantMsg.content
      .filter((c): c is PiThinkingContent => c.type === 'thinking')
      .map((c) => c.thinking)
      .join('\n');
  }
  return '';
}
