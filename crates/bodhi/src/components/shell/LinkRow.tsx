import type { MouseEvent } from 'react';

export interface LinkRowProps {
  /** Runs the row's select handler (open detail rail). Same callback the row's onClick uses. */
  onActivate: () => void;
  /** Accessible name announced by screen readers and shown by link-hint tools (e.g. Vimium). */
  label?: string;
}

/**
 * Empty, stretched <a> that turns a selectable list row into a real link target so
 * keyboard / link-hint tools (e.g. the Vimium extension) surface the whole row and screen
 * readers announce it. Rendered as the FIRST child of a `position: relative` row; it fills the
 * row but sits BEHIND the row's own controls (see `.l-rowlink` + the control-raising selector in
 * list.css / settings.css), so buttons / selects / switches stay clickable and a normal mouse
 * click still lands on the row. Selection is local state (no URL), so this is href="#" +
 * preventDefault — not a navigable link. stopPropagation prevents the row's own onClick from also
 * firing (which would run a second view transition on the same selection).
 *
 * onMouseDown preventDefault keeps the anchor from taking DOM focus on a mouse click, so a row
 * selected by mouse never lingers with a stale :focus-visible outline once focus later moves
 * elsewhere (e.g. activating a different row via a link-hint tool like Vimium).
 */
export function LinkRow({ onActivate, label }: LinkRowProps) {
  const handleClick = (e: MouseEvent<HTMLAnchorElement>) => {
    e.preventDefault();
    e.stopPropagation();
    onActivate();
  };
  return (
    <a
      className="l-rowlink"
      href="#"
      aria-label={label ?? 'Open details'}
      data-testid="row-link"
      onMouseDown={(e) => e.preventDefault()}
      onClick={handleClick}
    />
  );
}
