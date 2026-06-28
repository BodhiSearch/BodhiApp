import { Trash2 } from 'lucide-react';

import { ScrollArea } from '@/components/ui/scroll-area';
import { useChatStore } from '@/stores/chatStore';
import { cn } from '@/lib/utils';
import { Chat } from '@/types/chat';

interface HistoryGroupProps {
  title: string;
  children: React.ReactNode;
}

const HistoryGroup = ({ title, children }: HistoryGroupProps) => (
  <div className="space-y-2">
    <h3 className="text-xs font-medium text-muted-foreground">{title}</h3>
    {children}
  </div>
);

export function ChatHistory() {
  const chats = useChatStore((s) => s.chats);
  const deleteChat = useChatStore((s) => s.deleteChat);
  const currentChatId = useChatStore((s) => s.currentChatId);
  const setCurrentChatId = useChatStore((s) => s.setCurrentChatId);

  const nonEmptyChats = chats.filter((chat) => chat.messageCount > 0);
  const todayChats = nonEmptyChats.filter((chat) => chat.createdAt > new Date().setDate(new Date().getDate() - 1));
  const yesterdayChats = nonEmptyChats.filter(
    (chat) =>
      chat.createdAt > new Date().setDate(new Date().getDate() - 2) &&
      chat.createdAt < new Date().setDate(new Date().getDate() - 1)
  );
  const previousChats = nonEmptyChats.filter((chat) => chat.createdAt < new Date().setDate(new Date().getDate() - 2));

  const renderChat = (chat: Chat, selected: boolean) => (
    <div
      key={chat.id}
      data-testid={`chat-history-item-${chat.id}`}
      className="group/menu-item relative flex items-center"
    >
      <button
        type="button"
        onClick={() => setCurrentChatId(chat.id)}
        title={chat.title || 'Untitled Chat'}
        data-testid={`chat-history-button-${chat.id}`}
        className={cn(
          'flex-1 truncate rounded-md px-2 py-1.5 text-left text-sm',
          'hover:bg-muted/50',
          selected && 'bg-muted'
        )}
      >
        {chat.title || 'Untitled Chat'}
      </button>
      <button
        type="button"
        data-testid={`delete-chat-${chat.id}`}
        onClick={(e) => {
          e.stopPropagation();
          deleteChat(chat.id);
        }}
        className="ml-1 flex h-7 w-7 items-center justify-center rounded-md text-muted-foreground opacity-0 transition-opacity hover:bg-muted group-hover/menu-item:opacity-100"
        aria-label="Delete chat"
      >
        <Trash2 className="h-4 w-4" />
      </button>
    </div>
  );

  return (
    <div className="flex h-full flex-col" data-testid="chat-history-container">
      <ScrollArea className="flex-1">
        <div className="space-y-4 p-2">
          {todayChats.length > 0 && (
            <HistoryGroup title="TODAY">
              {todayChats.map((chat) => renderChat(chat, chat.id === currentChatId))}
            </HistoryGroup>
          )}
          {yesterdayChats.length > 0 && (
            <HistoryGroup title="YESTERDAY">
              {yesterdayChats.map((chat) => renderChat(chat, chat.id === currentChatId))}
            </HistoryGroup>
          )}
          {previousChats.length > 0 && (
            <HistoryGroup title="PREVIOUS 7 DAYS">
              {previousChats.map((chat) => renderChat(chat, chat.id === currentChatId))}
            </HistoryGroup>
          )}
        </div>
      </ScrollArea>

      <div className="border-t p-4">
        <p className="text-xs text-muted-foreground text-center">
          Chat history is stored in your browser and may be lost if you clear your data.
        </p>
      </div>
    </div>
  );
}
