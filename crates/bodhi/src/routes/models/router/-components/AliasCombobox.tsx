import { useMemo } from 'react';
import { AliasResponse } from '@bodhiapp/ts-client';

import { ShellIcon } from '@/components/shell';
import { Command, CommandEmpty, CommandInput, CommandItem, CommandList } from '@/components/ui/command';
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover';
import { isApiAlias } from '@/lib/utils';
import { aliasKind, AliasTypeBadge, ProviderBadge } from './AliasTypeBadge';

/** Identity used to reference an alias from a target: id for api, name for local. */
export function aliasIdentity(alias: AliasResponse): string {
  return isApiAlias(alias) ? alias.id : alias.alias;
}

interface AliasComboboxProps {
  value: string;
  options: AliasResponse[];
  byIdentity: Map<string, AliasResponse>;
  onSelect: (identity: string) => void;
  /** Trigger data-testid — kept verbatim for E2E (`target-alias-${idx}`). */
  testId: string;
  /** Controlled open state — the form keeps only one combobox open at a time. */
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

/**
 * Searchable alias picker (shadcn Command + Popover). Each option shows a type badge + the
 * identity + a provider badge. cmdk gives real listbox/option semantics + keyboard nav; the
 * option's accessible name is pinned to the identity so the E2E page object can select by it.
 */
export function AliasCombobox({
  value,
  options,
  byIdentity,
  onSelect,
  testId,
  open,
  onOpenChange,
}: AliasComboboxProps) {
  const selected = value ? byIdentity.get(value) : undefined;

  // cmdk filters on each item's `value`; include type + provider so search matches them too,
  // while the rendered accessible name stays the identity (E2E selects by it).
  const searchValues = useMemo(() => {
    const map = new Map<string, string>();
    options.forEach((a) => {
      const id = aliasIdentity(a);
      const provider = isApiAlias(a) ? a.api_format : '';
      map.set(id, `${id} ${aliasKind(a)} ${provider}`);
    });
    return map;
  }, [options]);

  return (
    <Popover open={open} onOpenChange={onOpenChange}>
      <PopoverTrigger asChild>
        <button type="button" role="combobox" aria-expanded={open} data-testid={testId} className="rf-combobox-trigger">
          <span className={selected ? 'rf-combobox-value mono' : 'rf-combobox-placeholder'}>
            {selected ? aliasIdentity(selected) : 'Select an alias…'}
          </span>
          <ShellIcon name="chevrons-up-down" size={13} />
        </button>
      </PopoverTrigger>
      <PopoverContent className="rf-combobox-pop" align="start">
        <Command>
          <CommandInput placeholder="Search aliases…" />
          <CommandList>
            <CommandEmpty>No aliases match.</CommandEmpty>
            {options.map((a) => {
              const id = aliasIdentity(a);
              return (
                <CommandItem
                  key={id}
                  value={searchValues.get(id)}
                  // Accessible name pinned to the identity (E2E `getByRole('option',{name:id})`).
                  aria-label={id}
                  onSelect={() => {
                    onSelect(id);
                    onOpenChange(false);
                  }}
                  className="rf-combobox-item"
                >
                  <AliasTypeBadge alias={a} small />
                  <span className="rf-combobox-item-name mono">{id}</span>
                  <ProviderBadge alias={a} />
                  {id === value && <ShellIcon name="check" size={12} />}
                </CommandItem>
              );
            })}
          </CommandList>
        </Command>
      </PopoverContent>
    </Popover>
  );
}
