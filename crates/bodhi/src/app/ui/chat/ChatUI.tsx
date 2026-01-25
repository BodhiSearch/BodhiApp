'use client';

import { FormEvent, RefObject, useEffect, useRef, memo, useMemo } from 'react';

import { Plus } from 'lucide-react';

import { ChatMessage } from '@/app/ui/chat/ChatMessage';
import { ToolsetsPopover } from '@/app/ui/chat/ToolsetsPopover';
import { Button } from '@/components/ui/button';
import { ScrollAnchor } from '@/components/ui/scroll-anchor';
import { useSidebar } from '@/components/ui/sidebar';
import { useChat } from '@/hooks/use-chat';
import { useChatDB } from '@/hooks/use-chat-db';
import { useChatSettings } from '@/hooks/use-chat-settings';
import { useResponsiveTestId } from '@/hooks/use-responsive-testid';
import { useToastMessages } from '@/hooks/use-toast-messages';
import { useToolsetSelection } from '@/hooks/use-toolset-selection';
import { useToolsets } from '@/hooks/useToolsets';
import { cn } from '@/lib/utils';
import { Message } from '@/types/chat';

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
  inputRef: RefObject<HTMLTextAreaElement>;
  isModelSelected: boolean;
  enabledTools: Record<string, string[]>;
  onToggleTool: (toolsetId: string, toolName: string) => void;
  onToggleToolset: (toolsetId: string, allToolNames: string[]) => void;
}

