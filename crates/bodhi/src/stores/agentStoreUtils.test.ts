import { describe, expect, it } from 'vitest';

import { Chat } from '@/types/chat';

import { getCurrentChat, getDefaultAgentMessageUsage } from './agentStoreUtils';

function makeChat(id: string): Chat {
  return {
    id,
    title: `Chat ${id}`,
    messages: [],
    messageCount: 0,
    createdAt: 0,
  };
}

describe('getCurrentChat', () => {
  it('returns null when currentChatId is null', () => {
    expect(getCurrentChat({ currentChatId: null, chats: [makeChat('a')] })).toBeNull();
  });

  it('returns null when currentChatId has no matching chat', () => {
    expect(getCurrentChat({ currentChatId: 'missing', chats: [makeChat('a')] })).toBeNull();
  });

  it('returns the matching chat when found', () => {
    const target = makeChat('b');
    expect(getCurrentChat({ currentChatId: 'b', chats: [makeChat('a'), target] })).toBe(target);
  });
});

describe('getDefaultAgentMessageUsage', () => {
  it('returns a zeroed usage object', () => {
    expect(getDefaultAgentMessageUsage()).toEqual({
      input: 0,
      output: 0,
      cacheRead: 0,
      cacheWrite: 0,
      totalTokens: 0,
      cost: { input: 0, output: 0, cacheRead: 0, cacheWrite: 0, total: 0 },
    });
  });

  it('returns a fresh object each call (no shared mutable reference)', () => {
    expect(getDefaultAgentMessageUsage()).not.toBe(getDefaultAgentMessageUsage());
  });
});
