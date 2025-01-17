'use client';

import { useChatCompletion } from '@/hooks/use-chat-completions';
import { useChatDB } from '@/hooks/use-chat-db';
import { useChatSettings } from '@/hooks/use-chat-settings';
import { Message } from '@/types/chat';
import { useCallback, useState } from 'react';

export function useChat() {
  const [input, setInput] = useState('');
  const [abortController, setAbortController] =
    useState<AbortController | null>(null);

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

        await append({
          request: {
            ...chatSettings.getRequestSettings(),
            messages: requestMessages,
          },
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
              messages: [...userMessages, { role: 'assistant' as const, content: message.content }],
              updatedAt: Date.now(),
            });
          },
          onFinish: () => { },
        });
      } catch (error) {
        console.error('Chat completion error:', error);
        throw error;
      }
    },
    [chatSettings, currentChat, append, createOrUpdateChat]
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
