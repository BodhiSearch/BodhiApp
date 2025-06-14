import { useChatCompletion } from '@/hooks/use-chat-completions';
import { ENDPOINT_OAI_CHAT_COMPLETIONS } from '@/hooks/useQuery';
import { act, renderHook } from '@testing-library/react';
import { rest } from 'msw';
import { setupServer } from 'msw/node';
import { QueryClient, QueryClientProvider } from 'react-query';
import { afterAll, beforeAll, beforeEach, describe, expect, it, vi } from 'vitest';

// Reuse the same server setup pattern as AppInitializer.test.tsx
const server = setupServer();

beforeAll(() => server.listen());
afterAll(() => server.close());
beforeEach(() => {
  server.resetHandlers();
});

describe('useChatCompletion', () => {
  let queryClient: QueryClient;

  beforeEach(() => {
    queryClient = new QueryClient({
      defaultOptions: {
        queries: {
          retry: false,
        },
      },
    });
  });

  const wrapper = ({ children }: { children: React.ReactNode }) => (
    <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
  );

  describe('non-streaming completion', () => {
    it('should handle successful completion request', async () => {
      const mockResponse = {
        choices: [
          {
            finish_reason: 'stop',
            index: 0,
            message: {
              content: 'The day that comes after Monday is Tuesday.',
              role: 'assistant',
            },
          },
        ],
        created: 1736234478,
        model: 'llama2-7B-chat',
        id: 'chatcmpl-test',
        object: 'chat.completion',
      };

      server.use(
        rest.post(`*${ENDPOINT_OAI_CHAT_COMPLETIONS}`, (req, res, ctx) => {
          return res(ctx.set('Content-Type', 'application/json'), ctx.json(mockResponse));
        })
      );

      const { result } = renderHook(() => useChatCompletion(), { wrapper });
      const onMessage = vi.fn();
      const onFinish = vi.fn();

      await act(async () => {
        await result.current.append({
          request: {
            model: 'llama2-7B-chat',
            messages: [
              {
                id: '1',
                role: 'user',
                content: 'What day comes after Monday?',
              },
            ],
          },
          onMessage,
          onFinish,
        });
      });

      expect(onMessage).toHaveBeenCalledWith(mockResponse.choices[0].message);
      expect(onFinish).toHaveBeenCalledWith(mockResponse.choices[0].message);
      expect(result.current.isLoading).toBe(false);
      expect(result.current.error).toBeNull();
    });
  });

  describe('streaming completion', () => {
    it('should handle streaming response with callbacks', async () => {
      const chunks = [
        '{"choices":[{"delta":{"content":" The"}}]}',
        '{"choices":[{"delta":{"content":" day"}}]}',
        '{"choices":[{"delta":{"content":" that"}}]}',
        '{"choices":[{"delta":{"content":" comes"}}]}',
        '{"choices":[{"delta":{"content":" after"}}]}',
        '{"choices":[{"delta":{"content":" Monday"}}]}',
        '{"choices":[{"delta":{"content":" is"}}]}',
        '{"choices":[{"delta":{"content":" Tuesday."}}]}',
        '[DONE]',
      ];

      server.use(
        rest.post(`*${ENDPOINT_OAI_CHAT_COMPLETIONS}`, (req, res, ctx) => {
          return res(
            ctx.status(200),
            ctx.set('Content-Type', 'text/event-stream'),
            ctx.body(chunks.map((chunk) => `data: ${chunk}\n\n`).join(''))
          );
        })
      );

      const onDelta = vi.fn();
      const onFinish = vi.fn();
      const { result } = renderHook(() => useChatCompletion(), { wrapper });

      await act(async () => {
        await result.current.append({
          request: {
            model: 'llama2-7B-chat',
            messages: [
              {
                id: '1',
                role: 'user',
                content: 'What day comes after Monday?',
              },
            ],
            stream: true,
          },
          onDelta,
          onFinish,
        });
      });

      expect(onDelta).toHaveBeenCalledWith(' The');
      expect(onDelta).toHaveBeenCalledWith(' day');
      expect(onDelta).toHaveBeenCalledWith(' that');
      expect(onDelta).toHaveBeenCalledWith(' comes');
      expect(onDelta).toHaveBeenCalledWith(' after');
      expect(onDelta).toHaveBeenCalledWith(' Monday');
      expect(onDelta).toHaveBeenCalledWith(' is');
      expect(onDelta).toHaveBeenCalledWith(' Tuesday.');
      expect(onFinish).toHaveBeenCalledWith({
        role: 'assistant',
        content: ' The day that comes after Monday is Tuesday.',
      });
    });

    it('should handle errors in event stream', async () => {
      const formatSSEMessage = (data: any) => `data: ${JSON.stringify(data)}\n\n`;

      const responseText = [
        formatSSEMessage({
          choices: [
            {
              delta: { content: 'Hello' },
              finish_reason: null,
            },
          ],
        }),
        formatSSEMessage({
          error: {
            message: 'Server error occurred',
            type: 'server_error',
          },
        }),
        'data: [DONE]\n\n',
      ].join('');

      server.use(
        rest.post(`*${ENDPOINT_OAI_CHAT_COMPLETIONS}`, (req, res, ctx) => {
          return res(
            ctx.status(200),
            ctx.set({
              'Content-Type': 'text/event-stream',
              'Cache-Control': 'no-cache',
              Connection: 'keep-alive',
            }),
            ctx.body(responseText)
          );
        })
      );

      const onDelta = vi.fn();
      const onFinish = vi.fn();
      const onError = vi.fn();
      const { result } = renderHook(() => useChatCompletion(), { wrapper });

      await act(async () => {
        await result.current.append({
          request: {
            model: 'llama2-7B-chat',
            messages: [
              {
                id: '1',
                role: 'user',
                content: 'What day comes after Monday?',
              },
            ],
            stream: true,
          },
          onDelta,
          onFinish,
          onError,
        });
      });

      // Verify we received the content before the error
      expect(onDelta).toHaveBeenCalledWith('Hello');

      // Current behavior: Stream continues and finishes with partial content
      expect(onFinish).toHaveBeenCalledWith({
        role: 'assistant',
        content: 'Hello',
      });

      // Current behavior: Error in stream is not reported
      expect(onError).not.toHaveBeenCalled();
    });
  });

  describe('metadata handling', () => {
    it('should include metadata in non-streaming response', async () => {
      const mockResponse = {
        choices: [
          {
            finish_reason: 'stop',
            index: 0,
            message: {
              content: 'Test response',
              role: 'assistant',
            },
          },
        ],
        created: 1736234478,
        model: 'test-model',
        usage: {
          completion_tokens: 16,
          prompt_tokens: 5,
          total_tokens: 21,
        },
        timings: {
          prompt_per_second: 41.7157,
          predicted_per_second: 31.04,
        },
        id: 'chatcmpl-test',
        object: 'chat.completion',
      };

      server.use(
        rest.post(`*${ENDPOINT_OAI_CHAT_COMPLETIONS}`, (req, res, ctx) => {
          return res(ctx.set('Content-Type', 'application/json'), ctx.json(mockResponse));
        })
      );

      const { result } = renderHook(() => useChatCompletion(), { wrapper });
      const onMessage = vi.fn();
      const onFinish = vi.fn();

      await act(async () => {
        await result.current.append({
          request: {
            model: 'test-model',
            messages: [{ role: 'user', content: 'test' }],
          },
          onMessage,
          onFinish,
        });
      });

      const expectedMetadata = {
        model: mockResponse.model,
        usage: mockResponse.usage,
        timings: {
          prompt_per_second: mockResponse.timings.prompt_per_second,
          predicted_per_second: mockResponse.timings.predicted_per_second,
        },
      };

      expect(onMessage).toHaveBeenCalledWith(
        expect.objectContaining({
          content: 'Test response',
          role: 'assistant',
          metadata: expectedMetadata,
        })
      );
      expect(onFinish).toHaveBeenCalledWith(
        expect.objectContaining({
          metadata: expectedMetadata,
        })
      );
    });

    it('should include metadata in streaming response', async () => {
      const streamChunks = [
        '{"choices":[{"delta":{"content":"Hello"},"finish_reason":null}]}',
        '{"choices":[{"delta":{"content":" world"},"finish_reason":null}]}',
        `{"choices":[{"delta":{},"finish_reason":"stop"}],"model":"test-model","usage":{"completion_tokens":16,"prompt_tokens":5,"total_tokens":21},"timings":{"prompt_per_second":41.7157,"predicted_per_second":31.04}}`,
      ];

      server.use(
        rest.post(`*${ENDPOINT_OAI_CHAT_COMPLETIONS}`, (req, res, ctx) => {
          return res(
            ctx.status(200),
            ctx.set('Content-Type', 'text/event-stream'),
            ctx.body(streamChunks.map((chunk) => `data: ${chunk}\n\n`).join(''))
          );
        })
      );

      const onDelta = vi.fn();
      const onFinish = vi.fn();
      const { result } = renderHook(() => useChatCompletion(), { wrapper });

      await act(async () => {
        await result.current.append({
          request: {
            model: 'test-model',
            messages: [{ role: 'user', content: 'test' }],
            stream: true,
          },
          onDelta,
          onFinish,
        });
      });

      expect(onDelta).toHaveBeenCalledWith('Hello');
      expect(onDelta).toHaveBeenCalledWith(' world');
      expect(onFinish).toHaveBeenCalledWith({
        role: 'assistant',
        content: 'Hello world',
        metadata: {
          model: 'test-model',
          usage: {
            completion_tokens: 16,
            prompt_tokens: 5,
            total_tokens: 21,
          },
          timings: {
            prompt_per_second: 41.7157,
            predicted_per_second: 31.04,
          },
        },
      });
    });

    it('should handle missing metadata fields gracefully', async () => {
      const mockResponse = {
        choices: [
          {
            finish_reason: 'stop',
            index: 0,
            message: {
              content: 'Test response',
              role: 'assistant',
            },
          },
        ],
        model: 'test-model',
        // No usage or timings data
        id: 'chatcmpl-test',
        object: 'chat.completion',
      };

      server.use(
        rest.post(`*${ENDPOINT_OAI_CHAT_COMPLETIONS}`, (req, res, ctx) => {
          return res(ctx.set('Content-Type', 'application/json'), ctx.json(mockResponse));
        })
      );

      const { result } = renderHook(() => useChatCompletion(), { wrapper });
      const onMessage = vi.fn();
      const onFinish = vi.fn();

      await act(async () => {
        await result.current.append({
          request: {
            model: 'test-model',
            messages: [{ role: 'user', content: 'test' }],
          },
          onMessage,
          onFinish,
        });
      });

      // Message should be delivered without metadata
      expect(onMessage).toHaveBeenCalledWith({
        content: 'Test response',
        role: 'assistant',
      });
      expect(onFinish).toHaveBeenCalledWith({
        content: 'Test response',
        role: 'assistant',
      });
    });
  });
});
