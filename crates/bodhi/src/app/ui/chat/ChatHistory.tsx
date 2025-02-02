'use client';

import { useChatDB } from '@/hooks/use-chat-db';
import {
  SidebarMenu,
  SidebarMenuItem,
  SidebarMenuButton,
  SidebarMenuAction,
} from '@/components/ui/sidebar';
import { ScrollArea } from '@/components/ui/scroll-area';
import { Trash2 } from 'lucide-react';
import { cn } from '@/lib/utils';
import { Chat } from '@/types/chat';

interface HistoryGroupProps {
  title: string;
  children: React.ReactNode;
}

const HistoryGroup = ({ title, children }: HistoryGroupProps) => (
  <div className="space-y-2">
    <h3 className="px-2 text-xs font-medium text-muted-foreground">{title}</h3>
    <div className="space-y-1">{children}</div>
  </div>
);

export function ChatHistory() {
  const { chats, deleteChat, currentChatId, setCurrentChatId } = useChatDB();

  // Filter out empty chats
  const nonEmptyChats = chats.filter((chat) => chat.messages.length > 0);
  const todayChats = nonEmptyChats.filter(
    (chat) => chat.createdAt > new Date().setDate(new Date().getDate() - 1)
  );
  const yesterdayChats = nonEmptyChats.filter(
    (chat) =>
      chat.createdAt > new Date().setDate(new Date().getDate() - 2) &&
      chat.createdAt < new Date().setDate(new Date().getDate() - 1)
  );
  const previousChats = nonEmptyChats.filter(
    (chat) => chat.createdAt < new Date().setDate(new Date().getDate() - 2)
  );

  const renderChat = (chat: Chat, selected: boolean) => (
    <SidebarMenuItem key={chat.id}>
      <SidebarMenuButton
        onClick={() => setCurrentChatId(chat.id)}
        isActive={chat.id === currentChatId}
        className={cn(
          'w-full justify-start px-2 text-sm font-normal truncate',
          'hover:bg-muted/50',
          selected && 'bg-muted'
        )}
        tooltip={chat.title || 'Untitled Chat'}
      >
        <span className="truncate">{chat.title || 'Untitled Chat'}</span>
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
    <div className="flex h-full flex-col">
      <div className="flex-1 overflow-y-auto">
        <SidebarMenu>
          <ScrollArea className="h-full">
            <div className="flex flex-col gap-4 p-2">
              {todayChats.length > 0 && (
                <div className="space-y-4">
                  <HistoryGroup title="TODAY">
                    {todayChats.map((chat) =>
                      renderChat(chat, chat.id === currentChatId)
                    )}
                  </HistoryGroup>
                </div>
              )}
              {yesterdayChats.length > 0 && (
                <div className="space-y-4">
                  <HistoryGroup title="YESTERDAY">
                    {yesterdayChats.map((chat) =>
                      renderChat(chat, chat.id === currentChatId)
                    )}
                  </HistoryGroup>
                </div>
              )}
              {previousChats.length > 0 && (
                <div className="space-y-4">
                  <HistoryGroup title="PREVIOUS 7 DAYS">
                    {previousChats.map((chat) =>
                      renderChat(chat, chat.id === currentChatId)
                    )}
                  </HistoryGroup>
                </div>
              )}
            </div>
          </ScrollArea>
        </SidebarMenu>
      </div>

      <div className="border-t p-4">
        <p className="text-xs text-muted-foreground text-center">
          Chat history is stored in your browser and may be lost if you clear
          your data.
        </p>
      </div>
    </div>
  );
}
