import { renderHook, act } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from 'react-query';
import { rest } from 'msw';
import { setupServer } from 'msw/node';
import { useChatCompletion } from './use-chat-completions';
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
            content: 'The day that comes after Monday is Tuesday.',
            role: "assistant"
          }
        }],
        created: 1736234478,
        model: "llama2-7B-chat",
        id: "chatcmpl-test",
        object: "chat.completion"
      };

      server.use(
        rest.post('*/v1/chat/completions', (req, res, ctx) => {
          return res(
            ctx.set('Content-Type', 'application/json'),
            ctx.json(mockResponse)
          );
        })
      );

      const { result } = renderHook(() => useChatCompletion(), { wrapper });
      const onMessage = vi.fn();

      await act(async () => {
        await result.current.append({
          request: {
            model: "llama2-7B-chat",
            messages: [
              {
                id: '1',
                role: "user",
                content: "What day comes after Monday?"
              }
            ]
          },
          onMessage
        });
      });

      expect(onMessage).toHaveBeenCalledWith(mockResponse.choices[0].message);
      expect(result.current.isLoading).toBe(false);
      expect(result.current.error).toBeNull();
    });
  });

  describe('streaming completion', () => {
    it('should handle streaming response with callback', async () => {
      const chunks = [
        '{"choices":[{"delta":{"content":" The"}}]}',
        '{"choices":[{"delta":{"content":" day"}}]}',
        '{"choices":[{"delta":{"content":" that"}}]}',
        '{"choices":[{"delta":{"content":" comes"}}]}',
        '{"choices":[{"delta":{"content":" after"}}]}',
        '{"choices":[{"delta":{"content":" Monday"}}]}',
        '{"choices":[{"delta":{"content":" is"}}]}',
        '{"choices":[{"delta":{"content":" Tuesday."}}]}',
        '[DONE]'
      ];

      server.use(
        rest.post('*/v1/chat/completions', (req, res, ctx) => {
          return res(
            ctx.status(200),
            ctx.set('Content-Type', 'text/event-stream'),
            ctx.body(chunks.map(chunk => `data: ${chunk}\n\n`).join(''))
          );
        })
      );

      const onDelta = vi.fn();
      const { result } = renderHook(() => useChatCompletion(), { wrapper });

      await act(async () => {
        await result.current.append({
          request: {
            model: "llama2-7B-chat",
            messages: [
              {
                id: '1',
                role: "user",
                content: "What day comes after Monday?"
              }
            ],
            stream: true
          },
          onDelta
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
    });
  });
});