import { MessageSquare } from 'lucide-react';

import { Button } from '@/components/ui/button';
import { cn } from '@/lib/utils';
import { useChatStore } from '@/stores/chatStore';

export const NewChatButton = () => {
  const createNewChat = useChatStore((s) => s.createNewChat);

  return (
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
  );
};
