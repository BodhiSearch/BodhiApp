import Link from 'next/link'
import { motion } from 'framer-motion'
import { buttonVariants } from '@/components/ui/button'
import { type Chat } from '@/lib/types'
import { RouteChat, cn } from '@/lib/utils'
import { IconMessage } from '@/components/ui/icons'
import { useLocalStorage } from '@/lib/hooks/use-local-storage'
import { useRouter } from 'next/router'
import { SidebarActions } from '@/components/sidebar-actions'
import { useEffect, useState } from 'react'

interface SidebarItemProps {
  index: number
  chat: Chat
}

export function SidebarItem({ index, chat }: SidebarItemProps) {
  const router = useRouter();
  const [isActive, setActive] = useState(false);
  const [newChatId, setNewChatId] = useLocalStorage('newChatId', null)
  const shouldAnimate = index === 0 && !!newChatId && chat.id === newChatId
  // mark item as active
  useEffect(() => {
    const { id } = router.query;
    if (!id) {
      setActive(false);
      return;
    }
    const isItemActive = chat.id === id;
    setActive(isItemActive);
  }, [router, chat, setActive]);

  if (!chat.id) return null

  return (
    <motion.div
      className="relative h-8"
      variants={{
        initial: {
          height: 0,
          opacity: 0
        },
        animate: {
          height: 'auto',
          opacity: 1
        }
      }}
      initial={shouldAnimate ? 'initial' : undefined}
      animate={shouldAnimate ? 'animate' : undefined}
      transition={{
        duration: 0.25,
        ease: 'easeIn'
      }}
    >
      <div className="absolute left-2 top-1 flex size-6 items-center justify-center">
        <IconMessage className="mr-2 mt-1 text-zinc-500" />
      </div>
      <Link
        href={RouteChat(chat.id)}
        className={cn(
          buttonVariants({ variant: 'ghost' }),
          'group w-full px-8 transition-colors hover:bg-zinc-200/40 dark:hover:bg-zinc-300/10',
          isActive && 'bg-zinc-200 pr-16 font-semibold dark:bg-zinc-800'
        )}
      >
        <div
          className="relative max-h-5 flex-1 select-none overflow-hidden text-ellipsis break-all"
          title={chat.title}
        >
          <span className="whitespace-nowrap">
            {shouldAnimate ? (
              chat.title.split('').map((character, index) => (
                <motion.span
                  key={index}
                  variants={{
                    initial: {
                      opacity: 0,
                      x: -100
                    },
                    animate: {
                      opacity: 1,
                      x: 0
                    }
                  }}
                  initial={shouldAnimate ? 'initial' : undefined}
                  animate={shouldAnimate ? 'animate' : undefined}
                  transition={{
                    duration: 0.25,
                    ease: 'easeIn',
                    delay: index * 0.05,
                    staggerChildren: 0.05
                  }}
                  onAnimationComplete={() => {
                    if (index === chat.title.length - 1) {
                      setNewChatId(null)
                      if (!router.pathname.includes('chat')) {
                        window.history.replaceState({}, '', `/chat?id=${chat.id}`)
                      }
                      setActive(true);
                    }
                  }}
                >
                  {character}
                </motion.span>
              ))
            ) : (
              <span>{chat.title}</span>
            )}
          </span>
        </div>
      </Link>
      {isActive && <div className="absolute right-2 top-1">
        <SidebarActions
          chat={chat}
        /></div>}
    </motion.div>
  )
}
