'use client';

import { MessageSquare } from 'lucide-react';

import { Button } from '@/components/ui/button';
import { SidebarMenu, SidebarMenuItem } from '@/components/ui/sidebar';
import { useChatDB } from '@/hooks/use-chat-db';
import { cn } from '@/lib/utils';

export const NewChatButton = () => {
  const { createNewChat } = useChatDB();

  return (
    <SidebarMenu>
      <SidebarMenuItem>
        <div className="flex flex-col gap-4 p-2 space-y-1 pb-2">
          <Button
            variant="ghost"
            className={cn('w-full justify-start gap-2 px-2 font-normal hover:bg-muted/50')}
            onClick={createNewChat}
            data-testid="new-chat-button"
          >
            <MessageSquare className="h-4 w-4" />
            New Chat
          </Button>
        </div>
      </SidebarMenuItem>
    </SidebarMenu>
  );
};
