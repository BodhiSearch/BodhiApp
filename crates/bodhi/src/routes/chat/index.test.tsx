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
import { useAgentStore } from '@/stores/agentStore';
import { useChatStore } from '@/stores/chatStore';
import { useChatSettingsStore } from '@/stores/chatSettingsStore';

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

vi.mock('@/stores/initStores', () => ({
  initChatStoreSubscriptions: vi.fn(),
}));

const mockPrompt = vi.fn();
const mockAbort = vi.fn();

vi.mock('@/stores/chatStore', () => {
  const { create } = require('zustand');
  return {
    useChatStore: create(() => ({
      chats: [],
      currentChatId: null,
      isLoaded: true,
      userId: 'default',
      loadChats: vi.fn(),
      setCurrentChatId: vi.fn(),
      createNewChat: vi.fn(),
      createOrUpdateChat: vi.fn(),
      deleteChat: vi.fn(),
      clearChats: vi.fn(),
      getChat: vi.fn(),
      saveChatSettings: vi.fn(),
      getChatSettings: vi.fn(),
    })),
  };
});

vi.mock('@/stores/chatSettingsStore', () => {
  const { create } = require('zustand');
  return {
    useChatSettingsStore: create(() => ({
      model: '',
      apiFormat: 'openai',
      stream: true,
      stream_enabled: true,
      temperature_enabled: false,
      top_p_enabled: false,
      n_enabled: false,
      max_tokens_enabled: false,
      presence_penalty_enabled: false,
      frequency_penalty_enabled: false,
      logit_bias_enabled: false,
      stop_enabled: false,
      seed_enabled: false,
      systemPrompt_enabled: false,
      response_format_enabled: false,
      api_token_enabled: false,
      maxToolIterations: 5,
      maxToolIterations_enabled: true,
      setModel: vi.fn(),
      setApiFormat: vi.fn(),
      setStream: vi.fn(),
      setStreamEnabled: vi.fn(),
      setTemperature: vi.fn(),
      setTemperatureEnabled: vi.fn(),
      setTopP: vi.fn(),
      setTopPEnabled: vi.fn(),
      setN: vi.fn(),
      setNEnabled: vi.fn(),
      setMaxTokens: vi.fn(),
      setMaxTokensEnabled: vi.fn(),
      setPresencePenalty: vi.fn(),
      setPresencePenaltyEnabled: vi.fn(),
      setFrequencyPenalty: vi.fn(),
      setFrequencyPenaltyEnabled: vi.fn(),
      setLogitBias: vi.fn(),
      setLogitBiasEnabled: vi.fn(),
      setStop: vi.fn(),
      setStopEnabled: vi.fn(),
      setSeed: vi.fn(),
      setSeedEnabled: vi.fn(),
      setSystemPrompt: vi.fn(),
      setSystemPromptEnabled: vi.fn(),
      setResponseFormat: vi.fn(),
      setResponseFormatEnabled: vi.fn(),
      setApiToken: vi.fn(),
      setApiTokenEnabled: vi.fn(),
      setMaxToolIterations: vi.fn(),
      setMaxToolIterationsEnabled: vi.fn(),
      getRequestSettings: () => ({}),
      reset: vi.fn(),
      loadForChat: vi.fn(),
      saveForChat: vi.fn(),
      setSetting: vi.fn(),
      setEnabled: vi.fn(),
    })),
  };
});

vi.mock('@/stores/agentStore', () => {
  const { create } = require('zustand');
  return {
    useAgentStore: create(() => ({
      input: '',
      isStreaming: false,
      messages: [],
      streamingMessage: undefined,
      pendingToolCalls: new Set(),
      errorMessage: undefined,
      setInput: vi.fn(),
      append: vi.fn(),
      stop: vi.fn(),
      reset: vi.fn(),
      syncAgentSettings: vi.fn(),
    })),
  };
});

vi.mock('@/stores/mcpSelectionStore', () => {
  const { create } = require('zustand');
  return {
    useMcpSelectionStore: create(() => ({
      enabledTools: {},
      hasChanges: false,
      toggleTool: vi.fn(),
      toggleMcp: vi.fn(),
      isMcpEnabled: () => false,
      isToolEnabled: () => false,
      getMcpCheckboxState: () => 'unchecked',
      setEnabledTools: vi.fn(),
      loadForChat: vi.fn(),
    })),
  };
});

const mockChatModel = {
  id: 'test-chat-model-id',
  alias: 'test-chat-model',
  source: 'user',
  repo: 'test/repo',
  filename: 'model.gguf',
  snapshot: 'main',
  model_params: {},
  request_params: {},
  context_params: [],
  created_at: '2024-01-01T00:00:00Z',
  updated_at: '2024-01-01T00:00:00Z',
};

setupMswV2();

beforeEach(() => {
  navigateMock.mockClear();
  toastMock.mockClear();
  mockPrompt.mockClear();
  mockAbort.mockClear();
  localStorage.clear();

  useAgentStore.setState({
    input: '',
    isStreaming: false,
    messages: [],
    streamingMessage: undefined,
    pendingToolCalls: new Set(),
    errorMessage: undefined,
    setInput: vi.fn((input: string) => useAgentStore.setState({ input })),
    append: mockPrompt,
    stop: mockAbort,
    reset: vi.fn(),
    syncAgentSettings: vi.fn(),
  });
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
      expect(navigateMock).toHaveBeenCalledWith({ to: '/setup/' });
    });
  });

  it('redirects to /ui/login if user is not logged in', async () => {
    server.use(...mockAppInfoReady(), ...mockUserLoggedOut());

    render(<ChatPage />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(navigateMock).toHaveBeenCalledWith({ to: '/login/' });
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

    // Set model directly in settings store so handleSubmit sees it
    useChatSettingsStore.setState({ model: mockChatModel.alias });

    // Make setInput actually update the store so handleSubmit can read the input
    useAgentStore.setState({
      setInput: vi.fn((input: string) => useAgentStore.setState({ input })),
      append: mockPrompt,
    });

    render(<ChatPage />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(screen.queryByTestId('chat-input')).toBeInTheDocument();
    });

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

    useAgentStore.setState({
      messages: [
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
          usage: {
            input: 5,
            output: 3,
            cacheRead: 0,
            cacheWrite: 0,
            totalTokens: 8,
            cost: { input: 0, output: 0, cacheRead: 0, cacheWrite: 0, total: 0 },
          },
          stopReason: 'stop',
          timestamp: Date.now(),
        } as AgentMessage,
      ],
    });

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

    useAgentStore.setState({
      messages: [
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
      ],
    });

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
