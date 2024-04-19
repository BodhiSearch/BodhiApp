import { getChats, clearChats, removeChat } from '@/lib/backend'
import { ClearHistory } from '@/components/clear-history'
import { SidebarItems } from '@/components/sidebar-items'
import { ThemeToggle } from '@/components/theme-toggle'
import { useState } from 'react'
import { useRouter } from 'next/navigation'
import React from 'react'
import { useAsync } from 'react-async-hook';
import { Skeleton } from './ui/skeleton'


interface SidebarListProps {
}

export function SidebarList({ }: SidebarListProps) {
  const chatHistory = useAsync(async () => (await getChats()).data, []);
  const router = useRouter();

  async function clearChatsFn() {
    let { status } = await clearChats();
    if (status == 200) {
      router.push('/');
    }
  }

  async function removeChatFn(chatId: string) {
    let { data, status } = await removeChat(chatId);
    if (status === 200) {
      router.push('/');
    } else {
      console.log(`error deleting chat: ${chatId}`);
    }
  }

  if (chatHistory.loading) {
    return <div className="flex flex-col flex-1 px-4 space-y-4 overflow-auto">
      {Array.from({ length: 10 }).map((_, i) => (
        <Skeleton
          key={i}
          className="w-full h-6 rounded-md shrink-0 animate-pulse bg-zinc-200 dark:bg-zinc-800"
        />
      ))}
    </div>
  }

  if (chatHistory.error) {
    console.log(`error retrieving chats`);
    return <div>Error retrieving chats...</div>;
  }

  const chats = chatHistory.result;
  return (
    <div className="flex flex-1 flex-col overflow-hidden">
      <div className="flex-1 overflow-auto">
        {chats.length ? (
          <div className="space-y-2 px-2">
            <SidebarItems chats={chats} removeChat={removeChatFn} />
          </div>
        ) : (
          <div className="p-8 text-center">
            <p className="text-sm text-muted-foreground">No chat history</p>
          </div>
        )}
      </div>
      <div className="flex items-center justify-between p-4">
        <ThemeToggle />
        <ClearHistory clearChats={clearChatsFn} isEnabled={chats.length > 0} />
      </div>
    </div>
  )
}
