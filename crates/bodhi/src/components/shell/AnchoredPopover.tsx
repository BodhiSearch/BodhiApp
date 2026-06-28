import { type ReactNode, type RefObject, useEffect, useLayoutEffect, useRef, useState } from 'react';

/* ── Anchored popover (fixed-position, escapes overflow) ────── */
export interface AnchoredPopoverProps {
  open: boolean;
  anchorRef: RefObject<HTMLElement | null>;
  onClose: () => void;
  children: ReactNode;
}

export function AnchoredPopover({ open, anchorRef, onClose, children }: AnchoredPopoverProps) {
  const popRef = useRef<HTMLDivElement>(null);
  const [pos, setPos] = useState<{ top: number; left: number } | null>(null);

  useLayoutEffect(() => {
    if (!open || !anchorRef.current) {
      setPos(null);
      return;
    }
    const a = anchorRef.current.getBoundingClientRect();
    const ph = popRef.current ? popRef.current.offsetHeight : 260;
    let top = a.top;
    if (top + ph > window.innerHeight - 8) top = Math.max(8, window.innerHeight - 8 - ph);
    setPos({ top, left: a.right + 8 });
  }, [open, anchorRef]);

  useEffect(() => {
    if (!open) return;
    const h = () => onClose();
    const k = (e: KeyboardEvent) => {
      if (e.key === 'Escape') onClose();
    };
    document.addEventListener('click', h);
    document.addEventListener('keydown', k);
    return () => {
      document.removeEventListener('click', h);
      document.removeEventListener('keydown', k);
    };
  }, [open, onClose]);

  if (!open) return null;
  return (
    <div
      ref={popRef}
      className="shell-pop"
      style={{ top: pos ? pos.top : -9999, left: pos ? pos.left : -9999 }}
      onClick={(e) => e.stopPropagation()}
    >
      {children}
    </div>
  );
}
