'use client';

import React from 'react';
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
import { ChatDBProvider } from '@/hooks/use-chat-db';
import { ChatSettingsProvider } from '@/hooks/use-chat-settings';
import { cn } from '@/lib/utils';
import { PanelLeftOpen, PanelLeftClose, Settings2, X } from 'lucide-react';
import { useSearchParams } from 'next/navigation';

const SIDEBAR_WIDTH = '24rem';

// Define custom CSS properties for TypeScript
const sidebarStyles = {
  '--sidebar-width': SIDEBAR_WIDTH,
  '--sidebar-width-mobile': '90vw',
} as React.CSSProperties;

function ChatWithSettings() {
  const { open, isMobile } = useSidebar();
  return (
    <>
      <div
        className={cn(
          'flex-1 flex flex-col min-w-0',
          'transition-[margin] duration-300 ease-in-out',
          !isMobile && open ? `mr-[${SIDEBAR_WIDTH}]` : ''
        )}
      >
        <ChatUI />
      </div>
      <SidebarTrigger
        variant="ghost"
        size="icon"
        className={cn(
          'fixed z-40 transition-all duration-300 right-0 top-20 h-7 w-7 -ml-1 md:right-0',
          open && `md:right-[${SIDEBAR_WIDTH}]`,
          !open && 'md:right-4'
        )}
        aria-label="Toggle settings"
      >
        {open ? <X className="h-5 w-5" /> : <Settings2 className="h-5 w-5" />}
      </SidebarTrigger>
      <SettingsSidebar />
    </>
  );
}

function ChatWithHistory() {
  const { open } = useSidebar();
  const searchParams = useSearchParams();
  const alias = searchParams.get('alias');
  const initialData = alias ? { model: alias } : undefined;

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
              open && `md:left-[calc(24.5rem)]`,
              !open && 'md:left-5'
            )}
            aria-label="Toggle history"
          >
            {open ? (
              <PanelLeftClose className="h-5 w-5" />
            ) : (
              <PanelLeftOpen className="h-5 w-5" />
            )}
          </SidebarTrigger>
          <ChatSettingsProvider initialData={initialData}>
            <SidebarProvider
              inner
              style={sidebarStyles}
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
      <SidebarProvider style={sidebarStyles}>
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
