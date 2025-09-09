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
import { useResponsiveTestId } from '@/hooks/use-responsive-testid';
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
  const getTestId = useResponsiveTestId();

  return (
    <>
      <div
        className={cn(
          'flex-1 flex flex-col min-w-0',
          'transition-[margin] duration-300 ease-in-out',
          !isMobile && open ? 'mr-[calc(24rem)]' : ''
        )}
        data-testid={getTestId('chat-main-content')}
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
        data-testid={getTestId('settings-toggle-button')}
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
  const model = searchParams?.get('model');
  const initialData = model ? { model: model } : undefined;
  const getTestId = useResponsiveTestId();

  return (
    <>
      <Sidebar side="left" data-testid={getTestId('chat-history-sidebar')}>
        <SidebarContent data-testid={getTestId('chat-history-content')}>
          <NewChatButton />
          <SidebarSeparator />
          <ChatHistory />
        </SidebarContent>
      </Sidebar>
      <div className="flex flex-1 flex-col w-full" data-testid={getTestId('chat-layout-main')}>
        <div className="flex flex-1 flex-col h-full" data-testid={getTestId('chat-layout-inner')}>
          <SidebarTrigger
            variant="ghost"
            size="icon"
            className={cn(
              'fixed z-40 transition-all duration-300 left-0 top-20 h-7 w-7 -ml-1',
              open && 'md:left-[262px]',
              !open && 'md:left-5'
            )}
            aria-label="Toggle history"
            data-testid={getTestId('chat-history-toggle')}
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
