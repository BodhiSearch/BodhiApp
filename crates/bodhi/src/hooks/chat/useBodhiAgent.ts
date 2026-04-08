import { useCallback, useEffect, useMemo, useRef, useState } from 'react';

import { streamSimple } from '@mariozechner/pi-ai';
import type { Model } from '@mariozechner/pi-ai';
import { Agent } from '@mariozechner/pi-agent-core';
import type { AgentEvent, AgentMessage, AgentTool, StreamFn } from '@mariozechner/pi-agent-core';

import { useChatDB } from './useChatDb';
import { useChatSettings } from './useChatSettings';
import type { ApiFormatSetting } from './useChatSettings';
import { useToastMessages } from '@/hooks/use-toast-messages';
import { nanoid } from '@/lib/utils';
import { extractTextFromAgentMessage } from '@/types/chat';

const DUMMY_API_KEY = 'bodhi-proxy';

// Creates a StreamFn that handles Authorization for Bodhi backend requests.
// When an API token is provided, it's sent as Bearer token.
// Otherwise, the Authorization header is stripped (set to null) so the Bodhi
// backend falls through to cookie-based session auth instead of rejecting
// the dummy API key that pi-ai's OpenAI SDK adds by default.
function createBodhiStreamFn(apiToken?: string): StreamFn {
  return (model, context, options) => {
    const headers =
      apiToken !== undefined
        ? { ...model.headers, Authorization: `Bearer ${apiToken}` }
        : { ...model.headers, Authorization: null as unknown as string };
    const patchedModel = { ...model, headers };
    return streamSimple(patchedModel, context, options);
  };
}

export interface UseBodhiAgentOptions {
  tools?: AgentTool[];
  enabledMcpTools?: Record<string, string[]>;
}

export interface UseBodhiAgentReturn {
  input: string;
  setInput: (input: string) => void;
  isStreaming: boolean;
  messages: AgentMessage[];
  streamingMessage: AgentMessage | undefined;
  pendingToolCalls: ReadonlySet<string>;
  errorMessage: string | undefined;
  append: (content: string) => Promise<void>;
  stop: () => void;
}

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

