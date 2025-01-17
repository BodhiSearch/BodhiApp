import { useMutation } from 'react-query';
import { AxiosError } from 'axios';
import apiClient from '@/lib/apiClient';
import { Message } from '@/types/chat';
import { ErrorResponse } from '@/types/models';

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
      apiClient.defaults.baseURL ||
      (typeof window !== 'undefined'
        ? window.location.origin
        : 'http://localhost');

    try {
      const response = await fetch(`${baseUrl}/v1/chat/completions`, {
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
            typeof errorData === 'string'
              ? errorData
              : errorData.error?.message || 'Unknown error occurred';
          throw new Error(errorMessage);
        }
        return;
      }

      if (contentType.includes('text/event-stream')) {
        const reader = response.body?.getReader();
        const decoder = new TextDecoder();
        let fullContent = '';

        while (reader) {
          const { done, value } = await reader.read();
          if (done) break;

          const chunk = decoder.decode(value);
          const lines = chunk
            .split('\n')
            .filter(
              (line) => line.trim() !== '' && line.trim() !== 'data: [DONE]'
            );

          for (const line of lines) {
            try {
              const jsonStr = line.replace(/^data: /, '');
              const json = JSON.parse(jsonStr);
              if (json.choices?.[0]?.delta?.content) {
                const content = json.choices[0].delta.content;
                fullContent += content;
                onDelta?.(content);
              }
            } catch (e) {
              console.warn('Failed to parse SSE message:', e);
            }
          }
        }

        // Call onFinish with the complete message after streaming is done
        const finalMessage: Message = {
          role: 'assistant',
          content: fullContent,
        };
        onFinish?.(finalMessage);
      } else {
        const data: ChatCompletionResponse = await response.json();
        if (data.choices?.[0]?.message) {
          const message = data.choices[0].message;
          onMessage?.(message);
          onFinish?.(message);
        }
      }
    } catch (error) {
      const errorMessage =
        error instanceof Error ? error.message : 'Unknown error occurred';

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
