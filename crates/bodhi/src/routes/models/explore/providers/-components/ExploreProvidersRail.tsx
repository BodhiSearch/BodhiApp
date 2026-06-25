import type {
  ListProviderModelsQuery,
  ProviderDetailResponse,
  ProviderModelRow,
  ProviderSummary,
} from '@bodhiapp/reference-api-types';

import { ShellIcon } from '@/components/shell';
import { Skeleton } from '@/components/ui/skeleton';
import {
  CAP_LABELS,
  CAP_TONE,
  fmtContext,
  fmtPrice,
  isFree,
  monogram,
  tintIndex,
} from '@/routes/models/explore/-shared/catalog-format';

export function ExploreProvidersRailHeader({ provider, onClose }: { provider: ProviderSummary; onClose: () => void }) {
  return (
    <div className="dp-head">
      <div className={`dp-head-icon cat-logo cat-tint-${tintIndex(provider.slug)}`} aria-hidden="true">
        {monogram(provider.name)}
      </div>
      <div className="dp-head-body">
        <div className="dp-head-title">{provider.name}</div>
        <div className="dp-head-sub">
          {provider.provider_shape} · {provider.model_count} models
        </div>
      </div>
      <button className="dp-close" onClick={onClose} title="Close" data-testid="cat-prov-detail-close">
        <ShellIcon name="x" size={15} />
      </button>
    </div>
  );
}

function Row({ k, v }: { k: string; v: string }) {
  return (
    <div className="dp-row">
      <span className="dp-row-k">{k}</span>
      <span className="dp-row-v mono">{v}</span>
    </div>
  );
}

type ModelSort = NonNullable<ListProviderModelsQuery['sort']>;
const MODEL_SORTS: { key: ModelSort; label: string }[] = [
  { key: 'context', label: 'Context' },
  { key: 'price', label: 'Price' },
  { key: 'name', label: 'Name' },
];

interface RailProps {
  provider: ProviderSummary;
  detail: ProviderDetailResponse | undefined;
  detailLoading: boolean;
  models: ProviderModelRow[];
  modelsLoading: boolean;
  modelSort: ModelSort;
  onModelSort: (s: ModelSort) => void;
}

export function ExploreProvidersRail({
  provider,
  detail,
  detailLoading,
  models,
  modelsLoading,
  modelSort,
  onModelSort,
}: RailProps) {
  return (
    <div className="dp-panel models-screen-rail" data-testid={`cat-prov-detail-${provider.slug}`}>
      <div className="dp-body">
        <div className="dp-section">
          <div className="dp-sec-lbl">Connection</div>
          {detailLoading && !detail ? (
            <Skeleton className="h-24 w-full" data-testid="cat-prov-detail-skeleton" />
          ) : (
            <div className="dp-rows" data-testid="cat-prov-detail-meta">
              <Row k="Base URL" v={detail?.api_base_url ?? '— (preset)'} />
              <Row k="API keys" v={detail?.env?.length ? detail.env.join(', ') : '—'} />
              <Row k="SDK" v={detail?.npm ?? '—'} />
              <Row k="API format" v={detail?.bridge.api_format ?? '—'} />
            </div>
          )}
          {detail?.doc_url && (
            <a
              className="cat-doc-link"
              href={detail.doc_url}
              target="_blank"
              rel="noreferrer"
              data-testid="cat-prov-doc-link"
            >
              <ShellIcon name="book-open" size={13} /> Documentation
            </a>
          )}
        </div>

        <div className="dp-section">
          <div className="dp-sec-lbl cat-prov-models-head">
            <span>Models ({models.length})</span>
            <span className="cat-prov-models-sort" data-testid="cat-prov-models-sort">
              {MODEL_SORTS.map((s) => (
                <button
                  key={s.key}
                  type="button"
                  className={`cat-sort-btn${modelSort === s.key ? ' on' : ''}`}
                  aria-pressed={modelSort === s.key}
                  onClick={() => onModelSort(s.key)}
                  data-testid={`cat-prov-models-sort-${s.key}`}
                >
                  {s.label}
                </button>
              ))}
            </span>
          </div>
          {modelsLoading ? (
            <Skeleton className="h-20 w-full" data-testid="cat-prov-models-skeleton" />
          ) : models.length === 0 ? (
            <div className="cat-sub">No models listed.</div>
          ) : (
            <div className="cat-prov-models" data-testid="cat-prov-models">
              {models.map((m) => (
                <ProviderModel key={m.model_id} model={m} />
              ))}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}

function ProviderModel({ model }: { model: ProviderModelRow }) {
  const free = isFree(model.pricing.input_per_m, model.pricing.output_per_m);
  return (
    <div className="cat-prov-model" data-testid={`cat-prov-model-${model.model_id}`}>
      <div className="cat-prov-model-head">
        <span className="cat-prov-model-name mono">{model.name}</span>
        <span className="cat-prov-model-price">{free ? 'Free' : `${fmtPrice(model.pricing.input_per_m)}/M`}</span>
      </div>
      <div className="cat-prov-model-sub">
        <span>{fmtContext(model.context_limit)} ctx</span>
        <div className="cat-caps">
          {model.caps.slice(0, 4).map((c) => (
            <span className={`cap-chip cap-${CAP_TONE[c]}`} key={c}>
              {CAP_LABELS[c]}
            </span>
          ))}
        </div>
      </div>
    </div>
  );
}
