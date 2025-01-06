'use client';

import { useChat } from '@/lib/hooks/use-chat-context';
import { cn } from '@/lib/utils';
import { Message } from '@/types/chat';
import { FormEvent } from 'react';

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
    <div data-testid="chat-ui" className="flex flex-col h-screen">
      {/* Messages area */}
      <div className="flex-1 overflow-auto p-4">
        {messages.map((message, i) => (
          <div
            key={i}
            className={cn(
              'mb-4 p-4 rounded',
              message.role === 'user'
                ? 'bg-blue-100 ml-8'
                : 'bg-gray-100 mr-8'
            )}
          >
            <div className="font-bold mb-1">
              {message.role === 'user' ? 'You' : 'Assistant'}
            </div>
            <div className="whitespace-pre-wrap">{message.content}</div>
          </div>
        ))}
        {(chatLoading || streamLoading) && (
          <div className="text-center text-gray-500">
            {chatLoading ? 'Loading chat...' : 'Assistant is typing...'}
          </div>
        )}
      </div>

      {/* Input area */}
      <form onSubmit={handleSubmit} className="border-t p-4">
        <div className="flex gap-2">
          <input
            type="text"
            value={input}
            onChange={(e) => setInput(e.target.value)}
            placeholder="Type your message..."
            className="flex-1 p-2 border rounded"
            disabled={streamLoading}
          />
          <button
            type="submit"
            disabled={streamLoading || !input.trim()}
            className={cn(
              'px-4 py-2 rounded',
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
              className="px-4 py-2 rounded bg-red-500 text-white hover:bg-red-600"
            >
              Stop
            </button>
          )}
        </div>
      </form>
    </div>
  );
}