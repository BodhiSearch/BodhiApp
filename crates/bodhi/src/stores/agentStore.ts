import { create } from 'zustand';

import { streamSimple } from '@mariozechner/pi-ai';
import type { Model } from '@mariozechner/pi-ai';
import { Agent } from '@mariozechner/pi-agent-core';
import type { AgentEvent, AgentMessage, AgentTool, StreamFn } from '@mariozechner/pi-agent-core';

import type { ApiFormat } from '@bodhiapp/ts-client';

import { useChatStore } from './chatStore';
import { useChatSettingsStore } from './chatSettingsStore';
import { nanoid } from '@/lib/utils';
import { extractTextFromAgentMessage } from '@/types/chat';

// pi-ai's SDK wrappers always send an auth header derived from `apiKey`.
// BodhiApp's chat UI authenticates through its own proxy using session cookies
// or a user-configured BodhiApp API token, so we hand pi-ai this sentinel and
// strip it server-side in the anthropic/openai auth middlewares.
export const SENTINEL_API_KEY = 'bodhiapp_sentinel_api_key_ignored';

type PiApi = 'openai-completions' | 'openai-responses' | 'anthropic-messages';

function apiFormatToPiApi(apiFormat: ApiFormat): PiApi {
  switch (apiFormat) {
    case 'openai_responses':
      return 'openai-responses';
    case 'anthropic':
    case 'anthropic_oauth':
      return 'anthropic-messages';
    default:
      return 'openai-completions';
  }
}

function apiFormatToProvider(apiFormat: ApiFormat): string {
  return apiFormat === 'anthropic' || apiFormat === 'anthropic_oauth' ? 'anthropic' : 'openai';
}

function buildModel(modelId: string, baseUrl: string, apiFormat: ApiFormat = 'openai'): Model<PiApi> {
  return {
    id: modelId,
    name: modelId,
    api: apiFormatToPiApi(apiFormat),
    provider: apiFormatToProvider(apiFormat),
    baseUrl,
    reasoning: false,
    input: ['text'],
    cost: { input: 0, output: 0, cacheRead: 0, cacheWrite: 0 },
    contextWindow: 128000,
    // pi-ai's google provider min-clamps to maxTokens; 0 produces invalid `maxOutputTokens: 0` in Gemini requests.
    maxTokens: 4096,
  };
}

function createBodhiStreamFn(getApiToken: () => string | undefined): StreamFn {
  return (model, context, options) => {
    const apiToken = getApiToken();
    const settings = useChatSettingsStore.getState();
    // With a BodhiApp API token, override both headers so it authenticates.
    // Without one, let the SDK send SENTINEL_API_KEY — middleware strips it
    // and session-cookie auth takes over.
    const headers = apiToken
      ? { ...model.headers, Authorization: `Bearer ${apiToken}`, 'x-api-key': apiToken }
      : model.headers;
    const patchedModel = headers ? { ...model, headers } : model;
    const maxTokens = settings.max_tokens_enabled && settings.max_tokens != null ? settings.max_tokens : undefined;
    return streamSimple(patchedModel, context, maxTokens !== undefined ? { ...options, maxTokens } : options);
  };
}

function getBaseUrl(apiFormat: ApiFormat = 'openai'): string {
  const origin = typeof window !== 'undefined' ? window.location.origin : 'http://localhost';
  // pi-ai's Anthropic provider uses the official @anthropic-ai/sdk which
  // appends `/v1/messages` to the configured baseURL, so we point it at
  // `/anthropic` so the final URL lands on BodhiApp's proxy endpoint
  // `/anthropic/v1/messages`.
  return apiFormat === 'anthropic' || apiFormat === 'anthropic_oauth' ? `${origin}/anthropic` : `${origin}/v1`;
}

export interface AgentStoreState {
  input: string;
  isStreaming: boolean;
  messages: AgentMessage[];
  streamingMessage: AgentMessage | undefined;
  pendingToolCalls: ReadonlySet<string>;
  errorMessage: string | undefined;
  /** Tracks the chat record the current agent session is writing to. */
  chatIdRef: { id: string; createdAt: number } | null;

  setInput: (input: string) => void;
  append: (
    content: string,
    options?: {
      tools?: AgentTool[];
      enabledMcpTools?: Record<string, string[]>;
      showError?: (title: string, msg: string) => void;
    }
  ) => Promise<void>;
  stop: () => void;
  reset: () => void;
  syncAgentSettings: (tools?: AgentTool[]) => void;
}

