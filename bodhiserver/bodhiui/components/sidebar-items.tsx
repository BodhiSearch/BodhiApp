'use client'

import { type Chat, type ServerActionResult } from '@/lib/types'
import { AnimatePresence, motion } from 'framer-motion'
import { SidebarActions } from '@/components/sidebar-actions'
import { SidebarItem } from '@/components/sidebar-item'

interface SidebarItemsProps {
  chats: Chat[],
  removeChat: (id: string) => ServerActionResult<void>;
}

export function SidebarItems({ chats, removeChat }: SidebarItemsProps) {
  if (!chats.length) return null
  return (
    <AnimatePresence>
      {chats.map(
        (chat, index) =>
          chat && (
            <motion.div
              key={chat?.id}
              exit={{
                opacity: 0,
                height: 0
              }}
            >
              <SidebarItem index={index} chat={chat}>
                <SidebarActions
                  chat={chat}
                  removeChat={removeChat}
                />
              </SidebarItem>
            </motion.div>
          )
      )}
    </AnimatePresence>
  )
}
