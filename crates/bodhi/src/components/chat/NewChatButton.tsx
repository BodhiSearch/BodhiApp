'use client';

import { Plus } from 'lucide-react';
import { useChatDB } from '@/hooks/use-chat-db';
import {
  SidebarMenu,
  SidebarMenuItem,
  SidebarMenuButton,
} from '@/components/ui/sidebar';

export const NewChatButton = () => {
  const { createNewChat } = useChatDB();

  return (
    <SidebarMenu>
      <SidebarMenuItem>
        <SidebarMenuButton
          onClick={createNewChat}
          className="bg-primary hover:bg-primary/90 text-primary-foreground"
          data-testid="new-chat-button"
        >
          <Plus className="h-4 w-4 shrink-0" />
          <span className="sidebar-expanded:inline hidden ml-2">New Chat</span>
        </SidebarMenuButton>
      </SidebarMenuItem>
    </SidebarMenu>
  );
};
