import type { ApiFormatHint, Capability, FacetBucket, ListProvidersQuery } from '@bodhiapp/reference-api-types';

import { ShellIcon } from '@/components/shell';
import { CAP_LABELS } from '@/routes/models/explore/-shared/catalog-format';
import '@/routes/models/-components/models.css';

/**
 * Faceted sidebar for Explore · API Providers. Controlled — selections drive the parent's
 * `ListProvidersQuery`. Facet OPTIONS + counts come from `ProviderListResponse.facets` (recomputed
 * per query). Zero-count options render disabled (not hidden) so a selected option can be cleared.
 */

export type ProviderPricing = NonNullable<ListProvidersQuery['pricing']>;

export interface ProviderFacets {
  capability?: Capability[];
  api_format?: ApiFormatHint[];
  pricing_max?: number;
  pricing?: ProviderPricing;
}

export function hasActiveProviderFacets(f: ProviderFacets): boolean {
  return Boolean(f.capability?.length || f.api_format?.length || f.pricing_max != null || f.pricing);
}

/** Build the API query params contributed by the provider facets (omitting defaults/empties). */
export function providerFacetsToQuery(f: ProviderFacets) {
  return {
    ...(f.capability?.length ? { capability: f.capability } : {}),
    ...(f.api_format?.length ? { api_format: f.api_format } : {}),
    ...(f.pricing_max != null ? { pricing_max: f.pricing_max } : {}),
    ...(f.pricing ? { pricing: f.pricing } : {}),
  };
}

const API_FORMAT_LABELS: Partial<Record<ApiFormatHint, string>> = {
  openai: 'OpenAI',
  openai_responses: 'OpenAI Responses',
  anthropic: 'Anthropic',
  anthropic_oauth: 'Anthropic OAuth',
  gemini: 'Gemini',
  other: 'Other',
};

function toggle<T>(list: T[] | undefined, value: T): T[] | undefined {
  const set = new Set(list ?? []);
  if (set.has(value)) set.delete(value);
  else set.add(value);
  const next = [...set];
  return next.length ? next : undefined;
}

interface SidebarProps {
  facets: ProviderFacets;
  capabilityCounts: FacetBucket;
  apiFormatCounts: FacetBucket;
  onFacetsChange: (next: ProviderFacets) => void;
  onClearAll: () => void;
}

export function ExploreProvidersSidebar({
  facets,
  capabilityCounts,
  apiFormatCounts,
  onFacetsChange,
  onClearAll,
}: SidebarProps) {
  return (
    <div className="m-facets" data-testid="cat-prov-facets">
      {hasActiveProviderFacets(facets) && (
        <button type="button" className="ld-clear-all" onClick={onClearAll} data-testid="cat-prov-clear-all">
          <ShellIcon name="x" size={11} /> Clear all filters
        </button>
      )}

      <FacetGroup icon="sparkles" title="Capability">
        <div className="m-facet-pills">
          {(Object.keys(CAP_LABELS) as Capability[]).map((c) => (
            <FacetPill
              key={c}
              label={CAP_LABELS[c]}
              count={capabilityCounts[c]}
              active={(facets.capability ?? []).includes(c)}
              testId={`cat-prov-cap-${c}`}
              onToggle={() => onFacetsChange({ ...facets, capability: toggle(facets.capability, c) })}
            />
          ))}
        </div>
      </FacetGroup>

      <FacetGroup icon="plug-zap" title="API format">
        <div className="m-facet-pills">
          {(Object.keys(API_FORMAT_LABELS) as ApiFormatHint[]).map((f) => (
            <FacetPill
              key={f}
              label={API_FORMAT_LABELS[f] ?? f}
              count={apiFormatCounts[f]}
              active={(facets.api_format ?? []).includes(f)}
              testId={`cat-prov-fmt-${f}`}
              onToggle={() => onFacetsChange({ ...facets, api_format: toggle(facets.api_format, f) })}
            />
          ))}
        </div>
      </FacetGroup>
    </div>
  );
}

/** A multi-select facet pill with a count. Zero-count + inactive → disabled (can't select an empty
 *  bucket), but a selected pill stays enabled so it can be cleared even when its count drops to 0. */
function FacetPill({
  label,
  count,
  active,
  testId,
  onToggle,
}: {
  label: string;
  count: number | undefined;
  active: boolean;
  testId: string;
  onToggle: () => void;
}) {
  const n = count ?? 0;
  const disabled = !active && n === 0;
  return (
    <button
      type="button"
      className={`m-facet-pill${active ? ' active' : ''}`}
      aria-pressed={active}
      disabled={disabled}
      onClick={onToggle}
      data-testid={testId}
    >
      {label}
      {n > 0 && <span className="cat-facet-count">{n}</span>}
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
