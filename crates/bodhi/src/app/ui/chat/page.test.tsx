/**
 * ChatPage Component Tests
 *
 * Purpose: Verify type migration to @bodhiapp/ts-client works correctly with comprehensive
 * scenario-based testing covering real-world chat usage patterns.
 *
 * Focus Areas:
 * - CreateChatCompletionResponse/StreamResponse type parsing from generated types
 * - API response metadata extraction (usage, timings) with llama.cpp extensions
 * - UI metadata display (tokens, speed) with conditional rendering
 * - Multi-turn conversations with context preservation across turns
 * - Chat history persistence and switching between conversations
 * - Error handling and state recovery (API errors, streaming errors)
 * - Optional field handling (standard OpenAI vs llama.cpp-specific fields)
 *
 * Test Coverage Structure:
 * 1. Baseline: Setup and authentication redirects (2 tests)
 * 2. Core Response Types: Non-streaming, streaming, validation (3 tests)
 * 3. Real Usage: Multi-turn conversations, history, optional fields (3 tests)
 * 4. Error Scenarios: API error with retry, streaming error with partial content (2 tests)
 *
 * Total: 10 comprehensive scenario-based tests
 *
 * Production Issues Fixed During Implementation:
 * - use-chat.tsx: Fixed non-streaming content bug where currentAssistantMessage (empty for
 *   non-streaming) was used instead of message.content, causing empty responses
 * - ChatMessage.tsx: Added streaming-message testid for streaming state detection
 *
 * Key Technical Patterns:
 * - Wait for persistent chat history, not ephemeral streaming state
 * - Update MSW handlers between turns for multi-turn conversations
 * - Use queryAllByTestId for dynamic message counts
 * - Verify metadata display conditional on optional fields
 * - Test input preservation on API errors for retry capability
 * - Test partial content preservation on streaming errors
 */

import ChatPage from '@/app/ui/chat/page';
import {
  mockChatModel,
  mockNonStreamingResponse,
  mockStandardOpenAIResponse,
  mockStreamingChunks,
} from '@/test-utils/fixtures/chat';
import {
  mockChatCompletions,
  mockChatCompletionsError,
  mockChatCompletionsStreaming,
  mockChatCompletionsStreamingWithError,
} from '@/test-utils/msw-v2/handlers/chat-completions';
import { mockAppInfoReady, mockAppInfoSetup } from '@/test-utils/msw-v2/handlers/info';
import { mockModels } from '@/test-utils/msw-v2/handlers/models';
import { mockUserLoggedIn, mockUserLoggedOut } from '@/test-utils/msw-v2/handlers/user';
import { server, setupMswV2 } from '@/test-utils/msw-v2/setup';
import { createWrapper } from '@/tests/wrapper';
import { act, render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, it, vi } from 'vitest';

// Mock the components
vi.mock('@/components/chat/ChatContainer', () => ({
  ChatContainer: () => <div data-testid="chat-container">Chat Content</div>,
}));

// Mock markdown component
vi.mock('@/components/ui/markdown', () => ({
  MemoizedReactMarkdown: ({ children }: { children: string }) => <div>{children}</div>,
}));

// Mock use-mobile hook
vi.mock('@/hooks/use-mobile', () => ({
  useIsMobile: () => false,
}));

const pushMock = vi.fn();
vi.mock('next/navigation', () => ({
  useRouter: () => ({
    push: pushMock,
  }),
  useSearchParams: () => null,
}));

const toastMock = vi.fn();
vi.mock('@/hooks/use-toast', () => ({
  useToast: () => ({
    toast: toastMock,
  }),
}));

setupMswV2();

beforeEach(() => {
  pushMock.mockClear();
  toastMock.mockClear();
  localStorage.clear();
});
afterEach(() => {
  vi.resetAllMocks();
});

