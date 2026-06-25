import { useState } from 'react';

import type { ModelDetailResponse, ModelLite, ServedBy } from '@bodhiapp/reference-api-types';
import { Link } from '@tanstack/react-router';

import { ShellIcon } from '@/components/shell';
import { Skeleton } from '@/components/ui/skeleton';
import { useCatalogProviderDetail } from '@/hooks/reference';
import {
  CAP_LABELS,
  CAP_TONE,
  fmtContext,
  fmtPrice,
  isFree,
  monogram,
  statusLabel,
  tintIndex,
} from '@/routes/models/explore/-shared/catalog-format';

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
          <div className="dp-sec-lbl">
            Served by ({detail?.served_by.length ?? model.provider_count}){' '}
            <span className="cat-sub">· $in / $out per M</span>
          </div>
          {loading && !detail ? (
            <Skeleton className="h-16 w-full" data-testid="cat-model-servedby-skeleton" />
          ) : (
            <div className="cat-servedby" data-testid="cat-model-servedby">
              {(detail?.served_by ?? []).map((s) => (
                <ServedByRow key={s.slug} served={s} modelId={model.model_id} />
              ))}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}

// A served-by provider row. Clicking the row toggles an inline connection detail (no navigation to
// the Providers page — with many providers that page may not list this one). The trailing "Add" icon
// jumps to the create-API-model form prefilled for this provider (api_format is openai for all
// catalog providers; base_url is the provider's own).
function ServedByRow({ served, modelId }: { served: ServedBy; modelId: string }) {
  const [open, setOpen] = useState(false);
  const { data: provider, isLoading } = useCatalogProviderDetail(open ? served.slug : null);
  const addSearch = {
    api_format: 'openai',
    ...(served.base_url ? { base_url: served.base_url } : {}),
    model: modelId,
  };
  return (
    <div className="cat-servedby-item" data-testid={`cat-model-servedby-${served.slug}`}>
      <div className="cat-servedby-row">
        <button
          type="button"
          className="cat-servedby-main"
          onClick={() => setOpen((v) => !v)}
          aria-expanded={open}
          data-testid={`cat-model-servedby-toggle-${served.slug}`}
        >
          <div className={`cat-logo cat-tint-${tintIndex(served.slug)}`} aria-hidden="true">
            {monogram(served.name)}
          </div>
          <span className="cat-servedby-name">{served.name}</span>
          <span className="cat-servedby-price">
            {isFree(served.pricing.input_per_m, served.pricing.output_per_m)
              ? 'Free'
              : `${fmtPrice(served.pricing.input_per_m)} / ${fmtPrice(served.pricing.output_per_m)}`}
          </span>
        </button>
        <Link
          to="/models/api/new/"
          search={addSearch}
          className="cat-servedby-add"
          title={`Add ${served.name} model`}
          data-testid={`cat-model-servedby-add-${served.slug}`}
        >
          <ShellIcon name="circle-plus" size={16} />
        </Link>
      </div>
      {open && (
        <div className="cat-servedby-detail" data-testid={`cat-model-servedby-detail-${served.slug}`}>
          {isLoading && !provider ? (
            <Skeleton className="h-12 w-full" />
          ) : (
            <div className="dp-rows">
              <Row k="Base URL" v={provider?.api_base_url ?? served.base_url ?? '— (preset)'} />
              <Row k="API format" v={provider?.bridge.api_format} />
              <Row k="API keys" v={provider?.env?.length ? provider.env.join(', ') : undefined} />
              <div className="cat-servedby-links">
                {/* Filter the Models page in place to this provider (provider facet = slug). */}
                <Link
                  to="/models/explore/api/"
                  search={{ provider: [served.slug] }}
                  className="cat-doc-link"
                  data-testid={`cat-model-servedby-allmodels-${served.slug}`}
                >
                  <ShellIcon name="layers" size={13} /> All Models from Provider
                </Link>
                {/* Open the Providers page searching for this provider by name. */}
                <Link
                  to="/models/explore/api-providers/"
                  search={{ q: served.name }}
                  className="cat-doc-link"
                  data-testid={`cat-model-servedby-view-${served.slug}`}
                >
                  <ShellIcon name="external-link" size={13} /> View
                </Link>
              </div>
            </div>
          )}
        </div>
      )}
    </div>
  );
}
