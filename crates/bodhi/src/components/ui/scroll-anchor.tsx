import { useEffect, useRef } from 'react';

interface ScrollAnchorProps {
  /**
   * Whether to track and scroll on content changes
   */
  trackVisibility?: boolean;
  /**
   * Custom CSS class for the anchor element
   */
  className?: string;
  /**
   * Behavior of the scroll animation
   */
  behavior?: ScrollBehavior;
}

export function ScrollAnchor({ trackVisibility = true, className = '', behavior = 'smooth' }: ScrollAnchorProps) {
  const anchorRef = useRef<HTMLDivElement>(null);
  const prevScrollRef = useRef<number>(0);

  useEffect(() => {
    if (!trackVisibility) return;

    const anchor = anchorRef.current;
    if (!anchor) return;

    const container = anchor.parentElement;
    if (!container) return;

    const isScrolledUp = container.scrollTop < prevScrollRef.current;

    // Only auto-scroll if at bottom or new content added
    const shouldScroll = !isScrolledUp || container.scrollHeight <= container.clientHeight;

    if (shouldScroll) {
      anchor.scrollIntoView({ behavior });
    }

    prevScrollRef.current = container.scrollTop;
  });

  return <div ref={anchorRef} className={className} />;
}
