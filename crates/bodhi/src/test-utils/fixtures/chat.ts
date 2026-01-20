/**
 * Test fixtures for chat completion testing
 * Uses types from @bodhiapp/ts-client with llama.cpp extensions
 */
import type { CreateChatCompletionResponse, ToolsetWithTools } from '@bodhiapp/ts-client';

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

// ============================================================================
// Tool Call Fixtures
// ============================================================================

/**
 * Mock tool call for web search
 */
export const mockToolCallWebSearch = {
  id: 'call_web_search_123',
  name: 'toolset__builtin-exa-web-search__search',
  arguments: JSON.stringify({ query: 'AI news 2024', num_results: 5 }),
};

/**
 * Mock tool call for a different tool
 */
export const mockToolCallCalculator = {
  id: 'call_calc_456',
  name: 'toolset__builtin-calculator__calculate',
  arguments: JSON.stringify({ expression: '2 + 2' }),
};

/**
 * Streaming chunks for tool call response
 */
export const mockStreamingChunksWithToolCalls = [
  // First tool call - id and name
  '{"choices":[{"index":0,"delta":{"tool_calls":[{"index":0,"id":"call_web_search_123","type":"function","function":{"name":"toolset__builtin-exa-web-search__search"}}]}}]}',
  // First tool call - arguments
  '{"choices":[{"index":0,"delta":{"tool_calls":[{"index":0,"function":{"arguments":"{\\"query\\":"}}]}}]}',
  '{"choices":[{"index":0,"delta":{"tool_calls":[{"index":0,"function":{"arguments":"\\"AI news 2024\\"}"}}]}}]}',
  // Finish reason: tool_calls
  '{"choices":[{"index":0,"delta":{},"finish_reason":"tool_calls"}]}',
  '[DONE]',
];

/**
 * Streaming chunks for final response after tool execution
 */
export const mockStreamingChunksFinalResponse = [
  '{"choices":[{"index":0,"delta":{"role":"assistant","content":""}}]}',
  '{"choices":[{"index":0,"delta":{"content":"Based on"}}]}',
  '{"choices":[{"index":0,"delta":{"content":" the search"}}]}',
  '{"choices":[{"index":0,"delta":{"content":" results,"}}]}',
  '{"choices":[{"index":0,"delta":{"content":" here is"}}]}',
  '{"choices":[{"index":0,"delta":{"content":" the latest"}}]}',
  '{"choices":[{"index":0,"delta":{"content":" AI news."}}]}',
  '{"choices":[{"index":0,"delta":{},"finish_reason":"stop"}],"usage":{"prompt_tokens":50,"completion_tokens":30,"total_tokens":80}}',
  '[DONE]',
];

/**
 * Mock tool execution result
 */
export const mockToolExecutionResult = {
  tool_call_id: 'call_web_search_123',
  result: {
    results: [
      { title: 'AI News Article 1', url: 'https://example.com/1', snippet: 'Latest AI developments...' },
      { title: 'AI News Article 2', url: 'https://example.com/2', snippet: 'Breaking news in AI...' },
    ],
  },
};

/**
 * Mock tool execution error
 */
export const mockToolExecutionError = {
  tool_call_id: 'call_web_search_123',
  error: 'API rate limit exceeded',
};

/**
 * Mock toolset with nested tools (configured and enabled)
 */
export const mockToolsetWithTools: ToolsetWithTools = {
  toolset_id: 'builtin-exa-web-search',
  name: 'Exa Web Search',
  description: 'Search the web using Exa AI',
  app_enabled: true,
  tools: [
    {
      type: 'function',
      function: {
        name: 'search',
        description: 'Search the web using Exa AI for real-time information',
        parameters: {
          type: 'object',
          properties: {
            query: {
              type: 'string',
              description: 'Search query',
            },
            num_results: {
              type: 'number',
              description: 'Number of results to return (default: 5)',
            },
          },
          required: ['query'],
        },
      },
    },
    {
      type: 'function',
      function: {
        name: 'findSimilar',
        description: 'Find similar pages to a given URL',
        parameters: {
          type: 'object',
          properties: {
            url: {
              type: 'string',
              description: 'URL to find similar pages for',
            },
            num_results: {
              type: 'number',
              description: 'Number of results to return (default: 5)',
            },
          },
          required: ['url'],
        },
      },
    },
    {
      type: 'function',
      function: {
        name: 'contents',
        description: 'Get contents of URLs',
        parameters: {
          type: 'object',
          properties: {
            urls: {
              type: 'array',
              items: { type: 'string' },
              description: 'URLs to get contents for',
            },
          },
          required: ['urls'],
        },
      },
    },
    {
      type: 'function',
      function: {
        name: 'answer',
        description: 'Get an answer to a question from Exa',
        parameters: {
          type: 'object',
          properties: {
            query: {
              type: 'string',
              description: 'Question to answer',
            },
          },
          required: ['query'],
        },
      },
    },
  ],
};

/**
 * Mock toolset not configured (missing API key)
 */
export const mockToolsetWithToolsNotConfigured: ToolsetWithTools = {
  toolset_id: 'builtin-calculator',
  name: 'Calculator',
  description: 'Perform mathematical calculations',
  app_enabled: true,
  tools: [
    {
      type: 'function',
      function: {
        name: 'calculate',
        description: 'Perform mathematical calculations',
        parameters: {
          type: 'object',
          properties: {
            expression: {
              type: 'string',
              description: 'Mathematical expression to evaluate',
            },
          },
          required: ['expression'],
        },
      },
    },
  ],
};
