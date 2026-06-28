import { FormEvent, RefObject, useCallback, useEffect, useRef, memo, useMemo } from 'react';

import type { AgentMessage, AgentTool } from '@mariozechner/pi-agent-core';
import type { AssistantMessage as PiAssistantMessage } from '@mariozechner/pi-ai';

import { Button } from '@/components/ui/button';
import { ScrollAnchor } from '@/components/ui/scroll-anchor';
import { useResponsiveTestId } from '@/hooks/use-responsive-testid';
import { useToastMessages } from '@/hooks/use-toast-messages';
import { cn } from '@/lib/utils';
import { useAgentStore } from '@/stores/agentStore';
import { useChatSettingsStore } from '@/stores/chatSettingsStore';
import { extractTextFromAgentMessage, extractThinkingFromAgentMessage, Message } from '@/types/chat';

import { ChatMessage } from './ChatMessage';
import { ThinkingBlock } from './ThinkingBlock';

import './chat.css';

const EmptyState = () => (
  <div className="flex h-full items-center justify-center" data-testid="empty-chat-state">
    <div className="text-center space-y-3">
      <h3 className="text-lg font-semibold">Welcome to Chat</h3>
      <p className="text-muted-foreground">Start a conversation by typing a message below.</p>
    </div>
  </div>
);

interface ChatInputProps {
  input: string;
  setInput: (value: string) => void;
  handleSubmit: (e: FormEvent) => void;
  streamLoading: boolean;
  onStop: () => void;
  inputRef: RefObject<HTMLTextAreaElement>;
  isModelSelected: boolean;
}

const ChatInput = memo(function ChatInput({
  input,
  setInput,
  handleSubmit,
  streamLoading,
  onStop,
  inputRef,
  isModelSelected,
}: ChatInputProps) {
  const getTestId = useResponsiveTestId();

  const autosize = (el: HTMLTextAreaElement | null) => {
    if (!el) return;
    el.style.height = 'auto';
    el.style.height = `${Math.min(el.scrollHeight, 160)}px`;
  };

  return (
    <div className="chat-composer" data-testid={getTestId('chat-input-panel')}>
      <form
        onSubmit={handleSubmit}
        className={cn('chat-composer-inner', !isModelSelected && 'no-model')}
        data-testid={getTestId('chat-form')}
        data-test-state={isModelSelected ? 'model-selected' : 'no-model'}
      >
        <textarea
          ref={inputRef}
          data-testid={getTestId('chat-input')}
          rows={1}
          placeholder={isModelSelected ? 'Ask me anything…   ⏎ to send' : 'Please select a model first'}
          value={input}
          onChange={(e) => {
            setInput(e.target.value);
            autosize(e.target);
          }}
          onKeyDown={(e) => {
            if (e.key === 'Enter' && !e.shiftKey) {
              e.preventDefault();
              handleSubmit(e);
            }
          }}
        />

        <div className="chat-composer-row">
          <div className="ml-auto">
            {streamLoading ? (
              <Button
                type="button"
                size="sm"
                variant="destructive"
                data-testid={getTestId('stop-button')}
                className="h-8 gap-1.5"
                onClick={onStop}
              >
                <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 16 16" fill="none" className="h-3.5 w-3.5">
                  <rect x="3" y="3" width="10" height="10" rx="1" fill="currentColor" />
                </svg>
                Stop
              </Button>
            ) : (
              <Button
                type="submit"
                size="sm"
                data-testid={getTestId('send-button')}
                disabled={!input.trim() || !isModelSelected}
                className="h-8 gap-1.5"
              >
                Send
                <svg
                  xmlns="http://www.w3.org/2000/svg"
                  viewBox="0 0 16 16"
                  fill="none"
                  className="h-3.5 w-3.5"
                  strokeWidth="2"
                >
                  <path
                    d="M.5 1.163A1 1 0 0 1 1.97.28l12.868 6.837a1 1 0 0 1 0 1.766L1.969 15.72A1 1 0 0 1 .5 14.836V10.33a1 1 0 0 1 .816-.983L8.5 8 1.316 6.653A1 1 0 0 1 .5 5.67V1.163Z"
                    fill="currentColor"
                  />
                </svg>
              </Button>
            )}
          </div>
        </div>
      </form>

      <p
        className="mx-auto max-w-[760px] px-2 pt-2 text-center text-xs text-muted-foreground"
        data-testid={getTestId('chat-disclaimer')}
      >
        Chat assistant can make mistakes.
      </p>
    </div>
  );
});

