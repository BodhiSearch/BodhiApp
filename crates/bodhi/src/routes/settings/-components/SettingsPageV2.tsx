import { useCallback, useMemo, useRef, useState } from 'react';

import { SettingInfo } from '@bodhiapp/ts-client';

import { ShellFilterTabs, ShellIcon, useCollapsibleSearch, useShellChrome } from '@/components/shell';
import '@/components/shell/api-keys.css';
import '@/components/shell/list.css';
import '@/components/shell/settings.css';
import { ErrorPage } from '@/components/ui/ErrorPage';
import { Skeleton } from '@/components/ui/skeleton';
import { useListSettings } from '@/hooks/settings';
import { useViewTransition } from '@/hooks/useViewTransition';
import { extractErrorMessage } from '@/lib/errorUtils';
import { SettingRailHeader } from '@/routes/settings/-components/SettingRailHeader';
import { SettingRailPanel } from '@/routes/settings/-components/SettingRailPanel';
import { SettingRow } from '@/routes/settings/-components/SettingRow';
import { EDITABLE_KEYS, isEnv, isModified } from '@/routes/settings/-components/settingsFormat';
import { SettingsGroupNav } from '@/routes/settings/-components/SettingsGroupNav';
import { GroupMeta, SettingsConfigV2 } from '@/routes/settings/-components/settingsTypes';

export { EDITABLE_KEYS };
export type { GroupMeta, SettingGroupConfig, SettingsConfigV2 } from '@/routes/settings/-components/settingsTypes';

const SETTINGS_BREADCRUMB = [{ label: 'Bodhi' }, { label: 'App Settings', current: true }];

type SettingFilter = 'all' | 'modified' | 'env';

const FILTER_TABS: { id: SettingFilter; label: string }[] = [
  { id: 'all', label: 'All' },
  { id: 'modified', label: 'Modified' },
  { id: 'env', label: 'Env' },
];

