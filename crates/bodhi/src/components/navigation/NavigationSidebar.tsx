'use client';

import { Button } from '@/components/ui/button';
import { PlusCircle, Home, Users } from 'lucide-react';
import Link from 'next/link';
import { RecentChats } from '@/components/navigation/RecentChats';
import {
  Sidebar,
  SidebarContent,
  SidebarHeader,
  SidebarGroup,
  SidebarGroupContent,
  SidebarGroupLabel,
  SidebarMenu,
  SidebarMenuItem,
  SidebarMenuButton,
} from '@/components/ui/sidebar';

const navigationItems = [
  {
    name: 'Home',
    href: '/ui/home',
    icon: Home,
  },
  {
    name: 'Assistants',
    href: '/ui/assistants',
    icon: Users,
  },
];

export function NavigationSidebar() {
  return (
    <Sidebar variant="inset">
      <SidebarHeader>
        <Link href="/ui/home">Bodhi</Link>
      </SidebarHeader>

      <SidebarContent>
        <SidebarGroup>
          <SidebarGroupContent>
            <Link href="/ui/chat" passHref>
              <Button>
                <PlusCircle />
                New Chat
              </Button>
            </Link>

            <SidebarMenu>
              {navigationItems.map((item) => {
                const Icon = item.icon;
                return (
                  <SidebarMenuItem key={item.href}>
                    <SidebarMenuButton asChild>
                      <Link href={item.href}>
                        <Icon />
                        <span>{item.name}</span>
                      </Link>
                    </SidebarMenuButton>
                  </SidebarMenuItem>
                );
              })}
            </SidebarMenu>
          </SidebarGroupContent>
        </SidebarGroup>
        <SidebarGroup>
          <SidebarGroupLabel>Recent Chats</SidebarGroupLabel>
          <SidebarGroupContent>
            <RecentChats />
          </SidebarGroupContent>
        </SidebarGroup>
      </SidebarContent>
    </Sidebar>
  );
}
