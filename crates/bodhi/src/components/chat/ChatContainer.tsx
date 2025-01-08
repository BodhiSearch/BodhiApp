'use client';

import { SettingsSidebar } from '@/components/settings/SettingsSidebar';
import { SidebarProvider } from '@/components/ui/sidebar';
import { useLocalStorage } from '@/hooks/useLocalStorage';
import { useChatDB } from '@/hooks/use-chat-db';
import { nanoid } from '@/lib/utils';
import { Chat } from '@/types/chat';
import { Settings2 } from 'lucide-react';
import { useRouter, useSearchParams } from 'next/navigation';
import { useEffect, useState } from 'react';
import { ChatProvider } from '@/hooks/use-chat';
import { ChatUI } from '@/components/chat/ChatUI';
import { cn } from '@/lib/utils';
import { Button } from '../ui/button';
import { useToast } from '@/hooks/use-toast';
import { MainLayout } from '@/components/layout/MainLayout';
import { ChatHistory } from './ChatHistory';
import { ChatSettingsProvider } from '@/hooks/use-chat-settings';

const SETTINGS_SIDEBAR_KEY = 'settings-sidebar-state';
const CURRENT_CHAT_KEY = 'current-chat';

export function ChatContainer() {
  const [settingsOpen, setSettingsOpen] = useLocalStorage(
    SETTINGS_SIDEBAR_KEY,
    true
  );
  const [currentChat, setCurrentChat] = useLocalStorage<Chat | null>(
    CURRENT_CHAT_KEY,
    null
  );
  const searchParams = useSearchParams();
  const router = useRouter();
  const { getChat } = useChatDB();
  const [isLoading, setIsLoading] = useState(true);
  const { toast } = useToast();

  useEffect(() => {
    const initializeChat = async () => {
      const id = searchParams.get('id');

      // Case 1: URL has an ID
      if (id) {
        try {
          const { data, status } = await getChat(id);

          if (status === 200) {
            setCurrentChat(data);
            setIsLoading(false);
            return;
          }

          // Show error toast and redirect
          toast({
            variant: 'destructive',
            title: 'Chat not found',
            description: 'The requested chat could not be found.',
          });
          router.push('/ui/chat');
          return;
        } catch (err) {
          // Show error toast and redirect
          toast({
            variant: 'destructive',
            title: 'Error loading chat',
            description: 'Failed to load the requested chat. Please try again.',
          });
          router.push('/ui/chat');
          return;
        }
      }

      // Case 2: No ID in URL - Check current chat
      if (currentChat) {
        if (currentChat.messages?.length > 0) {
          router.replace(`/ui/chat/?id=${currentChat.id}`);
          return;
        }

        setIsLoading(false);
        return;
      }

      // Case 3: Create new chat
      const newChat: Chat = {
        id: nanoid(),
        title: 'New Chat',
        messages: [],
        createdAt: Date.now(),
        updatedAt: Date.now(),
      };

      setCurrentChat(newChat);
      setIsLoading(false);
    };

    initializeChat();
  }, [searchParams, router, getChat, currentChat, setCurrentChat, toast]);

  const handleChatFinish = () => {
    const id = searchParams.get('id');
    // Only update URL if we don't already have an ID and there's a current chat
    if (!id && currentChat) {
      router.push(`/ui/chat/?id=${currentChat.id}`);
    }
  };

  if (isLoading) {
    return null;
  }
  return (
    <MainLayout sidebarContent={<ChatHistory />}>
      <ChatSettingsProvider>
        <SidebarProvider
          inner
          open={settingsOpen}
          onOpenChange={setSettingsOpen}
          className="flex-1 flex flex-col"
        >
          <div
            className={cn(
              'flex-1 flex flex-col min-w-0',
              'transition-[margin] duration-300 ease-in-out',
              settingsOpen && 'mr-64'
            )}
          >
            <ChatProvider chat={currentChat!}>
              <ChatUI isLoading={isLoading} onFinish={handleChatFinish} />
            </ChatProvider>
          </div>

          <div
            className={cn(
              'fixed top-4 z-40 transition-all duration-300',
              'right-0',
              settingsOpen && 'right-[16rem]',
              !settingsOpen && 'right-4'
            )}
          >
            <Button
              onClick={() => setSettingsOpen(!settingsOpen)}
              className={cn('p-2 rounded-lg hover:bg-accent mr-2 -ml-1')}
              aria-label="Toggle settings sidebar"
              aria-expanded={settingsOpen}
              variant="ghost"
            >
              <Settings2 />
            </Button>
          </div>
          <SettingsSidebar />
        </SidebarProvider>
      </ChatSettingsProvider>
    </MainLayout>
  );
}
