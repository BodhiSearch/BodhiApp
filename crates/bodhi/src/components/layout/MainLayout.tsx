'use client';

import { SidebarProvider } from '@/components/ui/sidebar';
import * as React from 'react';
import { useLocalStorage } from '@/hooks/useLocalStorage';
import { SidebarToggle } from '@/components/SidebarToggle';

interface MainLayoutProps {
  children: React.ReactNode;
  navigationSidebar?: React.ReactNode;
}

// Constants for localStorage keys
const NAV_SIDEBAR_KEY = 'nav-sidebar-state';

export function MainLayout({
  children,
  navigationSidebar,
}: MainLayoutProps) {
  const [navOpen, setNavOpen] = useLocalStorage(NAV_SIDEBAR_KEY, true);

  return (
    <div className="flex h-screen">
      <SidebarProvider open={navOpen} onOpenChange={setNavOpen}>
        {navigationSidebar}
      </SidebarProvider>
      <SidebarToggle open={navOpen} onOpenChange={setNavOpen} side="left" />

      <main className="flex-1">{children}</main>
    </div>
  );
}