// TODO: Remove shim when ChatMessage renders AgentMessage directly
function agentMessageToLegacy(msg: AgentMessage): Message | null {
  if (!('role' in msg)) return null;

  if (msg.role === 'user') {
    return {
      role: 'user',
      content: typeof msg.content === 'string' ? msg.content : extractTextFromAgentMessage(msg),
    };
  }

  if (msg.role === 'assistant') {
    const assistantMsg = msg as PiAssistantMessage;
    const text = assistantMsg.content
      .filter((c) => c.type === 'text')
      .map((c) => (c as { type: 'text'; text: string }).text)
      .join('');

    const toolCalls = assistantMsg.content
      .filter((c) => c.type === 'toolCall')
      .map((c) => {
        const tc = c as { type: 'toolCall'; id: string; name: string; arguments: Record<string, unknown> };
        return {
          id: tc.id,
          type: 'function' as const,
          function: {
            name: tc.name,
            arguments: JSON.stringify(tc.arguments),
          },
        };
      });

    return {
      role: 'assistant',
      content: text,
      tool_calls: toolCalls.length > 0 ? toolCalls : undefined,
      metadata:
        assistantMsg.usage || assistantMsg.model
          ? {
              // The model the request was sent under. For a model-router this is
              // the router alias (the pi SDK reports the request model, not the
              // upstream-echoed served model), so E2E distinguishes targets via
              // response content rather than this field.
              model: assistantMsg.model,
              usage: assistantMsg.usage
                ? {
                    prompt_tokens: assistantMsg.usage.input,
                    completion_tokens: assistantMsg.usage.output,
                    total_tokens: assistantMsg.usage.totalTokens,
                  }
                : undefined,
            }
          : undefined,
    };
  }

  if (msg.role === 'toolResult') {
    const toolMsg = msg as { role: 'toolResult'; content: Array<{ type: string; text?: string }>; toolCallId: string };
    const textContent = toolMsg.content
      .filter((c) => c.type === 'text')
      .map((c) => c.text ?? '')
      .join('');
    return {
      role: 'tool',
      content: textContent || JSON.stringify(toolMsg.content),
      tool_call_id: toolMsg.toolCallId,
    };
  }

  return null;
}

interface AgentMessageListProps {
  messages: AgentMessage[];
  streamingMessage: AgentMessage | undefined;
  isStreaming: boolean;
  pendingToolCalls: ReadonlySet<string>;
}

const AgentMessageList = memo(function AgentMessageList({
  messages,
  streamingMessage,
  isStreaming,
  pendingToolCalls,
}: AgentMessageListProps) {
  const legacyMessages: Message[] = useMemo(
    () => messages.map(agentMessageToLegacy).filter((m): m is Message => m !== null),
    [messages]
  );

  const streamingLegacy = useMemo(
    () => (streamingMessage ? agentMessageToLegacy(streamingMessage) : null),
    [streamingMessage]
  );

  const streamingThinking = useMemo(
    () => (streamingMessage ? extractThinkingFromAgentMessage(streamingMessage) : ''),
    [streamingMessage]
  );

  const hasStreamingContent = streamingLegacy && (streamingLegacy.content || streamingLegacy.tool_calls?.length);

  const lastUserIdx = useMemo(() => {
    for (let i = legacyMessages.length - 1; i >= 0; i--) {
      if (legacyMessages[i].role === 'user') return i;
    }
    return -1;
  }, [legacyMessages]);

  const lastAssistantIdx = useMemo(() => {
    for (let i = legacyMessages.length - 1; i >= 0; i--) {
      if (legacyMessages[i].role === 'assistant') return i;
    }
    return -1;
  }, [legacyMessages]);

  return (
    <div className="space-y-2 py-2" data-testid="message-list">
      {legacyMessages.map((message, i) => {
        if (message.role === 'tool') return null;

        const isLastOfRole =
          (message.role === 'user' && i === lastUserIdx) || (message.role === 'assistant' && i === lastAssistantIdx);
        // User messages are "latest" immediately (committed, not streaming);
        // assistant messages only become "latest" after streaming completes.
        const isLatest = isLastOfRole && (message.role === 'user' || (!isStreaming && !streamingMessage));
        const isArchived = !isLastOfRole || (message.role !== 'user' && !!streamingMessage);

        return (
          <ChatMessage
            key={`history-${i}`}
            message={message}
            isLatest={isLatest}
            isArchived={isArchived && !isLatest}
            allMessages={legacyMessages}
          />
        );
      })}

      {streamingThinking && <ThinkingBlock thinking={streamingThinking} isStreaming={isStreaming} />}

      {hasStreamingContent && (
        <ChatMessage
          key="streaming-current"
          message={streamingLegacy!}
          isStreaming={isStreaming}
          isLatest={!isStreaming}
          isArchived={false}
          allMessages={legacyMessages}
          isExecutingTools={pendingToolCalls.size > 0}
        />
      )}
      <ScrollAnchor />
    </div>
  );
});

