import { useCallback, useMemo, useRef, useState } from 'react';

import { SettingInfo } from '@bodhiapp/ts-client';

import { LinkRow, ShellFilterTabs, ShellIcon, useCollapsibleSearch, useShellChrome } from '@/components/shell';
import '@/components/shell/api-keys.css';
import '@/components/shell/list.css';
import '@/components/shell/settings.css';
import { ErrorPage } from '@/components/ui/ErrorPage';
import { Skeleton } from '@/components/ui/skeleton';
import { useDeleteSetting, useListSettings, useUpdateSetting } from '@/hooks/settings';
import { useToastMessages } from '@/hooks/use-toast-messages';
import { useViewTransition } from '@/hooks/useViewTransition';
import { parseSettingValue, SettingValueInput, settingValueHint } from '@/routes/settings/-shared/settingInput';

/** Backend `EDIT_SETTINGS_ALLOWED` (routes_settings.rs) — the only editable settings. */
const EDITABLE_KEYS = new Set(['BODHI_EXEC_VARIANT', 'BODHI_KEEP_ALIVE_SECS']);

const SETTINGS_BREADCRUMB = [{ label: 'Bodhi' }, { label: 'App Settings', current: true }];

type SettingFilter = 'all' | 'modified' | 'env';

const FILTER_TABS: { id: SettingFilter; label: string }[] = [
  { id: 'all', label: 'All' },
  { id: 'modified', label: 'Modified' },
  { id: 'env', label: 'Env' },
];

/** Group metadata: display name + kebab lucide icon. A setting's group is its config group. */
interface GroupMeta {
  id: string;
  name: string;
  icon: string;
}

export interface SettingGroupConfig {
  /** stable group id (matches the SETTINGS_CONFIG key) */
  id: string;
  /** uppercase section header name + sidebar label */
  name: string;
  /** short sidebar label */
  label: string;
  icon: string;
  /** keys belonging to this group, in display order */
  keys: string[];
  /** per-key description from the static config (optional) */
  descriptions: Record<string, string | undefined>;
}

export interface SettingsConfigV2 {
  groups: SettingGroupConfig[];
}

const isModified = (s: SettingInfo) => String(s.current_value) !== String(s.default_value);
const isEnv = (s: SettingInfo) => s.source === 'environment';

function sourceBadgeClass(source: string): string {
  switch (source) {
    case 'environment':
      return 's-badge-env';
    case 'command_line':
      return 's-badge-cmdline';
    case 'system':
      return 's-badge-system';
    case 'default':
      return 's-badge-default';
    default:
      return 's-badge-modified';
  }
}

function highlight(text: string, query: string) {
  if (!query) return text;
  const idx = text.toLowerCase().indexOf(query);
  if (idx < 0) return text;
  return (
    <>
      {text.slice(0, idx)}
      <mark>{text.slice(idx, idx + query.length)}</mark>
      {text.slice(idx + query.length)}
    </>
  );
}

interface SettingRowProps {
  setting: SettingInfo;
  description?: string;
  active: boolean;
  query: string;
  onSelect: () => void;
}

function SettingRow({ setting, description, active, query, onSelect }: SettingRowProps) {
  const editable = EDITABLE_KEYS.has(setting.key);
  const atDefault = !isModified(setting);
  const hideValue = setting.source === 'system';
  const valueText = hideValue ? '—' : String(setting.current_value ?? '—');

  return (
    <div
      className={`setting-row${isModified(setting) ? ' modified' : ''}${active ? ' active' : ''}`}
      onClick={onSelect}
      data-testid={`setting-row-${setting.key}`}
    >
      <LinkRow onActivate={onSelect} label={`Open setting ${setting.key}`} />
      <div className="row-key">
        <span className="key-name" data-testid={`setting-key-${setting.key}`}>
          {highlight(setting.key, query)}
        </span>
      </div>
      <div className={`row-value${atDefault ? ' at-default' : ''}`} data-testid={`setting-value-${setting.key}`}>
        {valueText}
      </div>
      <div className="row-actions">
        <span className={`s-badge ${sourceBadgeClass(setting.source)}`} data-testid={`setting-source-${setting.key}`}>
          {setting.source}
        </span>
        <span className="type-badge">{setting.metadata.type}</span>
        {editable && (
          <button
            className="row-edit-btn"
            onClick={(e) => {
              e.stopPropagation();
              onSelect();
            }}
            data-testid={`setting-edit-${setting.key}`}
            aria-label={`Edit ${setting.key}`}
          >
            <ShellIcon name="pencil" size={12} />
          </button>
        )}
      </div>
      {description && <div className="row-desc">{description}</div>}
    </div>
  );
}

function SettingRailHeader({
  setting,
  groupName,
  onClose,
}: {
  setting: SettingInfo;
  groupName: string;
  onClose: () => void;
}) {
  return (
    <div className="dp-head">
      <div className="dp-head-icon" style={{ background: 'var(--c-indigo-bg)', color: 'var(--c-indigo-text)' }}>
        <ShellIcon name="sliders-horizontal" size={15} />
      </div>
      <div className="dp-head-body">
        <div className="dp-head-title mono">{setting.key}</div>
        <div className="dp-head-sub">{groupName}</div>
      </div>
      <button className="dp-close" onClick={onClose} title="Close" data-testid="setting-detail-close">
        <ShellIcon name="x" size={15} />
      </button>
    </div>
  );
}

interface SettingRailPanelProps {
  setting: SettingInfo;
  description?: string;
}

