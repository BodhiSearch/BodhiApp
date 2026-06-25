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
  pricing?: ProviderPricing;
  is_lab?: boolean;
}

export function hasActiveProviderFacets(f: ProviderFacets): boolean {
  return Boolean(f.capability?.length || f.api_format?.length || f.pricing || f.is_lab);
}

/** Build the API query params contributed by the provider facets (omitting defaults/empties). */
export function providerFacetsToQuery(f: ProviderFacets) {
  return {
    ...(f.capability?.length ? { capability: f.capability } : {}),
    ...(f.api_format?.length ? { api_format: f.api_format } : {}),
    ...(f.pricing ? { pricing: f.pricing } : {}),
    ...(f.is_lab ? { is_lab: 'true' as const } : {}),
  };
}

// Display labels only — the OPTIONS rendered are driven by the API's api_format facet bucket, so
// synthetic/frontend-only formats (e.g. openai_responses, anthropic_oauth) never appear here.
const API_FORMAT_LABELS: Partial<Record<ApiFormatHint, string>> = {
  openai: 'OpenAI',
  anthropic: 'Anthropic',
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
}

export function ExploreProvidersSidebar({ facets, capabilityCounts, apiFormatCounts, onFacetsChange }: SidebarProps) {
  // OPTIONS come from the API's api_format bucket, so only formats the search backend actually
  // returns are filterable; a stored/selected value is kept so it can still be cleared. The
  // `openai_responses` variant is intentionally excluded as a filter option.
  const apiFormatKeys = Array.from(
    new Set<ApiFormatHint>([...(Object.keys(apiFormatCounts) as ApiFormatHint[]), ...(facets.api_format ?? [])])
  ).filter((f) => f !== 'openai_responses');

  return (
    <div className="m-facets" data-testid="cat-prov-facets">
      <FacetGroup icon="compass" title="Browse">
        <div className="m-facet-pills">
          <FacetPill
            label="Labs only"
            count={undefined}
            active={Boolean(facets.is_lab)}
            testId="cat-prov-labs"
            onToggle={() => onFacetsChange({ ...facets, is_lab: facets.is_lab ? undefined : true })}
          />
        </div>
      </FacetGroup>

      <FacetGroup icon="sparkles" title="Capability">
        <div className="m-facet-pills">
          {(Object.keys(CAP_LABELS) as Capability[]).map((c) => (
            <FacetPill
              key={c}
              label={CAP_LABELS[c]}
              count={capabilityCounts[c] ?? 0}
              active={(facets.capability ?? []).includes(c)}
              testId={`cat-prov-cap-${c}`}
              onToggle={() => onFacetsChange({ ...facets, capability: toggle(facets.capability, c) })}
            />
          ))}
        </div>
      </FacetGroup>

      <FacetGroup icon="plug-zap" title="API format">
        <div className="m-facet-pills">
          {apiFormatKeys.map((f) => (
            <FacetPill
              key={f}
              label={API_FORMAT_LABELS[f] ?? f}
              count={apiFormatCounts[f] ?? 0}
              active={(facets.api_format ?? []).includes(f)}
              testId={`cat-prov-fmt-${f}`}
              onToggle={() => onFacetsChange({ ...facets, api_format: toggle(facets.api_format, f) })}
            />
          ))}
        </div>
      </FacetGroup>

      <FacetGroup icon="dollar-sign" title="Pricing" note="cheapest model">
        <div className="m-facet-pills">
          {(['free', 'paid'] as ProviderPricing[]).map((p) => (
            <FacetPill
              key={p}
              label={p === 'free' ? 'Free' : 'Paid'}
              count={undefined}
              active={facets.pricing === p}
              testId={`cat-prov-pricing-${p}`}
              onToggle={() => onFacetsChange({ ...facets, pricing: facets.pricing === p ? undefined : p })}
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
  // Count-gating only applies to real facet buckets; synthetic chips (free/paid) stay enabled.
  const disabled = count != null && !active && n === 0;
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

function FacetGroup({
  icon,
  title,
  note,
  children,
}: {
  icon: string;
  title: string;
  note?: string;
  children: React.ReactNode;
}) {
  return (
    <div className="m-facet-group">
      <div className="m-facet-label">
        <ShellIcon name={icon} size={13} />
        <span>{title}</span>
        {note && <span className="m-facet-hint">({note})</span>}
      </div>
      {children}
    </div>
  );
}
