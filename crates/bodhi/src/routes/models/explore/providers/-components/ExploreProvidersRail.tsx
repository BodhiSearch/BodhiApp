import type { ProviderDetailResponse, ProviderModelRow, ProviderSummary } from '@bodhiapp/reference-api-types';
import { Link } from '@tanstack/react-router';

import { DetailRail, DetailRailBody, DetailRailRow, DetailRailRows, DetailRailSection } from '@/components/detail-rail';
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

// Formats the form recognizes; anything else (e.g. 'other') is forwarded as 'openai'.
const KNOWN_API_FORMATS = new Set(['openai', 'anthropic', 'gemini']);
function toFormParam(apiFormat: string | undefined): string {
  return apiFormat && KNOWN_API_FORMATS.has(apiFormat) ? apiFormat : 'openai';
}

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

interface RailProps {
  provider: ProviderSummary;
  detail: ProviderDetailResponse | undefined;
  detailLoading: boolean;
  models: ProviderModelRow[];
  modelsLoading: boolean;
}

export function ExploreProvidersRail({ provider, detail, detailLoading, models, modelsLoading }: RailProps) {
  // The provider's connection params, sourced from the loaded detail. base_url falls back to the
  // form preset (undefined) until detail arrives.
  const apiFormat = toFormParam(detail?.bridge.api_format ?? provider.api_format_hint);
  const baseUrl = detail?.api_base_url ?? undefined;
  const addProviderSearch = {
    api_format: apiFormat,
    name: provider.name,
    ...(baseUrl ? { base_url: baseUrl } : {}),
  };

  return (
    <DetailRail className="models-screen-rail" testId={`cat-prov-detail-${provider.slug}`}>
      <DetailRailBody>
        <DetailRailSection label="Connection">
          {detailLoading && !detail ? (
            <Skeleton className="h-24 w-full" data-testid="cat-prov-detail-skeleton" />
          ) : (
            <DetailRailRows testId="cat-prov-detail-meta">
              <DetailRailRow k="Base URL" v={detail?.api_base_url ?? '— (preset)'} />
              <DetailRailRow k="API keys" v={detail?.env?.length ? detail.env.join(', ') : '—'} />
              <DetailRailRow k="API format" v={detail?.bridge.api_format ?? '—'} />
            </DetailRailRows>
          )}
          <div className="cat-servedby-links">
            {/* Filter the API Models page in place to this provider (provider facet = slug). */}
            <Link
              to="/models/explore/api/"
              search={{ provider: [provider.slug] }}
              className="cat-doc-link"
              data-testid={`cat-prov-allmodels-${provider.slug}`}
            >
              <ShellIcon name="layers" size={13} /> See All Models from Provider
            </Link>
            {/* Jump to the create-API-model form prefilled for this provider. */}
            <Link
              to="/models/api/new/"
              search={addProviderSearch}
              className="cat-doc-link"
              data-testid={`cat-prov-add-${provider.slug}`}
            >
              <ShellIcon name="circle-plus" size={13} /> Add API Model
            </Link>
          </div>
        </DetailRailSection>

        <DetailRailSection label={`Models (${models.length})`}>
          {modelsLoading ? (
            <Skeleton className="h-20 w-full" data-testid="cat-prov-models-skeleton" />
          ) : models.length === 0 ? (
            <div className="cat-sub">No models listed.</div>
          ) : (
            <div className="cat-prov-models" data-testid="cat-prov-models">
              {models.map((m) => (
                <ProviderModel
                  key={m.model_id}
                  model={m}
                  apiFormat={apiFormat}
                  baseUrl={baseUrl}
                  name={provider.name}
                />
              ))}
            </div>
          )}
        </DetailRailSection>
      </DetailRailBody>
    </DetailRail>
  );
}

function ProviderModel({
  model,
  apiFormat,
  baseUrl,
  name,
}: {
  model: ProviderModelRow;
  apiFormat: string;
  baseUrl: string | undefined;
  name: string;
}) {
  const free = isFree(model.pricing.input_per_m, model.pricing.output_per_m);
  const addSearch = {
    api_format: apiFormat,
    name,
    model: model.model_id,
    ...(baseUrl ? { base_url: baseUrl } : {}),
  };
  return (
    <div className="cat-prov-model" data-testid={`cat-prov-model-${model.model_id}`}>
      <div className="cat-prov-model-head">
        <span className="cat-prov-model-name mono">{model.name}</span>
        <div className="cat-prov-model-head-right">
          <span className="cat-prov-model-price">{free ? 'Free' : `${fmtPrice(model.pricing.input_per_m)}/M`}</span>
          <Link
            to="/models/api/new/"
            search={addSearch}
            className="cat-prov-model-add"
            title={`Add ${model.name}`}
            data-testid={`cat-prov-model-add-${model.model_id}`}
          >
            <ShellIcon name="circle-plus" size={15} />
          </Link>
        </div>
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
