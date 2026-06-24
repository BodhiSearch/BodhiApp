import { useMemo, useState } from 'react';

import { ShellIcon } from '@/components/shell';
import { Command, CommandEmpty, CommandInput, CommandItem, CommandList } from '@/components/ui/command';
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover';

export interface FacetOption {
  value: string;
  /** Display label; defaults to the value. */
  label?: string;
  count?: number;
}

interface FacetComboboxProps {
  /** Options (typically derived from a facet `{value:count}` map, highest count first). */
  options: FacetOption[];
  /** Currently selected values. */
  selected: string[];
  onToggle: (value: string) => void;
  placeholder: string;
  searchPlaceholder: string;
  emptyText: string;
  /** testid prefix, e.g. `cat-model-provider` → trigger `cat-model-provider-combo`, chips `…-chip-<v>`. */
  testId: string;
}

/**
 * Multi-select autocomplete with removable chip tags, for high-cardinality facets (provider, family)
 * where a fixed chip set doesn't scale. Options + counts come from the catalog facet maps; selecting
 * adds a chip (and sends a repeated query param), removing a chip clears it. Built on cmdk Command +
 * Popover (same primitives as AliasCombobox) for real listbox semantics + keyboard nav.
 */
export function FacetCombobox({
  options,
  selected,
  onToggle,
  placeholder,
  searchPlaceholder,
  emptyText,
  testId,
}: FacetComboboxProps) {
  const [open, setOpen] = useState(false);
  const labelOf = useMemo(() => {
    const m = new Map<string, string>();
    options.forEach((o) => m.set(o.value, o.label ?? o.value));
    return m;
  }, [options]);

  return (
    <div className="cat-combo" data-testid={`${testId}-combo`}>
      {selected.length > 0 && (
        <div className="cat-combo-chips">
          {selected.map((v) => (
            <button
              key={v}
              type="button"
              className="cat-combo-chip"
              onClick={() => onToggle(v)}
              data-testid={`${testId}-chip-${v}`}
              aria-label={`Remove ${labelOf.get(v) ?? v}`}
            >
              {labelOf.get(v) ?? v}
              <ShellIcon name="x" size={10} />
            </button>
          ))}
        </div>
      )}
      <Popover open={open} onOpenChange={setOpen}>
        <PopoverTrigger asChild>
          <button
            type="button"
            role="combobox"
            aria-expanded={open}
            className="cat-combo-trigger"
            data-testid={`${testId}-trigger`}
          >
            <span className="cat-combo-placeholder">{placeholder}</span>
            <ShellIcon name="chevrons-up-down" size={12} />
          </button>
        </PopoverTrigger>
        <PopoverContent className="cat-combo-pop" align="start">
          <Command>
            <CommandInput placeholder={searchPlaceholder} />
            <CommandList>
              <CommandEmpty>{emptyText}</CommandEmpty>
              {options.map((o) => {
                const active = selected.includes(o.value);
                return (
                  <CommandItem
                    key={o.value}
                    value={`${o.label ?? o.value} ${o.value}`}
                    aria-label={o.value}
                    onSelect={() => onToggle(o.value)}
                    className="cat-combo-item"
                  >
                    {active && <ShellIcon name="check" size={12} />}
                    <span className="cat-combo-item-name">{o.label ?? o.value}</span>
                    {o.count != null && <span className="cat-facet-count">{o.count}</span>}
                  </CommandItem>
                );
              })}
            </CommandList>
          </Command>
        </PopoverContent>
      </Popover>
    </div>
  );
}

/** Turn a facet `{value: count}` map into options, highest-count first. */
export function facetOptions(bucket: Record<string, number> | undefined): FacetOption[] {
  if (!bucket) return [];
  return Object.entries(bucket)
    .sort((a, b) => b[1] - a[1])
    .map(([value, count]) => ({ value, count }));
}
