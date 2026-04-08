import { renderHook, act } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import type { AgentEvent, AgentMessage, AgentTool } from '@mariozechner/pi-agent-core';

type AgentEventListener = (event: AgentEvent, signal: AbortSignal) => Promise<void> | void;

const mockPrompt = vi.fn<[string | AgentMessage | AgentMessage[]], Promise<void>>();
const mockAbort = vi.fn();
let capturedAgentOptions: Record<string, unknown> = {};

let listeners: AgentEventListener[] = [];
let mockMessages: AgentMessage[] = [];

const mockModelSetter = vi.fn();
const mockToolsSetter = vi.fn();
const mockSystemPromptSetter = vi.fn();

vi.mock('@mariozechner/pi-agent-core', () => ({
  Agent: vi.fn().mockImplementation((opts: Record<string, unknown>) => {
    capturedAgentOptions = opts;
    return {
      subscribe: (listener: AgentEventListener) => {
        listeners.push(listener);
        return () => {
          listeners = listeners.filter((l) => l !== listener);
        };
      },
      get state() {
        return {
          get systemPrompt() {
            return '';
          },
          set systemPrompt(v: string) {
            mockSystemPromptSetter(v);
          },
          get model() {
            return null;
          },
          set model(v: unknown) {
            mockModelSetter(v);
          },
          get thinkingLevel() {
            return 'off';
          },
          set tools(v: AgentTool[]) {
            mockToolsSetter(v);
          },
          get tools() {
            return [];
          },
          get messages() {
            return mockMessages;
          },
          set messages(v: AgentMessage[]) {
            mockMessages = [...v];
          },
          get isStreaming() {
            return false;
          },
          get streamingMessage() {
            return undefined;
          },
          get pendingToolCalls() {
            return new Set<string>();
          },
          get errorMessage() {
            return undefined;
          },
        };
      },
      prompt: mockPrompt,
      abort: mockAbort,
    };
  }),
}));

vi.mock('@mariozechner/pi-ai', () => ({
  streamSimple: vi.fn(),
}));

const mockCreateOrUpdateChat = vi.fn().mockResolvedValue('chat-1');
const mockSetCurrentChatId = vi.fn();
vi.mock('@/hooks/chat/useChatDb', () => ({
  useChatDB: () => ({
    currentChat: null,
    createOrUpdateChat: mockCreateOrUpdateChat,
    setCurrentChatId: mockSetCurrentChatId,
  }),
}));

const mockShowError = vi.fn();
vi.mock('@/hooks/use-toast-messages', () => ({
  useToastMessages: () => ({
    showError: mockShowError,
  }),
}));

vi.mock('@/hooks/chat/useChatSettings', () => ({
  useChatSettings: () => ({
    model: 'test-model',
    systemPrompt: 'Be helpful',
    systemPrompt_enabled: true,
    maxToolIterations: 5,
    maxToolIterations_enabled: true,
    getRequestSettings: () => ({ model: 'test-model' }),
  }),
}));

vi.mock('@/lib/utils', () => ({
  nanoid: () => 'test-chat-id',
}));

import { useBodhiAgent } from './useBodhiAgent';

