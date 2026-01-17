/**
 * Chat completion hook with OpenAI API compatibility and llama.cpp extensions.
 *
 * This hook provides a React interface to the OpenAI-compatible chat completions API,
 * with support for both streaming and non-streaming responses.
 *
 * Type Architecture:
 * - API Layer: Uses generated types from @bodhiapp/ts-client (OpenAI-compatible)
 * - UI Layer: Uses Message type from @/types/chat (simplified for React state)
 * - Adapters: Convert between API and UI types at the hook boundary
 *
 * llama.cpp Extensions:
 * - Supports additional `timings` field in responses (not in standard OpenAI API)
 * - Types extended via ChatCompletionResponseWithTimings and ChatCompletionStreamResponseWithTimings
 *
 * Message Conversion:
 * - toApiMessage(): Converts UI Message → OpenAI ChatCompletionRequestMessage
 * - Response handlers: Convert OpenAI ChatCompletionResponseMessage → UI Message
 *
 * Tool Call Support:
 * - Handles streaming tool_calls chunks with accumulation by index
 * - Supports finish_reason: 'tool_calls' for agentic loop
 * - Converts tool messages to API format
 *
 * @see https://platform.openai.com/docs/api-reference/chat
 */

import { OpenAiApiError } from '@bodhiapp/ts-client';
import type {
  CreateChatCompletionRequest,
  ChatCompletionRequestMessage,
  CreateChatCompletionResponse,
  CreateChatCompletionStreamResponse,
  ChatCompletionMessageToolCallChunk,
  FinishReason,
} from '@bodhiapp/ts-client';
import { AxiosError } from 'axios';

import { useMutation } from '@/hooks/useQuery';
import apiClient from '@/lib/apiClient';
import { Message, MessageMetadata, ToolCall } from '@/types/chat';

// Generated OpenAI-compatible types from ts-client

// Type alias for compatibility
type ErrorResponse = OpenAiApiError;

/**
 * Convert UI Message to OpenAI ChatCompletionRequestMessage format.
 * UI messages use simple { role, content } structure.
 * OpenAI uses tagged union with role-specific structures.
 */
export function toApiMessage(message: Message): ChatCompletionRequestMessage {
  const role = message.role;
  const content = message.content;

  // Create appropriate message structure based on role
  switch (role) {
    case 'system':
      return { role: 'system', content } as ChatCompletionRequestMessage;
    case 'user':
      return { role: 'user', content } as ChatCompletionRequestMessage;
    case 'assistant':
      // Include tool_calls if present
      if (message.tool_calls && message.tool_calls.length > 0) {
        return {
          role: 'assistant',
          content: content || null,
          tool_calls: message.tool_calls.map((tc) => ({
            id: tc.id,
            type: 'function' as const,
            function: tc.function,
          })),
        } as ChatCompletionRequestMessage;
      }
      return { role: 'assistant', content } as ChatCompletionRequestMessage;
    case 'tool':
      return {
        role: 'tool',
        content,
        tool_call_id: message.tool_call_id!,
      } as ChatCompletionRequestMessage;
  }
}

/**
 * Accumulate tool calls from streaming chunks.
 * Tool calls arrive incrementally by index, requiring accumulation.
 */
export function accumulateToolCallChunk(
  toolCallMap: Map<number, ToolCall>,
  chunk: ChatCompletionMessageToolCallChunk
): void {
  const index = chunk.index;
  const existing = toolCallMap.get(index) || {
    id: '',
    type: 'function' as const,
    function: { name: '', arguments: '' },
  };

  if (chunk.id) {
    existing.id = chunk.id;
  }
  if (chunk.function?.name) {
    existing.function.name = chunk.function.name;
  }
  if (chunk.function?.arguments) {
    existing.function.arguments += chunk.function.arguments;
  }

  toolCallMap.set(index, existing);
}

/**
 * llama.cpp-specific timings extension.
 * These fields are added by llama.cpp server but not in standard OpenAI API.
 */
interface LlamaCppTimings {
  cache_n?: number;
  prompt_n?: number;
  prompt_ms?: number;
  prompt_per_token_ms?: number;
  prompt_per_second?: number;
  predicted_n?: number;
  predicted_ms?: number;
  predicted_per_token_ms?: number;
  predicted_per_second?: number;
}

/**
 * CreateChatCompletionResponse with llama.cpp timings extension.
 */
type ChatCompletionResponseWithTimings = CreateChatCompletionResponse & {
  timings?: LlamaCppTimings;
};

/**
 * CreateChatCompletionStreamResponse with llama.cpp timings extension.
 */
type ChatCompletionStreamResponseWithTimings = CreateChatCompletionStreamResponse & {
  timings?: LlamaCppTimings;
};

/**
 * Chat completion request using UI Message type.
 * This will be converted to CreateChatCompletionRequest format when sent to API.
 */
type ChatCompletionRequestWithUIMessages = Omit<CreateChatCompletionRequest, 'messages'> & {
  messages: Message[];
};

// Constants
export const ENDPOINT_OAI_CHAT_COMPLETIONS = '/v1/chat/completions';

/**
 * Completion result with finish reason and optional tool calls.
 */
export interface CompletionResult {
  message: Message;
  finishReason: FinishReason | null;
  toolCalls?: ToolCall[];
}

interface ChatCompletionCallbacks {
  onDelta?: (content: string) => void;
  onToolCallDelta?: (toolCalls: ToolCall[]) => void;
  onMessage?: (message: Message) => void;
  onFinish?: (result: CompletionResult) => void;
  onError?: (error: ErrorResponse | string) => void;
}