// Module-level singleton: one Agent instance per app session.
// Lives outside Zustand state because the Agent class manages its own
// internal streaming state and event subscriptions and cannot be serialized.
// Reset via the store's reset() action.
let _agent: Agent | null = null;
let _agentUnsubscribe: (() => void) | null = null;

function getOrCreateAgent(): Agent {
  if (!_agent) {
    const streamFn = createBodhiStreamFn(() => {
      const settings = useChatSettingsStore.getState();
      return settings.api_token_enabled ? settings.api_token : undefined;
    });
    _agent = new Agent({
      streamFn,
      getApiKey: () => SENTINEL_API_KEY,
    });
    _agentUnsubscribe?.();
    _agentUnsubscribe = _agent.subscribe((event: AgentEvent) => {
      const state = _agent!.state;
      const store = useAgentStore;

      switch (event.type) {
        case 'agent_start':
          store.setState({ isStreaming: true, errorMessage: undefined });
          break;
        case 'message_update':
          store.setState({
            messages: [...state.messages],
            streamingMessage: event.message,
            pendingToolCalls: new Set(state.pendingToolCalls),
          });
          break;
        case 'message_end':
          store.setState({ messages: [...state.messages], streamingMessage: undefined });
          break;
        case 'tool_execution_start':
          store.setState({ pendingToolCalls: new Set(state.pendingToolCalls) });
          break;
        case 'tool_execution_end':
          store.setState({ pendingToolCalls: new Set(state.pendingToolCalls) });
          break;
        case 'turn_end':
          store.setState({ messages: [...state.messages] });
          break;
        case 'agent_end':
          store.setState({
            messages: [...state.messages],
            streamingMessage: undefined,
            pendingToolCalls: new Set(),
            isStreaming: false,
            errorMessage: state.errorMessage,
          });
          break;
      }
    });
  }
  return _agent;
}

async function restoreMessagesForChat(): Promise<void> {
  const chatStore = useChatStore.getState();
  const settingsStore = useChatSettingsStore.getState();
  const currentChat = chatStore.currentChatId
    ? (chatStore.chats.find((c) => c.id === chatStore.currentChatId) ?? null)
    : null;

  if (!currentChat || currentChat.messageCount === 0) {
    useAgentStore.setState({ messages: [] });
    return;
  }

  const messages = await chatStore.loadMessagesForChat(currentChat.id);
  if (messages.length === 0) {
    useAgentStore.setState({ messages: [] });
    return;
  }

  const agent = getOrCreateAgent();
  const modelId = settingsStore.model || 'unknown';
  const apiFormat = settingsStore.apiFormat;

  const restored: AgentMessage[] = messages.map((m) => {
    if (m.role === 'user') {
      return { role: 'user' as const, content: m.content, timestamp: Date.now() };
    }
    try {
      const parsed = JSON.parse(m.content);
      if (parsed && typeof parsed === 'object' && 'role' in parsed) {
        return parsed as AgentMessage;
      }
      throw new Error('invalid shape');
    } catch {
      return {
        role: 'assistant' as const,
        content: [{ type: 'text' as const, text: m.content }],
        api: apiFormatToPiApi(apiFormat),
        provider: apiFormatToProvider(apiFormat),
        model: modelId,
        usage: {
          input: 0,
          output: 0,
          cacheRead: 0,
          cacheWrite: 0,
          totalTokens: 0,
          cost: { input: 0, output: 0, cacheRead: 0, cacheWrite: 0, total: 0 },
        },
        stopReason: 'stop' as const,
        timestamp: Date.now(),
      };
    }
  });

  agent.state.messages = restored;
  useAgentStore.setState({
    messages: [...restored],
    chatIdRef: { id: currentChat.id, createdAt: currentChat.createdAt },
  });
}

