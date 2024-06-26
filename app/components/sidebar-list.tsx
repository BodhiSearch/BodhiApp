import React from 'react'
import { ClearHistory } from '@/components/clear-history'
import { SidebarItems } from '@/components/sidebar-items'
import { ThemeToggle } from '@/components/theme-toggle'
import { Skeleton } from '@/components/ui/skeleton'
import { useChatHistory } from '@/lib/hooks/use-chat-history'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { useChatSettings } from '@/lib/hooks/use-chat-settings'

interface SidebarListProps {
}

export function SidebarList({ }: SidebarListProps) {
  const { status, chats, removeChat, clearChats } = useChatHistory();
  const { model, models, setModel } = useChatSettings();

  if (status === "loading") {
    return <div className="flex flex-col flex-1 px-4 space-y-4 overflow-auto">
      {Array.from({ length: 10 }).map((_, i) => (
        <Skeleton
          key={i}
          className="w-full h-6 rounded-md shrink-0 animate-pulse bg-zinc-200 dark:bg-zinc-800"
        />
      ))}
    </div>
  }

  if (status === "error") {
    return <div>Error retrieving chats...</div>;
  }

  return (
    <div className="flex flex-1 flex-col overflow-hidden">
      <div className="flex-1 overflow-auto">
        {chats.length ? (
          <div className="space-y-2 px-2">
            <SidebarItems chats={chats} removeChat={removeChat} />
          </div>
        ) : (
          <div className="p-8 text-center">
            <p className="text-sm text-muted-foreground">No chat history</p>
          </div>
        )}
      </div>
      <div className="flex items-center justify-between p-4">
        <Select onValueChange={(value) => setModel(value)} value={model || ''}>
          <SelectTrigger>
            <SelectValue placeholder="Model" />
          </SelectTrigger>
          <SelectContent>
            {models && models.map((model, index) => {
              return <SelectItem key={index} value={model.id}>{model.id}</SelectItem>
            })}
          </SelectContent>
        </Select>
      </div>
      <div className="flex items-center justify-between p-4">
        <ThemeToggle />
        <ClearHistory isEnabled={chats.length > 0} clearChats={clearChats} />
      </div>
    </div>
  )
}
