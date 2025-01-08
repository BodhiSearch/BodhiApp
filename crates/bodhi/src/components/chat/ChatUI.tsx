'use client';

import { useChat } from '@/hooks/use-chat';
import { cn } from '@/lib/utils';
import { Message } from '@/types/chat';
import { FormEvent } from 'react';
import { ChatMessage } from './ChatMessage';
import { Skeleton } from '@/components/ui/skeleton';

interface ChatUIProps {
  isLoading: boolean;
}

export function ChatUI({ isLoading: chatLoading }: ChatUIProps) {
  const {
    messages,
    input,
    setInput,
    isLoading: streamLoading,
    append,
    stop,
  } = useChat();

  const handleSubmit = async (e: FormEvent) => {
    e.preventDefault();
    if (!input.trim() || streamLoading) return;

    const userMessage: Message = {
      role: 'user',
      content: input.trim()
    };

    setInput('');
    await append(userMessage);
  };

  return (
    <div data-testid="chat-ui" className="flex flex-col flex-1 min-h-0">
      {/* Messages area */}
      <div className="flex-1 overflow-auto p-4">
        {chatLoading ? (
          // Loading skeletons
          <div className="space-y-4">
            {[...Array(2)].map((_, i) => (
              <div key={i} className="flex items-start gap-4">
                <Skeleton className="h-8 w-8 rounded-full" />
                <div className="space-y-2 flex-1">
                  <Skeleton className="h-4 w-24" />
                  <Skeleton className="h-16 w-full" />
                </div>
              </div>
            ))}
          </div>
        ) : messages.length === 0 ? (
          // Empty state
          <div className="flex h-full items-center justify-center text-center">
            <div className="space-y-4">
              <h3 className="text-lg font-semibold">Welcome to Chat</h3>
              <p className="text-muted-foreground">
                Start a conversation by typing a message below.
              </p>
            </div>
          </div>
        ) : (
          // Messages
          messages.map((message, i) => (
            <ChatMessage key={i} message={message} />
          ))
        )}

        {streamLoading && (
          <div className="flex items-center justify-center py-4">
            <div className="animate-pulse text-muted-foreground">
              Assistant is typing...
            </div>
          </div>
        )}
      </div>

      {/* Input form */}
      <form onSubmit={handleSubmit} className="border-t p-4">
        <div className="flex gap-2 max-w-2xl mx-auto">
          <textarea
            rows={1}
            value={input}
            onChange={(e) => setInput(e.target.value)}
            placeholder="Type your message..."
            className="flex-1 p-2 border rounded resize-none"
            disabled={streamLoading}
            onKeyDown={(e) => {
              if (e.key === 'Enter' && !e.shiftKey) {
                e.preventDefault();
                handleSubmit(e);
              }
            }}
          />
          <div className="flex gap-2">
            <button
              type="submit"
              disabled={streamLoading || !input.trim()}
              className={cn(
                'px-4 py-2 rounded transition-colors',
                streamLoading || !input.trim()
                  ? 'bg-gray-300'
                  : 'bg-blue-500 text-white hover:bg-blue-600'
              )}
            >
              Send
            </button>
            {streamLoading && (
              <button
                onClick={stop}
                type="button"
                className="px-4 py-2 rounded bg-red-500 text-white hover:bg-red-600"
              >
                Stop
              </button>
            )}
          </div>
        </div>
      </form>
    </div>
  );
}