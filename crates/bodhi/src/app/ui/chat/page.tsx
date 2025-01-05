'use client';

import { MainLayout } from '@/components/layout/MainLayout';
import AppInitializer from '@/components/AppInitializer';
import { NavigationSidebar } from '@/components/navigation/NavigationSidebar';
import { SettingsSidebar } from '@/components/settings/SettingsSidebar';

function ChatPageContent() {
  return (
    <MainLayout
      navigationSidebar={<NavigationSidebar />}
      settingsSidebar={<SettingsSidebar />}
    >
      <div>Chat Content</div>
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