describe('useBodhiAgent', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    listeners = [];
    mockMessages = [];
    capturedAgentOptions = {};
  });

  it('should initialize with idle state', () => {
    const { result } = renderHook(() => useBodhiAgent());

    expect(result.current.input).toBe('');
    expect(result.current.isStreaming).toBe(false);
    expect(result.current.messages).toEqual([]);
    expect(result.current.streamingMessage).toBeUndefined();
    expect(result.current.pendingToolCalls.size).toBe(0);
    expect(result.current.errorMessage).toBeUndefined();
  });

  it('should construct Agent with a streamFn that strips Authorization header', async () => {
    const { streamSimple: mockStreamSimple } = await import('@mariozechner/pi-ai');

    renderHook(() => useBodhiAgent());

    expect(capturedAgentOptions.streamFn).toBeDefined();
    expect(capturedAgentOptions.streamFn).not.toBe(mockStreamSimple);

    const fakeModel = { id: 'test', headers: { 'X-Custom': 'val' } };
    const fakeContext = { messages: [] };
    const fakeOptions = {};

    (capturedAgentOptions.streamFn as Function)(fakeModel, fakeContext, fakeOptions);

    expect(mockStreamSimple).toHaveBeenCalledWith(
      expect.objectContaining({
        id: 'test',
        headers: { 'X-Custom': 'val', Authorization: null },
      }),
      fakeContext,
      fakeOptions
    );
  });

  it('should call agent.prompt on append and configure model+system prompt', async () => {
    mockPrompt.mockResolvedValueOnce(undefined);

    const { result } = renderHook(() => useBodhiAgent());

    await act(async () => {
      await result.current.append('hello');
    });

    expect(mockPrompt).toHaveBeenCalledWith('hello');
    expect(mockModelSetter).toHaveBeenCalledWith(
      expect.objectContaining({
        id: 'test-model',
        api: 'openai-completions',
        provider: 'openai',
      })
    );
    expect(mockSystemPromptSetter).toHaveBeenCalledWith('Be helpful');
  });

  it('should set tools on agent state before prompt', async () => {
    mockPrompt.mockResolvedValueOnce(undefined);

    const mockTool = {
      name: 'test-tool',
      label: 'Test',
      description: 'A test tool',
      parameters: {},
      execute: vi.fn(),
    } as unknown as AgentTool;

    const { result } = renderHook(() => useBodhiAgent({ tools: [mockTool] }));

    await act(async () => {
      await result.current.append('hello');
    });

    expect(mockToolsSetter).toHaveBeenCalledWith([mockTool]);
  });

  it('should call abort on stop', () => {
    const { result } = renderHook(() => useBodhiAgent());

    act(() => {
      result.current.stop();
    });

    expect(mockAbort).toHaveBeenCalled();
  });

  it('should handle agent errors gracefully', async () => {
    mockPrompt.mockRejectedValueOnce(new Error('Stream failed'));

    const { result } = renderHook(() => useBodhiAgent());

    await act(async () => {
      await result.current.append('hello');
    });

    expect(mockShowError).toHaveBeenCalledWith('Error', 'Stream failed');
  });

  it('should clear input after append', async () => {
    mockPrompt.mockResolvedValueOnce(undefined);

    const { result } = renderHook(() => useBodhiAgent());

    act(() => {
      result.current.setInput('draft message');
    });
    expect(result.current.input).toBe('draft message');

    await act(async () => {
      await result.current.append('hello');
    });

    expect(result.current.input).toBe('');
  });

  it('should save chat after successful prompt', async () => {
    const completedMessages: AgentMessage[] = [
      { role: 'user', content: 'hello', timestamp: Date.now() },
      {
        role: 'assistant',
        content: [{ type: 'text', text: 'Hi!' }],
        api: 'openai-completions',
        provider: 'openai',
        model: 'test-model',
        usage: { inputTokens: 5, outputTokens: 2, cacheReadTokens: 0, cacheWriteTokens: 0 },
        stopReason: 'endTurn',
      } as AgentMessage,
    ];

    mockPrompt.mockImplementation(async () => {
      mockMessages = completedMessages;
    });

    const { result } = renderHook(() => useBodhiAgent());

    await act(async () => {
      await result.current.append('hello');
    });

    expect(mockCreateOrUpdateChat).toHaveBeenCalledWith(
      expect.objectContaining({
        id: 'test-chat-id',
      })
    );
    expect(mockSetCurrentChatId).toHaveBeenCalledWith('test-chat-id');
  });

  it('should update state when agent events fire', async () => {
    const streamingMsg: AgentMessage = {
      role: 'assistant',
      content: [{ type: 'text', text: 'hi' }],
      api: 'openai-completions',
      provider: 'openai',
      model: 'test-model',
      usage: { inputTokens: 0, outputTokens: 0, cacheReadTokens: 0, cacheWriteTokens: 0 },
      stopReason: 'endTurn',
    } as AgentMessage;

    mockPrompt.mockImplementation(async () => {
      const signal = new AbortController().signal;
      for (const listener of listeners) {
        await listener({ type: 'agent_start' }, signal);
      }
      for (const listener of listeners) {
        await listener({ type: 'turn_end', message: streamingMsg, toolResults: [] }, signal);
      }
      mockMessages = [streamingMsg];
      for (const listener of listeners) {
        await listener({ type: 'agent_end', messages: [streamingMsg] }, signal);
      }
    });

    const { result } = renderHook(() => useBodhiAgent());

    await act(async () => {
      await result.current.append('hello');
    });

    expect(result.current.messages).toHaveLength(1);
    expect(result.current.isStreaming).toBe(false);
  });

  it('should silence AbortError without showing toast', async () => {
    const abortError = new Error('The operation was aborted');
    abortError.name = 'AbortError';
    mockPrompt.mockRejectedValueOnce(abortError);

    const { result } = renderHook(() => useBodhiAgent());

    await act(async () => {
      await result.current.append('hello');
    });

    expect(mockShowError).not.toHaveBeenCalled();
  });
});
