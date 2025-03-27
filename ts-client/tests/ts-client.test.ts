import { paths } from '../src';
import { describe, it, expect, beforeAll, afterAll, afterEach } from 'vitest';
import { setupServer } from 'msw/node';
import { http, HttpResponse } from 'msw';

const TEST_PORT = 9135;
const API_BASE_URL = `http://localhost:${TEST_PORT}`;

// Mock response data
const mockChatCompletion = {
  id: "mock-completion-id",
  object: "chat.completion",
  created: Date.now(),
  model: "test-model",
  choices: [
    {
      index: 0,
      message: {
        role: "assistant",
        content: "4"
      },
      finish_reason: "stop"
    }
  ],
  usage: {
    prompt_tokens: 10,
    completion_tokens: 1,
    total_tokens: 11
  }
};

// Setup MSW server
const server = setupServer(
  http.post(`${API_BASE_URL}/v1/chat/completions`, () => {
    return HttpResponse.json(mockChatCompletion);
  })
);

// Start server before all tests
beforeAll(() => server.listen({ onUnhandledRequest: 'error' }));

// Reset handlers after each test
afterEach(() => server.resetHandlers());

// Clean up after all tests are done
afterAll(() => server.close());

describe('BodhiApp TypeScript Client', () => {
  describe('Chat Completion API', () => {
    it('should create a chat completion', async () => {
      type ChatCompletionRequest = paths['/v1/chat/completions']['post']['requestBody']['content']['application/json'];
      type ChatCompletionResponse = paths['/v1/chat/completions']['post']['responses']['200']['content']['application/json'];

      const request: ChatCompletionRequest = {
        model: "test-model",
        messages: [
          {
            role: "user",
            content: "What is 2+2?"
          }
        ],
        temperature: 0.7,
        max_tokens: 100,
        stream: false
      };

      const response = await fetch(`${API_BASE_URL}/v1/chat/completions`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(request)
      });

      expect(response.status).toBe(200);
      const data = (await response.json()) as ChatCompletionResponse;
      expect(data.choices[0].message).toBeDefined();
      expect(data.choices[0].message.content).toBe("4");
      expect(data.choices[0].finish_reason).toBe("stop");
    });
  });
}); 