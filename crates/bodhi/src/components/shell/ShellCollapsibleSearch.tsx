import { useId, useState } from 'react';

import { ShellIcon } from './ShellIcon';
import { ShellSearch } from './ShellSearch';

/**
 * Reusable collapsible search for list-page toolbars: a search icon button that
 * expands into a full-width search row above the toolbar. Collapses when the
 * input loses focus while empty, on Escape, or via the close button.
 *
 * Renders BOTH pieces in document order:
 *  - the expanded `l-searchrow` (when open), placed via `slot="row"`
 *  - the toggle button, placed via `slot="button"`
 * The toolbar composes them; see App Tokens for the canonical usage.
 *
 * First introduced on the App Tokens screen; reused across every list screen.
 */
export interface ShellCollapsibleSearchProps {
  value: string;
  onChange: (value: string) => void;
  placeholder?: string;
  /** testid for the toggle button */
  toggleTestId?: string;
  /** testid for the close button */
  closeTestId?: string;
}

interface UseCollapsibleSearch {
  open: boolean;
  /** The expanded search row — render above the toolbar (only when open). */
  row: React.ReactNode;
  /** The toggle button — render in the toolbar actions. */
  toggle: React.ReactNode;
}

/**
 * Hook form: returns the `row` (expanded input) and `toggle` (button) nodes so a
 * toolbar can place each in the right slot (the row must sit above the toolbar).
 */
export function useCollapsibleSearch({
  value,
  onChange,
  placeholder = 'Search…',
  toggleTestId,
  closeTestId,
}: ShellCollapsibleSearchProps): UseCollapsibleSearch {
  const [open, setOpen] = useState(false);
  const rowId = useId();

  const close = () => {
    onChange('');
    setOpen(false);
  };

  const row = open ? (
    <div className="l-searchrow" key={rowId}>
      <ShellSearch
        value={value}
        onChange={onChange}
        placeholder={placeholder}
        autoFocus
        onKeyDown={(e) => {
          if (e.key === 'Escape') setOpen(false);
        }}
        // Collapse when focus leaves an empty search.
        onBlur={() => {
          if (!value.trim()) setOpen(false);
        }}
      />
      <button className="l-iconbtn" title="Close search" onClick={close} data-testid={closeTestId}>
        <ShellIcon name="x" size={15} />
      </button>
    </div>
  ) : null;

  const toggle = (
    <button
      className={'l-iconbtn' + (open ? ' on' : '')}
      title="Search"
      onClick={() => setOpen((o) => !o)}
      data-testid={toggleTestId}
    >
      <ShellIcon name="search" size={15} />
      {value && !open && <span className="l-dot" />}
    </button>
  );

  return { open, row, toggle };
}
