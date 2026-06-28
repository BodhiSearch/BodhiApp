import { Sparkles } from 'lucide-react';

import { ToolCallsDisplay } from './ToolCallMessage';
import { CopyButton } from '@/components/CopyButton';
import { MemoizedReactMarkdown } from '@/components/ui/markdown';
import { cn } from '@/lib/utils';
import { Message } from '@/types/chat';

interface ChatMessageProps {
  message: Message;
  isStreaming?: boolean;
  isLatest?: boolean;
  isArchived?: boolean;
  allMessages?: Message[];
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

  // Streaming-state marker classes the E2E suite keys on for its wait conditions —
  // restyle the look around them, but keep the class names intact.
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

  const hasToolCalls = isAssistant && message.tool_calls && message.tool_calls.length > 0;

  return (
    <div
      data-testid={isUser ? 'user-message' : isStreaming ? 'streaming-message' : 'assistant-message'}
      // Served model echoed by the upstream (per-target for model-routers). Non-visible
      // signal used by E2E to assert which target served a response.
      data-served-model={!isUser && !isStreaming ? (metadata?.model ?? undefined) : undefined}
      className={cn(
        'chat-msg group',
        !isUser && isStreaming && 'message-streaming',
        !isUser && !isStreaming && 'message-completed',
        getMessageClasses()
      )}
    >
      <div className={cn('chat-avatar', isUser ? 'chat-avatar-user' : 'chat-avatar-ai')}>
        {isUser ? 'YO' : <Sparkles className="h-3.5 w-3.5" />}
      </div>

      <div className="chat-bubble">
        <div className="chat-name">
          {isUser ? 'You' : 'Bodhi'}
          {!isUser && metadata?.model && <span className="chat-model-tag">{metadata.model}</span>}
        </div>

        {/* Tool calls display (before content, as they are executed first) */}
        {hasToolCalls && (
          <ToolCallsDisplay toolCalls={message.tool_calls!} messages={allMessages} isExecuting={isExecutingTools} />
        )}

        {message.content && (
          <div
            className="chat-body"
            data-testid={`${isUser ? 'user' : isStreaming ? 'streaming' : 'assistant'}-message-content`}
          >
            {isUser ? (
              <div className="chat-user-body">
                <MemoizedReactMarkdown>{message.content}</MemoizedReactMarkdown>
              </div>
            ) : (
              <MemoizedReactMarkdown>{message.content}</MemoizedReactMarkdown>
            )}
          </div>
        )}

        {!isUser && !isStreaming && (
          <div className="chat-meta-strip" data-testid="message-metadata">
            {metadata?.usage && (
              <>
                <span className="chat-mi">
                  Query: <b>{metadata.usage.prompt_tokens}</b>&thinsp;tokens
                </span>
                <span className="chat-mi">
                  Response: <b>{metadata.usage.completion_tokens}</b>&thinsp;tokens
                </span>
              </>
            )}
            {metadata?.timings?.predicted_per_second && (
              <span className="chat-mi">
                <b>{formatNumber(metadata.timings.predicted_per_second)}</b>&thinsp;t/s
              </span>
            )}
            <span className="chat-meta-spacer" />
            {message.content && (
              <CopyButton
                text={message.content}
                className="opacity-0 group-hover:opacity-100 transition-opacity h-7 w-7"
                showToast={true}
              />
            )}
          </div>
        )}
      </div>
    </div>
  );
}
