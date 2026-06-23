import type { ModelDetailResponse, ModelLite } from '@bodhiapp/reference-api-types';
import { Link } from '@tanstack/react-router';

import { ShellIcon } from '@/components/shell';
import { Skeleton } from '@/components/ui/skeleton';
import { ROUTE_MODELS_EXPLORE_API_PROVIDERS } from '@/lib/constants';

import {
  CAP_LABELS,
  CAP_TONE,
  fmtContext,
  fmtPrice,
  isFree,
  monogram,
  statusLabel,
  tintIndex,
} from '../../-shared/catalog-format';

export function ExploreApiRailHeader({ model, onClose }: { model: ModelLite; onClose: () => void }) {
  return (
    <div className="dp-head">
      <div className={`dp-head-icon cat-logo cat-tint-${tintIndex(model.family ?? model.slug)}`} aria-hidden="true">
        {monogram(model.family ?? model.name)}
      </div>
      <div className="dp-head-body">
        <div className="dp-head-title">{model.name}</div>
        <div className="dp-head-sub">
          {model.family ?? model.slug} · {statusLabel(model.status)}
        </div>
      </div>
      <button className="dp-close" onClick={onClose} title="Close" data-testid="cat-model-detail-close">
        <ShellIcon name="x" size={15} />
      </button>
    </div>
  );
}

function Row({ k, v }: { k: string; v: string | null | undefined }) {
  if (v == null || v === '') return null;
  return (
    <div className="dp-row">
      <span className="dp-row-k">{k}</span>
      <span className="dp-row-v mono">{v}</span>
    </div>
  );
}

interface RailProps {
  model: ModelLite;
  detail: ModelDetailResponse | undefined;
  loading: boolean;
}

export function ExploreApiRail({ model, detail, loading }: RailProps) {
  const free = isFree(model.pricing.input_per_m, model.pricing.output_per_m);
  // Bridge: build the Configure-in-Bodhi target from detail.bridge. Omit base_url when null (the
  // form falls back to its preset). api_format maps 1:1 to the form's ApiFormat.
  const bridge = detail?.bridge;
  const configureSearch = bridge
    ? {
        api_format: bridge.api_format,
        ...(bridge.base_url ? { base_url: bridge.base_url } : {}),
        model: model.model_id,
      }
    : { model: model.model_id };

  return (
    <div className="dp-panel models-screen-rail" data-testid={`cat-model-detail-${model.slug}-${model.model_id}`}>
      <div className="dp-body">
        <div className="dp-section">
          <div className="dp-sec-lbl">Capabilities</div>
          <div className="cat-caps" data-testid="cat-model-detail-caps">
            {model.caps.map((c) => (
              <span className={`cap-chip cap-${CAP_TONE[c]}`} key={c}>
                {CAP_LABELS[c]}
              </span>
            ))}
          </div>
        </div>

        <div className="dp-section">
          <div className="dp-sec-lbl">Specs</div>
          {loading && !detail ? (
            <Skeleton className="h-24 w-full" data-testid="cat-model-detail-skeleton" />
          ) : (
            <div className="dp-rows" data-testid="cat-model-detail-specs">
              <Row k="Context" v={fmtContext(model.context_limit)} />
              <Row k="Max output" v={detail ? fmtContext(detail.output_limit) : fmtContext(model.output_limit)} />
              <Row k="Input" v={free ? 'Free' : `${fmtPrice(model.pricing.input_per_m)}/M`} />
              <Row k="Output" v={free ? 'Free' : `${fmtPrice(model.pricing.output_per_m)}/M`} />
              <Row k="Status" v={statusLabel(model.status)} />
              <Row k="Modalities" v={[...model.modalities_in, '→', ...model.modalities_out].join(' ')} />
              <Row k="Knowledge" v={detail?.knowledge_cutoff} />
              <Row k="Released" v={model.release_date} />
              <Row k="Open weights" v={model.open_weights ? 'Yes' : undefined} />
            </div>
          )}
        </div>

        <div className="dp-section">
          <div className="dp-sec-lbl">Served by ({detail?.served_by.length ?? model.provider_count})</div>
          {loading && !detail ? (
            <Skeleton className="h-16 w-full" data-testid="cat-model-servedby-skeleton" />
          ) : (
            <div className="cat-servedby" data-testid="cat-model-servedby">
              {(detail?.served_by ?? []).map((s) => (
                <Link
                  key={s.slug}
                  to={ROUTE_MODELS_EXPLORE_API_PROVIDERS}
                  search={{ select: s.slug }}
                  className="cat-servedby-row"
                  data-testid={`cat-model-servedby-${s.slug}`}
                >
                  <div className={`cat-logo cat-tint-${tintIndex(s.slug)}`} aria-hidden="true">
                    {monogram(s.name)}
                  </div>
                  <span className="cat-servedby-name">{s.name}</span>
                  <span className="cat-servedby-price">
                    {isFree(s.pricing.input_per_m, s.pricing.output_per_m)
                      ? 'Free'
                      : `${fmtPrice(s.pricing.input_per_m)}/M`}
                  </span>
                </Link>
              ))}
            </div>
          )}
        </div>

        <Link
          to="/models/api/new/"
          search={configureSearch}
          className="cat-configure-cta"
          data-testid="cat-model-configure-cta"
        >
          <ShellIcon name="plug-zap" size={15} /> Configure in Bodhi
        </Link>
      </div>
    </div>
  );
}
