import { beforeEach, describe, expect, it, vi } from 'vitest';

import { chatDb } from '@/lib/chatDb';
import { Chat } from '@/types/chat';
import { useChatStore } from './chatStore';

vi.mock('@/lib/utils', () => ({
  nanoid: () => 'new-id',
}));

// ── Helpers ───────────────────────────────────────────────────────────────────

function makeChat(overrides: Partial<Chat> = {}): Chat {
  return {
    id: 'chat-1',
    title: 'Test Chat',
    messages: [],
    messageCount: 0,
    createdAt: 1000,
    updatedAt: 2000,
    ...overrides,
  };
}

// ── Tests ─────────────────────────────────────────────────────────────────────

describe('chatStore', () => {
  beforeEach(async () => {
    localStorage.clear();
    vi.clearAllMocks();
    await chatDb.chats.clear();
    await chatDb.messages.clear();
    // Reset store to clean initial state
    useChatStore.setState({
      chats: [],
      currentChatId: null,
      isLoaded: false,
      userId: 'user-1',
    });
  });

  describe('loadChats', () => {
    it('sets isLoaded=true and loads chats for userId', async () => {
      await chatDb.chats.put({ id: 'c1', userId: 'user-1', title: 'Chat 1', createdAt: 1000, messageCount: 0 });
      await chatDb.chats.put({ id: 'c2', userId: 'other-user', title: 'Other', createdAt: 2000, messageCount: 0 });

      await useChatStore.getState().loadChats('user-1');

      const state = useChatStore.getState();
      expect(state.isLoaded).toBe(true);
      expect(state.chats).toHaveLength(1);
      expect(state.chats[0].id).toBe('c1');
    });

    it('switches userId and reloads when called with new userId', async () => {
      await chatDb.chats.put({ id: 'c1', userId: 'user-2', title: 'User2 Chat', createdAt: 1000, messageCount: 0 });

      await useChatStore.getState().loadChats('user-2');

      const state = useChatStore.getState();
      expect(state.userId).toBe('user-2');
      expect(state.chats).toHaveLength(1);
      expect(state.chats[0].id).toBe('c1');
    });

    it('restores currentChatId from localStorage for new userId', async () => {
      localStorage.setItem('current-chat-id:user-3', JSON.stringify('stored-id'));
      await useChatStore.getState().loadChats('user-3');
      expect(useChatStore.getState().currentChatId).toBe('stored-id');
    });
  });

  describe('createOrUpdateChat', () => {
    it('persists chat and messages to Dexie', async () => {
      const chat = makeChat({
        id: 'c1',
        messages: [{ role: 'user', content: 'hello' }],
        messageCount: 1,
      });

      await useChatStore.getState().loadChats('user-1');
      await useChatStore.getState().createOrUpdateChat(chat);

      const record = await chatDb.chats.get('c1');
      expect(record).toBeDefined();
      expect(record?.userId).toBe('user-1');

      const msgs = await chatDb.messages.where('chatId').equals('c1').toArray();
      expect(msgs).toHaveLength(1);
      expect(msgs[0].message.content).toBe('hello');
    });

    it('replaces messages on update', async () => {
      await useChatStore.getState().loadChats('user-1');

      const chat = makeChat({ id: 'c1', messages: [{ role: 'user', content: 'v1' }], messageCount: 1 });
      await useChatStore.getState().createOrUpdateChat(chat);

      const updated = { ...chat, messages: [{ role: 'user', content: 'v2' }] as Chat['messages'] };
      await useChatStore.getState().createOrUpdateChat(updated);

      const msgs = await chatDb.messages.where('chatId').equals('c1').toArray();
      expect(msgs).toHaveLength(1);
      expect(msgs[0].message.content).toBe('v2');
    });

    it('reloads chat list after update', async () => {
      await useChatStore.getState().loadChats('user-1');
      await useChatStore.getState().createOrUpdateChat(makeChat({ id: 'c1', title: 'My Chat' }));

      expect(useChatStore.getState().chats).toHaveLength(1);
      expect(useChatStore.getState().chats[0].title).toBe('My Chat');
    });
  });

  describe('deleteChat', () => {
    it('deletes a non-current chat', async () => {
      await useChatStore.getState().loadChats('user-1');
      await useChatStore.getState().createOrUpdateChat(makeChat({ id: 'c1' }));
      await useChatStore.getState().createOrUpdateChat(makeChat({ id: 'c2' }));
      useChatStore.setState({ currentChatId: 'c2' });

      await useChatStore.getState().deleteChat('c1');

      expect(useChatStore.getState().chats).toHaveLength(1);
      expect(useChatStore.getState().chats[0].id).toBe('c2');
      expect(await chatDb.chats.get('c1')).toBeUndefined();
    });

    it('deletes current chat and resets currentChatId to null', async () => {
      await useChatStore.getState().loadChats('user-1');
      await useChatStore.getState().createOrUpdateChat(
        makeChat({
          id: 'c1',
          messageCount: 2,
          messages: [
            { role: 'user', content: 'hi' },
            { role: 'assistant', content: 'hello' },
          ],
        })
      );
      await useChatStore.getState().createOrUpdateChat(makeChat({ id: 'c2', messageCount: 0, messages: [] }));
      useChatStore.setState({ currentChatId: 'c1' });

      await useChatStore.getState().deleteChat('c1');

      expect(useChatStore.getState().currentChatId).toBeNull();
      expect(await chatDb.chats.get('c1')).toBeUndefined();
    });

    it('deletes current chat when no empty sibling exists', async () => {
      await useChatStore.getState().loadChats('user-1');
      await useChatStore.getState().createOrUpdateChat(
        makeChat({
          id: 'c1',
          title: 'Old Title',
          messages: [{ role: 'user', content: 'hi' }],
          messageCount: 1,
        })
      );
      useChatStore.setState({ currentChatId: 'c1' });

      await useChatStore.getState().deleteChat('c1');

      expect(await chatDb.chats.get('c1')).toBeUndefined();
      expect(useChatStore.getState().currentChatId).toBeNull();
    });
  });

  describe('clearChats', () => {
    it('removes all user chats and resets currentChatId', async () => {
      await useChatStore.getState().loadChats('user-1');
      await useChatStore.getState().createOrUpdateChat(makeChat({ id: 'c1' }));
      await useChatStore.getState().createOrUpdateChat(makeChat({ id: 'c2' }));
      useChatStore.setState({ currentChatId: 'c1' });

      await useChatStore.getState().clearChats();

      expect(useChatStore.getState().chats).toHaveLength(0);
      expect(useChatStore.getState().currentChatId).toBeNull();
      expect(await chatDb.chats.count()).toBe(0);
    });

    it('does not delete chats belonging to other users', async () => {
      await useChatStore.getState().loadChats('user-1');
      await chatDb.chats.put({ id: 'other-c1', userId: 'other-user', title: 'Other', createdAt: 1000 });
      await useChatStore.getState().createOrUpdateChat(makeChat({ id: 'c1' }));

      await useChatStore.getState().clearChats();

      expect(await chatDb.chats.get('other-c1')).toBeDefined();
      expect(await chatDb.chats.get('c1')).toBeUndefined();
    });
  });

  describe('createNewChat', () => {
    it('no-ops when current chat has no messages', async () => {
      await useChatStore.getState().loadChats('user-1');
      await useChatStore.getState().createOrUpdateChat(makeChat({ id: 'c1', messageCount: 0 }));
      useChatStore.setState({ currentChatId: 'c1' });

      await useChatStore.getState().createNewChat();

      expect(useChatStore.getState().chats).toHaveLength(1);
    });

    it('reuses an existing empty chat', async () => {
      await useChatStore.getState().loadChats('user-1');
      await useChatStore
        .getState()
        .createOrUpdateChat(makeChat({ id: 'c1', messages: [{ role: 'user', content: 'hi' }], messageCount: 1 }));
      await useChatStore.getState().createOrUpdateChat(makeChat({ id: 'c2', messages: [], messageCount: 0 }));
      useChatStore.setState({ currentChatId: 'c1' });

      await useChatStore.getState().createNewChat();

      expect(useChatStore.getState().currentChatId).toBe('c2');
      expect(useChatStore.getState().chats).toHaveLength(2);
    });

    it('creates a new chat when all existing chats have messages', async () => {
      await useChatStore.getState().loadChats('user-1');
      await useChatStore
        .getState()
        .createOrUpdateChat(makeChat({ id: 'c1', messages: [{ role: 'user', content: 'hi' }], messageCount: 1 }));
      useChatStore.setState({ currentChatId: 'c1' });

      await useChatStore.getState().createNewChat();

      expect(useChatStore.getState().chats).toHaveLength(2);
      expect(useChatStore.getState().currentChatId).toBe('new-id');
    });
  });

  describe('getChat', () => {
    it('returns chat with messages for correct userId', async () => {
      await useChatStore.getState().loadChats('user-1');
      await useChatStore
        .getState()
        .createOrUpdateChat(makeChat({ id: 'c1', messages: [{ role: 'user', content: 'hello' }], messageCount: 1 }));

      const result = await useChatStore.getState().getChat('c1');

      expect(result.status).toBe(200);
      expect(result.data.id).toBe('c1');
      expect(result.data.messages).toHaveLength(1);
    });

    it('returns 404 for chat belonging to different userId', async () => {
      await chatDb.chats.put({ id: 'other-c1', userId: 'other-user', title: 'Other', createdAt: 1000 });
      await useChatStore.getState().loadChats('user-1');

      const result = await useChatStore.getState().getChat('other-c1');

      expect(result.status).toBe(404);
    });

    it('returns 404 for non-existent chat', async () => {
      await useChatStore.getState().loadChats('user-1');
      const result = await useChatStore.getState().getChat('does-not-exist');
      expect(result.status).toBe(404);
    });
  });

  describe('saveChatSettings / getChatSettings', () => {
    it('round-trips chat settings', async () => {
      await useChatStore.getState().loadChats('user-1');
      await useChatStore.getState().createOrUpdateChat(makeChat({ id: 'c1' }));

      const settings = {
        model: 'gpt-4',
        apiFormat: 'openai' as const,
        stream: true,
        stream_enabled: true,
        seed_enabled: false,
        systemPrompt_enabled: false,
        stop_enabled: false,
        max_tokens_enabled: false,
        n_enabled: false,
        temperature_enabled: false,
        top_p_enabled: false,
        presence_penalty_enabled: false,
        frequency_penalty_enabled: false,
        logit_bias_enabled: false,
        response_format_enabled: false,
        maxToolIterations: 5,
        maxToolIterations_enabled: true,
      };

      await useChatStore.getState().saveChatSettings('c1', settings);
      const loaded = await useChatStore.getState().getChatSettings('c1');

      expect(loaded?.model).toBe('gpt-4');
    });
  });

  describe('loadMessagesForChat', () => {
    it('loads messages from Dexie and caches in store', async () => {
      await useChatStore.getState().loadChats('user-1');
      await useChatStore.getState().createOrUpdateChat(
        makeChat({
          id: 'c1',
          messages: [{ role: 'user', content: 'cached message' }],
          messageCount: 1,
        })
      );

      // Clear messages from in-memory state to force DB load
      useChatStore.setState({
        chats: useChatStore.getState().chats.map((c) => (c.id === 'c1' ? { ...c, messages: [] } : c)),
      });

      const messages = await useChatStore.getState().loadMessagesForChat('c1');

      expect(messages).toHaveLength(1);
      expect(messages[0].content).toBe('cached message');
    });

    it('returns cached messages if already loaded', async () => {
      await useChatStore.getState().loadChats('user-1');
      const chat = makeChat({ id: 'c1', messages: [{ role: 'user', content: 'hi' }], messageCount: 1 });
      await useChatStore.getState().createOrUpdateChat(chat);
      await useChatStore.getState().loadMessagesForChat('c1'); // prime cache

      const messages = await useChatStore.getState().loadMessagesForChat('c1');
      expect(messages).toHaveLength(1);
    });
  });
});
