import { useEffect, useState } from 'react';

import type { Capability, Modality, ModelFacets } from '@bodhiapp/reference-api-types';

import { ShellIcon } from '@/components/shell';
import { Slider } from '@/components/ui/slider';

import { CAP_LABELS } from '../../-shared/catalog-format';
import '@/routes/models/-components/models.css';

/**
 * Faceted sidebar for Explore · API Models. Controlled — selections drive the parent's
 * `ListCatalogModelsQuery`. Facet OPTIONS come from a fixed enum set; counts come from
 * `ModelFacets` (recomputed per query). Range controls (pricing/context) are debounced so a slider
 * drag fires one query on release, not N.
 */

export type StatusFacet = 'stable' | 'alpha' | 'beta' | 'deprecated';
export type OpenWeights = 'open' | 'closed';

export interface ModelFacetsState {
  capability?: Capability[];
  modality?: Modality[];
  status?: StatusFacet[];
  provider?: string[];
  open_weights?: OpenWeights;
  pricing_max?: number;
  context_min?: number;
}

export function hasActiveModelFacets(f: ModelFacetsState): boolean {
  return Boolean(
    f.capability?.length ||
      f.modality?.length ||
      f.status?.length ||
      f.provider?.length ||
      f.open_weights ||
      f.pricing_max != null ||
      f.context_min != null
  );
}

const MODALITY_LABELS: Record<Modality, string> = {
  text: 'Text',
  image: 'Image',
  audio: 'Audio',
  video: 'Video',
  pdf: 'PDF',
};

const STATUS_LABELS: Record<StatusFacet, string> = {
  stable: 'Stable',
  beta: 'Beta',
  alpha: 'Alpha',
  deprecated: 'Deprecated',
};

const PRICE_MAX = 75; // $/Mtok slider ceiling
const CONTEXT_MAX = 1000; // K tokens slider ceiling

function toggle<T>(list: T[] | undefined, value: T): T[] | undefined {
  const set = new Set(list ?? []);
  if (set.has(value)) set.delete(value);
  else set.add(value);
  const next = [...set];
  return next.length ? next : undefined;
}

interface SidebarProps {
  facets: ModelFacetsState;
  facetCounts: ModelFacets | undefined;
  onFacetsChange: (next: ModelFacetsState) => void;
  onClearAll: () => void;
}

