'use client';

import { ChatMessage } from '@/app/ui/chat/ChatMessage';
import { Button } from '@/components/ui/button';
import { ScrollAnchor } from '@/components/ui/scroll-anchor';
import { useChat } from '@/hooks/use-chat';
import { useChatDB } from '@/hooks/use-chat-db';
import { useChatSettings } from '@/hooks/use-chat-settings';
import { Message } from '@/types/chat';
import { FormEvent, RefObject, useEffect, useRef, memo } from 'react';
import { Plus } from 'lucide-react';
import { useSidebar } from '@/components/ui/sidebar';
import { cn } from '@/lib/utils';
import { useToastMessages } from '@/hooks/use-toast-messages';

const EmptyState = () => (
  <div className="flex h-full items-center justify-center">
    <div className="text-center space-y-3">
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
  isModelSelected: boolean;
}

const ChatInput = memo(function ChatInput({
  input,
  setInput,
  handleSubmit,
  streamLoading,
  inputRef,
  isModelSelected,
}: ChatInputProps) {
  const { createNewChat } = useChatDB();

  return (
    <div className="sticky bottom-0 border-t bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/75">
      <div className="mx-auto max-w-3xl px-4 py-2">
        <div className="relative flex items-center rounded-lg border bg-background shadow-sm">
          <Button
            type="button"
            variant="ghost"
            size="icon"
            className="absolute left-2 h-8 w-8"
            onClick={createNewChat}
          >
            <Plus className="h-5 w-5" />
            <span className="sr-only">New chat</span>
          </Button>

          <form onSubmit={handleSubmit} className="flex w-full items-center">
            <textarea
              ref={inputRef}
              className={cn(
                'flex-1 resize-none bg-transparent px-12 py-3 text-sm outline-none disabled:opacity-50',
                !isModelSelected && 'ring-2 ring-destructive'
              )}
              rows={1}
              placeholder={
                isModelSelected
                  ? 'Ask me anything...'
                  : 'Please select a model first'
              }
              value={input}
              onChange={(e) => setInput(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === 'Enter' && !e.shiftKey) {
                  e.preventDefault();
                  handleSubmit(e);
                }
              }}
            />
            <Button
              type="submit"
              size="icon"
              disabled={!input.trim() || streamLoading || !isModelSelected}
              className="absolute right-2 h-8 w-8"
            >
              <svg
                xmlns="http://www.w3.org/2000/svg"
                viewBox="0 0 16 16"
                fill="none"
                className="h-4 w-4"
                strokeWidth="2"
              >
                <path
                  d="M.5 1.163A1 1 0 0 1 1.97.28l12.868 6.837a1 1 0 0 1 0 1.766L1.969 15.72A1 1 0 0 1 .5 14.836V10.33a1 1 0 0 1 .816-.983L8.5 8 1.316 6.653A1 1 0 0 1 .5 5.67V1.163Z"
                  fill="currentColor"
                />
              </svg>
              <span className="sr-only">Send message</span>
            </Button>
          </form>
        </div>

        <p className="px-2 py-2 text-center text-xs text-muted-foreground">
          Chat assistant can make mistakes.
        </p>
      </div>
    </div>
  );
});

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
    <div className="space-y-2 py-2">
      {messages.map((message, i) => (
        <ChatMessage key={`history-${i}`} message={message} />
      ))}
      {userMessage.content && (
        <ChatMessage key="user-current" message={userMessage} />
      )}
      {assistantMessage.content && (
        <ChatMessage
          key="assistant-current"
          message={assistantMessage}
          isStreaming={true}
        />
      )}
      <ScrollAnchor />
    </div>
  );
});

export function ChatUI() {
  const { currentChat } = useChatDB();
  const { showError } = useToastMessages();
  const { model } = useChatSettings();
  const { open: openSettings, setOpen: setOpenSettings } = useSidebar();
  const {
    input,
    setInput,
    isLoading: streamLoading,
    append,
    userMessage,
    assistantMessage,
  } = useChat();
  const inputRef = useRef<HTMLTextAreaElement>(null);

  useEffect(() => {
    if (!streamLoading && inputRef.current) {
      inputRef.current.focus();
    }
  }, [streamLoading]);

  const handleSubmit = async (e: FormEvent) => {
    e.preventDefault();
    if (!input.trim() || streamLoading) return;
    // Check if model is selected
    if (!model) {
      showError(
        'No Model Selected',
        'Please select an Alias/Model from settings before sending a message.'
      );
      // Open settings panel if it's not already open
      if (!openSettings) {
        setOpenSettings(true);
      }
      return;
    }

    const content = input.trim();
    setInput('');
    await append(content);
  };

  return (
    <div data-testid="chat-ui" className="flex h-full flex-col">
      <div className="relative flex-1 min-h-0">
        <div className="absolute inset-0 overflow-y-auto">
          <div className="sticky top-0 h-8 bg-background/80 backdrop-blur-sm z-30" />
          <div className="px-3">
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
        isModelSelected={!!model}
      />
    </div>
  );
}
