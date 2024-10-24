import * as React from 'react'
import { useInView } from 'react-intersection-observer'
import { useAtBottom } from '@/lib/hooks/use-at-bottom'
import { useEffect } from 'react'
import { useRouter } from 'next/router'

interface ChatScrollAnchorProps {
  trackVisibility?: boolean
}

export function ChatScrollAnchor({ trackVisibility }: ChatScrollAnchorProps) {
  const isAtBottom = useAtBottom()
  const router = useRouter();
  const { ref, entry, inView } = useInView({
    trackVisibility,
    delay: 100,
    rootMargin: '0px 0px -150px 0px'
  })
  useEffect(() => {
    if (isAtBottom && trackVisibility && !inView) {
      entry?.target.scrollIntoView({
        block: 'start'
      })
    }
  }, [inView, entry, isAtBottom, trackVisibility]);
  useEffect(() => {
    if (!router.isReady) return;
    entry?.target.scrollIntoView({ block: 'start', behavior: 'smooth' });
  }, [router.isReady, entry]);

  return <div ref={ref} className="h-px w-full" />
}