export function ExploreApiSidebar({ facets, facetCounts, onFacetsChange, onClearAll }: SidebarProps) {
  const capCounts = facetCounts?.capability ?? {};
  const modCounts = facetCounts?.modality ?? {};
  const statusCounts = facetCounts?.status ?? {};
  const providerCounts = facetCounts?.provider ?? {};

  return (
    <div className="m-facets" data-testid="cat-model-facets">
      {hasActiveModelFacets(facets) && (
        <button type="button" className="ld-clear-all" onClick={onClearAll} data-testid="cat-model-clear-all">
          <ShellIcon name="x" size={11} /> Clear all filters
        </button>
      )}

      <FacetGroup icon="sparkles" title="Capability">
        <Pills>
          {(Object.keys(CAP_LABELS) as Capability[]).map((c) => (
            <FacetPill
              key={c}
              label={CAP_LABELS[c]}
              count={capCounts[c]}
              active={(facets.capability ?? []).includes(c)}
              testId={`cat-model-cap-${c}`}
              onToggle={() => onFacetsChange({ ...facets, capability: toggle(facets.capability, c) })}
            />
          ))}
        </Pills>
      </FacetGroup>

      <FacetGroup icon="shapes" title="Modality" note="input & output">
        <Pills>
          {(Object.keys(MODALITY_LABELS) as Modality[]).map((m) => (
            <FacetPill
              key={m}
              label={MODALITY_LABELS[m]}
              count={modCounts[m]}
              active={(facets.modality ?? []).includes(m)}
              testId={`cat-model-mod-${m}`}
              onToggle={() => onFacetsChange({ ...facets, modality: toggle(facets.modality, m) })}
            />
          ))}
        </Pills>
      </FacetGroup>

      <FacetGroup icon="dollar-sign" title="Pricing" note="input $/Mtok, max">
        <RangeControl
          value={facets.pricing_max ?? PRICE_MAX}
          max={PRICE_MAX}
          step={0.5}
          format={(v) => (v >= PRICE_MAX ? 'Any' : `$${v}`)}
          testId="cat-model-pricing"
          onCommit={(v) => onFacetsChange({ ...facets, pricing_max: v >= PRICE_MAX ? undefined : v })}
        />
      </FacetGroup>

      <FacetGroup icon="ruler" title="Context" note="min, K tokens">
        <RangeControl
          value={facets.context_min ?? 0}
          max={CONTEXT_MAX}
          step={8}
          format={(v) => (v <= 0 ? 'Any' : `${v}K+`)}
          testId="cat-model-context"
          onCommit={(v) => onFacetsChange({ ...facets, context_min: v <= 0 ? undefined : v * 1000 })}
          // context_min is stored in tokens; show K.
          display={facets.context_min != null ? Math.round(facets.context_min / 1000) : 0}
        />
      </FacetGroup>

      <FacetGroup icon="activity" title="Status">
        <Pills>
          {(Object.keys(STATUS_LABELS) as StatusFacet[]).map((s) => (
            <FacetPill
              key={s}
              label={STATUS_LABELS[s]}
              count={statusCounts[s]}
              active={(facets.status ?? []).includes(s)}
              testId={`cat-model-status-${s}`}
              onToggle={() => onFacetsChange({ ...facets, status: toggle(facets.status, s) })}
            />
          ))}
        </Pills>
      </FacetGroup>

      <FacetGroup icon="unlock" title="Open weights">
        <Pills>
          {(['open', 'closed'] as OpenWeights[]).map((w) => (
            <FacetPill
              key={w}
              label={w === 'open' ? 'Open' : 'Closed'}
              count={facetCounts?.open_weights?.[w]}
              active={facets.open_weights === w}
              testId={`cat-model-ow-${w}`}
              // Tri-state: re-selecting the active value clears it.
              onToggle={() => onFacetsChange({ ...facets, open_weights: facets.open_weights === w ? undefined : w })}
            />
          ))}
        </Pills>
      </FacetGroup>

      {Object.keys(providerCounts).length > 0 && (
        <FacetGroup icon="at-sign" title="Provider">
          <Pills>
            {Object.keys(providerCounts)
              .sort((a, b) => (providerCounts[b] ?? 0) - (providerCounts[a] ?? 0))
              .slice(0, 12)
              .map((slug) => (
                <FacetPill
                  key={slug}
                  label={slug}
                  count={providerCounts[slug]}
                  active={(facets.provider ?? []).includes(slug)}
                  testId={`cat-model-provider-${slug}`}
                  onToggle={() => onFacetsChange({ ...facets, provider: toggle(facets.provider, slug) })}
                />
              ))}
          </Pills>
        </FacetGroup>
      )}
    </div>
  );
}

function Pills({ children }: { children: React.ReactNode }) {
  return <div className="m-facet-pills">{children}</div>;
}

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

/** A debounced range slider: emits onCommit only when the user releases (onValueCommit). */
function RangeControl({
  value,
  display,
  max,
  step,
  format,
  testId,
  onCommit,
}: {
  value: number;
  display?: number;
  max: number;
  step: number;
  format: (v: number) => string;
  testId: string;
  onCommit: (v: number) => void;
}) {
  const [local, setLocal] = useState(display ?? value);
  useEffect(() => {
    setLocal(display ?? value);
  }, [display, value]);

  return (
    <div className="cat-range" data-testid={testId}>
      <Slider
        value={[local]}
        min={0}
        max={max}
        step={step}
        onValueChange={(vals) => setLocal(vals[0])}
        onValueCommit={(vals) => onCommit(vals[0])}
        data-testid={`${testId}-slider`}
      />
      <span className="cat-range-val" data-testid={`${testId}-val`}>
        {format(local)}
      </span>
    </div>
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

/** Build the API query params contributed by the facets (omitting defaults/empties). */
export function modelFacetsToQuery(f: ModelFacetsState) {
  return {
    ...(f.capability?.length ? { capability: f.capability } : {}),
    ...(f.modality?.length ? { modality: f.modality } : {}),
    ...(f.status?.length ? { status: f.status } : {}),
    ...(f.provider?.length ? { provider: f.provider } : {}),
    ...(f.open_weights ? { open_weights: f.open_weights } : {}),
    ...(f.pricing_max != null ? { pricing_max: f.pricing_max } : {}),
    ...(f.context_min != null ? { context_min: f.context_min } : {}),
  };
}
