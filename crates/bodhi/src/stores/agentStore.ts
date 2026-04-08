import { create } from 'zustand';

import { streamSimple } from '@mariozechner/pi-ai';
import type { Model } from '@mariozechner/pi-ai';
import { Agent } from '@mariozechner/pi-agent-core';
import type { AgentEvent, AgentMessage, AgentTool, StreamFn } from '@mariozechner/pi-agent-core';

import { useChatStore } from './chatStore';
import { useChatSettingsStore } from './chatSettingsStore';
import type { ApiFormatSetting } from './chatSettingsStore';
import { nanoid } from '@/lib/utils';
import { extractTextFromAgentMessage } from '@/types/chat';

const DUMMY_API_KEY = 'bodhi-proxy';

type PiApi = 'openai-completions' | 'openai-responses';

function apiFormatToPiApi(apiFormat: ApiFormatSetting): PiApi {
  return apiFormat === 'openai_responses' ? 'openai-responses' : 'openai-completions';
}

function buildModel(modelId: string, baseUrl: string, apiFormat: ApiFormatSetting = 'openai'): Model<PiApi> {
  return {
    id: modelId,
    name: modelId,
    api: apiFormatToPiApi(apiFormat),
    provider: 'openai',
    baseUrl,
    reasoning: false,
    input: ['text'],
    cost: { input: 0, output: 0, cacheRead: 0, cacheWrite: 0 },
    contextWindow: 128000,
    maxTokens: 4096,
  };
}

function createBodhiStreamFn(getApiToken: () => string | undefined): StreamFn {
  return (model, context, options) => {
    const apiToken = getApiToken();
    const headers =
      apiToken !== undefined
        ? { ...model.headers, Authorization: `Bearer ${apiToken}` }
        : { ...model.headers, Authorization: null as unknown as string };
    const patchedModel = { ...model, headers };
    return streamSimple(patchedModel, context, options);
  };
}

function getBaseUrl(): string {
  const origin = typeof window !== 'undefined' ? window.location.origin : 'http://localhost';
  return `${origin}/v1`;
}

export interface AgentStoreState {
  input: string;
  isStreaming: boolean;
  messages: AgentMessage[];
  streamingMessage: AgentMessage | undefined;
  pendingToolCalls: ReadonlySet<string>;
  errorMessage: string | undefined;

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

let _agent: Agent | null = null;
let _agentUnsubscribe: (() => void) | null = null;
let _chatIdRef: { id: string; createdAt: number } | null = null;

function getOrCreateAgent(): Agent {
  if (!_agent) {
    const streamFn = createBodhiStreamFn(() => {
      const settings = useChatSettingsStore.getState();
      return settings.api_token_enabled ? settings.api_token : undefined;
    });
    _agent = new Agent({
      streamFn,
      getApiKey: () => DUMMY_API_KEY,
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
        provider: 'openai',
        model: modelId,
        usage: { inputTokens: 0, outputTokens: 0, cacheReadTokens: 0, cacheWriteTokens: 0 },
        stopReason: 'endTurn' as const,
      };
    }
  });

  agent.state.messages = restored;
  _chatIdRef = { id: currentChat.id, createdAt: currentChat.createdAt };
  useAgentStore.setState({ messages: [...restored] });
}

export const useAgentStore = create<AgentStoreState>((set, get) => ({
  input: '',
  isStreaming: false,
  messages: [],
  streamingMessage: undefined,
  pendingToolCalls: new Set<string>(),
  errorMessage: undefined,

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
    const baseUrl = getBaseUrl();
    const model = buildModel(modelId, baseUrl, settingsStore.apiFormat);

    agent.state.model = model;
    agent.state.tools = tools;
    agent.state.systemPrompt =
      settingsStore.systemPrompt_enabled && settingsStore.systemPrompt ? settingsStore.systemPrompt : '';

    if (!_chatIdRef) {
      const currentChat = chatStore.currentChatId
        ? (chatStore.chats.find((c) => c.id === chatStore.currentChatId) ?? null)
        : null;
      _chatIdRef = {
        id: currentChat?.id ?? nanoid(),
        createdAt: currentChat?.createdAt ?? Date.now(),
      };
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
        if (_chatIdRef) {
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
            id: _chatIdRef.id,
            title: titleText,
            messages: chatMessages,
            messageCount: chatMessages.length,
            createdAt: _chatIdRef.createdAt,
            updatedAt: Date.now(),
            enabledMcpTools: enabledMcpTools && Object.keys(enabledMcpTools).length > 0 ? enabledMcpTools : undefined,
          });

          // Save settings BEFORE setting currentChatId, so the cross-store
          // subscription that fires on setCurrentChatId finds persisted settings.
          await useChatSettingsStore.getState().saveForChat(_chatIdRef.id);

          if (!chatStore.currentChatId || chatStore.currentChatId !== _chatIdRef.id) {
            chatStore.setCurrentChatId(_chatIdRef.id);
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
    _chatIdRef = null;
    set({
      input: '',
      isStreaming: false,
      messages: [],
      streamingMessage: undefined,
      pendingToolCalls: new Set(),
      errorMessage: undefined,
    });

    void restoreMessagesForChat();
  },

  syncAgentSettings: (tools) => {
    const agent = getOrCreateAgent();
    const settings = useChatSettingsStore.getState();
    const baseUrl = getBaseUrl();
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
