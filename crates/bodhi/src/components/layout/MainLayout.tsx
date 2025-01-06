'use client';

import { SidebarInset, SidebarProvider, SidebarTrigger } from '@/components/ui/sidebar';
import * as React from 'react';
import { useLocalStorage } from '@/hooks/useLocalStorage';
import { SidebarToggle } from '@/components/SidebarToggle';
import { Separator } from '../ui/separator';
import { Breadcrumb, BreadcrumbItem, BreadcrumbLink, BreadcrumbList, BreadcrumbPage, BreadcrumbSeparator } from '../ui/breadcrumb';

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
  return (
    <SidebarProvider>
      {navigationSidebar}
      <SidebarInset>
        <header className="flex h-16 shrink-0 items-center gap-2 transition-[width,height] ease-linear group-has-[[data-collapsible=icon]]/sidebar-wrapper:h-12">
          <div className="flex items-center gap-2 px-4">
            <SidebarTrigger className="-ml-1" />
            <Separator orientation="vertical" className="mr-2 h-4" />
            <Breadcrumb>
              <BreadcrumbItem>
                <BreadcrumbLink href="#">
                  Chat
                </BreadcrumbLink>
              </BreadcrumbItem>
            </Breadcrumb>
          </div>
        </header>
        <div className="flex flex-1 flex-col gap-4 p-4 pt-0">
          <main>{children}</main>
        </div>
      </SidebarInset>
    </SidebarProvider>
  );
}
