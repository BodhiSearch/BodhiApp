'use client';

import { Plus } from 'lucide-react';
import { useRouter, useSearchParams } from 'next/navigation';
import { useLocalStorage } from '@/hooks/useLocalStorage';
import { CURRENT_CHAT_KEY } from '@/lib/constants';
import { Chat } from '@/types/chat';
import {
  SidebarMenu,
  SidebarMenuItem,
  SidebarMenuButton,
} from '@/components/ui/sidebar';

export const NewChatButton = () => {
  const router = useRouter();
  const searchParams = useSearchParams();
  const [currentChat, setCurrentChat] = useLocalStorage<Chat | null>(CURRENT_CHAT_KEY, null);

  const handleNewChat = () => {
    if (!currentChat || currentChat.messages.length === 0) {
      return;
    }
    setCurrentChat(null);
    router.replace('/ui/chat');
  };

  return (
    <SidebarMenu>
      <SidebarMenuItem>
        <SidebarMenuButton
          onClick={handleNewChat}
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