const ChatInput = memo(function ChatInput({
  input,
  setInput,
  handleSubmit,
  streamLoading,
  inputRef,
  isModelSelected,
  enabledTools,
  onToggleTool,
  onToggleToolset,
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
          {/* Left side buttons container */}
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

            <ToolsetsPopover
              enabledTools={enabledTools}
              onToggleTool={onToggleTool}
              onToggleToolset={onToggleToolset}
              disabled={streamLoading}
            />
          </div>

          <form onSubmit={handleSubmit} className="flex w-full items-center" data-testid={getTestId('chat-form')}>
            <textarea
              ref={inputRef}
              data-testid={getTestId('chat-input')}
              className={cn(
                'flex-1 resize-none bg-transparent pl-24 pr-12 py-3 text-sm outline-none disabled:opacity-50',
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
            <Button
              type="submit"
              size="icon"
              data-testid={getTestId('send-button')}
              disabled={!input.trim() || streamLoading || !isModelSelected}
              className="absolute right-2 h-8 w-8"
            >
              <svg
                xmlns="http://www.w3.org/2000/svg"
                viewBox="0 0 16 16"
                fill="none"
                className="h-4 w-4"
                strokeWidth="2"
              >
                <path
                  d="M.5 1.163A1 1 0 0 1 1.97.28l12.868 6.837a1 1 0 0 1 0 1.766L1.969 15.72A1 1 0 0 1 .5 14.836V10.33a1 1 0 0 1 .816-.983L8.5 8 1.316 6.653A1 1 0 0 1 .5 5.67V1.163Z"
                  fill="currentColor"
                />
              </svg>
              <span className="sr-only">Send message</span>
            </Button>
          </form>
        </div>

        <p className="px-2 py-2 text-center text-xs text-muted-foreground" data-testid={getTestId('chat-disclaimer')}>
          Chat assistant can make mistakes.
        </p>
      </div>
    </div>
  );
});

interface MessageListProps {
  messages: Message[];
  userMessage: Message;
  assistantMessage: Message;
  isStreaming: boolean;
  isExecutingTools: boolean;
}

const MessageList = memo(function MessageList({
  messages,
  userMessage,
  assistantMessage,
  isStreaming,
  isExecutingTools,
}: MessageListProps) {
  const hasCurrentUserMessage = userMessage.content.length > 0;
  const hasCurrentAssistantMessage =
    assistantMessage.content.length > 0 || (assistantMessage.tool_calls?.length ?? 0) > 0;

  // Combine persisted messages with current streaming message for tool result lookup
  const allMessages = [
    ...messages,
    ...(userMessage.content ? [userMessage] : []),
    ...(hasCurrentAssistantMessage ? [assistantMessage] : []),
  ];

  return (
    <div className="space-y-2 py-2" data-testid="message-list">
      {messages.map((message, i) => {
        // Skip tool messages - they are rendered inside ToolCallMessage
        if (message.role === 'tool') {
          return null;
        }

        const isLastMessage = i === messages.length - 1;
        const isUser = message.role === 'user';

        // Determine if this message should be marked as archived
        // User messages become archived when there's a current user message or streaming assistant message
        // Assistant messages become archived when there's a current user message
        const isArchived = isUser
          ? hasCurrentUserMessage || (hasCurrentAssistantMessage && isStreaming)
          : hasCurrentUserMessage;

        // Latest message logic: only applies to the last message of each type if no current messages
        const isLatest = isLastMessage && !hasCurrentUserMessage && !hasCurrentAssistantMessage;

        return (
          <ChatMessage
            key={`history-${i}`}
            message={message}
            isLatest={isLatest}
            isArchived={isArchived}
            allMessages={allMessages}
          />
        );
      })}

      {userMessage.content && (
        <ChatMessage key="user-current" message={userMessage} isLatest={true} isArchived={false} />
      )}

      {hasCurrentAssistantMessage && (
        <ChatMessage
          key="assistant-current"
          message={assistantMessage}
          isStreaming={isStreaming}
          isLatest={!isStreaming}
          isArchived={false}
          allMessages={allMessages}
          isExecutingTools={isExecutingTools}
          data-testid="streaming-message"
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

  // Toolset selection
  const { enabledTools, toggleTool, toggleToolset, setEnabledTools } = useToolsetSelection();
  const { data: toolsetsResponse } = useToolsets();
  const toolsets = useMemo(() => toolsetsResponse?.toolsets || [], [toolsetsResponse?.toolsets]);
  const toolsetTypes = useMemo(() => toolsetsResponse?.toolset_types || [], [toolsetsResponse?.toolset_types]);

  // Create scope enabled map from toolset_types
  const scopeEnabledMap = useMemo(() => {
    const map = new Map<string, boolean>();
    toolsetTypes.forEach((config) => map.set(config.scope, config.enabled));
    return map;
  }, [toolsetTypes]);

  // Auto-filter unavailable toolsets from selection
  useEffect(() => {
    if (toolsets.length === 0) return;

    const availableIds = new Set(
      toolsets.filter((t) => (scopeEnabledMap.get(t.scope) ?? true) && t.enabled && t.has_api_key).map((t) => t.id)
    );

    const filtered: Record<string, string[]> = {};
    let hasUnavailable = false;
    for (const [id, tools] of Object.entries(enabledTools)) {
      if (availableIds.has(id)) {
        filtered[id] = tools;
      } else {
        hasUnavailable = true;
      }
    }

    if (hasUnavailable) {
      setEnabledTools(filtered);
    }
  }, [toolsets, scopeEnabledMap, enabledTools, setEnabledTools]);

  // Chat with toolsets support
  const {
    input,
    setInput,
    isLoading: streamLoading,
    append,
    userMessage,
    assistantMessage,
    pendingToolCalls,
  } = useChat({
    enabledTools,
    toolsets,
    toolsetTypes,
  });

  const inputRef = useRef<HTMLTextAreaElement>(null);
  const getTestId = useResponsiveTestId();

  // Check if tools are being executed (have pending tool calls but no results yet)
  const isExecutingTools = pendingToolCalls.length > 0;

  useEffect(() => {
    if (!streamLoading && inputRef.current) {
      inputRef.current.focus();
    }
  }, [streamLoading]);

  const handleSubmit = async (e: FormEvent) => {
    e.preventDefault();
    if (!input.trim() || streamLoading) return;
    // Check if model is selected
    if (!model) {
      showError('No Model Selected', 'Please select an Alias/Model from settings before sending a message.');
      // Open settings panel if it's not already open
      if (!openSettings) {
        setOpenSettings(true);
      }
      return;
    }

    const content = input.trim();
    setInput('');
    await append(content);
  };

  return (
    <div data-testid={getTestId('chat-ui')} className="flex h-full flex-col">
      <div className="relative flex-1 min-h-0" data-testid={getTestId('chat-content-area')}>
        <div className="absolute inset-0 overflow-y-auto" data-testid={getTestId('chat-scroll-area')}>
          <div
            className="sticky top-0 h-8 bg-background/80 backdrop-blur-sm z-30"
            data-testid={getTestId('chat-header-spacer')}
          />
          <div className="px-3" data-testid={getTestId('chat-messages-container')}>
            {(currentChat === null || !currentChat?.messages?.length) && !userMessage.content ? (
              <EmptyState />
            ) : (
              <MessageList
                messages={currentChat?.messages || []}
                userMessage={userMessage}
                assistantMessage={assistantMessage}
                isStreaming={streamLoading}
                isExecutingTools={isExecutingTools}
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
        inputRef={inputRef}
        isModelSelected={!!model}
        enabledTools={enabledTools}
        onToggleTool={toggleTool}
        onToggleToolset={toggleToolset}
      />
    </div>
  );
}
