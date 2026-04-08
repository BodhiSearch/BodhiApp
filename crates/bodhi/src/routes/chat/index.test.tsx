import ChatPage from '@/routes/chat/index';
import { mockAppInfoReady, mockAppInfoSetup } from '@/test-utils/msw-v2/handlers/info';
import { mockModels } from '@/test-utils/msw-v2/handlers/models';
import { mockUserLoggedIn, mockUserLoggedOut } from '@/test-utils/msw-v2/handlers/user';
import { server, setupMswV2 } from '@/test-utils/msw-v2/setup';
import { createWrapper } from '@/tests/wrapper';
import { act, render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { http, HttpResponse } from 'msw';

import type { AgentMessage } from '@mariozechner/pi-agent-core';

vi.mock('@/components/ui/markdown', () => ({
  MemoizedReactMarkdown: ({ children }: { children: string }) => <div>{children}</div>,
}));

vi.mock('@/hooks/use-mobile', () => ({
  useIsMobile: () => false,
}));

const navigateMock = vi.fn();
vi.mock('@tanstack/react-router', async () => {
  const actual = await vi.importActual('@tanstack/react-router');
  return {
    ...actual,
    Link: ({ to, children, ...rest }: any) => (
      <a href={to} {...rest}>
        {children}
      </a>
    ),
    useNavigate: () => navigateMock,
    useSearch: () => ({}),
  };
});

const toastMock = vi.fn();
vi.mock('@/hooks/use-toast', () => ({
  useToast: () => ({
    toast: toastMock,
  }),
}));

const mockChatModel = {
  alias: 'test-chat-model',
  family: 'test',
  repo: 'test/repo',
  filename: 'model.gguf',
  features: ['chat'],
  chat_template: 'test-template',
};

const mockPrompt = vi.fn();
const mockAbort = vi.fn();

let mockAgentMessages: AgentMessage[] = [];
let mockIsStreaming = false;
let mockStreamingMessage: AgentMessage | undefined = undefined;
let mockPendingToolCalls = new Set<string>();
let mockErrorMessage: string | undefined = undefined;

vi.mock('@/hooks/chat/useBodhiAgent', () => {
  const { useState } = require('react');
  return {
    useBodhiAgent: () => {
      const [input, setInput] = useState('');
      return {
        input,
        setInput,
        isStreaming: mockIsStreaming,
        messages: mockAgentMessages,
        streamingMessage: mockStreamingMessage,
        pendingToolCalls: mockPendingToolCalls,
        errorMessage: mockErrorMessage,
        append: mockPrompt,
        stop: mockAbort,
      };
    },
  };
});

vi.mock('@/hooks/chat/useMcpAgentTools', () => ({
  useMcpAgentTools: () => [],
}));

vi.mock('@/hooks/mcps/useMcpClients', () => ({
  useMcpClients: () => ({
    clients: new Map(),
    allTools: new Map(),
    connectAll: vi.fn(),
    disconnectAll: vi.fn(),
    callTool: vi.fn(),
    isConnecting: false,
  }),
}));

setupMswV2();

beforeEach(() => {
  navigateMock.mockClear();
  toastMock.mockClear();
  mockPrompt.mockClear();
  mockAbort.mockClear();
  localStorage.clear();
  mockAgentMessages = [];
  mockIsStreaming = false;
  mockStreamingMessage = undefined;
  mockPendingToolCalls = new Set();
  mockErrorMessage = undefined;
});

afterEach(() => {
  vi.restoreAllMocks();
});

function setupDefaultHandlers() {
  server.use(
    ...mockAppInfoReady(),
    ...mockUserLoggedIn(),
    ...mockModels({ data: [mockChatModel], total: 1 }),
    http.get('http://localhost:3000/bodhi/v1/mcps', () => HttpResponse.json({ mcps: [] }))
  );
}

describe('ChatPage', () => {
  it('redirects to /ui/setup if status is setup', async () => {
    server.use(...mockAppInfoSetup(), ...mockUserLoggedIn());

    await act(async () => {
      render(<ChatPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(navigateMock).toHaveBeenCalledWith({ to: '/setup' });
    });
  });

  it('redirects to /ui/login if user is not logged in', async () => {
    server.use(...mockAppInfoReady(), ...mockUserLoggedOut());

    render(<ChatPage />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(navigateMock).toHaveBeenCalledWith({ to: '/login' });
    });
  });

  it('renders empty chat state when no messages', async () => {
    setupDefaultHandlers();
    render(<ChatPage />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(screen.queryByTestId('chat-input')).toBeInTheDocument();
    });

    expect(screen.getByTestId('empty-chat-state')).toBeInTheDocument();
  });

  it('requires model selection before sending messages', async () => {
    setupDefaultHandlers();
    render(<ChatPage />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(screen.queryByTestId('chat-input')).toBeInTheDocument();
    });

    const sendButton = screen.getByTestId('send-button');
    expect(sendButton).toBeDisabled();
  });

  it('calls append when user submits a message with model selected', async () => {
    const user = userEvent.setup();
    setupDefaultHandlers();

    mockPrompt.mockResolvedValue(undefined);

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
      expect(mockPrompt).toHaveBeenCalled();
    });

    const callArg = mockPrompt.mock.calls[0][0];
    expect(callArg).toBe('Hello AI');
  });

  it('displays agent messages after prompt completes', async () => {
    setupDefaultHandlers();

    mockAgentMessages = [
      {
        role: 'user',
        content: 'Hello',
        timestamp: Date.now(),
      } as AgentMessage,
      {
        role: 'assistant',
        content: [{ type: 'text', text: 'Hi there!' }],
        api: 'openai-completions',
        provider: 'openai',
        model: 'test-chat-model',
        usage: { inputTokens: 5, outputTokens: 3, cacheReadTokens: 0, cacheWriteTokens: 0 },
        stopReason: 'endTurn',
      } as AgentMessage,
    ];

    render(<ChatPage />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(screen.queryByTestId('chat-input')).toBeInTheDocument();
    });

    await waitFor(() => {
      expect(screen.getByTestId('user-message')).toBeInTheDocument();
    });

    const assistantMessages = screen.getAllByTestId('assistant-message');
    expect(assistantMessages.length).toBe(1);

    const content = assistantMessages[0].querySelector('[data-testid="assistant-message-content"]');
    expect(content).toHaveTextContent('Hi there!');
  });

  it('displays metadata (usage tokens) for assistant messages', async () => {
    setupDefaultHandlers();

    mockAgentMessages = [
      {
        role: 'user',
        content: 'Hello',
        timestamp: Date.now(),
      } as AgentMessage,
      {
        role: 'assistant',
        content: [{ type: 'text', text: 'Response with metadata.' }],
        api: 'openai-completions',
        provider: 'openai',
        model: 'test-chat-model',
        usage: {
          input: 15,
          output: 25,
          cacheRead: 0,
          cacheWrite: 0,
          totalTokens: 40,
          cost: { input: 0, output: 0, cacheRead: 0, cacheWrite: 0, total: 0 },
        },
        stopReason: 'stop',
        timestamp: Date.now(),
      } as AgentMessage,
    ];

    render(<ChatPage />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(screen.queryByTestId('chat-input')).toBeInTheDocument();
    });

    await waitFor(() => {
      expect(screen.getByTestId('message-metadata')).toBeInTheDocument();
    });

    const metadata = screen.getByTestId('message-metadata');
    expect(metadata).toHaveTextContent('Query: 15 tokens');
    expect(metadata).toHaveTextContent('Response: 25 tokens');
  });
});
