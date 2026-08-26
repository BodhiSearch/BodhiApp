import type { MouseEvent, ReactNode } from 'react';

export interface LinkRowProps {
  /** Runs the row's select handler (open detail rail). Same callback the row's onClick uses. */
  onActivate: () => void;
  /** Accessible name announced by screen readers and shown by link-hint tools (e.g. Vimium). */
  label?: string;
  /**
   * When provided, the anchor is COMPACT — wraps these children (typically the row's `#` index)
   * inline instead of stretching across the row. For horizontally-scrollable tables: a full-row
   * stretched anchor sits under cell content and can go off-screen under overflow, so link-hint
   * tools (Vimium) miss it — a small, leftmost cell anchor is reliably detected.
   */
  children?: ReactNode;
}

/**
 * Turns a selectable row into a real link target for keyboard/link-hint tools (e.g. Vimium) and
 * screen readers. href="#" + preventDefault keeps it non-navigable; stopPropagation stops the row's
 * own onClick from also firing (avoids a duplicate view transition).
 *
 * Compact mode (`children`) wraps the `#` index instead of stretching across the row, since a
 * stretched anchor under cell content is missed by link-hint tools once the row overflows horizontally.
 *
 * onMouseDown preventDefault keeps the anchor from taking focus on a mouse click, avoiding a stale
 * :focus-visible outline once focus later moves elsewhere.
 */
export function LinkRow({ onActivate, label, children }: LinkRowProps) {
  const handleClick = (e: MouseEvent<HTMLAnchorElement>) => {
    e.preventDefault();
    e.stopPropagation();
    onActivate();
  };
  return (
    <a
      className={children != null ? 'l-rowlink l-rowlink--cell' : 'l-rowlink'}
      href="#"
      aria-label={label ?? 'Open details'}
      data-testid="row-link"
      onMouseDown={(e) => e.preventDefault()}
      onClick={handleClick}
    >
      {children}
    </a>
  );
}
