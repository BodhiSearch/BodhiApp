/**
 * Manual MSW v2 handlers for chat completions endpoint
 * Updated to use types from @bodhiapp/ts-client with llama.cpp extensions
 */
import type {
  CreateChatCompletionRequest,
  CreateChatCompletionResponse,
  CreateChatCompletionStreamResponse,
} from '@bodhiapp/ts-client';
import { http, HttpResponse } from 'msw';

import { ENDPOINT_OAI_CHAT_COMPLETIONS } from '@/hooks/use-chat-completions';

import { INTERNAL_SERVER_ERROR } from '../setup';

/**
 * llama.cpp-specific timing extensions
 */
interface LlamaCppTimings {
  prompt_per_second?: number;
  predicted_per_second?: number;
}

/**
 * Extended response types with llama.cpp timings support
 */
type ChatCompletionResponseWithTimings = CreateChatCompletionResponse & {
  timings?: LlamaCppTimings;
};

type _ChatCompletionStreamResponseWithTimings = CreateChatCompletionStreamResponse & {
  timings?: LlamaCppTimings;
};

/**
 * Create streaming chat completion handler with configurable response
 * Uses generated types from @bodhiapp/ts-client
 */
export function mockChatCompletionsStreaming({
  chunks,
  captureRequest,
  ..._rest
}: {
  chunks?: string[];
  captureRequest?: (req: CreateChatCompletionRequest) => void;
} = {}) {
  const defaultChunks = [
    '{"choices":[{"delta":{"content":" Hello"}}]}',
    '{"choices":[{"delta":{"content":" world"}}]}',
    '[DONE]',
  ];

  return [
    http.post(ENDPOINT_OAI_CHAT_COMPLETIONS, async ({ request }) => {
      const requestData = (await request.json()) as CreateChatCompletionRequest;

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
 * Create non-streaming chat completion handler
 * Uses generated types from @bodhiapp/ts-client with llama.cpp timings support
 */
export function mockChatCompletions({
  response: responseConfig,
  captureRequest,
  request: requestMatch,
  ..._rest
}: {
  response?: Partial<ChatCompletionResponseWithTimings>;
  captureRequest?: (req: CreateChatCompletionRequest) => void;
  request?: {
    model?: string;
    messages?: Array<{ role: string; content: string }>;
  };
} = {}) {
  return [
    http.post(ENDPOINT_OAI_CHAT_COMPLETIONS, async ({ request }) => {
      const requestData = (await request.json()) as CreateChatCompletionRequest;

      if (captureRequest) {
        captureRequest(requestData);
      }

      // If request matching is specified, verify the request matches before responding
      if (requestMatch) {
        // Check model match if specified
        if (requestMatch.model !== undefined && requestData.model !== requestMatch.model) {
          return; // Pass through to next handler
        }

        // Check messages match if specified
        if (requestMatch.messages !== undefined) {
          if (!requestData.messages || requestData.messages.length !== requestMatch.messages.length) {
            return; // Pass through to next handler
          }

          // Compare each message (role + content)
          const messagesMatch = requestMatch.messages.every((expectedMsg, index) => {
            const actualMsg = requestData.messages[index];
            return actualMsg.role === expectedMsg.role && actualMsg.content === expectedMsg.content;
          });

          if (!messagesMatch) {
            return; // Pass through to next handler
          }
        }
      }

      const responseData: ChatCompletionResponseWithTimings = {
        id: 'chatcmpl-test',
        object: 'chat.completion',
        created: Date.now(),
        model: requestData.model,
        choices: [
          {
            index: 0,
            message: {
              // eslint-disable-next-line @typescript-eslint/no-explicit-any
              role: 'resource_admin' as any, // Work around Role type mismatch in generated types
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
        timings: {
          prompt_per_second: 200.0,
          predicted_per_second: 150.0,
        },
        ...responseConfig,
      };

      return HttpResponse.json(responseData);
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
  ..._rest
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
