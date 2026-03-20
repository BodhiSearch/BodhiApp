'use client';

import { Trash2 } from 'lucide-react';

import { ScrollArea } from '@/components/ui/scroll-area';
import { SidebarMenu, SidebarMenuItem, SidebarMenuButton, SidebarMenuAction } from '@/components/ui/sidebar';
import { useChatDB } from '@/hooks/use-chat-db';
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
  const { chats, deleteChat, currentChatId, setCurrentChatId } = useChatDB();

  // Filter out empty chats
  const nonEmptyChats = chats.filter((chat) => chat.messages.length > 0);
  const todayChats = nonEmptyChats.filter((chat) => chat.createdAt > new Date().setDate(new Date().getDate() - 1));
  const yesterdayChats = nonEmptyChats.filter(
    (chat) =>
      chat.createdAt > new Date().setDate(new Date().getDate() - 2) &&
      chat.createdAt < new Date().setDate(new Date().getDate() - 1)
  );
  const previousChats = nonEmptyChats.filter((chat) => chat.createdAt < new Date().setDate(new Date().getDate() - 2));

  const renderChat = (chat: Chat, selected: boolean) => (
    <SidebarMenuItem key={chat.id} data-testid={`chat-history-item-${chat.id}`}>
      <SidebarMenuButton
        onClick={() => setCurrentChatId(chat.id)}
        isActive={chat.id === currentChatId}
        className={cn('w-full justify-start truncate text-sm', 'hover:bg-muted/50', selected && 'bg-muted')}
        tooltip={chat.title || 'Untitled Chat'}
        data-testid={`chat-history-button-${chat.id}`}
      >
        {chat.title || 'Untitled Chat'}
      </SidebarMenuButton>
      <SidebarMenuAction
        data-testid={`delete-chat-${chat.id}`}
        onClick={(e) => {
          e.stopPropagation();
          deleteChat(chat.id);
        }}
        className="ml-2 opacity-0 group-hover/menu-item:opacity-100 transition-opacity"
      >
        <Trash2 className="h-4 w-4" />
      </SidebarMenuAction>
    </SidebarMenuItem>
  );

  return (
    <div className="flex h-full flex-col" data-testid="chat-history-container">
      <ScrollArea className="flex-1">
        <SidebarMenu>
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
        </SidebarMenu>
      </ScrollArea>

      <div className="border-t p-4">
        <p className="text-xs text-muted-foreground text-center">
          Chat history is stored in your browser and may be lost if you clear your data.
        </p>
      </div>
    </div>
  );
}
