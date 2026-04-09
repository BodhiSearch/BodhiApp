import { createFileRoute, useNavigate, useSearch } from '@tanstack/react-router';
import { z } from 'zod';
import React, { useEffect, useRef } from 'react';

import { PanelLeftOpen, PanelLeftClose, Settings2, X } from 'lucide-react';

import { ChatHistory } from './-components/ChatHistory';
import { ChatUI } from './-components/ChatUI';
import { NewChatButton } from './-components/NewChatButton';
import { SettingsSidebar } from './-components/settings/SettingsSidebar';
import AppInitializer from '@/components/AppInitializer';
import {
  Sidebar,
  SidebarContent,
  SidebarProvider,
  SidebarSeparator,
  SidebarTrigger,
  useSidebar,
} from '@/components/ui/sidebar';
import { useResponsiveTestId } from '@/hooks/use-responsive-testid';
import { useLocalStorage } from '@/hooks/useLocalStorage';
import { cn } from '@/lib/utils';
import { useChatStore } from '@/stores/chatStore';
import { useChatSettingsStore } from '@/stores/chatSettingsStore';
import { initChatStoreSubscriptions, hydrateStoresForCurrentChat } from '@/stores/initStores';

export const Route = createFileRoute('/chat/')({
  validateSearch: z.object({
    model: z.string().optional(),
    id: z.string().optional(),
  }),
  component: ChatPage,
});

const sidebarStyles = {
  '--sidebar-width': '260px',
  '--sidebar-width-mobile': '90vw',
} as React.CSSProperties;

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

function ChatUrlSync({ chatIdFromUrl, model }: { chatIdFromUrl?: string; model?: string }) {
  const currentChatId = useChatStore((s) => s.currentChatId);
  const isLoaded = useChatStore((s) => s.isLoaded);
  const chats = useChatStore((s) => s.chats);
  const setCurrentChatId = useChatStore((s) => s.setCurrentChatId);
  const navigate = useNavigate();
  const isInitialSync = useRef(true);

  useEffect(() => {
    if (!isLoaded) return;
    if (chatIdFromUrl && isInitialSync.current) {
      const chatExists = chats.some((c) => c.id === chatIdFromUrl);
      if (chatExists) {
        setCurrentChatId(chatIdFromUrl);
      }
    }
    isInitialSync.current = false;
  }, [chatIdFromUrl, chats, setCurrentChatId, isLoaded]);

  useEffect(() => {
    if (isInitialSync.current) return;

    const search: Record<string, string> = {};
    if (model) search.model = model;
    if (currentChatId) search.id = currentChatId;

    navigate({ to: '/chat/', search, replace: true });
  }, [currentChatId, model, navigate]);

  return null;
}

function ChatWithHistory() {
  const [isSidebarOpen, setIsSidebarOpen] = useLocalStorage('sidebar-settings-open', true);
  const { open, openMobile, isMobile } = useSidebar();
  const showHistoryPanel = isMobile ? openMobile : open;
  const search = useSearch({ from: '/chat/' });
  const model = search.model;
  const chatIdFromUrl = search.id;
  const getTestId = useResponsiveTestId();

  // Apply model from URL to settings store on initial load
  useEffect(() => {
    if (model) {
      useChatSettingsStore.getState().setModel(model);
    }
  }, [model]);

  return (
    <>
      <ChatUrlSync chatIdFromUrl={chatIdFromUrl} model={model} />
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
          <SidebarProvider
            inner
            style={settingsSidebarStyles}
            className="flex-1 flex flex-col overflow-hidden"
            open={isSidebarOpen}
            onOpenChange={setIsSidebarOpen}
          >
            <ChatWithSettings />
          </SidebarProvider>
        </div>
      </div>
    </>
  );
}

function ChatPageContent() {
  const [isSidebarOpen, setIsSidebarOpen] = useLocalStorage('sidebar-history-open', true);
  const loadChats = useChatStore((s) => s.loadChats);
  const search = useSearch({ from: '/chat/' });
  const urlModel = search.model;
  const urlChatId = search.id;

  useEffect(() => {
    initChatStoreSubscriptions();
    const result = loadChats();
    if (result && typeof result.then === 'function') {
      result.then(() => {
        if (urlModel && !urlChatId) {
          // URL has ?model=X without ?id=Y — start fresh with that model
          // Don't hydrate previous chat's settings which would overwrite the URL model
          useChatSettingsStore.getState().setModel(urlModel);
        } else {
          hydrateStoresForCurrentChat();
        }
      });
    }
  }, [loadChats, urlModel, urlChatId]);

  return (
    <SidebarProvider style={sidebarStyles} open={isSidebarOpen} onOpenChange={setIsSidebarOpen}>
      <ChatWithHistory />
    </SidebarProvider>
  );
}

export default function ChatPage() {
  return (
    <AppInitializer allowedStatus="ready" authenticated={true}>
      <ChatPageContent />
    </AppInitializer>
  );
}
