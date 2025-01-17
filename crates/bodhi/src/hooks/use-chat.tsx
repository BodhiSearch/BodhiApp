'use client';

import { useChatCompletion } from '@/hooks/use-chat-completions';
import { useChatDB } from '@/hooks/use-chat-db';
import { useChatSettings } from '@/hooks/use-chat-settings';
import { useToast } from '@/hooks/use-toast';
import { Message } from '@/types/chat';
import { useCallback, useState } from 'react';

export function useChat() {
  const [input, setInput] = useState('');
  const [abortController, setAbortController] =
    useState<AbortController | null>(null);
  const { toast } = useToast();

  const { append, isLoading } = useChatCompletion();
  const { currentChat, createOrUpdateChat } = useChatDB();
  const chatSettings = useChatSettings();

  const stop = useCallback(() => {
    if (abortController) {
      abortController.abort();
      setAbortController(null);
    }
  }, [abortController]);

  const processCompletion = useCallback(
    async (userMessages: Message[]) => {
      if (!currentChat) return;

      let assistantMessage = '';

      try {
        let requestMessages = [...userMessages];
        if (chatSettings.systemPrompt_enabled && chatSettings.systemPrompt) {
          requestMessages = [
            { role: 'system', content: chatSettings.systemPrompt },
            ...requestMessages,
          ];
        }

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
            assistantMessage += chunk;
            createOrUpdateChat({
              ...currentChat,
              messages: [
                ...userMessages,
                { role: 'assistant' as const, content: assistantMessage },
              ],
              updatedAt: Date.now(),
            });
          },
          onMessage: (message) => {
            createOrUpdateChat({
              ...currentChat,
              messages: [
                ...userMessages,
                { role: 'assistant' as const, content: message.content },
              ],
              updatedAt: Date.now(),
            });
          },
          onFinish: () => {},
          onError: (error) => {
            let errorMessage = 'Error sending message to AI assistant.';

            if (typeof error === 'object' && error !== null) {
              if (
                'error' in error &&
                typeof error.error === 'object' &&
                error.error !== null
              ) {
                errorMessage =
                  (error.error as { message?: string }).message || errorMessage;
              }
            } else if (typeof error === 'string') {
              errorMessage = error;
            } else if (typeof error === 'object' && error !== null) {
              errorMessage =
                (error as { message?: string }).message || errorMessage;
            }

            toast({
              title: 'Error',
              description: errorMessage,
              variant: 'destructive',
              duration: 5000,
            });
          },
        });
      } catch (error) {
        // Handle any unexpected errors that weren't caught by onError
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
    [chatSettings, currentChat, append, createOrUpdateChat, toast]
  );

  const appendMessage = useCallback(
    async (userMessage: Message) => {
      if (!currentChat) return;

      const newMessages = [...currentChat.messages, userMessage];

      const title = newMessages[0].content.substring(0, 100);
      currentChat.title = title;
      // Update chat with user message immediately
      await createOrUpdateChat({
        ...currentChat,
        messages: newMessages,
        title,
        updatedAt: Date.now(),
      });

      const controller = new AbortController();
      setAbortController(controller);

      try {
        await processCompletion(newMessages);
      } finally {
        setAbortController(null);
      }
    },
    [currentChat, processCompletion, createOrUpdateChat]
  );

  const reload = useCallback(async () => {
    if (!currentChat || currentChat.messages.length < 2) return;

    // Remove the last assistant message
    const lastUserMessageIndex = currentChat.messages
      .map((m) => m.role)
      .lastIndexOf('user');
    if (lastUserMessageIndex === -1) return;

    const messagesToKeep = currentChat.messages.slice(
      0,
      lastUserMessageIndex + 1
    );

    // Update chat with removed assistant message
    await createOrUpdateChat({
      ...currentChat,
      messages: messagesToKeep,
      updatedAt: Date.now(),
    });

    await processCompletion(messagesToKeep);
  }, [currentChat, processCompletion, createOrUpdateChat]);

  return {
    input,
    setInput,
    isLoading,
    append: appendMessage,
    stop,
    reload,
  };
}
