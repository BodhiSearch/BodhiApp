'use client';

import { SidebarProvider } from '@/components/ui/sidebar';
import { Settings2 } from 'lucide-react';
import * as React from 'react';
import { useLocalStorage } from '@/hooks/useLocalStorage';
import { SidebarToggle } from '@/components/SidebarToggle';

interface MainLayoutProps {
  children: React.ReactNode;
  navigationSidebar?: React.ReactNode;
  settingsSidebar?: React.ReactNode;
}

// Constants for localStorage keys
const NAV_SIDEBAR_KEY = 'nav-sidebar-state';
const SETTINGS_SIDEBAR_KEY = 'settings-sidebar-state';

export function MainLayout({
  children,
  navigationSidebar,
  settingsSidebar,
}: MainLayoutProps) {
  const [navOpen, setNavOpen] = useLocalStorage(NAV_SIDEBAR_KEY, true);
  const [settingsOpen, setSettingsOpen] = useLocalStorage(
    SETTINGS_SIDEBAR_KEY,
    true
  );

  return (
    <div className="flex h-screen">
      <SidebarProvider open={navOpen} onOpenChange={setNavOpen}>
        {navigationSidebar}
      </SidebarProvider>
      <SidebarToggle open={navOpen} onOpenChange={setNavOpen} side="left" />

      <main className="flex-1">{children}</main>

      <SidebarToggle
        open={settingsOpen}
        onOpenChange={setSettingsOpen}
        icon={<Settings2 />}
        side="right"
      />
      <SidebarProvider open={settingsOpen} onOpenChange={setSettingsOpen}>
        {settingsSidebar}
      </SidebarProvider>
    </div>
  );
}
