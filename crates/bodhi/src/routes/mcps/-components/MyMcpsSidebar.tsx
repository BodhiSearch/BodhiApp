import { ShellIcon } from '@/components/shell';
import '@/routes/models/-components/models.css';

/**
 * Faceted sidebar for My MCPs. The list is server-centric (registered servers); the Scope facet narrows
 * it: `all` ("Configured") shows every registered server, `mine` ("Connected") shows only servers the
 * user has at least one instance on. Single-select, tri-state (re-selecting clears).
 */

export type ScopeFacet = 'all' | 'mine';

export interface MyMcpsFacetsState {
  scope?: ScopeFacet;
}

export function hasActiveMyMcpsFacets(f: MyMcpsFacetsState): boolean {
  return Boolean(f.scope);
}

const SCOPE_OPTIONS: { value: ScopeFacet; label: string }[] = [
  { value: 'all', label: 'Configured' },
  { value: 'mine', label: 'Connected' },
];

interface SidebarProps {
  facets: MyMcpsFacetsState;
  onFacetsChange: (next: MyMcpsFacetsState) => void;
}

export function MyMcpsSidebar({ facets, onFacetsChange }: SidebarProps) {
  return (
    <div className="m-facets" data-testid="my-mcps-facets">
      <div className="m-facet-group">
        <div className="m-facet-label">
          <ShellIcon name="filter" size={13} />
          <span>Scope</span>
        </div>
        <div className="m-facet-pills">
          {SCOPE_OPTIONS.map(({ value, label }) => {
            const active = facets.scope === value;
            return (
              <button
                key={value}
                type="button"
                className={`m-facet-pill${active ? ' active' : ''}`}
                aria-pressed={active}
                data-testid={`my-mcps-scope-${value}`}
                onClick={() => onFacetsChange({ ...facets, scope: active ? undefined : value })}
              >
                {label}
              </button>
            );
          })}
        </div>
      </div>
    </div>
  );
}
