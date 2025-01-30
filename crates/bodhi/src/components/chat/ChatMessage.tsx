import { Message } from '@/types/chat';
import { MemoizedReactMarkdown } from '@/components/ui/markdown';
import { cn } from '@/lib/utils';
import { User, Bot, Copy } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { useToastMessages } from '@/hooks/use-toast-messages';

interface ChatMessageProps {
  message: Message;
  isStreaming?: boolean;
}

export function ChatMessage({
  message,
  isStreaming = false,
}: ChatMessageProps) {
  const isUser = message.role === 'user';
  const { showSuccess } = useToastMessages();

  const handleCopy = () => {
    navigator.clipboard.writeText(message.content);
    showSuccess('Copied', 'Chat message copied to clipboard');
  };

  return (
    <div
      className={cn(
        'group relative flex items-start gap-3 py-3 px-3',
        isUser && 'bg-background',
        !isUser && 'bg-muted/50'
      )}
    >
      <div
        className={cn(
          'flex h-7 w-7 shrink-0 select-none items-center justify-center rounded-md border shadow',
          isUser ? 'bg-primary text-primary-foreground' : 'bg-background'
        )}
      >
        {isUser ? <User className="h-4 w-4" /> : <Bot className="h-4 w-4" />}
      </div>

      <div className="flex-1 space-y-1">
        <div className="text-xs font-medium">
          {isUser ? 'You' : 'Assistant'}
        </div>

        <div
          className={cn(
            'prose prose-sm dark:prose-invert max-w-none break-words',
            'prose-p:leading-relaxed prose-pre:p-0',
            'prose-p:my-1 prose-pre:my-1'
          )}
        >
          <MemoizedReactMarkdown>{message.content}</MemoizedReactMarkdown>
        </div>

        {!isUser && !isStreaming && (
          <div className="flex justify-end pt-1">
            <div className="flex gap-2">
              <Button
                variant="ghost"
                size="icon"
                className="h-7 w-7 p-0"
                onClick={handleCopy}
              >
                <Copy className="h-3.5 w-3.5" />
                <span className="sr-only">Copy</span>
              </Button>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