export const useAgentStore = create<AgentStoreState>((set, get) => ({
  input: '',
  isStreaming: false,
  messages: [],
  streamingMessage: undefined,
  pendingToolCalls: new Set<string>(),
  errorMessage: undefined,
  chatIdRef: null,

  setInput: (input) => set({ input }),

  append: async (content, options) => {
    const { tools = [], enabledMcpTools, showError } = options ?? {};
    const settingsStore = useChatSettingsStore.getState();
    const chatStore = useChatStore.getState();
    const modelId = settingsStore.model;

    if (!modelId) {
      showError?.('Error', 'Please select a model first.');
      return;
    }

    const agent = getOrCreateAgent();
    const baseUrl = getBaseUrl(settingsStore.apiFormat);
    const model = buildModel(modelId, baseUrl, settingsStore.apiFormat);

    agent.state.model = model;
    agent.state.tools = tools;
    agent.state.systemPrompt =
      settingsStore.systemPrompt_enabled && settingsStore.systemPrompt ? settingsStore.systemPrompt : '';

    if (!get().chatIdRef) {
      const currentChat = chatStore.currentChatId
        ? (chatStore.chats.find((c) => c.id === chatStore.currentChatId) ?? null)
        : null;
      set({
        chatIdRef: {
          id: currentChat?.id ?? nanoid(),
          createdAt: currentChat?.createdAt ?? Date.now(),
        },
      });
    }

    // Restore messages if switching to existing chat and agent is empty
    const currentChat = chatStore.currentChatId
      ? (chatStore.chats.find((c) => c.id === chatStore.currentChatId) ?? null)
      : null;
    if (currentChat && currentChat.messageCount > 0 && agent.state.messages.length === 0) {
      await restoreMessagesForChat();
    }

    set({ input: '' });

    try {
      await agent.prompt(content);
      if (agent.state.errorMessage) {
        showError?.('Error', agent.state.errorMessage);
        agent.state.messages = agent.state.messages.filter(
          (m) => !('stopReason' in m && (m as { stopReason: string }).stopReason === 'error')
        );
        set({ messages: [...agent.state.messages] });
      } else {
        // Save chat
        const ref = get().chatIdRef;
        if (ref) {
          const piMessages = agent.state.messages;
          const firstUserContent = piMessages.find((m) => 'role' in m && m.role === 'user');
          const titleText =
            firstUserContent && 'content' in firstUserContent
              ? typeof firstUserContent.content === 'string'
                ? firstUserContent.content.slice(0, 20)
                : extractTextFromAgentMessage(firstUserContent).slice(0, 20) || 'New Chat'
              : 'New Chat';

          const chatMessages = piMessages.map((m) => {
            if ('role' in m && m.role === 'user') {
              return {
                role: 'user' as const,
                content: typeof m.content === 'string' ? m.content : extractTextFromAgentMessage(m),
              };
            }
            return {
              role: 'assistant' as const,
              content: JSON.stringify(m),
            };
          });
          await chatStore.createOrUpdateChat({
            id: ref.id,
            title: titleText,
            messages: chatMessages,
            messageCount: chatMessages.length,
            createdAt: ref.createdAt,
            updatedAt: Date.now(),
            enabledMcpTools: enabledMcpTools && Object.keys(enabledMcpTools).length > 0 ? enabledMcpTools : undefined,
          });

          // Save settings BEFORE setting currentChatId, so the cross-store
          // subscription that fires on setCurrentChatId finds persisted settings.
          await useChatSettingsStore.getState().saveForChat(ref.id);

          if (!chatStore.currentChatId || chatStore.currentChatId !== ref.id) {
            chatStore.setCurrentChatId(ref.id);
          }
        }
      }
    } catch (error) {
      if (error instanceof Error && error.name === 'AbortError') {
        return;
      }
      const msg = error instanceof Error ? error.message : 'Error sending message to AI assistant.';
      showError?.('Error', msg);
      set({ errorMessage: msg });
    }
  },

  stop: () => {
    _agent?.abort();
  },

  reset: () => {
    _agentUnsubscribe?.();
    _agent?.abort();
    _agent = null;
    _agentUnsubscribe = null;
    set({
      input: '',
      isStreaming: false,
      messages: [],
      streamingMessage: undefined,
      pendingToolCalls: new Set(),
      errorMessage: undefined,
      chatIdRef: null,
    });

    void restoreMessagesForChat();
  },

  syncAgentSettings: (tools) => {
    const agent = getOrCreateAgent();
    const settings = useChatSettingsStore.getState();
    const baseUrl = getBaseUrl(settings.apiFormat);
    if (settings.model) {
      agent.state.model = buildModel(settings.model, baseUrl, settings.apiFormat);
    }
    if (tools) {
      agent.state.tools = tools;
    }
    agent.state.systemPrompt = settings.systemPrompt_enabled && settings.systemPrompt ? settings.systemPrompt : '';
  },
}));

// Cross-store subscription: reset agent when currentChatId changes
let _chatStoreUnsubscribe: (() => void) | null = null;
export function initAgentSubscription() {
  _chatStoreUnsubscribe?.();
  _chatStoreUnsubscribe = useChatStore.subscribe((state, prevState) => {
    if (state.currentChatId !== prevState.currentChatId) {
      useAgentStore.getState().reset();
    }
  });
}
