/**
 * Manual MSW v2 handlers for chat completions endpoint
 * Note: OpenAPI schema has incomplete response definitions (unknown/never types)
 * Therefore using manual MSW for all handlers until schema is improved
 */
import { ENDPOINT_OAI_CHAT_COMPLETIONS } from '@/hooks/useQuery';
import { http, HttpResponse } from 'msw';
import { INTERNAL_SERVER_ERROR } from '../openapi-msw-setup';

/**
 * Chat completion message interface for handler responses
 */
interface ChatMessage {
  role: 'system' | 'user' | 'assistant';
  content: string;
}

/**
 * Chat completion request interface
 */
interface ChatCompletionRequest {
  model: string;
  messages: ChatMessage[];
  stream?: boolean;
  temperature?: number;
  max_tokens?: number;
}

/**
 * Chat completion choice interface
 */
interface ChatCompletionChoice {
  index: number;
  message?: ChatMessage;
  delta?: Partial<ChatMessage>;
  finish_reason?: string | null;
}

/**
 * Chat completion response interface
 */
interface ChatCompletionResponse {
  id?: string;
  object: 'chat.completion' | 'chat.completion.chunk';
  created?: number;
  model?: string;
  choices: ChatCompletionChoice[];
  usage?: {
    prompt_tokens: number;
    completion_tokens: number;
    total_tokens: number;
  };
}

/**
 * Create streaming chat completion handler with configurable response
 */
export function mockChatCompletionsStreaming({
  chunks,
  captureRequest,
  ...rest
}: {
  chunks?: string[];
  captureRequest?: (req: ChatCompletionRequest) => void;
} = {}) {
  const defaultChunks = [
    '{"choices":[{"delta":{"content":" Hello"}}]}',
    '{"choices":[{"delta":{"content":" world"}}]}',
    '[DONE]',
  ];

  return [
    http.post(ENDPOINT_OAI_CHAT_COMPLETIONS, async ({ request }) => {
      const requestData = (await request.json()) as ChatCompletionRequest;

      if (captureRequest) {
        captureRequest(requestData);
      }

      const chunksToUse = chunks || defaultChunks;
      const responseBody = chunksToUse.map((chunk) => `data: ${chunk}\n\n`).join('');

      const response = new Response(responseBody, {
        status: 200,
        headers: {
          'Content-Type': 'text/event-stream',
          'Cache-Control': 'no-cache',
          Connection: 'keep-alive',
        },
      });

      return response;
    }),
  ];
}

/**
 * Create non-streaming chat completion handler (manual MSW)
 */
export function mockChatCompletions({
  response: responseConfig,
  captureRequest,
  ...rest
}: {
  response?: Partial<ChatCompletionResponse>;
  captureRequest?: (req: ChatCompletionRequest) => void;
} = {}) {
  return [
    http.post(ENDPOINT_OAI_CHAT_COMPLETIONS, async ({ request }) => {
      const requestData = (await request.json()) as ChatCompletionRequest;

      if (captureRequest) {
        captureRequest(requestData);
      }

      const responseData: ChatCompletionResponse = {
        id: 'chatcmpl-test',
        object: 'chat.completion',
        created: Date.now(),
        model: requestData.model,
        choices: [
          {
            index: 0,
            message: {
              role: 'assistant',
              content: 'Test response',
            },
            finish_reason: 'stop',
          },
        ],
        usage: {
          prompt_tokens: 10,
          completion_tokens: 5,
          total_tokens: 15,
        },
        ...responseConfig,
      };

      const response = HttpResponse.json(responseData);

      return response;
    }),
  ];
}

/**
 * Create error handler for chat completions endpoint (manual MSW)
 */
export function mockChatCompletionsError({
  code = INTERNAL_SERVER_ERROR.code,
  message = INTERNAL_SERVER_ERROR.message,
  status = 400,
  ...rest
}: {
  status?: 400 | 401 | 403 | 500;
  code?: string;
  message?: string;
} = {}) {
  return [
    http.post(ENDPOINT_OAI_CHAT_COMPLETIONS, async () => {
      const errorData = {
        code,
        message,
        type: 'invalid_request_error',
        ...rest,
      };

      return HttpResponse.json({ error: errorData }, { status });
    }),
  ];
}

/**
 * Create network error handler for chat completions endpoint
 */
export function mockChatCompletionsNetworkError() {
  return [
    http.post(ENDPOINT_OAI_CHAT_COMPLETIONS, async () => {
      return HttpResponse.error();
    }),
  ];
}

/**
 * Create handler for streaming response with error in stream
 */
export function mockChatCompletionsStreamingWithError({
  initialChunks,
  errorMessage = 'Server error occurred',
  ...rest
}: {
  initialChunks?: string[];
  errorMessage?: string;
} = {}) {
  const defaultInitialChunks = ['{"choices":[{"delta":{"content":"Hello"}}]}'];

  const errorChunk = JSON.stringify({
    error: {
      message: errorMessage,
      type: 'server_error',
    },
  });

  return [
    http.post(ENDPOINT_OAI_CHAT_COMPLETIONS, async () => {
      const chunks = [...(initialChunks || defaultInitialChunks), errorChunk, '[DONE]'];

      const responseBody = chunks.map((chunk) => `data: ${chunk}\n\n`).join('');

      const response = new Response(responseBody, {
        status: 200,
        headers: {
          'Content-Type': 'text/event-stream',
          'Cache-Control': 'no-cache',
          Connection: 'keep-alive',
        },
      });

      return response;
    }),
  ];
}
