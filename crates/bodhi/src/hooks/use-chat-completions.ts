import { useMutation } from 'react-query';
import { AxiosError } from 'axios';
import apiClient from '@/lib/apiClient';
import { Message, MessageMetadata } from '@/types/chat';
import { OpenAiApiError } from '@bodhiapp/ts-client';

// Type alias for compatibility
type ErrorResponse = OpenAiApiError;

// Constants
export const ENDPOINT_OAI_CHAT_COMPLETIONS = '/v1/chat/completions';

interface ChatCompletionRequest {
  messages: Message[];
  stream?: boolean;
  model: string;
  temperature?: number;
  stop?: string[];
  max_tokens?: number;
  top_p?: number;
  frequency_penalty?: number;
  presence_penalty?: number;
}

interface ChatCompletionResponse {
  id: string;
  object: string;
  created: number;
  model: string;
  choices: {
    index: number;
    message: Message;
    finish_reason: string;
  }[];
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
      request: ChatCompletionRequest;
    } & ChatCompletionCallbacks &
      RequestExts
  >(async ({ request, headers, onDelta, onMessage, onFinish, onError }) => {
    const baseUrl =
      apiClient.defaults.baseURL || (typeof window !== 'undefined' ? window.location.origin : 'http://localhost');

    try {
      const response = await fetch(`${baseUrl}${ENDPOINT_OAI_CHAT_COMPLETIONS}`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          ...headers,
        },
        body: JSON.stringify(request),
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
              const json = JSON.parse(jsonStr);

              // Capture metadata from the last chunk
              if (json.choices?.[0]?.finish_reason === 'stop' && json.timings) {
                metadata = {
                  model: json.model,
                  usage: json.usage,
                  timings: {
                    prompt_per_second: json.timings?.prompt_per_second,
                    predicted_per_second: json.timings?.predicted_per_second,
                  },
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
        const data: ChatCompletionResponse = await response.json();
        if (data.choices?.[0]?.message) {
          const message = {
            ...data.choices[0].message,
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
