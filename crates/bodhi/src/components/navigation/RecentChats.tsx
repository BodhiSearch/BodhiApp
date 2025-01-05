'use client';

import { MessageSquare } from 'lucide-react';
import Link from 'next/link';
import {
  SidebarMenu,
  SidebarMenuItem,
  SidebarMenuButton,
} from '@/components/ui/sidebar';

interface ChatGroup {
  title: string;
  chats: {
    id: string;
    title: string;
  }[];
}

const mockChatGroups: ChatGroup[] = [
  {
    title: 'Today',
    chats: [
      { id: '1', title: 'How to add a figma file as library?' },
      { id: '2', title: 'Generate python code for fi...' },
    ],
  },
  {
    title: 'Last 7 Days',
    chats: [
      { id: '3', title: 'What is endothermic reactio...' },
      { id: '4', title: 'Convert given image to CSV' },
    ],
  },
];

export function RecentChats() {
  return (
    <div>
      {mockChatGroups.map((group) => (
        <div key={group.title}>
          <h4>{group.title}</h4>
          <SidebarMenu>
            {group.chats.map((chat) => (
              <SidebarMenuItem key={chat.id}>
                <SidebarMenuButton asChild>
                  <Link href={`/ui/chat/${chat.id}`}>
                    <MessageSquare />
                    <span>{chat.title}</span>
                  </Link>
                </SidebarMenuButton>
              </SidebarMenuItem>
            ))}
          </SidebarMenu>
        </div>
      ))}
    </div>
  );
}
