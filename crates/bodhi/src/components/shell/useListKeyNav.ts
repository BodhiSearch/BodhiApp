import { useEffect } from 'react';

export interface ListKeyNavOptions {
  /** Rows to navigate. Default `.l-listrow`. */
  rowSelector?: string;
  /** The scroll container that holds the rows. Default `.l-scroll`. */
  rootSelector?: string;
}

/**
 * Arrow-key navigation for a selectable master-detail list. ↑/↓ move the selection by one row —
 * EAGER: selecting also opens the detail rail, exactly like a click — Home/End jump to first/last,
 * and movement STOPS at the ends (no wrap). It works by activating the target row's stretched
 * `<a.l-rowlink>` (see {@link LinkRow}), reusing each page's existing select handler, so no
 * per-page state wiring is needed beyond one call from the list's container component.
 *
 * Scoped: ignores keys while focus is in a text field, the left sidebar, or the right detail rail,
 * so typing and panel use are never hijacked.
 */
export function useListKeyNav(opts: ListKeyNavOptions = {}) {
  const rowSelector = opts.rowSelector ?? '.l-listrow';
  const rootSelector = opts.rootSelector ?? '.l-scroll';

  useEffect(() => {
    // Skip rows hidden by a collapsed/filtered group. Uses computed style rather than
    // `offsetParent` so it behaves the same in a real browser and in jsdom (which never lays out,
    // so `offsetParent` is always null there).
    function isVisible(el: HTMLElement) {
      if (el.hidden) return false;
      const cs = getComputedStyle(el);
      return cs.display !== 'none' && cs.visibility !== 'hidden';
    }

    function scrollIntoView(root: HTMLElement, row: HTMLElement) {
      const cr = root.getBoundingClientRect();
      const rr = row.getBoundingClientRect();
      const head = root.querySelector<HTMLElement>('.l-listhead, .cat-listhead');
      const top = cr.top + (head ? head.offsetHeight : 0) + 6;
      if (rr.top < top) root.scrollTop -= top - rr.top;
      else if (rr.bottom > cr.bottom - 6) root.scrollTop += rr.bottom - cr.bottom + 6;
    }

    function onKey(e: KeyboardEvent) {
      if (e.defaultPrevented || e.ctrlKey || e.metaKey || e.altKey) return;
      if (!['ArrowDown', 'ArrowUp', 'Home', 'End'].includes(e.key)) return;
      const ae = document.activeElement;
      if (
        ae instanceof HTMLElement &&
        ae.closest(
          'input, textarea, select, [contenteditable=""], [contenteditable="true"], .shell-sidebar, .shell-rail'
        )
      ) {
        return;
      }
      const root = document.querySelector<HTMLElement>(rootSelector);
      if (!root) return;
      const rows = Array.from(root.querySelectorAll<HTMLElement>(rowSelector)).filter(isVisible);
      if (!rows.length) return;

      let cur = rows.findIndex((r) => r.classList.contains('active'));
      if (cur < 0 && ae instanceof HTMLElement) cur = rows.findIndex((r) => r.contains(ae));

      let next: number;
      if (e.key === 'Home') next = 0;
      else if (e.key === 'End') next = rows.length - 1;
      else if (e.key === 'ArrowDown') next = cur < 0 ? 0 : Math.min(rows.length - 1, cur + 1);
      else next = cur < 0 ? rows.length - 1 : Math.max(0, cur - 1);

      e.preventDefault();
      if (next === cur) return; // stop at ends — no wrap
      const row = rows[next];
      const link = row.querySelector<HTMLElement>('.l-rowlink') ?? row;
      link.focus({ preventScroll: true });
      link.click(); // reuse the row's existing select handler
      scrollIntoView(root, row);
    }

    document.addEventListener('keydown', onKey);
    return () => document.removeEventListener('keydown', onKey);
  }, [rowSelector, rootSelector]);
}
