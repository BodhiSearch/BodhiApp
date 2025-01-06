import { renderHook, act } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from 'react-query';
import { rest } from 'msw';
import { setupServer } from 'msw/node';
import { useChatCompletion } from './useQuery';
import { beforeAll, afterAll, beforeEach, describe, expect, it, vi } from 'vitest';

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
        choices: [{
          finish_reason: "stop",
          index: 0,
          message: {
            content: '{"answer": "Paris"}',
            role: "assistant"
          }
        }],
        created: 1736155698,
        model: "llama2-7B-chat",
        id: "chatcmpl-test",
        object: "chat.completion"
      };

      server.use(
        rest.post('*/v1/chat/completions', (req, res, ctx) => {
          return res(ctx.json(mockResponse));
        })
      );

      const { result } = renderHook(() => useChatCompletion(), { wrapper });

      let response;
      await act(async () => {
        response = await result.current.complete({
          model: "llama2-7B-chat",
          messages: [
            {
              id: '1',
              role: "assistant",
              content: "You are a helpful assistant."
            },
            {
              id: '2',
              role: "user",
              content: "What is the capital of France?"
            }
          ]
        });
      });

      expect(response).toEqual(mockResponse);
      expect(result.current.isLoading).toBe(false);
      expect(result.current.error).toBeNull();
    });

    it('should handle completion request error', async () => {
      server.use(
        rest.post('*/v1/chat/completions', (req, res, ctx) => {
          return res(ctx.status(500), ctx.json({ error: 'Server error' }));
        })
      );

      const { result } = renderHook(() => useChatCompletion(), { wrapper });

      await act(async () => {
        await expect(result.current.complete({
          model: "llama2-7B-chat",
          messages: [{
            id: '1',
            role: "user",
            content: "Test message"
          }]
        })).rejects.toThrow();
      });

      expect(result.current.error).toBeDefined();
    });
  });

  describe('streaming completion', () => {
    it('should handle streaming response with callback', async () => {
      const chunks = [
        '{"choices":[{"delta":{"content":" {"}}]}',
        '{"choices":[{"delta":{"content":"\\"answer\\""}}]}',
        '{"choices":[{"delta":{"content":": \\"Paris\\""}}]}',
        '{"choices":[{"delta":{"content":"}"}}]}',
        '[DONE]'
      ];

      // Create a ReadableStream to simulate SSE
      const stream = new ReadableStream({
        async start(controller) {
          for (const chunk of chunks) {
            const data = `data: ${chunk}\n\n`;
            controller.enqueue(new TextEncoder().encode(data));
          }
          controller.close();
        }
      });

      server.use(
        rest.post('*/v1/chat/completions', (req, res, ctx) => {
          // Create response stream
          const chunks = [
            '{"choices":[{"delta":{"content":" {"}}]}',
            '{"choices":[{"delta":{"content":"\\"answer\\""}}]}',
            '{"choices":[{"delta":{"content":": \\"Paris\\""}}]}',
            '{"choices":[{"delta":{"content":"}"}}]}',
            '[DONE]'
          ];

          return res(
            ctx.status(200),
            ctx.set('Content-Type', 'text/event-stream'),
            ctx.body(chunks.map(chunk => `data: ${chunk}\n\n`).join(''))
          );
        })
      );

      const onMessage = vi.fn();
      const { result } = renderHook(() => useChatCompletion(), { wrapper });

      await act(async () => {
        await result.current.stream({
          model: "llama2-7B-chat",
          messages: [
            {
              id: '1',
              role: "assistant",
              content: "You are a helpful assistant."
            }
          ],
          stream: true,
          onMessage
        });
      });

      // Wait for all microtasks to complete
      await new Promise(resolve => setTimeout(resolve, 0));

      expect(onMessage).toHaveBeenCalledWith(' {');
      expect(onMessage).toHaveBeenCalledWith('"answer"');
      expect(onMessage).toHaveBeenCalledWith(': "Paris"');
      expect(onMessage).toHaveBeenCalledWith('}');
    });

    it('should handle streaming error', async () => {
      server.use(
        rest.post('*/v1/chat/completions', (req, res, ctx) => {
          return res(ctx.status(500));
        })
      );

      const onMessage = vi.fn();
      const { result } = renderHook(() => useChatCompletion(), { wrapper });

      await act(async () => {
        await expect(result.current.stream({
          model: "llama2-7B-chat",
          messages: [{
            id: '1',
            role: "user",
            content: "Test message"
          }],
          stream: true,
          onMessage
        })).rejects.toThrow('Network response was not ok');
      });

      expect(onMessage).not.toHaveBeenCalled();
    });
  });
});