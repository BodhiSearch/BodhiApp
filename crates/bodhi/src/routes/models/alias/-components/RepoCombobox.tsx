import { useEffect, useMemo, useState } from 'react';

import { ShellIcon } from '@/components/shell';
import { Command, CommandEmpty, CommandGroup, CommandInput, CommandItem, CommandList } from '@/components/ui/command';
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover';
import { useListModelFiles } from '@/hooks/models';
import { useSearchRepos } from '@/hooks/reference';

type RepoOrigin = 'local' | 'remote';
interface RepoOption {
  id: string;
  origin: RepoOrigin;
}

const ORIGIN_LABEL: Record<RepoOrigin, string> = {
  local: 'Downloaded',
  remote: 'HuggingFace',
};

interface RepoComboboxProps {
  value: string;
  onChange: (repo: string) => void;
  /** Trigger data-testid — kept verbatim for E2E (`repo-input`). */
  testId: string;
}

/**
 * Searchable repo (`<org>/<repo>`) picker for the local-model form. Merges two sources, local-first:
 * already-downloaded repos (`/bodhi/v1/models/files`, filtered client-side) lead, then live
 * HuggingFace GGUF suggestions (`/api/v1/repos`) with any id already shown as local dropped.
 *
 * Stays free-text: a typed repo with no matching suggestion still commits (the "Use … " row), so any
 * `<org>/<repo>` works and `QuantSelector` resolves its quants. On empty input only local repos show
 * (the reference endpoint requires a non-empty search).
 */
export function RepoCombobox({ value, onChange, testId }: RepoComboboxProps) {
  const [open, setOpen] = useState(false);
  const [input, setInput] = useState('');
  const [debounced, setDebounced] = useState('');

  useEffect(() => {
    const t = setTimeout(() => setDebounced(input.trim()), 200);
    return () => clearTimeout(t);
  }, [input]);

  const { data: filesData } = useListModelFiles(1, 100, 'repo', 'asc');
  const { data: remoteItems } = useSearchRepos({ search: debounced, filter: 'gguf', limit: 10 });

  const localRepos = useMemo(() => {
    const set = new Set<string>();
    (filesData?.data ?? []).forEach((f) => set.add(f.repo));
    return [...set].sort();
  }, [filesData]);

  const options = useMemo<RepoOption[]>(() => {
    const needle = input.trim().toLowerCase();
    const locals = (needle ? localRepos.filter((r) => r.toLowerCase().includes(needle)) : localRepos).map(
      (id): RepoOption => ({ id, origin: 'local' })
    );
    const localSet = new Set(locals.map((o) => o.id));
    const remotes = (remoteItems ?? [])
      .filter((r) => !localSet.has(r.id))
      .map((r): RepoOption => ({ id: r.id, origin: 'remote' }));
    return [...locals, ...remotes];
  }, [localRepos, remoteItems, input]);

  const typed = input.trim();
  const showFreeText = typed.length > 0 && !options.some((o) => o.id === typed);

  const commit = (repo: string) => {
    onChange(repo);
    setOpen(false);
    setInput('');
  };

  return (
    <Popover open={open} onOpenChange={setOpen}>
      <PopoverTrigger asChild>
        <button type="button" role="combobox" aria-expanded={open} data-testid={testId} className="lf-rc-trigger">
          <span className={value ? 'lf-rc-value lf-mono' : 'lf-rc-placeholder'}>{value || 'org/repo'}</span>
          <ShellIcon name="chevrons-up-down" size={13} />
        </button>
      </PopoverTrigger>
      <PopoverContent className="lf-rc-pop" align="start">
        {/* shouldFilter off — we order/dedup the two sources ourselves; cmdk would re-sort them. */}
        <Command shouldFilter={false}>
          <CommandInput placeholder="Search HuggingFace repos…" value={input} onValueChange={setInput} />
          <CommandList>
            {options.length === 0 && !showFreeText && (
              <CommandEmpty>{typed ? 'No matching repos.' : 'No downloaded repos yet — type to search.'}</CommandEmpty>
            )}
            {options.length > 0 && (
              <CommandGroup>
                {options.map((o) => (
                  <CommandItem
                    key={`${o.origin}:${o.id}`}
                    value={o.id}
                    aria-label={o.id}
                    onSelect={() => commit(o.id)}
                    className="lf-rc-item"
                  >
                    <span className="lf-rc-item-name lf-mono">{o.id}</span>
                    <span className={`lf-rc-badge lf-rc-badge-${o.origin}`}>{ORIGIN_LABEL[o.origin]}</span>
                    {o.id === value && <ShellIcon name="check" size={12} />}
                  </CommandItem>
                ))}
              </CommandGroup>
            )}
            {showFreeText && (
              <CommandGroup>
                <CommandItem
                  value={typed}
                  aria-label={typed}
                  onSelect={() => commit(typed)}
                  className="lf-rc-item"
                  data-testid="repo-freetext"
                >
                  <span className="lf-rc-item-name lf-mono">{typed}</span>
                  <span className="lf-rc-badge lf-rc-badge-freetext">Use this</span>
                </CommandItem>
              </CommandGroup>
            )}
          </CommandList>
        </Command>
      </PopoverContent>
    </Popover>
  );
}
