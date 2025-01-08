'use client';

import { MainLayout } from '@/components/layout/MainLayout';
import AppInitializer from '@/components/AppInitializer';
import { ChatContainer } from '@/components/chat/ChatContainer';
import { ChatDBProvider } from '@/hooks/use-chat-db';

function ChatPageContent() {
  return (
    <MainLayout>
      <ChatContainer />
    </MainLayout>
  );
}

export default function ChatPage() {
  return (
    <AppInitializer allowedStatus="ready" authenticated={true}>
      <ChatDBProvider>
        <ChatPageContent />
      </ChatDBProvider>
    </AppInitializer>
  );
}
