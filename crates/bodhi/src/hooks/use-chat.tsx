import { useChatCompletion } from '@/hooks/use-chat-completions';
import { useChatDB } from '@/hooks/use-chat-db';
import { useChatSettings } from '@/hooks/use-chat-settings';
import { useToastMessages } from '@/hooks/use-toast-messages';
import { nanoid } from '@/lib/utils';
import { Message } from '@/types/chat';
import { useCallback, useState } from 'react';

export function useChat() {
  const [input, setInput] = useState('');
  const [abortController, setAbortController] = useState<AbortController | null>(null);
  const [userMessage, setUserMessage] = useState<Message>({
    role: 'user',
    content: '',
  });
  const [assistantMessage, setAssistantMessage] = useState<Message>({
    role: 'assistant',
    content: '',
  });
  const { showError } = useToastMessages();
  const { append, isLoading } = useChatCompletion();
  const { currentChat, createOrUpdateChat, setCurrentChatId } = useChatDB();
  const chatSettings = useChatSettings();

  // Reset state to before user submission
  const resetToPreSubmissionState = useCallback((userContent: string) => {
    setInput(userContent); // Restore input
    setUserMessage({ role: 'user', content: '' }); // Clear user message
    setAssistantMessage({ role: 'assistant', content: '' }); // Clear assistant message
  }, []);

  const processCompletion = useCallback(
    async (userMessages: Message[]) => {
      let currentAssistantMessage = '';
      const userContent = userMessages[userMessages.length - 1].content;

      try {
        const requestMessages =
          chatSettings.systemPrompt_enabled && chatSettings.systemPrompt
            ? [{ role: 'system' as const, content: chatSettings.systemPrompt }, ...userMessages]
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
          onFinish: (message) => {
            const id = currentChat?.id || nanoid();
            const createdAt = currentChat?.createdAt || Date.now();
            const messages = [
              ...userMessages,
              {
                role: 'assistant' as const,
                content: currentAssistantMessage,
                metadata: message.metadata,
              },
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
            setInput(''); // Clear input after successful completion
            if (!currentChat) {
              setCurrentChatId(id);
            }
          },
          onError: (error) => {
            const errorMessage = extractErrorMessage(error);
            showError('Error', errorMessage);

            // Only reset if we haven't started receiving assistant's response
            if (!currentAssistantMessage) {
              resetToPreSubmissionState(userContent);
            }
            // If we have partial response, we might handle it differently in the future
          },
        });
      } catch (error) {
        const errorMessage = error instanceof Error ? error.message : 'An unexpected error occurred';

        showError('Error', errorMessage);
        // Reset state since this is a complete failure (couldn't even make the request)
        resetToPreSubmissionState(userContent);
      }
    },
    [chatSettings, currentChat, append, createOrUpdateChat, showError, setCurrentChatId, resetToPreSubmissionState]
  );

  // Helper function to extract error message
  const extractErrorMessage = (error: unknown): string => {
    if (typeof error === 'string') return error;

    if (error && typeof error === 'object') {
      if ('error' in error && error.error && typeof error.error === 'object') {
        return (error.error as { message?: string }).message || 'Error sending message to AI assistant.';
      }
      return (error as { message?: string }).message || 'Error sending message to AI assistant.';
    }

    return 'Error sending message to AI assistant.';
  };

  const appendMessage = useCallback(
    async (content: string) => {
      setAssistantMessage({ role: 'assistant' as const, content: '' });
      setUserMessage({ role: 'user' as const, content });

      const existingMessages = currentChat?.messages || [];
      const newMessages = [...existingMessages, { role: 'user' as const, content }];

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
    stop: () => {
      if (abortController) {
        abortController.abort();
        setAbortController(null);
      }
    },
    userMessage,
    assistantMessage,
  };
}
