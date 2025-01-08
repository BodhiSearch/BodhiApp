'use client';

import { useNavigation } from '@/hooks/use-navigation';
import { SidebarInset, SidebarTrigger } from '@/components/ui/sidebar';
import * as React from 'react';
import { Separator } from '../ui/separator';
import { Breadcrumb, BreadcrumbItem, BreadcrumbLink } from '../ui/breadcrumb';
import { NavigationSidebar } from '../navigation/NavigationSidebar';

interface MainLayoutProps {
  sidebarContent?: React.ReactNode;
  children: React.ReactNode;
}

export function MainLayout({ sidebarContent, children }: MainLayoutProps) {
  const { currentPage } = useNavigation();
  return (
    <>
      <NavigationSidebar>{sidebarContent}</NavigationSidebar>
      <SidebarInset>
        <header className="flex h-16 shrink-0 items-center gap-2 transition-[width,height] ease-linear group-has-[[data-collapsible=icon]]/sidebar-wrapper:h-12">
          <div className="flex items-center gap-2 px-4">
            <SidebarTrigger className="-ml-1" />
            <Separator orientation="vertical" className="mr-2 h-4" />
            <Breadcrumb>
              <BreadcrumbItem>
                <BreadcrumbLink href="#">{currentPage.title}</BreadcrumbLink>
              </BreadcrumbItem>
            </Breadcrumb>
          </div>
        </header>
        <main className="flex flex-1 flex-col overflow-hidden">{children}</main>
      </SidebarInset>
    </>
  );
}
