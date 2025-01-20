'use client';

import { useChatCompletion } from '@/hooks/use-chat-completions';
import { useChatDB } from '@/hooks/use-chat-db';
import { useChatSettings } from '@/hooks/use-chat-settings';
import { useToast } from '@/hooks/use-toast';
import { nanoid } from '@/lib/utils';
import { Message } from '@/types/chat';
import { useCallback, useState } from 'react';

export function useChat() {
  const [input, setInput] = useState('');
  const [abortController, setAbortController] =
    useState<AbortController | null>(null);
  const [userMessage, setUserMessage] = useState<Message>({
    role: 'user',
    content: '',
  });
  const [assistantMessage, setAssistantMessage] = useState<Message>({
    role: 'assistant',
    content: '',
  });

  const { toast } = useToast();
  const { append, isLoading } = useChatCompletion();
  const { currentChat, createOrUpdateChat, setCurrentChatId } = useChatDB();
  const chatSettings = useChatSettings();

  const stop = useCallback(() => {
    if (abortController) {
      abortController.abort();
      setAbortController(null);
    }
  }, [abortController]);

  const processCompletion = useCallback(
    async (userMessages: Message[]) => {
      let currentAssistantMessage = '';

      try {
        const requestMessages =
          chatSettings.systemPrompt_enabled && chatSettings.systemPrompt
            ? [
                { role: 'system' as const, content: chatSettings.systemPrompt },
                ...userMessages,
              ]
            : userMessages;

        const headers: Record<string, string> = {};
        if (chatSettings.api_token_enabled && chatSettings.api_token) {
          headers.Authorization = `Bearer ${chatSettings.api_token}`;
        }

        await append({
          request: {
            ...chatSettings.getRequestSettings(),
            messages: requestMessages,
          },
          headers,
          onDelta: (chunk) => {
            currentAssistantMessage += chunk;
            setAssistantMessage((prevMessage) => ({
              role: 'assistant' as const,
              content: prevMessage.content + chunk,
            }));
          },
          onMessage: (message) => {
            setAssistantMessage({
              role: 'assistant' as const,
              content: message.content,
            });
          },
          onFinish: () => {
            const id = currentChat?.id || nanoid();
            const createdAt = currentChat?.createdAt || Date.now();
            const messages = [
              ...userMessages,
              { role: 'assistant' as const, content: currentAssistantMessage },
            ];

            createOrUpdateChat({
              id,
              title: messages[0]?.content.slice(0, 20) || 'New Chat',
              messages,
              createdAt,
              updatedAt: Date.now(),
            });
            setAssistantMessage({ role: 'assistant' as const, content: '' });
            setUserMessage({ role: 'user' as const, content: '' });
            if (!currentChat) {
              setCurrentChatId(id);
            }
          },
          onError: (error) => {
            const errorMessage = extractErrorMessage(error);
            toast({
              title: 'Error',
              description: errorMessage,
              variant: 'destructive',
              duration: 5000,
            });
          },
        });
      } catch (error) {
        const errorMessage =
          error instanceof Error
            ? error.message
            : 'An unexpected error occurred';

        toast({
          title: 'Error',
          description: errorMessage,
          variant: 'destructive',
          duration: 5000,
        });
      }
    },
    [
      chatSettings,
      currentChat,
      append,
      createOrUpdateChat,
      toast,
      setCurrentChatId,
    ]
  );

  // Helper function to extract error message
  const extractErrorMessage = (error: unknown): string => {
    if (typeof error === 'string') return error;

    if (error && typeof error === 'object') {
      if ('error' in error && error.error && typeof error.error === 'object') {
        return (
          (error.error as { message?: string }).message ||
          'Error sending message to AI assistant.'
        );
      }
      return (
        (error as { message?: string }).message ||
        'Error sending message to AI assistant.'
      );
    }

    return 'Error sending message to AI assistant.';
  };

  const appendMessage = useCallback(
    async (content: string) => {
      setAssistantMessage({ role: 'assistant' as const, content: '' });
      setUserMessage({ role: 'user' as const, content });

      const existingMessages = currentChat?.messages || [];
      const newMessages = [
        ...existingMessages,
        { role: 'user' as const, content },
      ];

      const controller = new AbortController();
      setAbortController(controller);

      try {
        await processCompletion(newMessages);
      } finally {
        setAbortController(null);
      }
    },
    [currentChat, processCompletion]
  );

  return {
    input,
    setInput,
    isLoading,
    append: appendMessage,
    stop,
    userMessage,
    assistantMessage,
  };
}
