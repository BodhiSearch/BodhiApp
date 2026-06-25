import type { Capability, Modality, ModelFacets } from '@bodhiapp/reference-api-types';

import { ShellIcon } from '@/components/shell';
import { CAP_LABELS } from '@/routes/models/explore/-shared/catalog-format';
import { FacetCombobox, facetOptions } from '@/routes/models/explore/-shared/FacetCombobox';
import { DualRangeControl, RangeControl } from '@/routes/models/explore/-shared/RangeControls';
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
  family?: string[];
  open_weights?: OpenWeights;
  pricing?: 'free' | 'paid';
  pricing_in_min?: number;
  pricing_in_max?: number;
  pricing_out_min?: number;
  pricing_out_max?: number;
  context_min?: number;
}

export function hasActiveModelFacets(f: ModelFacetsState): boolean {
  return Boolean(
    f.capability?.length ||
      f.modality?.length ||
      f.status?.length ||
      f.provider?.length ||
      f.family?.length ||
      f.open_weights ||
      f.pricing ||
      f.pricing_in_min != null ||
      f.pricing_in_max != null ||
      f.pricing_out_min != null ||
      f.pricing_out_max != null ||
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

// Frontend-owned slider ceilings (the backend has no price-range facet). Most models fall well under
// these; a slider parked at its ceiling means "no upper bound" and sends nothing.
const PRICE_IN_MAX = 30; // input $/Mtok slider ceiling
const PRICE_OUT_MAX = 60; // output $/Mtok slider ceiling
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
}

export function ExploreApiSidebar({ facets, facetCounts, onFacetsChange }: SidebarProps) {
  const capCounts = facetCounts?.capability ?? {};
  const modCounts = facetCounts?.modality ?? {};
  const statusCounts = facetCounts?.status ?? {};
  const providerOptions = facetOptions(facetCounts?.provider);
  const familyOptions = facetOptions(facetCounts?.family);

  return (
    <div className="m-facets" data-testid="cat-model-facets">
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

      <FacetGroup icon="dollar-sign" title="Pricing" note="$/Mtok">
        <Pills>
          <FacetPill
            label="Free"
            count={undefined}
            active={facets.pricing === 'free'}
            testId="cat-model-pricing-free"
            // Free pins input AND output to $0 server-side; clearing the price ranges avoids sending
            // redundant bounds. Re-click clears.
            onToggle={() =>
              onFacetsChange(
                facets.pricing === 'free'
                  ? { ...facets, pricing: undefined }
                  : {
                      ...facets,
                      pricing: 'free',
                      pricing_in_min: undefined,
                      pricing_in_max: undefined,
                      pricing_out_min: undefined,
                      pricing_out_max: undefined,
                    }
              )
            }
          />
        </Pills>
        <DualRangeControl
          axis="Input"
          min={facets.pricing_in_min ?? 0}
          max={facets.pricing_in_max ?? PRICE_IN_MAX}
          ceiling={PRICE_IN_MAX}
          step={0.25}
          format={(v) => `$${v}`}
          maxLabel="Any"
          disabled={facets.pricing === 'free'}
          testId="cat-model-pricing-in"
          onCommit={(lo, hi) =>
            onFacetsChange({
              ...facets,
              pricing_in_min: lo <= 0 ? undefined : lo,
              pricing_in_max: hi >= PRICE_IN_MAX ? undefined : hi,
            })
          }
        />
        <DualRangeControl
          axis="Output"
          min={facets.pricing_out_min ?? 0}
          max={facets.pricing_out_max ?? PRICE_OUT_MAX}
          ceiling={PRICE_OUT_MAX}
          step={0.5}
          format={(v) => `$${v}`}
          maxLabel="Any"
          disabled={facets.pricing === 'free'}
          testId="cat-model-pricing-out"
          onCommit={(lo, hi) =>
            onFacetsChange({
              ...facets,
              pricing_out_min: lo <= 0 ? undefined : lo,
              pricing_out_max: hi >= PRICE_OUT_MAX ? undefined : hi,
            })
          }
        />
      </FacetGroup>

      <FacetGroup icon="ruler" title="Context" note="min tokens">
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

      <FacetGroup icon="boxes" title="Family">
        <FacetCombobox
          options={familyOptions}
          selected={facets.family ?? []}
          onToggle={(v) => onFacetsChange({ ...facets, family: toggle(facets.family, v) })}
          placeholder="Any family"
          searchPlaceholder="Search families…"
          emptyText="No families match."
          testId="cat-model-family"
        />
      </FacetGroup>

      <FacetGroup icon="at-sign" title="Provider">
        <FacetCombobox
          options={providerOptions}
          selected={facets.provider ?? []}
          onToggle={(v) => onFacetsChange({ ...facets, provider: toggle(facets.provider, v) })}
          placeholder="Any provider"
          searchPlaceholder="Search providers…"
          emptyText="No providers match."
          testId="cat-model-provider"
        />
      </FacetGroup>
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
  // Count-gating only applies to real facet buckets; synthetic chips (no count) stay enabled.
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

/** Build the API query params contributed by the facets (omitting defaults/empties). */
export function modelFacetsToQuery(f: ModelFacetsState) {
  return {
    ...(f.capability?.length ? { capability: f.capability } : {}),
    ...(f.modality?.length ? { modality: f.modality } : {}),
    ...(f.status?.length ? { status: f.status } : {}),
    ...(f.provider?.length ? { provider: f.provider } : {}),
    ...(f.family?.length ? { family: f.family } : {}),
    ...(f.open_weights ? { open_weights: f.open_weights } : {}),
    ...(f.pricing ? { pricing: f.pricing } : {}),
    // When Free is set the price ranges are cleared; never send both (redundant — backend ANDs them).
    ...(f.pricing !== 'free' && f.pricing_in_min != null ? { pricing_in_min: f.pricing_in_min } : {}),
    ...(f.pricing !== 'free' && f.pricing_in_max != null ? { pricing_in_max: f.pricing_in_max } : {}),
    ...(f.pricing !== 'free' && f.pricing_out_min != null ? { pricing_out_min: f.pricing_out_min } : {}),
    ...(f.pricing !== 'free' && f.pricing_out_max != null ? { pricing_out_max: f.pricing_out_max } : {}),
    ...(f.context_min != null ? { context_min: f.context_min } : {}),
  };
}
