import { Message } from '@/types/chat';
import { MemoizedReactMarkdown } from '@/components/ui/markdown';
import { cn } from '@/lib/utils';

interface ChatMessageProps {
  message: Message;
}

export function ChatMessage({ message }: ChatMessageProps) {
  return (
    <div
      className={cn(
        'mb-4 p-4 rounded',
        message.role === 'user' ? 'message-user' : 'message-assistant'
      )}
    >
      <div className="font-bold mb-2">
        {message.role === 'user' ? 'You' : 'Assistant'}
      </div>

      <MemoizedReactMarkdown
        className="prose break-words prose-p:leading-relaxed prose-pre:p-0"
      >
        {message.content}
      </MemoizedReactMarkdown>
    </div>
  );
}