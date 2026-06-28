import { MessageCircle, Trash2 } from 'lucide-react';

import { cn } from '@/lib/utils';
import { useChatStore } from '@/stores/chatStore';
import { Chat } from '@/types/chat';

interface ChatHistoryProps {
  /** Client-side filter over chat titles. */
  search?: string;
  /** Render the compact list used inside the collapsed-rail popover. */
  compact?: boolean;
  /** Fired after a chat is selected (so the popover can close). */
  onSelect?: () => void;
}

function groupChats(chats: Chat[]) {
  const nonEmpty = chats.filter((chat) => chat.messageCount > 0);
  const dayAgo = new Date().setDate(new Date().getDate() - 1);
  const twoDaysAgo = new Date().setDate(new Date().getDate() - 2);
  return {
    today: nonEmpty.filter((c) => c.createdAt > dayAgo),
    yesterday: nonEmpty.filter((c) => c.createdAt > twoDaysAgo && c.createdAt < dayAgo),
    previous: nonEmpty.filter((c) => c.createdAt < twoDaysAgo),
  };
}

export function ChatHistory({ search = '', compact = false, onSelect }: ChatHistoryProps) {
  const chats = useChatStore((s) => s.chats);
  const deleteChat = useChatStore((s) => s.deleteChat);
  const currentChatId = useChatStore((s) => s.currentChatId);
  const setCurrentChatId = useChatStore((s) => s.setCurrentChatId);

  const q = search.trim().toLowerCase();
  const matches = (c: Chat) => !q || (c.title || 'Untitled Chat').toLowerCase().includes(q);
  const visible = chats.filter(matches);
  const groups = groupChats(visible);

  const select = (id: string) => {
    setCurrentChatId(id);
    onSelect?.();
  };

  if (compact) {
    const order: Array<[string, Chat[]]> = [
      ['Today', groups.today],
      ['Yesterday', groups.yesterday],
      ['Previous 7 days', groups.previous],
    ];
    return (
      <div className="chat-hist-pop" data-testid="chat-history-container">
        {order.map(([label, items]) =>
          items.length === 0 ? null : (
            <div key={label}>
              <div className="chat-hist-pop-group">{label}</div>
              {items.map((chat) => (
                <button
                  key={chat.id}
                  data-testid={`chat-history-button-${chat.id}`}
                  className={cn('chat-hist-pop-item', chat.id === currentChatId && 'on')}
                  onClick={() => select(chat.id)}
                >
                  <MessageCircle className="h-3.5 w-3.5 shrink-0" />
                  <span className="chat-item-label">{chat.title || 'Untitled Chat'}</span>
                </button>
              ))}
            </div>
          )
        )}
      </div>
    );
  }

  const renderChat = (chat: Chat) => {
    const selected = chat.id === currentChatId;
    return (
      <div key={chat.id} data-testid={`chat-history-item-${chat.id}`} className={cn('chat-item', selected && 'on')}>
        <button
          type="button"
          className="chat-item-label"
          title={chat.title || 'Untitled Chat'}
          data-testid={`chat-history-button-${chat.id}`}
          onClick={() => select(chat.id)}
        >
          {chat.title || 'Untitled Chat'}
        </button>
        <button
          type="button"
          className="chat-item-more"
          aria-label="Delete chat"
          data-testid={`delete-chat-${chat.id}`}
          onClick={(e) => {
            e.stopPropagation();
            deleteChat(chat.id);
          }}
        >
          <Trash2 className="h-3.5 w-3.5" />
        </button>
      </div>
    );
  };

  return (
    <div className="chat-hist" data-testid="chat-history-container">
      <div className="chat-list">
        {groups.today.length > 0 && (
          <>
            <div className="chat-group">TODAY</div>
            {groups.today.map(renderChat)}
          </>
        )}
        {groups.yesterday.length > 0 && (
          <>
            <div className="chat-group">YESTERDAY</div>
            {groups.yesterday.map(renderChat)}
          </>
        )}
        {groups.previous.length > 0 && (
          <>
            <div className="chat-group">PREVIOUS 7 DAYS</div>
            {groups.previous.map(renderChat)}
          </>
        )}
      </div>

      <div className="chat-hist-foot">
        Chat history is stored in your browser and may be lost if you clear your data.
      </div>
    </div>
  );
}
