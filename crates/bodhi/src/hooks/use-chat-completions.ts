import { useMutation } from 'react-query';
import { AxiosError } from 'axios';
import apiClient from '@/lib/apiClient';
import { Message } from '@/types/chat';

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

export function useChatCompletion() {
  const appendMutation = useMutation<
    void,
    AxiosError,
    {
      request: ChatCompletionRequest;
      onDelta?: (content: string) => void;
      onMessage?: (message: Message) => void;
    }
  >(async ({ request, onDelta, onMessage }) => {
    const baseUrl =
      apiClient.defaults.baseURL ||
      (typeof window !== 'undefined'
        ? window.location.origin
        : 'http://localhost');

    const response = await fetch(`${baseUrl}/v1/chat/completions`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(request),
    });

    if (!response.ok) {
      throw new Error('Network response was not ok');
    }

    const contentType = response.headers.get('Content-Type') || '';

    if (contentType.includes('text/event-stream')) {
      const reader = response.body?.getReader();
      const decoder = new TextDecoder();

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
              onDelta?.(json.choices[0].delta.content);
            }
          } catch (e) {
            console.warn('Failed to parse SSE message:', e);
          }
        }
      }
    } else {
      const data: ChatCompletionResponse = await response.json();
      if (data.choices?.[0]?.message) {
        onMessage?.(data.choices[0].message);
      }
    }
  });

  return {
    append: appendMutation.mutateAsync,
    isLoading: appendMutation.isLoading,
    error: appendMutation.error,
  };
}
