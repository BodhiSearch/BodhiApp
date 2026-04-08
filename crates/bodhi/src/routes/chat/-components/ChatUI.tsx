import { FormEvent, RefObject, useEffect, useRef, memo, useMemo } from 'react';

import type { AgentMessage } from '@mariozechner/pi-agent-core';
import type { AssistantMessage as PiAssistantMessage } from '@mariozechner/pi-ai';
import { Plus } from 'lucide-react';

import { ChatMessage } from './ChatMessage';
import { McpsPopover } from './McpsPopover';
import { ThinkingBlock } from './ThinkingBlock';
import { Button } from '@/components/ui/button';
import { MemoizedReactMarkdown } from '@/components/ui/markdown';
import { ScrollAnchor } from '@/components/ui/scroll-anchor';
import { useSidebar } from '@/components/ui/sidebar';
import { useChatDB, useChatSettings } from '@/hooks/chat';
import { useBodhiAgent } from '@/hooks/chat/useBodhiAgent';
import { useMcpAgentTools } from '@/hooks/chat/useMcpAgentTools';
import { useMcpSelection, useListMcps } from '@/hooks/mcps';
import type { McpClientTool, McpConnectionStatus } from '@/hooks/mcps/useMcpClient';
import { useMcpClients } from '@/hooks/mcps/useMcpClients';
import { useResponsiveTestId } from '@/hooks/use-responsive-testid';
import { useToastMessages } from '@/hooks/use-toast-messages';
import { cn } from '@/lib/utils';
import { extractTextFromAgentMessage, extractThinkingFromAgentMessage, Message } from '@/types/chat';

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
  enabledMcpTools: Record<string, string[]>;
  onToggleMcpTool: (mcpId: string, toolName: string) => void;
  onToggleMcp: (mcpId: string, allToolNames: string[]) => void;
  mcpTools: Map<string, McpClientTool[]>;
  mcpConnectionStatus: Map<string, McpConnectionStatus>;
}

