import { useState } from 'react';

import type { GetModelResponse, Model, Quant } from '@bodhiapp/reference-api-types';

import { ShellIcon } from '@/components/shell';
import { Skeleton } from '@/components/ui/skeleton';

const MONTHS = ['Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun', 'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec'];

function fmtDate(iso?: string | null): string {
  if (!iso) return '—';
  const d = new Date(iso);
  return `${d.getDate()} ${MONTHS[d.getMonth()]} ${d.getFullYear()}`;
}

/** Bytes → GB (catalog sizes are bytes); "—" when null. */
function fmtSize(bytes?: number | null): string {
  if (bytes == null) return '—';
  const gb = bytes / 1_000_000_000;
  return gb >= 1 ? `${gb.toFixed(1)} GB` : `${(bytes / 1_000_000).toFixed(0)} MB`;
}

function fmtCount(n?: number | null): string {
  if (n == null) return '—';
  if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1).replace(/\.0$/, '')}M`;
  if (n >= 1_000) return `${(n / 1_000).toFixed(1).replace(/\.0$/, '')}k`;
  return String(n);
}

export function LocalDiscoveryRailHeader({ model, onClose }: { model: Model; onClose: () => void }) {
  return (
    <div className="dp-head">
      <div className="dp-head-icon" style={{ background: 'var(--c-saffron-bg)', color: 'var(--c-saffron-text)' }}>
        <ShellIcon name="hard-drive" size={15} />
      </div>
      <div className="dp-head-body">
        <div className="dp-head-title mono">
          {model.namespace}/{model.repo}
        </div>
        <div className="dp-head-sub">GGUF · HuggingFace</div>
      </div>
      <button className="dp-close" onClick={onClose} title="Close" data-testid="ld-detail-close">
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

interface RailProps {
  /** The list-row model (always available) — header + summary render immediately. */
  model: Model;
  /** The single-model detail (quants + detail-only fields); undefined while loading. */
  detail: GetModelResponse | undefined;
  loading: boolean;
  onPull: (quant: Quant) => void;
}

export function LocalDiscoveryRail({ model, detail, loading, onPull }: RailProps) {
  const [tab, setTab] = useState<'overview' | 'quants'>('overview');
  const quants = detail?.quants ?? [];
  const quantCount = detail?.quant_count ?? model.quant_count ?? quants.length;

  return (
    <div className="dp-panel models-screen-rail" data-testid={`ld-detail-${model.namespace}-${model.repo}`}>
      <div className="ld-detail-metabar">
        <div className="ld-meta-chips">
          {model.params_b && <span className="ld-meta-chip-lg">{model.params_b}B</span>}
          {(detail?.architecture ?? model.architecture) && (
            <span className="ld-meta-chip-lg mono">{detail?.architecture ?? model.architecture}</span>
          )}
          <span className="ld-meta-chip-lg mono">GGUF</span>
        </div>
        <div className="ld-meta-stats">
          <span title="Downloads">
            <ShellIcon name="download" size={11} /> {fmtCount(model.downloads)}
          </span>
          <span title="Likes">
            <ShellIcon name="heart" size={11} /> {fmtCount(model.likes)}
          </span>
          <span title={`Updated ${fmtDate(model.last_modified)}`}>
            <ShellIcon name="calendar" size={11} /> {fmtDate(model.last_modified)}
          </span>
        </div>
      </div>

      <div className="ld-tabs">
        <button
          className={`ld-tab${tab === 'overview' ? ' on' : ''}`}
          onClick={() => setTab('overview')}
          data-testid="ld-tab-overview"
        >
          Overview
        </button>
        <button
          className={`ld-tab${tab === 'quants' ? ' on' : ''}`}
          onClick={() => setTab('quants')}
          data-testid="ld-tab-quants"
        >
          Download options ({quantCount})
        </button>
      </div>

      <div className="dp-body">
        {tab === 'overview' ? (
          <OverviewTab model={model} detail={detail} loading={loading} />
        ) : (
          <QuantsTab quants={quants} loading={loading} onPull={onPull} />
        )}
      </div>
    </div>
  );
}

function OverviewTab({
  model,
  detail,
  loading,
}: {
  model: Model;
  detail: GetModelResponse | undefined;
  loading: boolean;
}) {
  const caps = detail?.capabilities ?? model.capabilities ?? [];
  return (
    <>
      {caps.length > 0 && (
        <div className="dp-section">
          <div className="dp-sec-lbl">Capabilities</div>
          <div className="m-cap-chips" data-testid="ld-detail-capabilities">
            {caps.map((c) => (
              <span className="m-cap-chip" key={c}>
                {c}
              </span>
            ))}
          </div>
        </div>
      )}

      <div className="dp-section">
        <div className="dp-sec-lbl">Specs</div>
        {loading && !detail ? (
          <Skeleton className="h-20 w-full" data-testid="ld-detail-specs-skeleton" />
        ) : (
          <div className="dp-rows" data-testid="ld-detail-specs">
            <Row k="Context" v={detail?.context_max != null ? `${detail.context_max.toLocaleString()} tokens` : '—'} />
            <Row k="Architecture" v={detail?.architecture ?? model.architecture ?? '—'} />
            <Row k="Parameters" v={model.params_b ? `${model.params_b}B` : '—'} />
            <Row k="License" v={model.license ?? '—'} />
            <Row k="Created" v={fmtDate(model.created_at)} />
          </div>
        )}
      </div>
    </>
  );
}

function QuantsTab({ quants, loading, onPull }: { quants: Quant[]; loading: boolean; onPull: (q: Quant) => void }) {
  if (loading && quants.length === 0) {
    return <Skeleton className="h-32 w-full" data-testid="ld-quants-skeleton" />;
  }
  if (quants.length === 0) {
    return <div className="dp-desc">No download options found for this repository.</div>;
  }
  return (
    <div className="ld-quants" data-testid="ld-quants">
      {quants.map((q) => (
        <div className={`ld-quant-row${q.recommended ? ' rec' : ''}`} key={q.name} data-testid={`ld-quant-${q.name}`}>
          <div className="ld-quant-main">
            <span className="ld-quant-name mono">{q.name}</span>
            {q.recommended && (
              <span className="ld-rec-badge" data-testid={`ld-quant-rec-${q.name}`}>
                <ShellIcon name="thumbs-up" size={10} /> Recommended
              </span>
            )}
          </div>
          <div className="ld-quant-side">
            <span className="ld-quant-size">{fmtSize(q.size)}</span>
            <button
              className="ld-quant-pull"
              title={`Pull ${q.name}`}
              onClick={() => onPull(q)}
              data-testid={`ld-quant-pull-${q.name}`}
            >
              <ShellIcon name="download" size={14} />
            </button>
          </div>
        </div>
      ))}
    </div>
  );
}
