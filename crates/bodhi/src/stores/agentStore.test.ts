import { act } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import type { AgentEvent, AgentMessage, AgentTool } from '@mariozechner/pi-agent-core';

// ── Agent mock ────────────────────────────────────────────────────────────────

type AgentEventListener = (event: AgentEvent) => void;

const mockPrompt = vi.fn<(input: string | AgentMessage | AgentMessage[]) => Promise<void>>();
const mockAbort = vi.fn();
const mockModelSetter = vi.fn();
const mockToolsSetter = vi.fn();
const mockSystemPromptSetter = vi.fn();

let capturedListeners: AgentEventListener[] = [];
let mockAgentMessages: AgentMessage[] = [];
let mockErrorMessage: string | undefined = undefined;

vi.mock('@mariozechner/pi-agent-core', () => ({
  Agent: vi.fn().mockImplementation(() => ({
    subscribe: (listener: AgentEventListener) => {
      capturedListeners.push(listener);
      return () => {
        capturedListeners = capturedListeners.filter((l) => l !== listener);
      };
    },
    get state() {
      return {
        get model() {
          return null;
        },
        set model(v: unknown) {
          mockModelSetter(v);
        },
        get tools() {
          return [];
        },
        set tools(v: AgentTool[]) {
          mockToolsSetter(v);
        },
        get systemPrompt() {
          return '';
        },
        set systemPrompt(v: string) {
          mockSystemPromptSetter(v);
        },
        get messages() {
          return mockAgentMessages;
        },
        set messages(v: AgentMessage[]) {
          mockAgentMessages = [...v];
        },
        get pendingToolCalls() {
          return new Set<string>();
        },
        get errorMessage() {
          return mockErrorMessage;
        },
        get isStreaming() {
          return false;
        },
      };
    },
    prompt: mockPrompt,
    abort: mockAbort,
  })),
}));

vi.mock('@mariozechner/pi-ai', () => ({
  streamSimple: vi.fn(),
}));

// ── Store mocks ───────────────────────────────────────────────────────────────

vi.mock('@/stores/chatStore', () => {
  const { create } = require('zustand');
  return {
    useChatStore: create(() => ({
      chats: [],
      currentChatId: null,
      isLoaded: true,
      userId: 'user-1',
      loadChats: vi.fn(),
      setCurrentChatId: vi.fn(),
      createNewChat: vi.fn(),
      createOrUpdateChat: vi.fn().mockResolvedValue('chat-1'),
      deleteChat: vi.fn(),
      clearChats: vi.fn(),
      getChat: vi.fn(),
      saveChatSettings: vi.fn(),
      getChatSettings: vi.fn(),
      loadMessagesForChat: vi.fn().mockResolvedValue([]),
    })),
  };
});

vi.mock('@/stores/chatSettingsStore', () => {
  const { create } = require('zustand');
  return {
    useChatSettingsStore: create(() => ({
      model: 'test-model',
      apiFormat: 'openai',
      systemPrompt: 'Be helpful',
      systemPrompt_enabled: true,
      api_token: undefined,
      api_token_enabled: false,
      saveForChat: vi.fn().mockResolvedValue(undefined),
      getRequestSettings: () => ({ model: 'test-model' }),
    })),
  };
});

vi.mock('@/lib/utils', () => ({
  nanoid: () => 'new-chat-id',
}));

// ── Import subject under test AFTER mocks ────────────────────────────────────

import { useAgentStore, initAgentSubscription } from './agentStore';
import { useChatStore } from './chatStore';
import { useChatSettingsStore } from './chatSettingsStore';

// ── Helpers ───────────────────────────────────────────────────────────────────

function emitAgentEvent(event: AgentEvent) {
  capturedListeners.forEach((l) => l(event));
}

// ── Tests ─────────────────────────────────────────────────────────────────────

