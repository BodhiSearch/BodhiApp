'use client';

import { useChatDB } from '@/hooks/use-chat-db';
import {
  SidebarMenu,
  SidebarMenuItem,
  SidebarMenuButton,
  SidebarMenuAction,
} from '@/components/ui/sidebar';
import { Trash2 } from 'lucide-react';
import { useRouter, useSearchParams } from 'next/navigation';

export function ChatHistory() {
  const { chats, deleteChat } = useChatDB();
  const router = useRouter();
  const searchParams = useSearchParams();
  const currentChatId = searchParams.get('id');

  return (
    <SidebarMenu>
      {chats.map((chat) => (
        <SidebarMenuItem key={chat.id}>
          <SidebarMenuButton
            onClick={() => router.push(`/ui/chat/?id=${chat.id}`)}
            isActive={chat.id === currentChatId}
          >
            {chat.title || 'Untitled Chat'}
          </SidebarMenuButton>
          <SidebarMenuAction
            data-testid={`delete-chat-${chat.id}`}
            onClick={(e) => {
              e.stopPropagation();
              deleteChat(chat.id);
            }}
            showOnHover
          >
            <Trash2 className="h-4 w-4" />
          </SidebarMenuAction>
        </SidebarMenuItem>
      ))}
    </SidebarMenu>
  );
}
