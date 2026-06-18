/**
 * Reusable single-select filter tabs for list pages — the themed pill group
 * (l-cat) used in the list toolbar (App Tokens, Access Requests, …). Each tab
 * shows a label and an optional count badge; the active one is filled.
 *
 * First introduced on the App Tokens screen; reused across every list screen.
 */
export interface ShellFilterTab<T extends string = string> {
  id: T;
  label: string;
  count?: number;
}

export interface ShellFilterTabsProps<T extends string = string> {
  tabs: ShellFilterTab<T>[];
  value: T;
  onChange: (id: T) => void;
  /** aria-label for the tablist group */
  label?: string;
  /** testid prefix → `${testIdPrefix}-${tab.id}` per tab */
  testIdPrefix?: string;
  /** while true, badges show a shimmer placeholder instead of (possibly zero) counts */
  loading?: boolean;
}

export function ShellFilterTabs<T extends string = string>({
  tabs,
  value,
  onChange,
  label = 'Filter',
  testIdPrefix,
  loading = false,
}: ShellFilterTabsProps<T>) {
  return (
    <div className="l-cats" role="tablist" aria-label={label}>
      {tabs.map((tab) => (
        <button
          key={tab.id}
          role="tab"
          aria-selected={value === tab.id}
          className={'l-cat' + (value === tab.id ? ' on' : '')}
          onClick={() => onChange(tab.id)}
          data-testid={testIdPrefix ? `${testIdPrefix}-${tab.id}` : undefined}
        >
          {tab.label}
          {loading ? (
            <span className="l-cat-badge l-cat-badge--loading" aria-label="Loading count" />
          ) : (
            tab.count != null && <span className="l-cat-badge">{tab.count}</span>
          )}
        </button>
      ))}
    </div>
  );
}
