'use client';

import { useChatCompletion } from '@/hooks/use-chat-completions';
import { useChatDB } from '@/hooks/use-chat-db';
import { useChatSettings } from '@/hooks/use-chat-settings';
import { Chat, Message } from '@/types/chat';
import { createContext, useCallback, useContext, useState } from 'react';

interface ChatContextType {
  messages: Message[];
  input: string;
  setInput: (input: string) => void;
  isLoading: boolean;
  append: (message: Message) => Promise<void>;
  stop: () => void;
  reload: () => Promise<void>;
}

const ChatContext = createContext<ChatContextType | undefined>(undefined);

interface ChatProviderProps {
  children: React.ReactNode;
  chat: Chat;
}

export function ChatProvider({ children, chat }: ChatProviderProps) {
  const [messages, setMessages] = useState<Message[]>(chat?.messages || []);
  const [input, setInput] = useState('');
  const [abortController, setAbortController] =
    useState<AbortController | null>(null);

  const { append, isLoading } = useChatCompletion();
  const { createOrUpdateChat } = useChatDB();
  const chatSettings = useChatSettings();

  const stop = useCallback(() => {
    if (abortController) {
      abortController.abort();
      setAbortController(null);
    }
  }, [abortController]);

  const processCompletion = useCallback(
    async (requestMessages: Message[]) => {
      let assistantMessage = '';

      try {
        await append({
          request: {
            ...chatSettings.getRequestSettings(),
            messages: requestMessages,
          },
          onDelta: (chunk) => {
            assistantMessage += chunk;
            setMessages([
              ...requestMessages,
              { role: 'assistant' as const, content: assistantMessage },
            ]);
          },
          onFinish: (message) => {
            const finalMessages = [...requestMessages, message];
            setMessages(finalMessages);
            createOrUpdateChat({
              ...chat,
              messages: finalMessages,
              title: finalMessages[0].content.substring(0, 100),
              updatedAt: Date.now(),
            });
          },
        });
      } catch (error) {
        console.error('Chat completion error:', error);
        throw error;
      }
    },
    [chatSettings, chat, append, createOrUpdateChat]
  );

  const appendMessage = useCallback(
    async (userMessage: Message) => {
      const newMessages = [...messages, userMessage];
      setMessages(newMessages);

      const controller = new AbortController();
      setAbortController(controller);

      try {
        await processCompletion(newMessages);
      } finally {
        setAbortController(null);
      }
    },
    [messages, processCompletion]
  );

  const reload = useCallback(async () => {
    if (messages.length < 2) return;

    // Remove the last assistant message
    const lastUserMessageIndex = messages
      .map((m) => m.role)
      .lastIndexOf('user');
    if (lastUserMessageIndex === -1) return;

    const messagesToKeep = messages.slice(0, lastUserMessageIndex + 1);
    setMessages(messagesToKeep);

    await processCompletion(messagesToKeep);
  }, [messages, processCompletion]);

  return (
    <ChatContext.Provider
      value={{
        messages,
        input,
        setInput,
        isLoading,
        append: appendMessage,
        stop,
        reload,
      }}
    >
      {children}
    </ChatContext.Provider>
  );
}

export function useChat() {
  const context = useContext(ChatContext);
  if (!context) {
    throw new Error('useChat must be used within a ChatProvider');
  }
  return context;
}
