import { useState } from 'react';

import { ChevronRight, Edit3, MessageCircle } from 'lucide-react';

import { useChatStore } from '@/stores/chatStore';

/**
 * The chat breadcrumb: a "Chat ›" crumb plus the current conversation's title, editable inline.
 * Published into the shell breadcrumb slot (which renders a custom node as-is). Rename saves through
 * the store (getChat → createOrUpdateChat with the new title).
 */
export function ChatTitle() {
  const currentChatId = useChatStore((s) => s.currentChatId);
  const chats = useChatStore((s) => s.chats);
  const getChat = useChatStore((s) => s.getChat);
  const createOrUpdateChat = useChatStore((s) => s.createOrUpdateChat);

  const current = chats.find((c) => c.id === currentChatId);
  const title = current?.title || 'New Chat';

  const [editing, setEditing] = useState(false);
  const [draft, setDraft] = useState(title);

  const startEditing = () => {
    setDraft(title);
    setEditing(true);
  };

  const commit = async () => {
    setEditing(false);
    const next = draft.trim();
    if (!currentChatId || !next || next === title) return;
    const { data, status } = await getChat(currentChatId);
    if (status !== 200) return;
    await createOrUpdateChat({ ...data, title: next });
  };

  return (
    <div className="chat-title" data-testid="chat-title">
      <div className="chat-crumb">
        <MessageCircle className="h-2.5 w-2.5" />
        Chat
        <ChevronRight className="h-2.5 w-2.5" />
      </div>
      {editing ? (
        <input
          autoFocus
          className="chat-title-input"
          data-testid="chat-title-input"
          value={draft}
          onFocus={(e) => e.currentTarget.select()}
          onChange={(e) => setDraft(e.target.value)}
          onBlur={commit}
          onKeyDown={(e) => {
            if (e.key === 'Enter') commit();
            if (e.key === 'Escape') setEditing(false);
          }}
        />
      ) : (
        <button
          type="button"
          className="chat-title-name"
          data-testid="chat-title-edit"
          onClick={() => current && startEditing()}
          disabled={!current}
          title={current ? 'Rename chat' : undefined}
        >
          <span className="name">{title}</span>
          {current && <Edit3 className="chat-title-edit-icon h-3 w-3" />}
        </button>
      )}
    </div>
  );
}
