'use client';

import { useChat } from '@/hooks/use-chat';
import { Message } from '@/types/chat';
import { FormEvent, useRef, useEffect, RefObject } from 'react';
import { ChatMessage } from './ChatMessage';
import { Skeleton } from '@/components/ui/skeleton';
import { Button } from '@/components/ui/button';
import { Textarea } from '@/components/ui/textarea';
import { ScrollAnchor } from '@/components/ui/scroll-anchor';

// Extracted components outside the main component
const LoadingSkeletons = () => (
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
);

const EmptyState = () => (
  <div className="flex h-full items-center justify-center text-center">
    <div className="space-y-4">
      <h3 className="text-lg font-semibold">Welcome to Chat</h3>
      <p className="text-muted-foreground">
        Start a conversation by typing a message below.
      </p>
    </div>
  </div>
);

interface ChatInputProps {
  input: string;
  setInput: (value: string) => void;
  handleSubmit: (e: FormEvent) => void;
  streamLoading: boolean;
  stop: () => void;
  inputRef: RefObject<HTMLTextAreaElement>;
}

const ChatInput = ({
  input,
  setInput,
  handleSubmit,
  streamLoading,
  stop,
  inputRef,
}: ChatInputProps) => (
  <form onSubmit={handleSubmit} className="border-t p-4">
    <div className="flex gap-2 max-w-2xl mx-auto">
      <Textarea
        ref={inputRef}
        rows={1}
        value={input}
        onChange={(e) => setInput(e.target.value)}
        placeholder="Type your message..."
        className="flex-1 min-h-[44px] resize-none"
        disabled={streamLoading}
        onKeyDown={(e) => {
          if (e.key === 'Enter' && !e.shiftKey) {
            e.preventDefault();
            handleSubmit(e);
          }
        }}
      />
      <div className="flex gap-2">
        <Button
          type="submit"
          disabled={streamLoading || !input.trim()}
          variant="default"
        >
          Send
        </Button>
        {streamLoading && (
          <Button onClick={stop} type="button" variant="destructive">
            Stop
          </Button>
        )}
      </div>
    </div>
  </form>
);

interface MessageListProps {
  messages: Message[];
}

const MessageList = ({ messages }: MessageListProps) => (
  <>
    {messages.map((message, i) => (
      <ChatMessage key={i} message={message} />
    ))}
    <ScrollAnchor />
  </>
);

interface ChatUIProps {
  isLoading: boolean;
}

export function ChatUI({ isLoading: chatLoading }: ChatUIProps) {
  const {
    messages = [],
    input,
    setInput,
    isLoading: streamLoading,
    append,
    stop,
  } = useChat();

  const inputRef = useRef<HTMLTextAreaElement>(null);

  // Focus input after loading completes
  useEffect(() => {
    if (!streamLoading && inputRef.current) {
      inputRef.current.focus();
    }
  }, [streamLoading]);

  const handleSubmit = async (e: FormEvent) => {
    e.preventDefault();
    if (!input.trim() || streamLoading) return;

    const userMessage: Message = {
      role: 'user',
      content: input.trim(),
    };

    setInput('');
    await append(userMessage);
  };

  return (
    <div data-testid="chat-ui" className="flex flex-col h-full">
      <div className="flex-1 min-h-0 relative">
        <div className="absolute inset-0 overflow-y-auto p-4">
          {chatLoading ? (
            <LoadingSkeletons />
          ) : messages.length === 0 ? (
            <EmptyState />
          ) : (
            <MessageList messages={messages} />
          )}
        </div>
      </div>

      <div className="flex-none border-t bg-background">
        <ChatInput
          input={input}
          setInput={setInput}
          handleSubmit={handleSubmit}
          streamLoading={streamLoading}
          stop={stop}
          inputRef={inputRef}
        />
      </div>
    </div>
  );
}