const ChatInput = memo(function ChatInput({
  input,
  setInput,
  handleSubmit,
  streamLoading,
  onStop,
  inputRef,
  isModelSelected,
  enabledMcpTools,
  onToggleMcpTool,
  onToggleMcp,
  mcpTools,
  mcpConnectionStatus,
}: ChatInputProps) {
  const { createNewChat } = useChatDB();
  const getTestId = useResponsiveTestId();

  return (
    <div
      className="sticky bottom-0 border-t bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/75"
      data-testid={getTestId('chat-input-panel')}
    >
      <div className="mx-auto max-w-3xl px-4 py-2">
        <div
          className="relative flex items-center rounded-lg border bg-background shadow-sm"
          data-testid={getTestId('chat-input-container')}
        >
          <div className="absolute left-2 flex items-center gap-1">
            <Button
              type="button"
              variant="ghost"
              size="icon"
              className="h-8 w-8"
              onClick={createNewChat}
              data-testid={getTestId('new-chat-inline-button')}
            >
              <Plus className="h-5 w-5" />
              <span className="sr-only">New chat</span>
            </Button>

            <McpsPopover
              enabledMcpTools={enabledMcpTools}
              onToggleTool={onToggleMcpTool}
              onToggleMcp={onToggleMcp}
              disabled={streamLoading}
              mcpTools={mcpTools}
              mcpConnectionStatus={mcpConnectionStatus}
            />
          </div>

          <form onSubmit={handleSubmit} className="flex w-full items-center" data-testid={getTestId('chat-form')}>
            <textarea
              ref={inputRef}
              data-testid={getTestId('chat-input')}
              className={cn(
                'flex-1 resize-none bg-transparent pl-32 pr-12 py-3 text-sm outline-none disabled:opacity-50',
                !isModelSelected && 'ring-2 ring-destructive'
              )}
              rows={1}
              placeholder={isModelSelected ? 'Ask me anything...' : 'Please select a model first'}
              value={input}
              onChange={(e) => setInput(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === 'Enter' && !e.shiftKey) {
                  e.preventDefault();
                  handleSubmit(e);
                }
              }}
            />
            {streamLoading ? (
              <Button
                type="button"
                size="icon"
                variant="destructive"
                data-testid={getTestId('stop-button')}
                className="absolute right-2 h-8 w-8"
                onClick={onStop}
              >
                <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 16 16" fill="none" className="h-4 w-4">
                  <rect x="3" y="3" width="10" height="10" rx="1" fill="currentColor" />
                </svg>
                <span className="sr-only">Stop generating</span>
              </Button>
            ) : (
              <Button
                type="submit"
                size="icon"
                data-testid={getTestId('send-button')}
                disabled={!input.trim() || !isModelSelected}
                className="absolute right-2 h-8 w-8"
              >
                <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 16 16" fill="none" className="h-4 w-4" strokeWidth="2">
                  <path
                    d="M.5 1.163A1 1 0 0 1 1.97.28l12.868 6.837a1 1 0 0 1 0 1.766L1.969 15.72A1 1 0 0 1 .5 14.836V10.33a1 1 0 0 1 .816-.983L8.5 8 1.316 6.653A1 1 0 0 1 .5 5.67V1.163Z"
                    fill="currentColor"
                  />
                </svg>
                <span className="sr-only">Send message</span>
              </Button>
            )}
          </form>
        </div>

        <p className="px-2 py-2 text-center text-xs text-muted-foreground" data-testid={getTestId('chat-disclaimer')}>
          Chat assistant can make mistakes.
        </p>
      </div>
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
      metadata: assistantMsg.usage
        ? {
            usage: {
              prompt_tokens: assistantMsg.usage.input,
              completion_tokens: assistantMsg.usage.output,
              total_tokens: assistantMsg.usage.totalTokens,
            },
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

export function ChatUI() {
  const { currentChat } = useChatDB();
  const { showError } = useToastMessages();
  const { model } = useChatSettings();
  const { open: openSettings, setOpen: setOpenSettings } = useSidebar();

  const {
    enabledTools: enabledMcpTools,
    toggleTool: toggleMcpTool,
    toggleMcp,
    setEnabledTools: setEnabledMcpTools,
  } = useMcpSelection();
  const { data: mcpsResponse } = useListMcps();
  const mcps = useMemo(() => mcpsResponse?.mcps || [], [mcpsResponse?.mcps]);

  const mcpClients = useMcpClients();

  useEffect(() => {
    const enabledMcps = mcps.filter((m) => m.mcp_server.enabled && m.enabled && m.path);
    mcpClients.connectAll(enabledMcps);
    return () => {
      mcpClients.disconnectAll();
    };
  }, [mcps]); // eslint-disable-line react-hooks/exhaustive-deps

  const mcpSlugs = useMemo(() => {
    const map = new Map<string, string>();
    mcps.forEach((m) => map.set(m.id, m.slug));
    return map;
  }, [mcps]);

  const mcpConnectionStatus = useMemo(() => {
    const map = new Map<string, McpConnectionStatus>();
    for (const [id, state] of mcpClients.clients) {
      map.set(id, state.status);
    }
    return map;
  }, [mcpClients.clients]);

  useEffect(() => {
    if (mcps.length === 0) return;

    const availableIds = new Set(mcps.filter((m) => m.mcp_server.enabled && m.enabled).map((m) => m.id));

    const filtered: Record<string, string[]> = {};
    let hasUnavailable = false;
    for (const [id, tools] of Object.entries(enabledMcpTools)) {
      if (availableIds.has(id)) {
        filtered[id] = tools;
      } else {
        hasUnavailable = true;
      }
    }

    if (hasUnavailable) {
      setEnabledMcpTools(filtered);
    }
  }, [mcps, enabledMcpTools, setEnabledMcpTools]);

  const agentTools = useMcpAgentTools({
    enabledMcpTools,
    allTools: mcpClients.allTools,
    slugs: mcpSlugs,
    callTool: mcpClients.callTool,
  });

  const {
    input,
    setInput,
    isStreaming: streamLoading,
    messages: agentMessages,
    streamingMessage,
    pendingToolCalls,
    append,
    stop,
  } = useBodhiAgent({
    tools: agentTools,
    enabledMcpTools,
  });

  const inputRef = useRef<HTMLTextAreaElement>(null);
  const getTestId = useResponsiveTestId();

  useEffect(() => {
    if (!streamLoading && inputRef.current) {
      inputRef.current.focus();
    }
  }, [streamLoading]);

  const handleSubmit = async (e: FormEvent) => {
    e.preventDefault();
    if (!input.trim() || streamLoading) return;
    if (!model) {
      showError('No Model Selected', 'Please select an Alias/Model from settings before sending a message.');
      if (!openSettings) {
        setOpenSettings(true);
      }
      return;
    }

    const content = input.trim();
    setInput('');
    await append(content);
  };

  const isEmpty = agentMessages.length === 0 && !streamingMessage;

  return (
    <div data-testid={getTestId('chat-ui')} className="flex h-full flex-col">
      <div className="relative flex-1 min-h-0" data-testid={getTestId('chat-content-area')}>
        <div className="absolute inset-0 overflow-y-auto" data-testid={getTestId('chat-scroll-area')}>
          <div
            className="sticky top-0 h-8 bg-background/80 backdrop-blur-sm z-30"
            data-testid={getTestId('chat-header-spacer')}
          />
          <div className="px-3" data-testid={getTestId('chat-messages-container')}>
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
        enabledMcpTools={enabledMcpTools}
        onToggleMcpTool={toggleMcpTool}
        onToggleMcp={toggleMcp}
        mcpTools={mcpClients.allTools}
        mcpConnectionStatus={mcpConnectionStatus}
      />
    </div>
  );
}
