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
 * @see https://platform.openai.com/docs/api-reference/chat
 */

import { OpenAiApiError } from '@bodhiapp/ts-client';
import type {
  CreateChatCompletionRequest,
  ChatCompletionRequestMessage,
  CreateChatCompletionResponse,
  CreateChatCompletionStreamResponse,
} from '@bodhiapp/ts-client';
import { AxiosError } from 'axios';

import { useMutation } from '@/hooks/useQuery';
import apiClient from '@/lib/apiClient';
import { Message, MessageMetadata } from '@/types/chat';

// Generated OpenAI-compatible types from ts-client

// Type alias for compatibility
type ErrorResponse = OpenAiApiError;

/**
 * Convert UI Message to OpenAI ChatCompletionRequestMessage format.
 * UI messages use simple { role, content } structure.
 * OpenAI uses tagged union with role-specific structures.
 */
function toApiMessage(message: Message): ChatCompletionRequestMessage {
  const role = message.role;
  const content = message.content;

  // Create appropriate message structure based on role
  switch (role) {
    case 'system':
      return { role: 'system', content } as ChatCompletionRequestMessage;
    case 'user':
      return { role: 'user', content } as ChatCompletionRequestMessage;
    case 'assistant':
      return { role: 'assistant', content } as ChatCompletionRequestMessage;
  }
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

interface ChatCompletionCallbacks {
  onDelta?: (content: string) => void;
  onMessage?: (message: Message) => void;
  onFinish?: (message: Message) => void;
  onError?: (error: ErrorResponse | string) => void;
}

interface RequestExts {
  headers?: Record<string, string>;
}

export function useChatCompletion() {
  const appendMutation = useMutation<
    void,
    AxiosError,
    {
      request: ChatCompletionRequestWithUIMessages;
    } & ChatCompletionCallbacks &
      RequestExts
  >(async ({ request, headers, onDelta, onMessage, onFinish, onError }) => {
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

        while (reader) {
          const { done, value } = await reader.read();
          if (done) break;

          const chunk = decoder.decode(value);
          const lines = chunk.split('\n').filter((line) => line.trim() !== '' && line.trim() !== 'data: [DONE]');

          for (const line of lines) {
            try {
              const jsonStr = line.replace(/^data: /, '');
              const json: ChatCompletionStreamResponseWithTimings = JSON.parse(jsonStr);

              // Capture metadata from the last chunk
              if (json.choices?.[0]?.finish_reason === 'stop') {
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
              } else if (json.choices?.[0]?.delta?.content) {
                const content = json.choices[0].delta.content;
                fullContent += content;
                onDelta?.(content);
              }
            } catch (e) {
              console.warn('Failed to parse SSE message:', e);
            }
          }
        }

        // Include metadata in the final message
        const finalMessage: Message = {
          role: 'assistant',
          content: fullContent,
        };
        if (metadata) {
          finalMessage.metadata = metadata;
        }
        onFinish?.(finalMessage);
      } else {
        const data: ChatCompletionResponseWithTimings = await response.json();
        if (data.choices?.[0]?.message) {
          // Convert OpenAI ChatCompletionResponseMessage to UI Message
          const apiMessage = data.choices[0].message;
          const message: Message = {
            role: apiMessage.role as 'assistant',
            content: apiMessage.content || '',
          };
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
          onFinish?.(message);
        }
      }
    } catch (error) {
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
