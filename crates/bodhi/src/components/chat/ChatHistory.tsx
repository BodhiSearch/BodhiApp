'use client';

import { useChatDB } from '@/hooks/use-chat-db';
import {
  SidebarMenu,
  SidebarMenuItem,
  SidebarMenuButton,
  SidebarMenuAction,
} from '@/components/ui/sidebar';
import { Trash2 } from 'lucide-react';

export function ChatHistory() {
  const { chats, deleteChat, currentChatId, setCurrentChatId } = useChatDB();

  // Filter out empty chats
  const nonEmptyChats = chats.filter((chat) => chat.messages.length > 0);

  return (
    <SidebarMenu>
      {nonEmptyChats.map((chat) => (
        <SidebarMenuItem key={chat.id}>
          <SidebarMenuButton
            onClick={() => setCurrentChatId(chat.id)}
            isActive={chat.id === currentChatId}
            className="w-full justify-between"
            tooltip={chat.title || 'Untitled Chat'}
          >
            <span className="truncate">{chat.title || 'Untitled Chat'}</span>
            <SidebarMenuAction
              data-testid={`delete-chat-${chat.id}`}
              onClick={(e) => {
                e.stopPropagation();
                deleteChat(chat.id);
              }}
              className="opacity-0 group-hover:opacity-100 transition-opacity"
            >
              <Trash2 className="h-4 w-4" />
            </SidebarMenuAction>
          </SidebarMenuButton>
        </SidebarMenuItem>
      ))}
    </SidebarMenu>
  );
}
