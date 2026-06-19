import { useCallback } from 'react';

import { ShellIcon } from '@/components/shell';
import { ApiFormatFacet, CapabilityFacet, ModelsFilter, ModelTypeFacet } from '@/hooks/models';

/** Bytes per GB used by the SIZE dual-slider (binary GiB, matching the GGUF-size convention). */
export const GB = 1024 * 1024 * 1024;
/** The slider's upper bound; the max thumb at this value means "no upper limit". */
const SIZE_MAX_GB = 16;

const TYPE_FACETS: { id: ModelTypeFacet; label: string }[] = [
  { id: 'local_file', label: 'Local File' },
  { id: 'model_alias', label: 'Model Alias' },
  { id: 'api_model', label: 'API Model' },
  { id: 'fallback', label: 'Router' },
];

// Capability subset that maps to real backend metadata fields (chat/embeddings dropped — no field).
const CAPABILITY_FACETS: { id: CapabilityFacet; label: string }[] = [
  { id: 'vision', label: 'vision' },
  { id: 'tool_use', label: 'tool-use' },
  { id: 'reasoning', label: 'reasoning' },
];

const API_FORMAT_FACETS: { id: ApiFormatFacet; label: string }[] = [
  { id: 'openai', label: 'OpenAI' },
  { id: 'responses', label: 'Responses' },
  { id: 'anthropic', label: 'Anthropic' },
  { id: 'gemini', label: 'Gemini' },
  { id: 'liberty', label: 'Liberty' },
];

function toggle<T>(list: T[] | undefined, value: T): T[] | undefined {
  const set = new Set(list ?? []);
  if (set.has(value)) set.delete(value);
  else set.add(value);
  const next = [...set];
  return next.length ? next : undefined;
}

interface FacetPillsProps<T extends string> {
  facets: { id: T; label: string }[];
  selected: T[] | undefined;
  onToggle: (id: T) => void;
  testIdPrefix: string;
}

function FacetPills<T extends string>({ facets, selected, onToggle, testIdPrefix }: FacetPillsProps<T>) {
  const active = new Set(selected ?? []);
  return (
    <div className="m-facet-pills">
      {facets.map((f) => (
        <button
          key={f.id}
          type="button"
          className={`m-facet-pill${active.has(f.id) ? ' active' : ''}`}
          aria-pressed={active.has(f.id)}
          onClick={() => onToggle(f.id)}
          data-testid={`${testIdPrefix}-${f.id}`}
        >
          {f.label}
        </button>
      ))}
    </div>
  );
}

interface ModelSidebarFacetsProps {
  filter: ModelsFilter;
  onChange: (next: ModelsFilter) => void;
}

export function ModelSidebarFacets({ filter, onChange }: ModelSidebarFacetsProps) {
  const onToggleType = useCallback(
    (id: ModelTypeFacet) => onChange({ ...filter, types: toggle(filter.types, id) }),
    [filter, onChange]
  );
  const onToggleCapability = useCallback(
    (id: CapabilityFacet) => onChange({ ...filter, capabilities: toggle(filter.capabilities, id) }),
    [filter, onChange]
  );
  const onToggleApiFormat = useCallback(
    (id: ApiFormatFacet) => onChange({ ...filter, apiFormats: toggle(filter.apiFormats, id) }),
    [filter, onChange]
  );

  // SIZE dual-slider state, expressed in GB for the UI; persisted to bytes in the filter.
  const minGb = filter.sizeMin != null ? Math.round(filter.sizeMin / GB) : 0;
  const maxGb = filter.sizeMax != null ? Math.round(filter.sizeMax / GB) : SIZE_MAX_GB;

  const onSizeChange = useCallback(
    (nextMinGb: number, nextMaxGb: number) => {
      const lo = Math.min(nextMinGb, nextMaxGb);
      const hi = Math.max(nextMinGb, nextMaxGb);
      onChange({
        ...filter,
        // 0 GB min and 16+ GB max mean "no bound" → omit so the facet is inactive.
        sizeMin: lo > 0 ? lo * GB : undefined,
        sizeMax: hi < SIZE_MAX_GB ? hi * GB : undefined,
      });
    },
    [filter, onChange]
  );

  const sizeActive = filter.sizeMin != null || filter.sizeMax != null;

  return (
    <div className="m-facets" data-testid="models-facets">
      <FacetGroup icon="shapes" title="Type">
        <FacetPills
          facets={TYPE_FACETS}
          selected={filter.types}
          onToggle={onToggleType}
          testIdPrefix="models-facet-type"
        />
      </FacetGroup>

      <FacetGroup icon="sparkles" title="Capability">
        <FacetPills
          facets={CAPABILITY_FACETS}
          selected={filter.capabilities}
          onToggle={onToggleCapability}
          testIdPrefix="models-facet-capability"
        />
      </FacetGroup>

      <FacetGroup icon="ruler" title="Size" hint="local files">
        <div className="m-size">
          <div className="m-size-labels">
            <span>{minGb === 0 ? '0 GB' : `${minGb} GB`}</span>
            <span>{maxGb >= SIZE_MAX_GB ? '16+ GB' : `${maxGb} GB`}</span>
          </div>
          <div className={`m-size-slider${sizeActive ? ' active' : ''}`} data-testid="models-facet-size">
            <input
              type="range"
              min={0}
              max={SIZE_MAX_GB}
              step={1}
              value={minGb}
              aria-label="Minimum model size (GB)"
              data-testid="models-facet-size-min"
              onChange={(e) => onSizeChange(Number(e.target.value), maxGb)}
            />
            <input
              type="range"
              min={0}
              max={SIZE_MAX_GB}
              step={1}
              value={maxGb}
              aria-label="Maximum model size (GB)"
              data-testid="models-facet-size-max"
              onChange={(e) => onSizeChange(minGb, Number(e.target.value))}
            />
          </div>
        </div>
      </FacetGroup>

      <FacetGroup icon="plug" title="API Format" hint="API only">
        <FacetPills
          facets={API_FORMAT_FACETS}
          selected={filter.apiFormats}
          onToggle={onToggleApiFormat}
          testIdPrefix="models-facet-format"
        />
      </FacetGroup>
    </div>
  );
}

function FacetGroup({
  icon,
  title,
  hint,
  children,
}: {
  icon: string;
  title: string;
  hint?: string;
  children: React.ReactNode;
}) {
  return (
    <div className="m-facet-group">
      <div className="m-facet-label">
        <ShellIcon name={icon} size={13} />
        <span>{title}</span>
        {hint && <span className="m-facet-hint">({hint})</span>}
      </div>
      {children}
    </div>
  );
}
