import type { MouseEvent, ReactNode } from 'react';

export interface LinkRowProps {
  /** Runs the row's select handler (open detail rail). Same callback the row's onClick uses. */
  onActivate: () => void;
  /** Accessible name announced by screen readers and shown by link-hint tools (e.g. Vimium). */
  label?: string;
  /**
   * When provided, the anchor is COMPACT — it wraps these children (typically the row's `#` index)
   * and flows inline instead of stretching across the whole row. Use this in horizontally-scrollable
   * tables: a full-row stretched anchor is covered by the cell content (z-index:1) and can sit
   * partly off-screen under horizontal overflow, so link-hint tools (Vimium) fail to surface it.
   * A small, leftmost, always-visible cell anchor is reliably detected. The row's own `onClick`
   * still handles plain mouse selection, so the rest of the row stays selectable.
   */
  children?: ReactNode;
}

/**
 * Turns a selectable list row into a real link target so keyboard / link-hint tools (e.g. the
 * Vimium extension) surface it and screen readers announce it. Selection is local state (no URL),
 * so this is href="#" + preventDefault — not a navigable link. stopPropagation prevents the row's
 * own onClick from also firing (which would run a second view transition on the same selection).
 *
 * Two shapes:
 *   - **Stretched (default)**: an empty `<a>` filling the row, BEHIND the row's controls (see
 *     `.l-rowlink` + the control-raising selectors in list.css / settings.css). Rendered as the
 *     FIRST child of a `position: relative` row. Used by flex list rows (settings, tokens, users …).
 *   - **Compact (`children` given)**: an inline `<a.l-rowlink--cell>` wrapping the children (the `#`
 *     index). Used by the catalog tables, where a stretched anchor is unreliable for link-hint tools
 *     under horizontal overflow.
 *
 * onMouseDown preventDefault keeps the anchor from taking DOM focus on a mouse click, so a row
 * selected by mouse never lingers with a stale :focus-visible outline once focus later moves
 * elsewhere (e.g. activating a different row via a link-hint tool like Vimium).
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
