import { User, Bot } from 'lucide-react';

import { ToolCallsDisplay } from '@/app/ui/chat/ToolCallMessage';
import { CopyButton } from '@/components/CopyButton';
import { MemoizedReactMarkdown } from '@/components/ui/markdown';
import { cn } from '@/lib/utils';
import { Message } from '@/types/chat';

interface ChatMessageProps {
  message: Message;
  isStreaming?: boolean;
  isLatest?: boolean;
  isArchived?: boolean;
  /** All messages in the conversation (for finding tool results) */
  allMessages?: Message[];
  /** Whether tools are currently being executed */
  isExecutingTools?: boolean;
}

export function ChatMessage({
  message,
  isStreaming = false,
  isLatest = false,
  isArchived = false,
  allMessages = [],
  isExecutingTools = false,
}: ChatMessageProps) {
  const isUser = message.role === 'user';
  const isAssistant = message.role === 'assistant';
  const isTool = message.role === 'tool';
  const metadata = message.metadata;

  // Skip rendering tool messages - they are displayed nested under ToolCallMessage
  if (isTool) {
    return null;
  }

  const formatNumber = (num: number) => num.toFixed(2);

  // Determine CSS classes based on message state
  const getMessageClasses = () => {
    if (isUser) {
      if (isLatest) return 'chat-user-message';
      if (isArchived) return 'chat-user-message-archive';
      return '';
    } else {
      if (isStreaming) return 'chat-ai-streaming';
      if (isLatest) return 'chat-ai-message';
      if (isArchived) return 'chat-ai-archive';
      return '';
    }
  };

  // Check if this assistant message has tool calls
  const hasToolCalls = isAssistant && message.tool_calls && message.tool_calls.length > 0;

  return (
    <div
      data-testid={isUser ? 'user-message' : isStreaming ? 'streaming-message' : 'assistant-message'}
      className={cn(
        'group relative flex items-start gap-3 p-3',
        isUser ? 'bg-background' : 'bg-muted/30',
        !isUser && isStreaming && 'message-streaming',
        !isUser && !isStreaming && 'message-completed',
        getMessageClasses()
      )}
    >
      <div
        className={cn(
          'flex h-7 w-7 shrink-0 items-center justify-center rounded-md border shadow mt-1',
          isUser ? 'bg-primary text-primary-foreground' : 'bg-background'
        )}
      >
        {isUser ? <User className="h-4 w-4" /> : <Bot className="h-4 w-4" />}
      </div>

      <div className="flex-1 min-w-0">
        <div className="text-xs font-medium mb-1.5">{isUser ? 'You' : 'Assistant'}</div>

        {/* Tool calls display (before content, as they are executed first) */}
        {hasToolCalls && (
          <ToolCallsDisplay toolCalls={message.tool_calls!} messages={allMessages} isExecuting={isExecutingTools} />
        )}

        {/* Message content */}
        {message.content && (
          <div data-testid={`${isUser ? 'user' : isStreaming ? 'streaming' : 'assistant'}-message-content`}>
            <MemoizedReactMarkdown>{message.content}</MemoizedReactMarkdown>
          </div>
        )}

        {!isUser && !isStreaming && (
          <div
            className="flex justify-between items-center mt-2 text-xs text-muted-foreground"
            data-testid="message-metadata"
          >
            <div className="flex items-center gap-4">
              {metadata?.usage && (
                <div className="flex items-center gap-2">
                  <span>Query: {metadata.usage.prompt_tokens} tokens</span>
                  <span>•</span>
                  <span>Response: {metadata.usage.completion_tokens} tokens</span>
                </div>
              )}
              {metadata?.timings?.prompt_per_second && metadata?.timings?.predicted_per_second && (
                <div className="flex items-center gap-2">
                  <span>•</span>
                  <span>Speed: {formatNumber(metadata.timings.predicted_per_second)} t/s</span>
                </div>
              )}
            </div>
            {message.content && (
              <CopyButton
                text={message.content}
                className="opacity-0 group-hover:opacity-100 transition-opacity h-8 w-8"
                showToast={true}
              />
            )}
          </div>
        )}
      </div>
    </div>
  );
}