interface RequestExts {
  headers?: Record<string, string>;
  signal?: AbortSignal;
}

export function useChatCompletion() {
  const appendMutation = useMutation<
    void,
    AxiosError,
    {
      request: ChatCompletionRequestWithUIMessages;
    } & ChatCompletionCallbacks &
      RequestExts
  >(async ({ request, headers, signal, onDelta, onToolCallDelta, onMessage, onFinish, onError }) => {
    const baseUrl =
      apiClient.defaults.baseURL || (typeof window !== 'undefined' ? window.location.origin : 'http://localhost');

    const url = `${baseUrl}${ENDPOINT_OAI_CHAT_COMPLETIONS}`;

    try {
      const response = await fetch(url, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          ...headers,
        },
        body: JSON.stringify({
          ...request,
          messages: request.messages.map(toApiMessage),
        }),
        signal,
      });

      const contentType = response.headers.get('Content-Type') || '';

      if (!response.ok) {
        let errorData: ErrorResponse | string;

        if (contentType.includes('application/json')) {
          errorData = (await response.json()) as ErrorResponse;
        } else {
          errorData = await response.text();
        }

        if (onError) {
          onError(errorData);
        } else {
          const errorMessage =
            typeof errorData === 'string' ? errorData : errorData.error?.message || 'Unknown error occurred';
          throw new Error(errorMessage);
        }
        return;
      }

      if (contentType.includes('text/event-stream')) {
        const reader = response.body?.getReader();
        const decoder = new TextDecoder();
        let fullContent = '';
        let metadata: MessageMetadata | undefined;
        let finishReason: FinishReason | null = null;
        const toolCallMap = new Map<number, ToolCall>();

        while (reader) {
          const { done, value } = await reader.read();
          if (done) break;

          const chunk = decoder.decode(value);
          const lines = chunk.split('\n').filter((line) => line.trim() !== '' && line.trim() !== 'data: [DONE]');

          for (const line of lines) {
            try {
              const jsonStr = line.replace(/^data: /, '');
              const json: ChatCompletionStreamResponseWithTimings = JSON.parse(jsonStr);
              const choice = json.choices?.[0];

              // Capture finish reason
              if (choice?.finish_reason) {
                finishReason = choice.finish_reason;
                metadata = {
                  model: json.model,
                  usage: json.usage ?? undefined,
                  timings: json.timings
                    ? {
                        prompt_per_second: json.timings.prompt_per_second,
                        predicted_per_second: json.timings.predicted_per_second,
                      }
                    : undefined,
                };
              }

              // Handle content delta
              if (choice?.delta?.content) {
                const content = choice.delta.content;
                fullContent += content;
                onDelta?.(content);
              }

              // Handle tool call chunks
              if (choice?.delta?.tool_calls) {
                for (const toolCallChunk of choice.delta.tool_calls) {
                  accumulateToolCallChunk(toolCallMap, toolCallChunk);
                }
                // Emit current accumulated tool calls for UI updates
                const currentToolCalls = Array.from(toolCallMap.values());
                onToolCallDelta?.(currentToolCalls);
              }
            } catch (e) {
              console.warn('Failed to parse SSE message:', e);
            }
          }
        }

        // Build the final message
        const toolCalls = toolCallMap.size > 0 ? Array.from(toolCallMap.values()) : undefined;
        const finalMessage: Message = {
          role: 'assistant',
          content: fullContent,
        };
        if (toolCalls) {
          finalMessage.tool_calls = toolCalls;
        }
        if (metadata) {
          finalMessage.metadata = metadata;
        }

        onFinish?.({
          message: finalMessage,
          finishReason,
          toolCalls,
        });
      } else {
        const data: ChatCompletionResponseWithTimings = await response.json();
        const choice = data.choices?.[0];
        if (choice?.message) {
          // Convert OpenAI ChatCompletionResponseMessage to UI Message
          const apiMessage = choice.message;
          const finishReason = choice.finish_reason ?? null;

          // Extract tool calls if present (only function type tool calls)
          const toolCalls: ToolCall[] | undefined = apiMessage.tool_calls
            ?.filter((tc): tc is typeof tc & { type: 'function' } => tc.type === 'function')
            .map((tc) => ({
              id: tc.id,
              type: 'function' as const,
              function: {
                name: tc.function.name,
                arguments: tc.function.arguments,
              },
            }));

          const message: Message = {
            role: apiMessage.role as 'assistant',
            content: apiMessage.content || '',
          };
          if (toolCalls) {
            message.tool_calls = toolCalls;
          }
          if (data.usage) {
            message.metadata = {
              model: data.model,
              usage: data.usage,
              timings: {
                prompt_per_second: data.timings?.prompt_per_second,
                predicted_per_second: data.timings?.predicted_per_second,
              },
            };
          }
          onMessage?.(message);
          onFinish?.({
            message,
            finishReason,
            toolCalls,
          });
        }
      }
    } catch (error) {
      // Don't treat abort as an error
      if (error instanceof Error && error.name === 'AbortError') {
        return;
      }

      const errorMessage = error instanceof Error ? error.message : 'Unknown error occurred';

      if (onError) {
        onError(errorMessage);
      } else {
        throw error;
      }
    }
  });

  return {
    append: appendMutation.mutateAsync,
    isLoading: appendMutation.isLoading,
    error: appendMutation.error,
  };
}
