'use client';

import { SettingsSidebar } from '@/components/settings/SettingsSidebar';
import { SidebarToggle } from '@/components/SidebarToggle';
import { SidebarProvider } from '@/components/ui/sidebar';
import { useLocalStorage } from '@/hooks/useLocalStorage';
import { useChatDB } from '@/lib/hooks/use-chat-db';
import { nanoid } from '@/lib/utils';
import { Chat } from '@/types/chat';
import { Settings2 } from 'lucide-react';
import { useRouter, useSearchParams } from 'next/navigation';
import { useEffect, useState } from 'react';
import { ChatProvider } from '@/lib/hooks/use-chat';
import { ChatUI } from '@/components/chat/ChatUI';

const SETTINGS_SIDEBAR_KEY = 'settings-sidebar-state';

export function ChatContainer() {
  const [settingsOpen, setSettingsOpen] = useLocalStorage(SETTINGS_SIDEBAR_KEY, true);
  const searchParams = useSearchParams();
  const router = useRouter();
  const { getChat } = useChatDB();
  const [currentChat, setCurrentChat] = useState<Chat | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [isInitialized, setIsInitialized] = useState(false);

  useEffect(() => {
    const initializeChat = async () => {
      const id = searchParams.get('id');

      if (!id) {
        const newId = nanoid();
        const newChat: Chat = {
          id: newId,
          title: 'New Chat',
          messages: [],
          createdAt: Date.now(),
          updatedAt: Date.now()
        };

        setCurrentChat(newChat);
        setIsLoading(false);
        setIsInitialized(true);

        // Use replace instead of push to avoid navigation loop
        router.replace(`/ui/chat/?id=${newId}`);
        return;
      }

      try {
        const { data, status } = await getChat(id);
        const chatData = status === 200 ? data : {
          id,
          title: 'New Chat',
          messages: [],
          createdAt: Date.now(),
          updatedAt: Date.now()
        };
        setCurrentChat(chatData);
      } catch (err) {
        console.error('Failed to load chat:', err);
        setCurrentChat({
          id,
          title: 'New Chat',
          messages: [],
          createdAt: Date.now(),
          updatedAt: Date.now()
        });
      } finally {
        setIsLoading(false);
        setIsInitialized(true);
      }
    };

    if (!isInitialized) {
      initializeChat();
    }
  }, [searchParams, router, getChat, isInitialized]);

  // Show loading state or nothing while initializing
  if (!isInitialized) {
    return null;
  }

  return (
    <>
      <SidebarProvider open={settingsOpen} onOpenChange={setSettingsOpen}>
        <SidebarToggle
          className="-ml-1"
          open={settingsOpen}
          onOpenChange={setSettingsOpen}
          side='right'
          icon={<Settings2 />}
        />
        <SettingsSidebar />
      </SidebarProvider>

      <ChatProvider chat={currentChat!}>
        <ChatUI isLoading={isLoading} />
      </ChatProvider>
    </>
  );
}