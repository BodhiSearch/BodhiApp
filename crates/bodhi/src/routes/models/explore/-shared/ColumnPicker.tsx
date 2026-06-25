import { useCallback, useMemo, useState } from 'react';

import { ShellIcon } from '@/components/shell';
import {
  DropdownMenu,
  DropdownMenuCheckboxItem,
  DropdownMenuContent,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';

/** A column that can appear in the picker (`optional` columns are the only ones offered). */
export interface PickerColumn {
  key: string;
  label: string;
  optional?: boolean;
}

/**
 * Hidden-column state for a catalog table: which optional columns the user has toggled off, plus a
 * `visibleColumns()` filter that drops them. Replaces the `useState<Set>` + `toggleColumn` +
 * `visibleColumns` useMemo boilerplate that each screen used to hand-roll.
 */
export function useHiddenColumns() {
  const [hidden, setHidden] = useState<Set<string>>(() => new Set());
  const toggle = useCallback((key: string) => {
    setHidden((prev) => {
      const next = new Set(prev);
      if (next.has(key)) next.delete(key);
      else next.add(key);
      return next;
    });
  }, []);
  const visibleColumns = useCallback(
    <T extends PickerColumn>(cols: T[]): T[] => cols.filter((c) => !c.optional || !hidden.has(c.key)),
    [hidden]
  );
  return { hidden, toggle, visibleColumns };
}

/**
 * The show/hide column dropdown shared across catalog/list screens. Only `optional` columns appear.
 * `testIdPrefix` keeps each page's existing testids stable (`cat-model`, `cat-prov`, …).
 */
export function ColumnPicker({
  columns,
  hidden,
  onToggle,
  testIdPrefix,
}: {
  columns: PickerColumn[];
  hidden: Set<string>;
  onToggle: (key: string) => void;
  testIdPrefix: string;
}) {
  const optional = useMemo(() => columns.filter((c) => c.optional), [columns]);
  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <button
          type="button"
          className="cat-sort-btn cat-toolbar-icon-btn"
          data-testid={`${testIdPrefix}-columns`}
          aria-label="Columns"
          title="Columns"
        >
          <ShellIcon name="columns-3" size={13} />
        </button>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="end">
        <DropdownMenuLabel>Columns</DropdownMenuLabel>
        <DropdownMenuSeparator />
        {optional.map((col) => (
          <DropdownMenuCheckboxItem
            key={col.key}
            checked={!hidden.has(col.key)}
            onCheckedChange={() => onToggle(col.key)}
            onSelect={(e) => e.preventDefault()}
            data-testid={`${testIdPrefix}-col-${col.key}`}
          >
            {col.label}
          </DropdownMenuCheckboxItem>
        ))}
      </DropdownMenuContent>
    </DropdownMenu>
  );
}
