import type { ApiFormat } from '@bodhiapp/ts-client';
import { Agent } from '@mariozechner/pi-agent-core';
import type { AgentEvent, AgentMessage, AgentTool, StreamFn } from '@mariozechner/pi-agent-core';
import { streamSimple } from '@mariozechner/pi-ai';
import type { Model } from '@mariozechner/pi-ai';
import { create } from 'zustand';

import { nanoid } from '@/lib/utils';
import { extractTextFromAgentMessage } from '@/types/chat';

import { getCurrentChat, getDefaultAgentMessageUsage } from './agentStoreUtils';
import { useChatSettingsStore } from './chatSettingsStore';
import { useChatStore } from './chatStore';

// Sentinel key for pi-ai SDK: real auth uses session cookies or BodhiApp API token; middleware strips this.
export const SENTINEL_API_KEY = 'bodhiapp_sentinel_api_key_ignored';

type PiApi = 'openai-completions' | 'openai-responses' | 'anthropic-messages' | 'google-generative-ai';

function apiFormatToPiApi(apiFormat: ApiFormat, provider?: string | null): PiApi {
  switch (apiFormat) {
    case 'openai_responses':
      return 'openai-responses';
    case 'anthropic':
    case 'anthropic_oauth':
      return 'anthropic-messages';
    case 'gemini':
      return 'google-generative-ai';
    case 'llm_liberty_oauth':
      if (provider === 'anthropic') return 'anthropic-messages';
      if (provider === 'openai-codex') return 'openai-responses';
      return 'openai-completions';
    default:
      return 'openai-completions';
  }
}

function apiFormatToProvider(apiFormat: ApiFormat, provider?: string | null): string {
  if (apiFormat === 'anthropic' || apiFormat === 'anthropic_oauth') return 'anthropic';
  if (apiFormat === 'gemini') return 'google';
  if (apiFormat === 'llm_liberty_oauth' && provider === 'anthropic') return 'anthropic';
  if (apiFormat === 'llm_liberty_oauth' && provider === 'openai-codex') return 'openai';
  return 'openai';
}

function buildModel(
  modelId: string,
  baseUrl: string,
  apiFormat: ApiFormat = 'openai',
  provider?: string | null
): Model<PiApi> {
  return {
    id: modelId,
    name: modelId,
    api: apiFormatToPiApi(apiFormat, provider),
    provider: apiFormatToProvider(apiFormat, provider),
    baseUrl,
    reasoning: false,
    input: ['text'],
    cost: { input: 0, output: 0, cacheRead: 0, cacheWrite: 0 },
    contextWindow: 128000,
    // 0 disables pi-ai's model.maxTokens fallback in buildBaseOptions; the user-facing toggle is authoritative.
    maxTokens: 0,
  };
}

function createBodhiStreamFn(getApiToken: () => string | undefined): StreamFn {
  return (model, context, options) => {
    const apiToken = getApiToken();
    const settings = useChatSettingsStore.getState();
    // API token overrides headers; without one, sentinel key is sent and stripped by middleware.
    const headers = apiToken
      ? { ...model.headers, Authorization: `Bearer ${apiToken}`, 'x-api-key': apiToken }
      : model.headers;
    const patchedModel = headers ? { ...model, headers } : model;
    const maxTokens = settings.max_tokens_enabled && settings.max_tokens != null ? settings.max_tokens : undefined;
    return streamSimple(patchedModel, context, maxTokens !== undefined ? { ...options, maxTokens } : options);
  };
}

function getBaseUrl(apiFormat: ApiFormat = 'openai', provider?: string | null): string {
  const origin = typeof window !== 'undefined' ? window.location.origin : 'http://localhost';
  // Anthropic SDK appends `/v1/messages`; Gemini SDK appends `/models/{id}:generateContent`.
  if (apiFormat === 'anthropic' || apiFormat === 'anthropic_oauth') return `${origin}/anthropic`;
  if (apiFormat === 'gemini') return `${origin}/v1beta`;
  if (apiFormat === 'llm_liberty_oauth' && provider === 'anthropic') return `${origin}/anthropic`;
  if (apiFormat === 'llm_liberty_oauth' && provider === 'openai-codex') return `${origin}/v1`;
  return `${origin}/v1`;
}

export interface AgentStoreState {
  input: string;
  isStreaming: boolean;
  messages: AgentMessage[];
  streamingMessage: AgentMessage | undefined;
  pendingToolCalls: ReadonlySet<string>;
  errorMessage: string | undefined;
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

// Module-level singleton: lives outside Zustand because Agent manages its own streaming state and cannot be serialized.
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

const isAgentMessage = (x: unknown): x is AgentMessage =>
  typeof x === 'object' && x !== null && 'role' in x && 'api' in x && 'provider' in x;

async function restoreMessagesForChat(): Promise<void> {
  const chatStore = useChatStore.getState();
  const settingsStore = useChatSettingsStore.getState();
  const currentChat = getCurrentChat(chatStore);

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
  const llmLibertyProvider = settingsStore.llmLibertyProvider;

  const restored: AgentMessage[] = messages.flatMap((m): AgentMessage[] => {
    if (m.role === 'user') {
      return [{ role: 'user' as const, content: m.content, timestamp: Date.now() }];
    }
    try {
      const parsed = JSON.parse(m.content);
      if (isAgentMessage(parsed)) {
        return [parsed];
      }
      if (parsed && typeof parsed === 'object') {
        console.warn(
          'restoreMessagesForChat: dropping malformed assistant message missing required fields (api, provider)',
          parsed
        );
        return [];
      }
      throw new Error('invalid shape');
    } catch {
      return [
        {
          role: 'assistant' as const,
          content: [{ type: 'text' as const, text: m.content }],
          api: apiFormatToPiApi(apiFormat, llmLibertyProvider),
          provider: apiFormatToProvider(apiFormat, llmLibertyProvider),
          model: modelId,
          usage: getDefaultAgentMessageUsage(),
          stopReason: 'stop' as const,
          timestamp: Date.now(),
        },
      ];
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
    const baseUrl = getBaseUrl(settingsStore.apiFormat, settingsStore.llmLibertyProvider);
    const model = buildModel(modelId, baseUrl, settingsStore.apiFormat, settingsStore.llmLibertyProvider);

    agent.state.model = model;
    agent.state.tools = tools;
    agent.state.systemPrompt =
      settingsStore.systemPrompt_enabled && settingsStore.systemPrompt ? settingsStore.systemPrompt : '';

    if (!get().chatIdRef) {
      const currentChat = getCurrentChat(chatStore);
      set({
        chatIdRef: {
          id: currentChat?.id ?? nanoid(),
          createdAt: currentChat?.createdAt ?? Date.now(),
        },
      });
    }

    // Restore messages if switching to existing chat and agent is empty
    const currentChat = getCurrentChat(chatStore);
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

          // Save settings before setCurrentChatId so the cross-store subscription finds persisted settings.
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
    const baseUrl = getBaseUrl(settings.apiFormat, settings.llmLibertyProvider);
    if (settings.model) {
      agent.state.model = buildModel(settings.model, baseUrl, settings.apiFormat, settings.llmLibertyProvider);
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
