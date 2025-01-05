'use client';

import { MainLayout } from '@/components/layout/MainLayout';
import AppInitializer from '@/components/AppInitializer';
import { NavigationSidebar } from '@/components/navigation/NavigationSidebar';
import { SettingsSidebar } from '@/components/settings/SettingsSidebar';
import { ChatContainer } from '@/components/chat/ChatContainer';

function ChatPageContent() {
  return (
    <MainLayout
      navigationSidebar={<NavigationSidebar />}
      settingsSidebar={<SettingsSidebar />}
    >
      <ChatContainer />
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
