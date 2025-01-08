'use client';

import AppInitializer from '@/components/AppInitializer';
import { ChatContainer } from '@/components/chat/ChatContainer';
import { MainLayout } from '@/components/layout/MainLayout';
import { ChatDBProvider } from '@/hooks/use-chat-db';
import { ChatSettingsProvider } from '@/hooks/use-chat-settings';

function ChatPageContent() {
  return (
    <MainLayout>
      <ChatDBProvider>
        <ChatSettingsProvider>
          <ChatContainer />
        </ChatSettingsProvider>
      </ChatDBProvider>
    </MainLayout>
  );
}

export default function ChatPage() {
  return (
    <AppInitializer allowedStatus="ready" authenticated={true}>
      <ChatPageContent />
    </AppInitializer>
  );
}
