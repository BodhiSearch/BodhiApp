'use client';

import { ChatMessage } from '@/components/chat/ChatMessage';
import { Button } from '@/components/ui/button';
import { ScrollAnchor } from '@/components/ui/scroll-anchor';
import { useChat } from '@/hooks/use-chat';
import { useChatDB } from '@/hooks/use-chat-db';
import { Message } from '@/types/chat';
import { FormEvent, RefObject, useEffect, useRef, memo } from 'react';

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
  inputRef: RefObject<HTMLTextAreaElement>;
}

const ChatInput = ({
  input,
  setInput,
  handleSubmit,
  streamLoading,
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
      </div>
    </form>
  </div>
);

interface MessageListProps {
  messages: Message[];
  userMessage: Message;
  assistantMessage: Message;
}

const MessageList = memo(function MessageList({
  messages,
  userMessage,
  assistantMessage,
}: MessageListProps) {
  return (
    <>
      {messages.map((message, i) => (
        <ChatMessage key={`history-${i}`} message={message} />
      ))}
      {userMessage.content && (
        <ChatMessage key="user-current" message={userMessage} />
      )}
      {assistantMessage.content && (
        <ChatMessage key="assistant-current" message={assistantMessage} />
      )}
      <ScrollAnchor />
    </>
  );
});

export function ChatUI() {
  const { currentChat } = useChatDB();
  const {
    input,
    setInput,
    isLoading: streamLoading,
    append,
    userMessage,
    assistantMessage,
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
    const content = input.trim();
    setInput('');
    await append(content);
  };

  return (
    <div data-testid="chat-ui" className="flex-1 flex flex-col min-h-0">
      <div className="flex-1 overflow-hidden relative">
        <div className="absolute inset-0 overflow-y-auto">
          <div className="p-4">
            {(currentChat === null || !currentChat?.messages?.length) &&
            !userMessage.content ? (
              <EmptyState />
            ) : (
              <MessageList
                messages={currentChat?.messages || []}
                userMessage={userMessage}
                assistantMessage={assistantMessage}
              />
            )}
          </div>
        </div>
      </div>
      <ChatInput
        input={input}
        setInput={setInput}
        handleSubmit={handleSubmit}
        streamLoading={streamLoading}
        inputRef={inputRef}
      />
    </div>
  );
}
