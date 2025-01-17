'use client';

import AppInitializer from '@/components/AppInitializer';
import { ChatHistory } from '@/components/chat/ChatHistory';
import { ChatUI } from '@/components/chat/ChatUI';
import { NewChatButton } from '@/components/chat/NewChatButton';
import { SettingsSidebar } from '@/components/settings/SettingsSidebar';
import {
  Sidebar,
  SidebarContent,
  SidebarGroup,
  SidebarProvider,
  SidebarTrigger,
  useSidebar,
} from '@/components/ui/sidebar';
import { ChatDBProvider, useChatDB } from '@/hooks/use-chat-db';
import { ChatSettingsProvider } from '@/hooks/use-chat-settings';
import { cn } from '@/lib/utils';
import { Settings2 } from 'lucide-react';
import { useEffect, useState } from 'react';

function ChatWithSettings() {
  const { open, isMobile } = useSidebar();
  const { initializeCurrentChatId } = useChatDB();
  const [isLoading, setIsLoading] = useState(true);

  useEffect(() => {
    let mounted = true;
    const initialize = async () => {
      try {
        await initializeCurrentChatId();
        if (mounted) {
          setIsLoading(false);
        }
      } catch (error) {
        console.error('Failed to initialize chat:', error);
        if (mounted) {
          setIsLoading(false);
        }
      }
    };

    initialize();

    return () => {
      mounted = false;
    };
  }, []);

  return (
    <>
      <div
        className={cn(
          'flex-1 flex flex-col min-w-0',
          'transition-[margin] duration-300 ease-in-out',
          !isMobile && open ? 'mr-64' : ''
        )}
      >
        <ChatUI isLoading={isLoading} />
      </div>
      <SidebarTrigger
        variant="ghost"
        size="icon"
        className={cn(
          'fixed z-40 transition-all duration-300 right-0 top-20 h-7 w-7 -ml-1 md:right-0',
          open && 'md:right-[16rem]',
          !open && 'md:right-4'
        )}
        aria-label="Toggle settings"
      >
        <Settings2 />
      </SidebarTrigger>
      <SettingsSidebar />
    </>
  );
}

function ChatWithHistory() {
  const { open } = useSidebar();
  return (
    <>
      <Sidebar side="left">
        <SidebarContent>
          <SidebarGroup>
            <NewChatButton />
          </SidebarGroup>
          <SidebarGroup>
            <ChatHistory />
          </SidebarGroup>
        </SidebarContent>
      </Sidebar>
      <div className="flex flex-1 flex-col w-full">
        <div className="flex flex-1 flex-col h-full">
          <SidebarTrigger
            variant="ghost"
            size="icon"
            className={cn(
              'fixed z-40 transition-all duration-300 left-0 top-20 h-7 w-7 -ml-1',
              open && 'md:left-[16rem]',
              !open && 'md:left-4'
            )}
            aria-label="Toggle settings"
          />
          <ChatSettingsProvider>
            <SidebarProvider
              inner
              className="flex-1 flex flex-col overflow-hidden"
            >
              <ChatWithSettings />
            </SidebarProvider>
          </ChatSettingsProvider>
        </div>
      </div>
    </>
  );
}

function ChatPageContent() {
  return (
    <ChatDBProvider>
      <SidebarProvider>
        <ChatWithHistory />
      </SidebarProvider>
    </ChatDBProvider>
  );
}

export default function ChatPage() {
  return (
    <AppInitializer allowedStatus="ready" authenticated={true}>
      <ChatPageContent />
    </AppInitializer>
  );
}