function SettingRailPanel({ setting, description }: SettingRailPanelProps) {
  const editable = EDITABLE_KEYS.has(setting.key);
  const hideValue = setting.source === 'system';
  const { showSuccess, showError } = useToastMessages();

  const [draft, setDraft] = useState<string>(() => String(setting.current_value ?? ''));
  // Reset the draft when a different setting is selected.
  const keyRef = useRef(setting.key);
  if (keyRef.current !== setting.key) {
    keyRef.current = setting.key;
    setDraft(String(setting.current_value ?? ''));
  }

  const updateSetting = useUpdateSetting({
    onSuccess: () => showSuccess('Success', `Setting ${setting.key} updated`),
    onError: (message) => showError('Error', message),
  });
  const deleteSetting = useDeleteSetting({
    onSuccess: () => showSuccess('Reset', `Setting ${setting.key} reset to default`),
    onError: (message) => showError('Error', message),
  });

  const dirty = draft !== String(setting.current_value ?? '');
  const busy = updateSetting.isPending || deleteSetting.isPending;

  const onSave = () => {
    const parsed = parseSettingValue(setting.metadata, draft);
    if (!parsed.ok) {
      showError('Error', parsed.error);
      return;
    }
    updateSetting.mutate({ key: setting.key, value: parsed.value });
  };

  return (
    <div className="dp-panel settings-screen-rail" data-testid={`setting-detail-${setting.key}`}>
      <div className="dp-status-row">
        <span className={`s-badge ${sourceBadgeClass(setting.source)}`}>{setting.source}</span>
        <span className="type-badge">{setting.metadata.type}</span>
      </div>

      <div className="dp-body">
        {description && (
          <div className="dp-section">
            <p className="dp-desc">{description}</p>
          </div>
        )}

        <div className="dp-section">
          <div className="dp-sec-lbl">Values</div>
          <div className="dp-rows">
            {!hideValue && (
              <div className="dp-row">
                <span className="dp-row-k">
                  <ShellIcon name="circle-dot" size={13} /> Current
                </span>
                <span className="dp-row-v mono">{String(setting.current_value ?? '—')}</span>
              </div>
            )}
            <div className="dp-row">
              <span className="dp-row-k">
                <ShellIcon name="rotate-ccw" size={13} /> Default
              </span>
              <span className="dp-row-v mono">{String(setting.default_value ?? '—')}</span>
            </div>
          </div>
        </div>

        {editable ? (
          <div className="dp-section">
            <div className="dp-sec-lbl">New value</div>
            <div className="dp-field">
              <SettingValueInput setting={setting} value={draft} onChange={setDraft} testId="setting-new-value" />
              <span className="dp-field-hint">{settingValueHint(setting.metadata)}</span>
            </div>
          </div>
        ) : (
          <div className="dp-section">
            <div className="dp-readonly-note" data-testid="setting-readonly-note">
              <ShellIcon name="lock" size={14} />
              <div>This setting is read-only (set via {setting.source}).</div>
            </div>
          </div>
        )}
      </div>

      {editable && (
        <div className="dp-foot">
          <button
            className="dp-btn dp-btn-accent"
            disabled={!dirty || busy}
            onClick={onSave}
            data-testid="setting-save"
          >
            <ShellIcon name="check" size={14} /> {dirty ? 'Save changes' : 'Saved'}
          </button>
          <div className="dp-foot-row">
            <button
              className="dp-btn dp-btn-outline"
              onClick={() => setDraft(String(setting.current_value ?? ''))}
              data-testid="setting-cancel"
            >
              Cancel
            </button>
            {String(setting.current_value) !== String(setting.default_value) && (
              <button
                className="dp-btn dp-btn-outline"
                disabled={busy}
                onClick={() => deleteSetting.mutate({ key: setting.key })}
                data-testid="setting-reset"
              >
                <ShellIcon name="rotate-ccw" size={13} /> Reset
              </button>
            )}
          </div>
        </div>
      )}
    </div>
  );
}

interface SettingsGroupNavProps {
  groups: GroupMeta[];
  counts: Record<string, number>;
  active: string;
  onNavigate: (id: string) => void;
}

function SettingsGroupNav({ groups, counts, active, onNavigate }: SettingsGroupNavProps) {
  return (
    <div className="settings-screen-nav" data-testid="settings-group-nav">
      <div className="snav-label">Settings Groups</div>
      {groups.map((g) => (
        <button
          key={g.id}
          className={`snav-item${active === g.id ? ' active' : ''}`}
          onClick={() => onNavigate(g.id)}
          data-testid={`settings-group-${g.id}`}
        >
          <ShellIcon name={g.icon} size={13} />
          {g.name}
          <span className="snav-count">{counts[g.id] || 0}</span>
        </button>
      ))}
      <div className="snav-legend">
        <div className="snav-legend-title">Legend</div>
        <div className="snav-legend-rows">
          <div className="snav-legend-row">
            <span className="s-badge s-badge-default">default</span> At default value
          </div>
          <div className="snav-legend-row">
            <span className="s-badge s-badge-modified">modified</span> Overridden
          </div>
          <div className="snav-legend-row">
            <span className="s-badge s-badge-env">env</span> From env var
          </div>
          <div className="snav-legend-row">
            <span className="s-badge s-badge-cmdline">cmd</span> Command line
          </div>
        </div>
      </div>
    </div>
  );
}

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
    const message = error.response?.data?.error?.message || error.message;
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
