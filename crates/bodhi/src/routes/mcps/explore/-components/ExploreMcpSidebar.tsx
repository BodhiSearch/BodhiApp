import type { McpAuthType, McpFacets } from '@bodhiapp/reference-api-types';

import { ShellIcon } from '@/components/shell';
import '@/routes/models/-components/models.css';

/**
 * Faceted sidebar for Explore · MCP Servers. Controlled — selections drive the parent's
 * `ListMcpServersQuery`. Unlike the API-Models sidebar (fixed enum + availability mask), the MCP
 * facet OPTIONS are themselves DATA-DRIVEN from the response `facets` arrays: Category and Auth render
 * exactly the values the catalog currently has. In v1 `facets.category` is empty (the group hides) and
 * `facets.auth` is `['http']` (a single chip), so the rails light up automatically as enrichment lands
 * — no hard-coded taxonomy. `Verified` is the one boolean-backed pill.
 */

export type InstalledFacet = 'installed' | 'not_installed';

export interface McpFacetsState {
  category?: string[];
  auth?: McpAuthType[];
  verified?: boolean;
  installed?: InstalledFacet;
}

export function hasActiveMcpFacets(f: McpFacetsState): boolean {
  return Boolean(f.category?.length || f.auth?.length || f.verified || f.installed);
}

function toggle<T>(list: T[] | undefined, value: T): T[] | undefined {
  const set = new Set(list ?? []);
  if (set.has(value)) set.delete(value);
  else set.add(value);
  const next = [...set];
  return next.length ? next : undefined;
}

interface SidebarProps {
  facets: McpFacetsState;
  facetValues: McpFacets | undefined;
  onFacetsChange: (next: McpFacetsState) => void;
}

export function ExploreMcpSidebar({ facets, facetValues, onFacetsChange }: SidebarProps) {
  const categoryValues = facetValues?.category ?? [];
  const authValues = (facetValues?.auth ?? []) as McpAuthType[];

  return (
    <div className="m-facets" data-testid="cat-mcp-facets">
      <FacetGroup icon="download" title="Availability">
        <Pills>
          {(['installed', 'not_installed'] as InstalledFacet[]).map((v) => (
            <FacetPill
              key={v}
              label={v === 'installed' ? 'Installed' : 'Not installed'}
              active={facets.installed === v}
              testId={`cat-mcp-installed-${v}`}
              // Tri-state: re-selecting the active value clears it.
              onToggle={() => onFacetsChange({ ...facets, installed: facets.installed === v ? undefined : v })}
            />
          ))}
        </Pills>
      </FacetGroup>

      {categoryValues.length > 0 && (
        <FacetGroup icon="shapes" title="Category">
          <Pills>
            {categoryValues.map((c) => (
              <FacetPill
                key={c}
                label={c}
                active={(facets.category ?? []).includes(c)}
                testId={`cat-mcp-category-${c}`}
                onToggle={() => onFacetsChange({ ...facets, category: toggle(facets.category, c) })}
              />
            ))}
          </Pills>
        </FacetGroup>
      )}

      {authValues.length > 0 && (
        <FacetGroup icon="lock" title="Auth">
          <Pills>
            {authValues.map((a) => (
              <FacetPill
                key={a}
                label={a}
                active={(facets.auth ?? []).includes(a)}
                testId={`cat-mcp-auth-${a}`}
                onToggle={() => onFacetsChange({ ...facets, auth: toggle(facets.auth, a) })}
              />
            ))}
          </Pills>
        </FacetGroup>
      )}

      <FacetGroup icon="badge-check" title="Publisher">
        <Pills>
          <FacetPill
            label="Verified"
            active={!!facets.verified}
            testId="cat-mcp-verified"
            onToggle={() => onFacetsChange({ ...facets, verified: facets.verified ? undefined : true })}
          />
        </Pills>
      </FacetGroup>
    </div>
  );
}

function Pills({ children }: { children: React.ReactNode }) {
  return <div className="m-facet-pills">{children}</div>;
}

function FacetPill({
  label,
  active,
  testId,
  onToggle,
}: {
  label: string;
  active: boolean;
  testId: string;
  onToggle: () => void;
}) {
  return (
    <button
      type="button"
      className={`m-facet-pill${active ? ' active' : ''}`}
      aria-pressed={active}
      onClick={onToggle}
      data-testid={testId}
    >
      {label}
    </button>
  );
}

function FacetGroup({ icon, title, children }: { icon: string; title: string; children: React.ReactNode }) {
  return (
    <div className="m-facet-group">
      <div className="m-facet-label">
        <ShellIcon name={icon} size={13} />
        <span>{title}</span>
      </div>
      {children}
    </div>
  );
}
