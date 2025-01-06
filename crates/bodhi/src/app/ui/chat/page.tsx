'use client';

import { MainLayout } from '@/components/layout/MainLayout';
import AppInitializer from '@/components/AppInitializer';
import { NavigationSidebar } from '@/components/navigation/NavigationSidebar';
import { ChatContainer } from '@/components/chat/ChatContainer';
import { ChatsProvider } from '@/lib/hooks/use-chats';

function ChatPageContent() {
  return (
    <MainLayout navigationSidebar={<NavigationSidebar />}>
      <ChatContainer />
    </MainLayout>
  );
}

export default function ChatPage() {
  return (
    <AppInitializer allowedStatus="ready" authenticated={true}>
      <ChatsProvider>
        <ChatPageContent />
      </ChatsProvider>
    </AppInitializer>
  );
}
