import { Message } from '@/types/chat';
import { MemoizedReactMarkdown } from '@/components/ui/markdown';
import { cn } from '@/lib/utils';
import { User, Bot } from 'lucide-react';
import { CopyButton } from '@/components/CopyButton';

interface ChatMessageProps {
  message: Message;
  isStreaming?: boolean;
}

export function ChatMessage({
  message,
  isStreaming = false,
}: ChatMessageProps) {
  const isUser = message.role === 'user';

  return (
    <div
      className={cn(
        'group relative flex items-start gap-3 p-3',
        isUser ? 'bg-background' : 'bg-muted/30'
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
        <div className="text-xs font-medium mb-1.5">
          {isUser ? 'You' : 'Assistant'}
        </div>

        <MemoizedReactMarkdown>{message.content}</MemoizedReactMarkdown>

        {!isUser && !isStreaming && (
          <div className="flex justify-end mt-2">
            <CopyButton
              text={message.content}
              className="opacity-0 group-hover:opacity-100 transition-opacity h-8 w-8"
              showToast={true}
            />
          </div>
        )}
      </div>
    </div>
  );
}