interface ChatUIProps {
  /** Agent tools + selection from the chat's shared MCP wiring (see useChatMcp in the route). */
  agentTools: AgentTool[];
  enabledMcpTools: Record<string, string[]>;
}

export function ChatUI({ agentTools, enabledMcpTools }: ChatUIProps) {
  const { showError } = useToastMessages();
  const model = useChatSettingsStore((s) => s.model);

  const input = useAgentStore((s) => s.input);
  const setInput = useAgentStore((s) => s.setInput);
  const streamLoading = useAgentStore((s) => s.isStreaming);
  const agentMessages = useAgentStore((s) => s.messages);
  const streamingMessage = useAgentStore((s) => s.streamingMessage);
  const pendingToolCalls = useAgentStore((s) => s.pendingToolCalls);
  const stop = useAgentStore((s) => s.stop);
  const syncAgentSettings = useAgentStore((s) => s.syncAgentSettings);

  useEffect(() => {
    syncAgentSettings(agentTools);
  }, [syncAgentSettings, agentTools]);

  const inputRef = useRef<HTMLTextAreaElement>(null);
  const getTestId = useResponsiveTestId();

  useEffect(() => {
    if (!streamLoading && inputRef.current) {
      inputRef.current.focus();
    }
  }, [streamLoading]);

  const handleSubmit = useCallback(
    async (e: FormEvent) => {
      e.preventDefault();
      const currentInput = useAgentStore.getState().input;
      const currentModel = useChatSettingsStore.getState().model;
      const isStreaming = useAgentStore.getState().isStreaming;

      if (!currentInput.trim() || isStreaming) return;
      if (!currentModel) {
        showError('No Model Selected', 'Please select an Alias/Model from settings before sending a message.');
        return;
      }

      const content = currentInput.trim();
      await useAgentStore.getState().append(content, {
        tools: agentTools,
        enabledMcpTools,
        showError,
      });
    },
    [showError, agentTools, enabledMcpTools]
  );

  const isEmpty = agentMessages.length === 0 && !streamingMessage;

  return (
    <div data-testid={getTestId('chat-ui')} className="flex h-full flex-col">
      <div className="relative flex-1 min-h-0" data-testid={getTestId('chat-content-area')}>
        <div className="chat-conv absolute inset-0" data-testid={getTestId('chat-scroll-area')}>
          <div className="chat-conv-inner" data-testid={getTestId('chat-messages-container')}>
            {isEmpty ? (
              <EmptyState />
            ) : (
              <AgentMessageList
                messages={agentMessages}
                streamingMessage={streamingMessage}
                isStreaming={streamLoading}
                pendingToolCalls={pendingToolCalls}
              />
            )}
          </div>
        </div>
      </div>
      <ChatInput
        input={input}
        setInput={setInput}
        handleSubmit={handleSubmit}
        streamLoading={streamLoading}
        onStop={stop}
        inputRef={inputRef}
        isModelSelected={!!model}
      />
    </div>
  );
}
