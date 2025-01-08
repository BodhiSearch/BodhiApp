'use client';

import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';
import {
  Sidebar,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
} from '@/components/ui/sidebar';
import { ChevronsUpDown, Check } from 'lucide-react';
import { useRouter } from 'next/navigation';
import { IconMapper } from '../ui/icon-mapper';
import { useNavigation } from '@/hooks/use-navigation';

export function NavigationSidebar() {
  const router = useRouter();
  const { pages, currentPage } = useNavigation();

  return (
    <Sidebar variant="inset" collapsible="icon">
      <SidebarMenu>
        <SidebarMenuItem>
          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <SidebarMenuButton
                size="lg"
                className="data-[state=open]:bg-sidebar-accent data-[state=open]:text-sidebar-accent-foreground"
              >
                <div className="flex aspect-square size-8 items-center justify-center rounded-lg bg-sidebar-primary text-sidebar-primary-foreground">
                  <IconMapper name={currentPage.iconName} className="size-4" />
                </div>
                <div className="flex flex-col gap-0.5 leading-none">
                  <span>{currentPage.title}</span>
                </div>
                <ChevronsUpDown className="ml-auto" />
              </SidebarMenuButton>
            </DropdownMenuTrigger>
            <DropdownMenuContent
              className="w-[--radix-dropdown-menu-trigger-width]"
              align="start"
            >
              {pages.map((page) => (
                <DropdownMenuItem
                  key={page.url}
                  onSelect={() => router.push(page.url)}
                >
                  {page.title}
                  {currentPage.url === page.url && (
                    <Check className="ml-auto" />
                  )}
                </DropdownMenuItem>
              ))}
            </DropdownMenuContent>
          </DropdownMenu>
        </SidebarMenuItem>
      </SidebarMenu>
    </Sidebar>
  );
}
