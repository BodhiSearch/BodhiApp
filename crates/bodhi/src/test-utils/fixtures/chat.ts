/**
 * Test fixtures for chat completion testing
 * Uses types from @bodhiapp/ts-client with llama.cpp extensions
 */
import type { CreateChatCompletionResponse } from '@bodhiapp/ts-client';

/**
 * llama.cpp-specific timing extensions
 */
interface LlamaCppTimings {
  prompt_per_second?: number;
  predicted_per_second?: number;
}

/**
 * Extended response type with llama.cpp timings
 */
type ChatCompletionResponseWithTimings = CreateChatCompletionResponse & {
  timings?: LlamaCppTimings;
};

/**
 * Simple model fixture for chat tests
 */
export const mockChatModel = {
  source: 'user' as const,
  alias: 'test-chat-model',
  repo: 'test/repo',
  filename: 'model.gguf',
  snapshot: 'abc123',
  request_params: {},
  context_params: [],
};

/**
 * Non-streaming response with full metadata (including llama.cpp timings)
 */
export const mockNonStreamingResponse: ChatCompletionResponseWithTimings = {
  id: 'chatcmpl-test-123',
  object: 'chat.completion',
  created: Date.now(),
  model: 'test-chat-model',
  choices: [
    {
      index: 0,
      message: {
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        role: 'resource_admin' as any, // Work around Role type mismatch in generated types
        content: 'This is a detailed test response from the AI assistant.',
      },
      finish_reason: 'stop',
    },
  ],
  usage: {
    prompt_tokens: 15,
    completion_tokens: 25,
    total_tokens: 40,
  },
  timings: {
    prompt_per_second: 250.5,
    predicted_per_second: 180.3,
  },
};

/**
 * Streaming response chunks with final metadata
 * Final chunk includes usage and timings
 */
export const mockStreamingChunks = [
  '{"choices":[{"index":0,"delta":{"role":"assistant","content":""}}]}',
  '{"choices":[{"index":0,"delta":{"content":"Hello"}}]}',
  '{"choices":[{"index":0,"delta":{"content":" there"}}]}',
  '{"choices":[{"index":0,"delta":{"content":"!"}}]}',
  '{"choices":[{"index":0,"delta":{"content":" How"}}]}',
  '{"choices":[{"index":0,"delta":{"content":" can"}}]}',
  '{"choices":[{"index":0,"delta":{"content":" I"}}]}',
  '{"choices":[{"index":0,"delta":{"content":" help"}}]}',
  '{"choices":[{"index":0,"delta":{"content":"?"}}]}',
  '{"choices":[{"index":0,"delta":{},"finish_reason":"stop"}],"usage":{"prompt_tokens":10,"completion_tokens":20,"total_tokens":30},"timings":{"prompt_per_second":200.0,"predicted_per_second":195.7}}',
  '[DONE]',
];

/**
 * Response without timings (standard OpenAI, not llama.cpp)
 */
export const mockStandardOpenAIResponse: CreateChatCompletionResponse = {
  id: 'chatcmpl-standard-456',
  object: 'chat.completion',
  created: Date.now(),
  model: 'test-chat-model',
  choices: [
    {
      index: 0,
      message: {
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        role: 'resource_admin' as any, // Work around Role type mismatch in generated types
        content: 'Standard OpenAI response without timings.',
      },
      finish_reason: 'stop',
    },
  ],
  usage: {
    prompt_tokens: 10,
    completion_tokens: 5,
    total_tokens: 15,
  },
  // No timings field - standard OpenAI response
};
