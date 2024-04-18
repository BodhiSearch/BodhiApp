import { getChats, clearChats, removeChat } from '@/lib/backend'
import { ClearHistory } from '@/components/clear-history'
import { SidebarItems } from '@/components/sidebar-items'
import { ThemeToggle } from '@/components/theme-toggle'
import { useEffect, useState } from 'react'
import { useRouter } from 'next/navigation'


interface SidebarListProps {
}

export function SidebarList({ }: SidebarListProps) {
  const [chats, setChats] = useState([]);
  const router = useRouter();
  const refreshChats = async () => {
    const { data: chats } = await getChats();
    setChats(chats);
  };
  useEffect(() => {
    refreshChats()
  }, []);

  async function clearChatsFn() {
    let { status } = await clearChats();
    if (status == 200) {
      await refreshChats();
      await router.push('/');
    }
  }

  async function removeChatFn(chatId: string) {
    let { data, status } = await removeChat(chatId);
    if (status === 200) {
      await refreshChats();
      await router.push('/');
    } else {
      return data
    }
  }

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
