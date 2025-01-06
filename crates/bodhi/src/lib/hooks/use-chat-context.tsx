'use client';

import { createContext, useContext, useState, useCallback } from 'react';
import { Message, Chat } from '@/types/chat';
import { useChatCompletion } from '@/hooks/useQuery';
import { useChats } from '@/lib/hooks/use-chats';
import { useChatSettings } from '@/lib/hooks/use-chat-settings';

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

export function ChatProvider({
  children,
  chat,
}: ChatProviderProps) {
  const [messages, setMessages] = useState<Message[]>(chat.messages);
  const [input, setInput] = useState('');
  const [abortController, setAbortController] = useState<AbortController | null>(null);

  const { complete, stream, isLoading } = useChatCompletion();
  const { createOrUpdateChat } = useChats();
  const chatSettings = useChatSettings();

  const stop = useCallback(() => {
    if (abortController) {
      abortController.abort();
      setAbortController(null);
    }
  }, [abortController]);

  const append = useCallback(async (userMessage: Message) => {
    const newMessages = [...messages, userMessage];
    setMessages(newMessages);

    const controller = new AbortController();
    setAbortController(controller);

    try {
      let assistantMessage = '';

      await stream({
        messages: newMessages,
        ...chatSettings.getRequestSettings(),
        onMessage: (chunk) => {
          assistantMessage += chunk;
          setMessages([
            ...newMessages,
            { role: 'assistant', content: assistantMessage }
          ]);
        }
      });

      const finalMessages = [
        ...newMessages,
        { role: 'assistant', content: assistantMessage }
      ];

      await createOrUpdateChat({
        ...chat,
        messages: finalMessages,
        title: finalMessages[0].content.substring(0, 100),
        updatedAt: Date.now()
      });

    } catch (error) {
      console.error('Chat completion error:', error);
    } finally {
      setAbortController(null);
    }
  }, [messages, chatSettings, chat, stream, createOrUpdateChat]);

  const reload = useCallback(async () => {
    if (messages.length < 2) return;

    // Remove the last assistant message
    const lastUserMessageIndex = messages.map(m => m.role).lastIndexOf('user');
    if (lastUserMessageIndex === -1) return;

    const messagesToKeep = messages.slice(0, lastUserMessageIndex + 1);
    setMessages(messagesToKeep);

    // Re-send the last user message
    const lastUserMessage = messages[lastUserMessageIndex];
    await append(lastUserMessage);
  }, [messages, append]);

  return (
    <ChatContext.Provider value={{
      messages,
      input,
      setInput,
      isLoading,
      append,
      stop,
      reload
    }}>
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