describe('ChatPage', () => {
  it('redirects to /ui/setup if status is setup', async () => {
    server.use(...mockAppInfoSetup(), ...mockUserLoggedIn());

    await act(async () => {
      render(<ChatPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(pushMock).toHaveBeenCalledWith('/ui/setup');
    });
  });

  it('redirects to /ui/login if user is not logged in', async () => {
    server.use(...mockAppInfoReady(), ...mockUserLoggedOut());

    render(<ChatPage />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(pushMock).toHaveBeenCalledWith('/ui/login');
    });
  });

  it('displays non-streaming response with full metadata (tokens and speed)', async () => {
    const user = userEvent.setup();
    server.use(
      ...mockAppInfoReady(),
      ...mockUserLoggedIn(),
      ...mockModels({ data: [mockChatModel], total: 1 }),
      ...mockChatCompletions({
        request: { messages: [{ role: 'user', content: 'Hello AI' }] },
        response: mockNonStreamingResponse,
      })
    );
    render(<ChatPage />, { wrapper: createWrapper() });
    await waitFor(() => {
      expect(screen.queryByTestId('chat-input')).toBeInTheDocument();
    });
    const modelSelector = await screen.findByTestId('model-selector-trigger');
    await user.click(modelSelector);
    const modelOption = await screen.findByRole('option', { name: mockChatModel.alias });
    await user.click(modelOption);
    const chatInput = screen.getByTestId('chat-input');
    await user.type(chatInput, 'Hello AI');
    const sendButton = screen.getByTestId('send-button');
    await user.click(sendButton);

    await waitFor(() => {
      expect(screen.getByTestId('user-message')).toBeInTheDocument();
    });

    await waitFor(() => {
      const assistantMessages = screen.queryAllByTestId('assistant-message');
      expect(assistantMessages.length).toBeGreaterThan(0);
    });

    await waitFor(() => {
      const assistantMessages = screen.getAllByTestId('assistant-message');
      const lastAssistantMessage = assistantMessages[assistantMessages.length - 1];
      const contentElement = lastAssistantMessage.querySelector('[data-testid="assistant-message-content"]');
      expect(contentElement?.textContent).toBeTruthy();
    });

    const assistantMessages = screen.getAllByTestId('assistant-message');
    const lastAssistantMessage = assistantMessages[assistantMessages.length - 1];
    const assistantMessageContent = lastAssistantMessage.querySelector('[data-testid="assistant-message-content"]');
    expect(assistantMessageContent).toHaveTextContent('This is a detailed test response from the AI assistant.');

    const metadata = screen.getByTestId('message-metadata');
    expect(metadata).toHaveTextContent('Query: 15 tokens');
    expect(metadata).toHaveTextContent('Response: 25 tokens');
    expect(metadata).toHaveTextContent('Speed: 180.30 t/s');
  });

  it('displays streaming response with progressive content and final metadata', async () => {
    const user = userEvent.setup();

    server.use(
      ...mockAppInfoReady(),
      ...mockUserLoggedIn(),
      ...mockModels({ data: [mockChatModel], total: 1 }),
      ...mockChatCompletionsStreaming({ chunks: mockStreamingChunks })
    );

    render(<ChatPage />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(screen.queryByTestId('chat-input')).toBeInTheDocument();
    });

    const modelSelector = await screen.findByTestId('model-selector-trigger');
    await user.click(modelSelector);

    const modelOption = await screen.findByRole('option', { name: mockChatModel.alias });
    await user.click(modelOption);

    const chatInput = screen.getByTestId('chat-input');
    await user.type(chatInput, 'Test streaming');

    const sendButton = screen.getByTestId('send-button');
    await user.click(sendButton);

    await waitFor(() => {
      expect(screen.getAllByTestId('user-message').length).toBeGreaterThan(0);
    });

    // For streaming tests, we check that the final message appears in chat history
    // The streaming-message is ephemeral and may not be caught in tests due to timing
    await waitFor(() => {
      const assistantMessages = screen.queryAllByTestId('assistant-message');
      expect(assistantMessages.length).toBeGreaterThan(0);
    });

    // Verify content and metadata
    const assistantMessages = screen.getAllByTestId('assistant-message');
    const lastAssistantMessage = assistantMessages[assistantMessages.length - 1];
    const assistantMessageContent = lastAssistantMessage.querySelector('[data-testid="assistant-message-content"]');
    expect(assistantMessageContent).toHaveTextContent('Hello there! How can I help?');

    const metadata = screen.getByTestId('message-metadata');
    expect(metadata).toHaveTextContent('Query: 10 tokens');
    expect(metadata).toHaveTextContent('Response: 20 tokens');
    expect(metadata).toHaveTextContent('Speed: 195.70 t/s');
  });

  it('handles multi-turn conversation with metadata preserved across turns', async () => {
    const user = userEvent.setup();
    const turn1Response = {
      ...mockNonStreamingResponse,
      choices: [
        {
          index: 0,
          message: {
            role: 'resource_admin' as any,
            content: 'Hello! I am happy to help.',
          },
          finish_reason: 'stop' as const,
        },
      ],
      usage: {
        prompt_tokens: 5,
        completion_tokens: 10,
        total_tokens: 15,
      },
      timings: {
        prompt_per_second: 300.0,
        predicted_per_second: 200.0,
      },
    };

    const turn2Response = {
      ...mockNonStreamingResponse,
      choices: [
        {
          index: 0,
          message: {
            role: 'resource_admin' as any,
            content: 'React is a JavaScript library for building user interfaces.',
          },
          finish_reason: 'stop' as const,
        },
      ],
      usage: {
        prompt_tokens: 20,
        completion_tokens: 15,
        total_tokens: 35,
      },
      timings: {
        prompt_per_second: 280.0,
        predicted_per_second: 190.0,
      },
    };

    const turn3Response = {
      ...mockNonStreamingResponse,
      choices: [
        {
          index: 0,
          message: {
            role: 'resource_admin' as any,
            content: 'Yes, React uses a virtual DOM for efficient updates.',
          },
          finish_reason: 'stop' as const,
        },
      ],
      usage: {
        prompt_tokens: 35,
        completion_tokens: 12,
        total_tokens: 47,
      },
      timings: {
        prompt_per_second: 290.0,
        predicted_per_second: 185.0,
      },
    };
    server.use(...mockAppInfoReady(), ...mockUserLoggedIn(), ...mockModels({ data: [mockChatModel], total: 1 }));
    render(<ChatPage />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(screen.queryByTestId('chat-input')).toBeInTheDocument();
    });

    const modelSelector = await screen.findByTestId('model-selector-trigger');
    await user.click(modelSelector);

    const modelOption = await screen.findByRole('option', { name: mockChatModel.alias });
    await user.click(modelOption);

    const chatInput = screen.getByTestId('chat-input');
    const sendButton = screen.getByTestId('send-button');

    server.use(
      ...mockChatCompletions({
        request: { messages: [{ role: 'user', content: 'Hello' }] },
        response: turn1Response,
      })
    );
    await user.type(chatInput, 'Hello');
    await user.click(sendButton);

    await waitFor(() => {
      const assistantMessages = screen.queryAllByTestId('assistant-message');
      expect(assistantMessages.length).toBeGreaterThan(0);
    });

    let assistantMessages = screen.getAllByTestId('assistant-message');
    expect(assistantMessages[0].textContent).toContain('Hello! I am happy to help.');

    server.use(
      ...mockChatCompletions({
        request: {
          messages: [
            { role: 'user', content: 'Hello' },
            { role: 'assistant', content: 'Hello! I am happy to help.' },
            { role: 'user', content: 'What is React?' },
          ],
        },
        response: turn2Response,
      })
    );
    await user.type(chatInput, 'What is React?');
    await user.click(sendButton);

    await waitFor(() => {
      const msgs = screen.queryAllByTestId('assistant-message');
      expect(msgs.length).toBe(2);
    });

    assistantMessages = screen.getAllByTestId('assistant-message');
    expect(assistantMessages[1].textContent).toContain('React is a JavaScript library');

    server.use(
      ...mockChatCompletions({
        request: {
          messages: [
            { role: 'user', content: 'Hello' },
            { role: 'assistant', content: 'Hello! I am happy to help.' },
            { role: 'user', content: 'What is React?' },
            { role: 'assistant', content: 'React is a JavaScript library for building user interfaces.' },
            { role: 'user', content: 'Does it use virtual DOM?' },
          ],
        },
        response: turn3Response,
      })
    );
    await user.type(chatInput, 'Does it use virtual DOM?');
    await user.click(sendButton);

    await waitFor(() => {
      const msgs = screen.queryAllByTestId('assistant-message');
      expect(msgs.length).toBe(3);
    });

    assistantMessages = screen.getAllByTestId('assistant-message');
    expect(assistantMessages[2].textContent).toContain('Yes, React uses a virtual DOM');

    const allMetadata = screen.getAllByTestId('message-metadata');
    expect(allMetadata.length).toBe(3);
    expect(allMetadata[0]).toHaveTextContent('Query: 5 tokens');
    expect(allMetadata[0]).toHaveTextContent('Response: 10 tokens');
    expect(allMetadata[1]).toHaveTextContent('Query: 20 tokens');
    expect(allMetadata[1]).toHaveTextContent('Response: 15 tokens');
    expect(allMetadata[2]).toHaveTextContent('Query: 35 tokens');
    expect(allMetadata[2]).toHaveTextContent('Response: 12 tokens');
  });

  it('persists chat history and restores messages when switching chats', async () => {
    const user = userEvent.setup();
    server.use(...mockAppInfoReady(), ...mockUserLoggedIn(), ...mockModels({ data: [mockChatModel], total: 1 }));

    const chat1Response1 = {
      ...mockNonStreamingResponse,
      choices: [
        {
          index: 0,
          message: {
            role: 'resource_admin' as any,
            content: 'First chat, first response.',
          },
          finish_reason: 'stop' as const,
        },
      ],
      usage: {
        prompt_tokens: 10,
        completion_tokens: 8,
        total_tokens: 18,
      },
    };

    const chat1Response2 = {
      ...mockNonStreamingResponse,
      choices: [
        {
          index: 0,
          message: {
            role: 'resource_admin' as any,
            content: 'First chat, second response.',
          },
          finish_reason: 'stop' as const,
        },
      ],
      usage: {
        prompt_tokens: 20,
        completion_tokens: 10,
        total_tokens: 30,
      },
    };

    const chat2Response = {
      ...mockNonStreamingResponse,
      choices: [
        {
          index: 0,
          message: {
            role: 'resource_admin' as any,
            content: 'Second chat, first response.',
          },
          finish_reason: 'stop' as const,
        },
      ],
      usage: {
        prompt_tokens: 12,
        completion_tokens: 9,
        total_tokens: 21,
      },
    };

    render(<ChatPage />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(screen.queryByTestId('chat-input')).toBeInTheDocument();
    });

    const modelSelector = await screen.findByTestId('model-selector-trigger');
    await user.click(modelSelector);

    const modelOption = await screen.findByRole('option', { name: mockChatModel.alias });
    await user.click(modelOption);

    const chatInput = screen.getByTestId('chat-input');
    const sendButton = screen.getByTestId('send-button');

    server.use(
      ...mockChatCompletions({
        request: { messages: [{ role: 'user', content: 'Message 1' }] },
        response: chat1Response1,
      })
    );
    await user.type(chatInput, 'Message 1');
    await user.click(sendButton);

    await waitFor(() => {
      expect(screen.queryAllByTestId('assistant-message').length).toBe(1);
    });

    server.use(
      ...mockChatCompletions({
        request: {
          messages: [
            { role: 'user', content: 'Message 1' },
            { role: 'assistant', content: 'First chat, first response.' },
            { role: 'user', content: 'Message 2' },
          ],
        },
        response: chat1Response2,
      })
    );
    await user.type(chatInput, 'Message 2');
    await user.click(sendButton);

    await waitFor(() => {
      expect(screen.queryAllByTestId('assistant-message').length).toBe(2);
    });

    let assistantMessages = screen.getAllByTestId('assistant-message');
    expect(assistantMessages[0].textContent).toContain('First chat, first response');
    expect(assistantMessages[1].textContent).toContain('First chat, second response');

    const newChatButton = screen.getByTestId('new-chat-button');
    await user.click(newChatButton);

    await waitFor(() => {
      expect(screen.queryByTestId('empty-chat-state')).toBeInTheDocument();
    });

    expect(screen.queryAllByTestId('assistant-message').length).toBe(0);

    server.use(
      ...mockChatCompletions({
        request: { messages: [{ role: 'user', content: 'New chat message' }] },
        response: chat2Response,
      })
    );
    await user.type(chatInput, 'New chat message');
    await user.click(sendButton);

    await waitFor(() => {
      expect(screen.queryAllByTestId('assistant-message').length).toBe(1);
    });

    assistantMessages = screen.getAllByTestId('assistant-message');
    expect(assistantMessages[0].textContent).toContain('Second chat, first response');

    const chatHistoryContainer = screen.getByTestId('chat-history-container');
    const chatHistoryButtons = chatHistoryContainer.querySelectorAll('button[data-testid^="chat-history-button-"]');
    expect(chatHistoryButtons.length).toBeGreaterThanOrEqual(2);

    await user.click(chatHistoryButtons[1] as HTMLElement);

    await waitFor(() => {
      expect(screen.queryAllByTestId('assistant-message').length).toBe(2);
    });

    assistantMessages = screen.getAllByTestId('assistant-message');
    expect(assistantMessages[0].textContent).toContain('First chat, first response');
    expect(assistantMessages[1].textContent).toContain('First chat, second response');
  });

  it('displays response without timings field (standard OpenAI format)', async () => {
    const user = userEvent.setup();

    server.use(...mockAppInfoReady());
    server.use(...mockUserLoggedIn());
    server.use(...mockModels({ data: [mockChatModel], total: 1 }));
    server.use(
      ...mockChatCompletions({
        request: { messages: [{ role: 'user', content: 'Test OpenAI' }] },
        response: { ...mockStandardOpenAIResponse, timings: undefined },
      })
    );

    render(<ChatPage />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(screen.queryByTestId('chat-input')).toBeInTheDocument();
    });

    const modelSelector = await screen.findByTestId('model-selector-trigger');
    await user.click(modelSelector);

    const modelOption = await screen.findByRole('option', { name: mockChatModel.alias });
    await user.click(modelOption);

    const chatInput = screen.getByTestId('chat-input');
    await user.type(chatInput, 'Test OpenAI');

    const sendButton = screen.getByTestId('send-button');
    await user.click(sendButton);

    await waitFor(() => {
      const assistantMessages = screen.queryAllByTestId('assistant-message');
      expect(assistantMessages.length).toBeGreaterThan(0);
    });

    const assistantMessages = screen.getAllByTestId('assistant-message');
    const lastAssistantMessage = assistantMessages[assistantMessages.length - 1];
    const assistantMessageContent = lastAssistantMessage.querySelector('[data-testid="assistant-message-content"]');
    expect(assistantMessageContent).toHaveTextContent('Standard OpenAI response without timings.');

    const metadata = screen.getByTestId('message-metadata');
    expect(metadata).toHaveTextContent('Query: 10 tokens');
    expect(metadata).toHaveTextContent('Response: 5 tokens');
    expect(metadata).not.toHaveTextContent('Speed:');
  });

  it('requires model selection before sending messages', async () => {
    const user = userEvent.setup();

    server.use(...mockAppInfoReady());
    server.use(...mockUserLoggedIn());
    server.use(...mockModels({ data: [mockChatModel], total: 1 }));

    render(<ChatPage />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(screen.queryByTestId('chat-input')).toBeInTheDocument();
    });

    const sendButton = screen.getByTestId('send-button');
    expect(sendButton).toBeDisabled();

    const modelSelector = await screen.findByTestId('model-selector-trigger');
    await user.click(modelSelector);

    const modelOption = await screen.findByRole('option', { name: mockChatModel.alias });
    await user.click(modelOption);

    const chatInput = screen.getByTestId('chat-input');
    await user.type(chatInput, 'Test message');

    await waitFor(() => {
      expect(sendButton).not.toBeDisabled();
    });
  });

  it('handles API error with input restoration for retry', async () => {
    const user = userEvent.setup();

    server.use(...mockAppInfoReady());
    server.use(...mockUserLoggedIn());
    server.use(...mockModels({ data: [mockChatModel], total: 1 }));
    server.use(
      ...mockChatCompletionsError({ status: 500, code: 'internal_error', message: 'Model failed to respond' })
    );

    render(<ChatPage />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(screen.queryByTestId('chat-input')).toBeInTheDocument();
    });

    const modelSelector = await screen.findByTestId('model-selector-trigger');
    await user.click(modelSelector);

    const modelOption = await screen.findByRole('option', { name: mockChatModel.alias });
    await user.click(modelOption);

    const chatInput = screen.getByTestId('chat-input');
    const originalMessage = 'Hello, this should fail';
    await user.type(chatInput, originalMessage);

    const sendButton = screen.getByTestId('send-button');
    await user.click(sendButton);

    await waitFor(() => {
      expect(toastMock).toHaveBeenCalledWith(
        expect.objectContaining({
          title: 'Error',
          description: expect.stringContaining('Model failed to respond'),
        })
      );
    });

    expect(chatInput).toHaveValue(originalMessage);

    expect(screen.queryByTestId('user-message')).not.toBeInTheDocument();

    expect(sendButton).not.toBeDisabled();

    server.use(
      ...mockChatCompletions({
        request: { messages: [{ role: 'user', content: originalMessage }] },
        response: mockNonStreamingResponse,
      })
    );

    await user.click(sendButton);

    await waitFor(() => {
      const assistantMessages = screen.queryAllByTestId('assistant-message');
      expect(assistantMessages.length).toBeGreaterThan(0);
    });

    const assistantMessages = screen.getAllByTestId('assistant-message');
    const lastAssistantMessage = assistantMessages[assistantMessages.length - 1];
    const assistantMessageContent = lastAssistantMessage.querySelector('[data-testid="assistant-message-content"]');
    expect(assistantMessageContent).toHaveTextContent('This is a detailed test response from the AI assistant.');
  });

  it('handles streaming error with partial response preserved', async () => {
    const user = userEvent.setup();

    server.use(...mockAppInfoReady());
    server.use(...mockUserLoggedIn());
    server.use(...mockModels({ data: [mockChatModel], total: 1 }));

    const initialChunks = [
      '{"choices":[{"delta":{"content":"Partial "}}]}',
      '{"choices":[{"delta":{"content":"response "}}]}',
      '{"choices":[{"delta":{"content":"before "}}]}',
      '{"choices":[{"delta":{"content":"error"}}]}',
    ];

    server.use(...mockChatCompletionsStreamingWithError({ initialChunks, errorMessage: 'Stream interrupted' }));

    render(<ChatPage />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(screen.queryByTestId('chat-input')).toBeInTheDocument();
    });

    const modelSelector = await screen.findByTestId('model-selector-trigger');
    await user.click(modelSelector);

    const modelOption = await screen.findByRole('option', { name: mockChatModel.alias });
    await user.click(modelOption);

    const chatInput = screen.getByTestId('chat-input');
    await user.type(chatInput, 'Test streaming error');

    const sendButton = screen.getByTestId('send-button');
    await user.click(sendButton);

    await waitFor(() => {
      expect(screen.getByTestId('user-message')).toBeInTheDocument();
    });

    await waitFor(() => {
      const assistantMessages = screen.queryAllByTestId('assistant-message');
      expect(assistantMessages.length).toBeGreaterThan(0);
    });

    const assistantMessages = screen.getAllByTestId('assistant-message');
    const lastAssistantMessage = assistantMessages[assistantMessages.length - 1];
    const assistantMessageContent = lastAssistantMessage.querySelector('[data-testid="assistant-message-content"]');
    expect(assistantMessageContent?.textContent).toContain('Partial response before error');

    const metadata = screen.queryByTestId('message-metadata');
    if (metadata) {
      expect(metadata.textContent).not.toContain('Query:');
      expect(metadata.textContent).not.toContain('Response:');
      expect(metadata.textContent).not.toContain('Speed:');
    }

    expect(chatInput).not.toBeDisabled();
    expect(chatInput).toHaveValue('');

    await user.type(chatInput, 'Can continue chatting');
    await waitFor(() => {
      expect(sendButton).not.toBeDisabled();
    });
  });
});
