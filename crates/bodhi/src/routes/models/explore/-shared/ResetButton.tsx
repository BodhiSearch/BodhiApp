import { ShellIcon } from '@/components/shell';

export type ResetMode = 'filters' | 'query' | 'none';

/**
 * The always-visible toolbar reset, shared across the catalog/list screens. It waterfalls in
 * precedence order: clear active filters first, else clear the search query, else there is nothing
 * to reset (the button is disabled, an inert noop). The parent owns `onReset` and the `mode`
 * computation (`hasFilters ? 'filters' : hasQuery ? 'query' : 'none'`).
 */
export function ResetButton({ mode, onReset, testId }: { mode: ResetMode; onReset: () => void; testId: string }) {
  const label = mode === 'filters' ? 'Clear all filters' : mode === 'query' ? 'Clear search' : 'Nothing to reset';
  return (
    <button
      type="button"
      className="cat-sort-btn cat-toolbar-icon-btn"
      onClick={onReset}
      disabled={mode === 'none'}
      data-testid={testId}
      data-test-state={mode}
      aria-label={label}
      title={label}
    >
      <ShellIcon name="rotate-ccw" size={13} />
    </button>
  );
}