export function SettingsPageV2({ config: staticConfig }: { config: SettingsConfigV2 }) {
  const { data: settings, isLoading, error } = useListSettings();

  // Merge the static groups with the data-driven dynamic groups (variant args + ungrouped),
  // so the V2 screen shows every real setting — not just the prototype's four groups.
  const config = useMemo<SettingsConfigV2>(() => {
    if (!settings) return staticConfig;
    const known = new Set(staticConfig.groups.flatMap((g) => g.keys));
    // Dedup keys: the backend can return a setting more than once (e.g. BODHI_DEPLOYMENT),
    // which would otherwise produce duplicate React keys in the dynamic groups.
    const variantArgs = [
      ...new Set(
        settings
          .filter((s) => s.key.startsWith('BODHI_LLAMACPP_ARGS_') && s.key !== 'BODHI_LLAMACPP_ARGS')
          .map((s) => s.key)
      ),
    ];
    const variantSet = new Set(variantArgs);
    const ungrouped = [
      ...new Set(settings.filter((s) => !known.has(s.key) && !variantSet.has(s.key)).map((s) => s.key)),
    ];

    const groups = [...staticConfig.groups];
    if (variantArgs.length) {
      groups.push({
        id: 'variant-args',
        name: 'Server Arguments by Variant',
        label: 'Variant Args',
        icon: 'terminal',
        keys: variantArgs,
        descriptions: Object.fromEntries(
          variantArgs.map((k) => [
            k,
            `Arguments specific to the ${k.replace('BODHI_LLAMACPP_ARGS_', '').toLowerCase()} variant.`,
          ])
        ),
      });
    }
    if (ungrouped.length) {
      groups.push({
        id: 'misc',
        name: 'Miscellaneous Settings',
        label: 'Miscellaneous',
        icon: 'settings',
        keys: ungrouped,
        descriptions: {},
      });
    }
    return { groups };
  }, [settings, staticConfig]);

  const [filter, setFilter] = useState<SettingFilter>('all');
  const [search, setSearch] = useState('');
  const [selectedKey, setSelectedKey] = useState<string | null>(null);
  const [activeSection, setActiveSection] = useState<string>(config.groups[0]?.id ?? '');

  const scrollRef = useRef<HTMLDivElement>(null);
  const sectionRefs = useRef<Record<string, HTMLDivElement | null>>({});
  const rafPending = useRef(false);

  const withViewTransition = useViewTransition();
  const select = useCallback(
    (key: string | null) => withViewTransition(() => setSelectedKey(key)),
    [withViewTransition]
  );

  const byKey = useMemo(() => {
    const m = new Map<string, SettingInfo>();
    (settings ?? []).forEach((s) => m.set(s.key, s));
    return m;
  }, [settings]);

  const q = search.trim().toLowerCase();
  const matches = useCallback(
    (s: SettingInfo, description?: string) => {
      if (filter === 'modified' && !isModified(s)) return false;
      if (filter === 'env' && !isEnv(s)) return false;
      if (!q) return true;
      const hay = `${s.key} ${description ?? ''} ${String(s.current_value ?? '')}`.toLowerCase();
      return hay.includes(q);
    },
    [filter, q]
  );

  // Per-group counts (total settings present in each group).
  const counts = useMemo(() => {
    const c: Record<string, number> = {};
    config.groups.forEach((g) => {
      c[g.id] = g.keys.filter((k) => byKey.has(k)).length;
    });
    return c;
  }, [config.groups, byKey]);

  // Filter-tab counts.
  const filterCounts = useMemo(() => {
    const present = (settings ?? []).filter((s) => config.groups.some((g) => g.keys.includes(s.key)));
    return {
      all: present.length,
      modified: present.filter(isModified).length,
      env: present.filter(isEnv).length,
    };
  }, [settings, config.groups]);

  const filterTabs = useMemo(() => FILTER_TABS.map((t) => ({ ...t, count: filterCounts[t.id] })), [filterCounts]);

  const searchNode = useCollapsibleSearch({
    value: search,
    onChange: setSearch,
    placeholder: 'Search settings…',
    toggleTestId: 'settings-search-toggle',
    closeTestId: 'settings-search-close',
  });

  // Visible rows per group (after filter + search).
  const visibleGroups = useMemo(
    () =>
      config.groups
        .map((g) => ({
          ...g,
          rows: g.keys
            .map((k) => byKey.get(k))
            .filter((s): s is SettingInfo => !!s && matches(s, g.descriptions[s.key])),
        }))
        .filter((g) => g.rows.length > 0),
    [config.groups, byKey, matches]
  );

  const anyVisible = visibleGroups.length > 0;

  const groupMeta: GroupMeta[] = useMemo(
    () => config.groups.map((g) => ({ id: g.id, name: g.label, icon: g.icon })),
    [config.groups]
  );

  const onNavigate = useCallback((id: string) => {
    setActiveSection(id);
    const el = sectionRefs.current[id];
    const scroll = scrollRef.current;
    if (el && scroll) scroll.scrollTo({ top: el.offsetTop - 2, behavior: 'smooth' });
  }, []);

  // Scroll-spy: active group follows scroll (rAF-throttled, change-guarded).
  const onScroll = useCallback(() => {
    if (rafPending.current) return;
    rafPending.current = true;
    requestAnimationFrame(() => {
      rafPending.current = false;
      const scroll = scrollRef.current;
      if (!scroll) return;
      let next: string | null = null;
      for (const g of config.groups) {
        const el = sectionRefs.current[g.id];
        if (el && el.offsetTop - scroll.scrollTop <= 60) next = g.id;
      }
      if (next) setActiveSection((prev) => (prev === next ? prev : next));
    });
  }, [config.groups]);

  const selected = selectedKey ? (byKey.get(selectedKey) ?? null) : null;
  const selectedGroup = useMemo(
    () => (selected ? config.groups.find((g) => g.keys.includes(selected.key)) : undefined),
    [selected, config.groups]
  );

  const sidebar = useMemo(
    () => <SettingsGroupNav groups={groupMeta} counts={counts} active={activeSection} onNavigate={onNavigate} />,
    [groupMeta, counts, activeSection, onNavigate]
  );

  const railHeader = useMemo(
    () =>
      selected ? (
        <SettingRailHeader setting={selected} groupName={selectedGroup?.name ?? ''} onClose={() => select(null)} />
      ) : null,
    [selected, selectedGroup, select]
  );

  const rail = useMemo(
    () =>
      selected ? <SettingRailPanel setting={selected} description={selectedGroup?.descriptions[selected.key]} /> : null,
    [selected, selectedGroup]
  );

  useShellChrome({ breadcrumb: SETTINGS_BREADCRUMB, sidebar, rail, railHeader, railDefaultOpen: false });

  if (error) {
    const message = extractErrorMessage(error, 'An unexpected error occurred');
    return <ErrorPage message={message} />;
  }

  return (
    <div
      className="settings-screen l-page"
      data-testid="settings-page"
      data-pagestatus={isLoading ? 'loading' : 'ready'}
    >
      <div className="l-controls">
        {searchNode.row}
        <div className="l-toolbar">
          <ShellFilterTabs
            tabs={filterTabs}
            value={filter}
            onChange={setFilter}
            label="Filter settings"
            testIdPrefix="settings-filter"
            loading={isLoading}
          />
          <div className="l-tb-actions">{searchNode.toggle}</div>
        </div>
      </div>

      <div className="l-scroll" ref={scrollRef} onScroll={onScroll} data-testid="settings-list">
        {isLoading ? (
          <div style={{ padding: 16 }} data-testid="settings-skeleton-container">
            {Array.from({ length: 5 }).map((_, i) => (
              <Skeleton key={i} className="h-12 w-full mb-3" data-testid="settings-skeleton" />
            ))}
          </div>
        ) : !anyVisible ? (
          <div className="empty-state" data-testid="no-settings">
            <div className="empty-icon">
              <ShellIcon name="search-x" size={28} />
            </div>
            <div className="empty-title">No settings match</div>
            <div className="empty-sub">
              {search ? 'Try a different search term.' : 'No settings match this filter.'}
            </div>
          </div>
        ) : (
          visibleGroups.map((g) => (
            <div
              className="section-group"
              key={g.id}
              ref={(el) => {
                sectionRefs.current[g.id] = el;
              }}
            >
              <div className="section-header">
                <ShellIcon name={g.icon} size={14} />
                <span className="section-header-name">{g.name}</span>
              </div>
              {g.rows.map((s) => (
                <SettingRow
                  key={s.key}
                  setting={s}
                  description={g.descriptions[s.key]}
                  active={s.key === selectedKey}
                  query={q}
                  onSelect={() => select(s.key)}
                />
              ))}
            </div>
          ))
        )}
      </div>
    </div>
  );
}
