import { useMemo, useState } from 'react';

import { Check } from 'lucide-react';

import { Button } from '@/components/ui/button';
import { Sheet, SheetContent, SheetDescription, SheetTitle } from '@/components/ui/sheet';

import type { AccessItem } from './types';

interface AccessPickerPanelProps {
  open: boolean;
  onClose: () => void;
  title: string;
  subtitle: string;
  items: AccessItem[];
  selectedIds: string[];
  onToggle: (id: string) => void;
  noun: string;
  testIdPrefix: string;
}

type Group = { label: string; items: AccessItem[] };

/** Slide-in side panel (shadcn Sheet) for picking specific resources: search,
 *  optional Local/API type filter, grouped list, footer count + Done. */
export function AccessPickerPanel({
  open,
  onClose,
  title,
  subtitle,
  items,
  selectedIds,
  onToggle,
  noun,
  testIdPrefix,
}: AccessPickerPanelProps) {
  const [search, setSearch] = useState('');
  const [typeFilter, setTypeFilter] = useState<'all' | 'local' | 'api'>('all');

  const hasTypes = useMemo(() => items.some((i) => i.type), [items]);

  const filtered = useMemo(() => {
    const q = search.trim().toLowerCase();
    return items.filter((i) => {
      if (hasTypes && typeFilter !== 'all' && i.type !== typeFilter) return false;
      if (!q) return true;
      return i.label.toLowerCase().includes(q) || i.id.toLowerCase().includes(q) || !!i.meta?.toLowerCase().includes(q);
    });
  }, [items, search, typeFilter, hasTypes]);

  const groups = useMemo<Group[]>(() => {
    if (!hasTypes) return filtered.length ? [{ label: `${noun}s`, items: filtered }] : [];
    const local = filtered.filter((i) => i.type === 'local');
    const api = filtered.filter((i) => i.type === 'api');
    const out: Group[] = [];
    if (local.length) out.push({ label: 'Local Models', items: local });
    if (api.length) out.push({ label: 'API Models', items: api });
    return out;
  }, [filtered, hasTypes, noun]);

  return (
    <Sheet open={open} onOpenChange={(o) => !o && onClose()}>
      <SheetContent
        side="right"
        className="access-picker-panel w-full p-0 sm:max-w-[420px]"
        data-testid={`${testIdPrefix}-panel`}
      >
        <div className="ap-panel-head">
          <SheetTitle className="ap-panel-title">{title}</SheetTitle>
          <SheetDescription className="ap-panel-subtitle">{subtitle}</SheetDescription>
        </div>

        <div className="ap-panel-filters">
          <input
            className="ap-panel-search"
            placeholder={`Search ${noun}s…`}
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            data-testid={`${testIdPrefix}-panel-search`}
            autoFocus
          />
          {hasTypes && (
            <select
              className="ap-panel-type"
              value={typeFilter}
              onChange={(e) => setTypeFilter(e.target.value as 'all' | 'local' | 'api')}
              data-testid={`${testIdPrefix}-panel-type`}
            >
              <option value="all">All</option>
              <option value="local">Local</option>
              <option value="api">API</option>
            </select>
          )}
        </div>

        <div className="ap-panel-body">
          {groups.length === 0 && (
            <div className="ap-panel-empty" data-testid={`${testIdPrefix}-panel-empty`}>
              No {noun}s match “{search}”
            </div>
          )}
          {groups.map((g) => (
            <div key={g.label}>
              <div className="ap-panel-group">{g.label}</div>
              {g.items.map((item) => {
                const checked = selectedIds.includes(item.id);
                return (
                  <button
                    type="button"
                    key={item.id}
                    className={`ap-panel-row${checked ? ' is-added' : ''}`}
                    onClick={() => onToggle(item.id)}
                    data-testid={`${testIdPrefix}-panel-item-${item.id}`}
                    aria-pressed={checked}
                  >
                    <span className="ap-panel-check">{checked && <Check strokeWidth={3} />}</span>
                    <span className="ap-panel-name">{item.label}</span>
                    <span className="ap-panel-tags">
                      {item.meta && <span className="ap-panel-meta">{item.meta}</span>}
                      {item.type === 'local' && <span className="ap-type ap-type-local">local</span>}
                      {item.type === 'api' && <span className="ap-type ap-type-api">api</span>}
                    </span>
                  </button>
                );
              })}
            </div>
          ))}
        </div>

        <div className="ap-panel-foot">
          <span className="ap-panel-count" data-testid={`${testIdPrefix}-panel-count`}>
            {selectedIds.length} {noun}
            {selectedIds.length !== 1 ? 's' : ''} selected
          </span>
          <Button type="button" size="sm" onClick={onClose} data-testid={`${testIdPrefix}-panel-done`}>
            Done
          </Button>
        </div>
      </SheetContent>
    </Sheet>
  );
}
