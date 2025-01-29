import { Message } from '@/types/chat';
import { MemoizedReactMarkdown } from '@/components/ui/markdown';
import { cn } from '@/lib/utils';
import { User, Bot, Copy, RefreshCw, ThumbsUp, ThumbsDown } from 'lucide-react';
import { Button } from '@/components/ui/button';

interface ChatMessageProps {
  message: Message;
  isStreaming?: boolean;
}

export function ChatMessage({
  message,
  isStreaming = false,
}: ChatMessageProps) {
  const isUser = message.role === 'user';

  const handleCopy = () => {
    navigator.clipboard.writeText(message.content);
  };

  const handleRewrite = () => {};

  return (
    <div
      className={cn(
        'group relative flex items-start gap-4 py-4 px-4',
        isUser && 'bg-background',
        !isUser && 'bg-muted/50'
      )}
    >
      <div
        className={cn(
          'flex h-8 w-8 shrink-0 select-none items-center justify-center rounded-md border shadow',
          isUser ? 'bg-primary text-primary-foreground' : 'bg-background'
        )}
      >
        {isUser ? <User className="h-4 w-4" /> : <Bot className="h-4 w-4" />}
      </div>

      <div className="flex-1 space-y-2">
        <div className="text-sm font-semibold">
          {isUser ? 'You' : 'Assistant'}
        </div>

        <div
          className={cn(
            'prose prose-sm dark:prose-invert max-w-none break-words',
            'prose-p:leading-relaxed prose-pre:p-0'
          )}
        >
          <MemoizedReactMarkdown>{message.content}</MemoizedReactMarkdown>
        </div>

        {!isUser && !isStreaming && (
          <div className="flex justify-between pt-2">
            <div className="flex gap-2">
              <Button
                variant="ghost"
                size="icon"
                className="h-8 w-8 p-0"
                onClick={handleRewrite}
              >
                <RefreshCw className="h-4 w-4" />
                <span className="sr-only">Rewrite</span>
              </Button>
              <Button
                variant="ghost"
                size="icon"
                className="h-8 w-8 p-0"
                onClick={handleCopy}
              >
                <Copy className="h-4 w-4" />
                <span className="sr-only">Copy</span>
              </Button>
            </div>
            <div className="flex gap-2">
              <Button variant="ghost" size="icon" className="h-8 w-8 p-0">
                <ThumbsUp className="h-4 w-4" />
                <span className="sr-only">Helpful</span>
              </Button>
              <Button variant="ghost" size="icon" className="h-8 w-8 p-0">
                <ThumbsDown className="h-4 w-4" />
                <span className="sr-only">Not helpful</span>
              </Button>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
