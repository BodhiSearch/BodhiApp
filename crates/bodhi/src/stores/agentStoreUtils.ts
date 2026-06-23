import type { AssistantMessage } from '@mariozechner/pi-ai';

import { Chat } from '@/types/chat';

interface CurrentChatSource {
  currentChatId: string | null;
  chats: Chat[];
}

// Resolves the current chat from a chatStore snapshot, or null when none is selected/found.
export function getCurrentChat(chatStore: CurrentChatSource): Chat | null {
  return chatStore.currentChatId ? (chatStore.chats.find((c) => c.id === chatStore.currentChatId) ?? null) : null;
}

// Zero-cost usage fallback for assistant messages restored from plain-text (non-JSON) persistence.
export function getDefaultAgentMessageUsage(): AssistantMessage['usage'] {
  return {
    input: 0,
    output: 0,
    cacheRead: 0,
    cacheWrite: 0,
    totalTokens: 0,
    cost: { input: 0, output: 0, cacheRead: 0, cacheWrite: 0, total: 0 },
  };
}
