import { Message } from '@/types/chat';
import { MemoizedReactMarkdown } from '@/components/ui/markdown';
import { cn } from '@/lib/utils';

interface ChatMessageProps {
  message: Message;
}

export function ChatMessage({ message }: ChatMessageProps) {
  const isUser = message.role === 'user';

  return (
    <div
      className={cn(
        'rounded-lg p-4',
        isUser
          ? 'bg-primary/10 text-primary-foreground'
          : 'bg-muted/50 text-foreground'
      )}
    >
      <div className="mb-2 text-sm font-medium">
        {isUser ? 'You' : 'Assistant'}
      </div>
      <MemoizedReactMarkdown className="prose prose-sm dark:prose-invert max-w-none break-words prose-p:leading-relaxed prose-pre:p-0">
        {message.content}
      </MemoizedReactMarkdown>
    </div>
  );
}
