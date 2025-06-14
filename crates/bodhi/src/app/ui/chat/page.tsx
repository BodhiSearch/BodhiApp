'use client';

import React from 'react';
import AppInitializer from '@/components/AppInitializer';
import { ChatHistory } from '@/app/ui/chat/ChatHistory';
import { ChatUI } from '@/app/ui/chat/ChatUI';
import { NewChatButton } from '@/app/ui/chat/NewChatButton';
import { SettingsSidebar } from '@/app/ui/chat/settings/SettingsSidebar';
import {
  Sidebar,
  SidebarContent,
  SidebarProvider,
  SidebarSeparator,
  SidebarTrigger,
  useSidebar,
} from '@/components/ui/sidebar';
import { ChatDBProvider } from '@/hooks/use-chat-db';
import { ChatSettingsProvider } from '@/hooks/use-chat-settings';
import { cn } from '@/lib/utils';
import { PanelLeftOpen, PanelLeftClose, Settings2, X } from 'lucide-react';
import { useSearchParams } from 'next/navigation';
import { useLocalStorage } from '@/hooks/useLocalStorage';

// Define custom CSS properties for TypeScript
const sidebarStyles = {
  '--sidebar-width': '260px',
  '--sidebar-width-mobile': '90vw',
} as React.CSSProperties;

// Settings sidebar should keep original width
const settingsSidebarStyles = {
  '--sidebar-width': '24rem',
  '--sidebar-width-mobile': '90vw',
} as React.CSSProperties;

function ChatWithSettings() {
  const { open, openMobile, isMobile } = useSidebar();
  const showSettingsPanel = isMobile ? openMobile : open;

  return (
    <>
      <div
        className={cn(
          'flex-1 flex flex-col min-w-0',
          'transition-[margin] duration-300 ease-in-out',
          !isMobile && open ? 'mr-[calc(24rem)]' : ''
        )}
      >
        <ChatUI />
      </div>
      <SidebarTrigger
        variant="ghost"
        size="icon"
        className={cn(
          'fixed z-40 transition-all duration-300 right-0 top-20 h-7 w-7 -ml-1 md:right-0',
          open && 'md:right-[calc(24rem)]',
          !open && 'md:right-4'
        )}
        aria-label="Toggle settings"
      >
        {showSettingsPanel ? <X className="h-5 w-5" /> : <Settings2 className="h-5 w-5" />}
      </SidebarTrigger>
      <SettingsSidebar />
    </>
  );
}

function ChatWithHistory() {
  const [isSidebarOpen, setIsSidebarOpen] = useLocalStorage('sidebar-settings-open', true);
  const { open, openMobile, isMobile } = useSidebar();
  const showHistoryPanel = isMobile ? openMobile : open;
  const searchParams = useSearchParams();
  const alias = searchParams?.get('alias');
  const initialData = alias ? { model: alias } : undefined;

  return (
    <>
      <Sidebar side="left">
        <SidebarContent>
          <NewChatButton />
          <SidebarSeparator />
          <ChatHistory />
        </SidebarContent>
      </Sidebar>
      <div className="flex flex-1 flex-col w-full">
        <div className="flex flex-1 flex-col h-full">
          <SidebarTrigger
            variant="ghost"
            size="icon"
            className={cn(
              'fixed z-40 transition-all duration-300 left-0 top-20 h-7 w-7 -ml-1',
              open && 'md:left-[262px]',
              !open && 'md:left-5'
            )}
            aria-label="Toggle history"
          >
            {showHistoryPanel ? <PanelLeftClose className="h-5 w-5" /> : <PanelLeftOpen className="h-5 w-5" />}
          </SidebarTrigger>
          <ChatSettingsProvider initialData={initialData}>
            <SidebarProvider
              inner
              style={settingsSidebarStyles}
              className="flex-1 flex flex-col overflow-hidden"
              open={isSidebarOpen}
              onOpenChange={setIsSidebarOpen}
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
  const [isSidebarOpen, setIsSidebarOpen] = useLocalStorage('sidebar-history-open', true);
  return (
    <ChatDBProvider>
      <SidebarProvider style={sidebarStyles} open={isSidebarOpen} onOpenChange={setIsSidebarOpen}>
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
