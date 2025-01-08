'use client';

import AppInitializer from '@/components/AppInitializer';
import { ChatContainer } from '@/components/chat/ChatContainer';
import { ChatDBProvider } from '@/hooks/use-chat-db';

function ChatPageContent() {
  return (
    <ChatDBProvider>
      <ChatContainer />
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
