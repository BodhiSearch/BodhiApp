'use client';

import { ChatMessage } from '@/components/chat/ChatMessage';
import { Button } from '@/components/ui/button';
import { ScrollAnchor } from '@/components/ui/scroll-anchor';
import { Skeleton } from '@/components/ui/skeleton';
import { useChat } from '@/hooks/use-chat';
import { useChatDB } from '@/hooks/use-chat-db';
import { Message } from '@/types/chat';
import { FormEvent, RefObject, useEffect, useRef } from 'react';

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
  <div className="flex-1 flex items-center justify-center">
    <div className="space-y-4 text-center">
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
  <div className="sticky bottom-0 border-t bg-background">
    <form onSubmit={handleSubmit} className="flex gap-2 max-w-2xl mx-auto p-4">
      <textarea
        ref={inputRef}
        className="flex w-full rounded-md border border-input bg-background px-3 py-2 text-base ring-offset-background placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50 md:text-sm flex-1 min-h-[44px] resize-none"
        rows={1}
        placeholder="Type your message..."
        value={input}
        onChange={(e) => setInput(e.target.value)}
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
          disabled={!input.trim() || streamLoading}
          onClick={handleSubmit}
        >
          Send
        </Button>
        {streamLoading && (
          <Button onClick={stop} type="button" variant="destructive">
            Stop
          </Button>
        )}
      </div>
    </form>
  </div>
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

export function ChatUI({ isLoading }: ChatUIProps) {
  const { currentChat } = useChatDB();
  const { input, setInput, isLoading: streamLoading, append, stop } = useChat();

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
    <div data-testid="chat-ui" className="flex-1 flex flex-col min-h-0">
      <div className="flex-1 overflow-hidden relative">
        <div className="absolute inset-0 overflow-y-auto">
          <div className="p-4">
            {isLoading ? (
              <LoadingSkeletons />
            ) : !currentChat?.messages?.length ? (
              <EmptyState />
            ) : (
              <MessageList messages={currentChat.messages} />
            )}
          </div>
        </div>
      </div>
      <ChatInput
        input={input}
        setInput={setInput}
        handleSubmit={handleSubmit}
        streamLoading={streamLoading}
        stop={stop}
        inputRef={inputRef}
      />
    </div>
  );
}
