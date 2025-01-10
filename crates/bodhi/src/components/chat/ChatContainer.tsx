'use client';

import { SettingsSidebar } from '@/components/settings/SettingsSidebar';
import { SidebarProvider, SidebarTrigger } from '@/components/ui/sidebar';
import { useLocalStorage } from '@/hooks/useLocalStorage';
import { useChatDB } from '@/hooks/use-chat-db';
import { Settings2 } from 'lucide-react';
import { useEffect, useState } from 'react';
import { ChatUI } from '@/components/chat/ChatUI';
import { cn } from '@/lib/utils';
import { MainLayout } from '@/components/layout/MainLayout';
import { ChatHistory } from './ChatHistory';
import { ChatSettingsProvider } from '@/hooks/use-chat-settings';
import { NewChatButton } from './NewChatButton';
import { Separator } from '@/components/ui/separator';

const SETTINGS_SIDEBAR_KEY = 'settings-sidebar-state';

export function ChatContainer() {
  const [settingsOpen, setSettingsOpen] = useLocalStorage(
    SETTINGS_SIDEBAR_KEY,
    true
  );
  const { initializeCurrentChatId } = useChatDB();
  const [isLoading, setIsLoading] = useState(true);

  useEffect(() => {
    const initialize = async () => {
      await initializeCurrentChatId();
      setIsLoading(false);
    };

    initialize();
  }, [initializeCurrentChatId]);

  if (isLoading) {
    return null;
  }

  return (
    <MainLayout
      sidebarContent={
        <div className="flex flex-col h-full">
          <div className="p-2">
            <NewChatButton />
          </div>
          <Separator className="my-2" />
          <div className="flex-1 overflow-auto">
            <ChatHistory />
          </div>
        </div>
      }
    >
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
            <ChatUI isLoading={isLoading} />
          </div>
          <div
            className={cn(
              'fixed top-4 z-40 transition-all duration-300',
              'right-0',
              settingsOpen && 'right-[16rem]',
              !settingsOpen && 'right-4'
            )}
          >
            <SidebarTrigger icon={<Settings2 />} />
          </div>
          <SettingsSidebar />
        </SidebarProvider>
      </ChatSettingsProvider>
    </MainLayout>
  );
}