describe('agentStore', () => {
  beforeEach(() => {
    // Reset module-level _agent singleton so each test gets a fresh agent.
    // reset() sets _agent = null; restoreMessagesForChat() exits early
    // because chats=[] means no currentChat.
    useAgentStore.getState().reset();

    vi.clearAllMocks();
    capturedListeners = [];
    mockAgentMessages = [];
    mockErrorMessage = undefined;

    // Ensure clean Zustand state after reset
    useAgentStore.setState({
      input: '',
      isStreaming: false,
      messages: [],
      streamingMessage: undefined,
      pendingToolCalls: new Set(),
      errorMessage: undefined,
      chatIdRef: null,
    });

    // Reset chat store mock state
    useChatStore.setState({
      chats: [],
      currentChatId: null,
      createOrUpdateChat: vi.fn().mockResolvedValue('chat-1'),
      setCurrentChatId: vi.fn(),
      loadMessagesForChat: vi.fn().mockResolvedValue([]),
    });

    // Reset settings store state
    useChatSettingsStore.setState({
      model: 'test-model',
      apiFormat: 'openai',
      systemPrompt: 'Be helpful',
      systemPrompt_enabled: true,
      api_token: undefined,
      api_token_enabled: false,
      saveForChat: vi.fn().mockResolvedValue(undefined),
    });
  });

  describe('setInput', () => {
    it('updates input state', () => {
      useAgentStore.getState().setInput('hello');
      expect(useAgentStore.getState().input).toBe('hello');
    });
  });

  describe('append', () => {
    it('does nothing when no model is set', async () => {
      useChatSettingsStore.setState({ model: '' });
      const showError = vi.fn();

      await act(async () => {
        await useAgentStore.getState().append('hello', { showError });
      });

      expect(mockPrompt).not.toHaveBeenCalled();
      expect(showError).toHaveBeenCalledWith('Error', 'Please select a model first.');
    });

    it('calls agent.prompt with content', async () => {
      mockPrompt.mockResolvedValueOnce(undefined);
      useChatSettingsStore.setState({ model: 'test-model' });

      await act(async () => {
        await useAgentStore.getState().append('hello world');
      });

      expect(mockPrompt).toHaveBeenCalledWith('hello world');
    });

    it('configures model on agent before prompting', async () => {
      mockPrompt.mockResolvedValueOnce(undefined);
      useChatSettingsStore.setState({ model: 'my-model', apiFormat: 'openai' });

      await act(async () => {
        await useAgentStore.getState().append('hi');
      });

      expect(mockModelSetter).toHaveBeenCalledWith(
        expect.objectContaining({ id: 'my-model', api: 'openai-completions', provider: 'openai' })
      );
    });

    it('configures system prompt on agent', async () => {
      mockPrompt.mockResolvedValueOnce(undefined);
      useChatSettingsStore.setState({ systemPrompt: 'You are helpful', systemPrompt_enabled: true });

      await act(async () => {
        await useAgentStore.getState().append('hi');
      });

      expect(mockSystemPromptSetter).toHaveBeenCalledWith('You are helpful');
    });

    it('clears input after appending', async () => {
      mockPrompt.mockResolvedValueOnce(undefined);
      useAgentStore.setState({ input: 'some input' });

      await act(async () => {
        await useAgentStore.getState().append('hi');
      });

      expect(useAgentStore.getState().input).toBe('');
    });

    it('sets tools on agent from options', async () => {
      mockPrompt.mockResolvedValueOnce(undefined);
      const tool = {
        name: 'my-tool',
        label: 'My Tool',
        description: 'does stuff',
        parameters: {},
        execute: vi.fn(),
      } as unknown as AgentTool;

      await act(async () => {
        await useAgentStore.getState().append('hi', { tools: [tool] });
      });

      expect(mockToolsSetter).toHaveBeenCalledWith([tool]);
    });

    it('saves chat after successful prompt', async () => {
      mockPrompt.mockResolvedValueOnce(undefined);
      const userMsg: AgentMessage = { role: 'user', content: 'hello', timestamp: Date.now() };
      mockAgentMessages = [userMsg];

      await act(async () => {
        await useAgentStore.getState().append('hello');
      });

      expect(useChatStore.getState().createOrUpdateChat).toHaveBeenCalledWith(
        expect.objectContaining({ title: 'hello', messageCount: 1 })
      );
    });

    it('sets chatIdRef in store state after first append', async () => {
      mockPrompt.mockResolvedValueOnce(undefined);

      await act(async () => {
        await useAgentStore.getState().append('hello');
      });

      const ref = useAgentStore.getState().chatIdRef;
      expect(ref).not.toBeNull();
      expect(ref?.id).toBe('new-chat-id'); // nanoid mock
    });

    it('reuses existing chatIdRef across multiple appends', async () => {
      mockPrompt.mockResolvedValue(undefined);

      await act(async () => {
        await useAgentStore.getState().append('first');
      });
      const firstRef = useAgentStore.getState().chatIdRef;

      await act(async () => {
        await useAgentStore.getState().append('second');
      });
      const secondRef = useAgentStore.getState().chatIdRef;

      expect(firstRef?.id).toBe(secondRef?.id);
    });

    it('shows error and filters messages on agent errorMessage', async () => {
      mockPrompt.mockResolvedValueOnce(undefined);
      mockErrorMessage = 'API failed';
      const showError = vi.fn();

      await act(async () => {
        await useAgentStore.getState().append('hello', { showError });
      });

      expect(showError).toHaveBeenCalledWith('Error', 'API failed');
      expect(useChatStore.getState().createOrUpdateChat).not.toHaveBeenCalled();
    });

    it('handles AbortError silently', async () => {
      const abortError = new Error('aborted');
      abortError.name = 'AbortError';
      mockPrompt.mockRejectedValueOnce(abortError);
      const showError = vi.fn();

      await act(async () => {
        await useAgentStore.getState().append('hello', { showError });
      });

      expect(showError).not.toHaveBeenCalled();
      expect(useAgentStore.getState().errorMessage).toBeUndefined();
    });

    it('shows and sets error for non-abort errors', async () => {
      mockPrompt.mockRejectedValueOnce(new Error('network error'));
      const showError = vi.fn();

      await act(async () => {
        await useAgentStore.getState().append('hello', { showError });
      });

      expect(showError).toHaveBeenCalledWith('Error', 'network error');
      expect(useAgentStore.getState().errorMessage).toBe('network error');
    });
  });

  describe('stop', () => {
    it('calls agent.abort()', () => {
      // Ensure agent is created first
      useAgentStore.getState().syncAgentSettings();
      useAgentStore.getState().stop();
      expect(mockAbort).toHaveBeenCalled();
    });
  });

  describe('reset', () => {
    it('clears all state including chatIdRef', async () => {
      mockPrompt.mockResolvedValueOnce(undefined);
      // Set some state
      useAgentStore.setState({ input: 'hi', isStreaming: true, chatIdRef: { id: 'x', createdAt: 1 } });

      act(() => {
        useAgentStore.getState().reset();
      });

      const state = useAgentStore.getState();
      expect(state.input).toBe('');
      expect(state.isStreaming).toBe(false);
      expect(state.messages).toEqual([]);
      expect(state.chatIdRef).toBeNull();
    });

    it('aborts the current agent', () => {
      useAgentStore.getState().syncAgentSettings(); // create agent
      act(() => {
        useAgentStore.getState().reset();
      });
      expect(mockAbort).toHaveBeenCalled();
    });
  });

  describe('syncAgentSettings', () => {
    it('updates agent model from settings store', () => {
      useChatSettingsStore.setState({ model: 'synced-model', apiFormat: 'openai_responses' });
      useAgentStore.getState().syncAgentSettings();
      expect(mockModelSetter).toHaveBeenCalledWith(
        expect.objectContaining({ id: 'synced-model', api: 'openai-responses' })
      );
    });

    it('updates agent tools when provided', () => {
      const tool = {
        name: 'tool-x',
        label: 'X',
        description: 'x',
        parameters: {},
        execute: vi.fn(),
      } as unknown as AgentTool;
      useAgentStore.getState().syncAgentSettings([tool]);
      expect(mockToolsSetter).toHaveBeenCalledWith([tool]);
    });

    it('does not update tools when not provided', () => {
      useAgentStore.getState().syncAgentSettings();
      expect(mockToolsSetter).not.toHaveBeenCalled();
    });
  });

  describe('agent events → store state', () => {
    beforeEach(() => {
      // Trigger agent creation
      useAgentStore.getState().syncAgentSettings();
    });

    it('agent_start sets isStreaming=true', () => {
      emitAgentEvent({ type: 'agent_start' } as AgentEvent);
      expect(useAgentStore.getState().isStreaming).toBe(true);
      expect(useAgentStore.getState().errorMessage).toBeUndefined();
    });

    it('agent_end sets isStreaming=false and syncs messages', () => {
      const msg: AgentMessage = { role: 'user', content: 'hi', timestamp: 0 };
      mockAgentMessages = [msg];
      emitAgentEvent({ type: 'agent_end' } as AgentEvent);
      const state = useAgentStore.getState();
      expect(state.isStreaming).toBe(false);
      expect(state.messages).toHaveLength(1);
      expect(state.streamingMessage).toBeUndefined();
    });

    it('message_update sets streamingMessage', () => {
      const streamingMsg: AgentMessage = { role: 'user', content: 'stream', timestamp: 0 };
      emitAgentEvent({ type: 'message_update', message: streamingMsg } as AgentEvent);
      expect(useAgentStore.getState().streamingMessage).toBe(streamingMsg);
    });

    it('message_end clears streamingMessage', () => {
      useAgentStore.setState({ streamingMessage: { role: 'user', content: 'x', timestamp: 0 } });
      emitAgentEvent({ type: 'message_end' } as AgentEvent);
      expect(useAgentStore.getState().streamingMessage).toBeUndefined();
    });
  });

  describe('initAgentSubscription', () => {
    it('resets agent when currentChatId changes', () => {
      initAgentSubscription();
      useAgentStore.setState({ chatIdRef: { id: 'old-chat', createdAt: 0 } });

      useChatStore.setState({ currentChatId: 'chat-2' });

      expect(useAgentStore.getState().chatIdRef).toBeNull();
    });
  });
});