export function useBodhiAgent(options?: UseBodhiAgentOptions): UseBodhiAgentReturn {
  const { tools = [], enabledMcpTools } = options ?? {};

  const [input, setInput] = useState('');
  const [isStreaming, setIsStreaming] = useState(false);
  const [messages, setMessages] = useState<AgentMessage[]>([]);
  const [streamingMessage, setStreamingMessage] = useState<AgentMessage | undefined>(undefined);
  const [pendingToolCalls, setPendingToolCalls] = useState<ReadonlySet<string>>(new Set());
  const [errorMessage, setErrorMessage] = useState<string | undefined>(undefined);

  const { showError } = useToastMessages();
  const { currentChat, createOrUpdateChat, setCurrentChatId } = useChatDB();
  const chatSettings = useChatSettings();

  const agentRef = useRef<Agent | null>(null);
  const chatIdRef = useRef<{ id: string; createdAt: number } | null>(null);
  const apiTokenRef = useRef<string | undefined>(undefined);

  const baseUrl = useMemo(() => {
    const origin = typeof window !== 'undefined' ? window.location.origin : 'http://localhost';
    return `${origin}/v1`;
  }, []);

  // StreamFn reads API token from ref so it always uses the current value
  const streamFnRef = useRef<StreamFn>((model, context, options) => {
    return createBodhiStreamFn(apiTokenRef.current)(model, context, options);
  });

  const getAgent = useCallback(() => {
    if (!agentRef.current) {
      agentRef.current = new Agent({
        streamFn: streamFnRef.current,
        getApiKey: () => DUMMY_API_KEY,
      });
    }
    return agentRef.current;
  }, []);

  useEffect(() => {
    const agent = getAgent();

    const unsubscribe = agent.subscribe((event: AgentEvent) => {
      const state = agent.state;

      switch (event.type) {
        case 'agent_start':
          setIsStreaming(true);
          setErrorMessage(undefined);
          break;

        case 'message_update':
          // Sync committed messages (includes user message) so they render during streaming
          setMessages([...state.messages]);
          setStreamingMessage(event.message);
          setPendingToolCalls(new Set(state.pendingToolCalls));
          break;

        case 'message_end':
          // Sync committed messages so user messages appear immediately
          setMessages([...state.messages]);
          setStreamingMessage(undefined);
          break;

        case 'tool_execution_start':
          setPendingToolCalls(new Set(state.pendingToolCalls));
          break;

        case 'tool_execution_end':
          setPendingToolCalls(new Set(state.pendingToolCalls));
          break;

        case 'turn_end':
          setMessages([...state.messages]);
          break;

        case 'agent_end':
          setMessages([...state.messages]);
          setStreamingMessage(undefined);
          setPendingToolCalls(new Set());
          setIsStreaming(false);
          setErrorMessage(state.errorMessage);
          break;
      }
    });

    return () => {
      unsubscribe();
      agentRef.current?.abort();
    };
  }, [getAgent]);

  // Sync agent state with React state so settings changes take effect without waiting for next append
  useEffect(() => {
    const agent = getAgent();
    if (chatSettings.model) {
      agent.state.model = buildModel(chatSettings.model, baseUrl, chatSettings.apiFormat);
    }
    agent.state.tools = tools;
    agent.state.systemPrompt =
      chatSettings.systemPrompt_enabled && chatSettings.systemPrompt ? chatSettings.systemPrompt : '';
  }, [
    chatSettings.model,
    chatSettings.apiFormat,
    chatSettings.systemPrompt,
    chatSettings.systemPrompt_enabled,
    tools,
    getAgent,
    baseUrl,
  ]);

  const saveChat = useCallback(
    async (agentMessages: AgentMessage[]) => {
      if (!chatIdRef.current) return;

      const piMessages = agentMessages;
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
      await createOrUpdateChat({
        id: chatIdRef.current.id,
        title: titleText,
        messages: chatMessages,
        messageCount: chatMessages.length,
        createdAt: chatIdRef.current.createdAt,
        updatedAt: Date.now(),
        enabledMcpTools: enabledMcpTools && Object.keys(enabledMcpTools).length > 0 ? enabledMcpTools : undefined,
      });

      if (!currentChat) {
        setCurrentChatId(chatIdRef.current.id);
      }
    },
    [createOrUpdateChat, currentChat, setCurrentChatId, enabledMcpTools]
  );

  const append = useCallback(
    async (content: string) => {
      const modelId = chatSettings.model;
      if (!modelId) {
        showError('Error', 'Please select a model first.');
        return;
      }

      // Update API token ref so the streamFn uses the current value
      apiTokenRef.current = chatSettings.api_token_enabled ? chatSettings.api_token : undefined;

      const agent = getAgent();
      const model = buildModel(modelId, baseUrl, chatSettings.apiFormat);

      agent.state.model = model;
      agent.state.tools = tools;

      if (chatSettings.systemPrompt_enabled && chatSettings.systemPrompt) {
        agent.state.systemPrompt = chatSettings.systemPrompt;
      } else {
        agent.state.systemPrompt = '';
      }

      if (!chatIdRef.current) {
        chatIdRef.current = {
          id: currentChat?.id ?? nanoid(),
          createdAt: currentChat?.createdAt ?? Date.now(),
        };
      }

      if (currentChat?.messages && currentChat.messages.length > 0 && agent.state.messages.length === 0) {
        const restored: AgentMessage[] = currentChat.messages.map((m) => {
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
              api: apiFormatToPiApi(chatSettings.apiFormat),
              provider: 'openai',
              model: modelId,
              usage: { inputTokens: 0, outputTokens: 0, cacheReadTokens: 0, cacheWriteTokens: 0 },
              stopReason: 'endTurn' as const,
            };
          }
        });
        agent.state.messages = restored;
      }

      setInput('');

      try {
        await agent.prompt(content);
        // pi-agent-core catches errors internally and sets state.errorMessage
        // instead of rethrowing, so we check after prompt completes.
        if (agent.state.errorMessage) {
          showError('Error', agent.state.errorMessage);
          // Remove the error assistant message (empty content) and the failed user
          // message from agent state so they don't render as blank bubbles or
          // pollute context for the next prompt.
          agent.state.messages = agent.state.messages.filter(
            (m) => !('stopReason' in m && (m as { stopReason: string }).stopReason === 'error')
          );
          setMessages([...agent.state.messages]);
        } else {
          await saveChat(agent.state.messages);
        }
      } catch (error) {
        if (error instanceof Error && error.name === 'AbortError') {
          return;
        }
        const msg = error instanceof Error ? error.message : 'Error sending message to AI assistant.';
        showError('Error', msg);
        setErrorMessage(msg);
      }
    },
    [chatSettings, getAgent, baseUrl, tools, currentChat, saveChat, showError]
  );

  const stop = useCallback(() => {
    agentRef.current?.abort();
  }, []);

  return {
    input,
    setInput,
    isStreaming,
    messages,
    streamingMessage,
    pendingToolCalls,
    errorMessage,
    append,
    stop,
  };
}
