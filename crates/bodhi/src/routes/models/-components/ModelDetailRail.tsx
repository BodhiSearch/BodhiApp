import { AliasResponse, ApiAliasResponse, ModelRouterResponse } from '@bodhiapp/ts-client';

import { ShellIcon } from '@/components/shell';
import { isApiAlias, isModelRouterAlias, isUserAlias } from '@/lib/utils';
import { RoutingChainPreview } from './RoutingChainPreview';

/** Bytes per GB (binary) — re-exported so the screen can share one constant. */
export const GB = 1024 * 1024 * 1024;

function formatSize(bytes?: number | null): string {
  if (bytes == null) return '—';
  if (bytes >= GB) return `${(bytes / GB).toFixed(2)} GB`;
  const mb = bytes / (1024 * 1024);
  return `${mb.toFixed(0)} MB`;
}

function railTitle(alias: AliasResponse): string {
  if (isApiAlias(alias)) return alias.name || alias.id;
  return alias.alias;
}

function railIcon(alias: AliasResponse): string {
  if (isApiAlias(alias)) return 'cloud';
  if (isModelRouterAlias(alias)) return 'route';
  if (isUserAlias(alias)) return 'tag';
  return 'hard-drive';
}

function railSubtitle(alias: AliasResponse): string {
  switch (alias.source) {
    case 'api':
      return 'API Model';
    case 'model':
      return 'Local File';
    case 'user':
      return 'Model Alias';
    default:
      return 'Router';
  }
}

export function ModelRailHeader({ alias, onClose }: { alias: AliasResponse; onClose: () => void }) {
  return (
    <div className="dp-head">
      <div className="dp-head-icon" style={{ background: 'var(--c-indigo-bg)', color: 'var(--c-indigo-text)' }}>
        <ShellIcon name={railIcon(alias)} size={15} />
      </div>
      <div className="dp-head-body">
        <div className="dp-head-title mono">{railTitle(alias)}</div>
        <div className="dp-head-sub">{railSubtitle(alias)}</div>
      </div>
      <button className="dp-close" onClick={onClose} title="Close" data-testid="model-detail-close">
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

interface ModelDetailRailProps {
  alias: AliasResponse;
  onEdit: () => void;
}

export function ModelDetailRail({ alias, onEdit }: ModelDetailRailProps) {
  const id = isApiAlias(alias) || isModelRouterAlias(alias) ? alias.id : alias.alias;
  return (
    <div className="dp-panel models-screen-rail" data-testid={`model-detail-${id}`}>
      <div className="dp-body">
        {isApiAlias(alias) ? (
          <ApiRailBody alias={alias} />
        ) : isModelRouterAlias(alias) ? (
          <FallbackRailBody alias={alias} />
        ) : (
          <LocalRailBody alias={alias} />
        )}
      </div>

      <div className="dp-foot">
        <button className="dp-btn dp-btn-accent" onClick={onEdit} data-testid="model-detail-edit">
          <ShellIcon name="pencil" size={14} /> Edit {railSubtitle(alias).toLowerCase()}
        </button>
      </div>
    </div>
  );
}

function LocalRailBody({ alias }: { alias: AliasResponse }) {
  // user + model aliases share the local-file shape; size/metadata are optional real fields.
  const local = alias as Extract<AliasResponse, { repo: string }>;
  const size = 'size' in local ? (local as { size?: number | null }).size : undefined;
  const metadata = 'metadata' in local ? local.metadata : undefined;
  return (
    <>
      <div className="dp-section">
        <div className="dp-sec-lbl">File</div>
        <div className="dp-rows">
          <Row k="repo" v={local.repo} />
          <Row k="filename" v={local.filename} />
          <Row k="snapshot" v={local.snapshot} />
          {size != null && <Row k="size" v={formatSize(size)} />}
        </div>
      </div>

      {metadata?.capabilities && (
        <div className="dp-section">
          <div className="dp-sec-lbl">Capabilities</div>
          <div className="m-cap-chips" data-testid="model-detail-capabilities">
            {metadata.capabilities.vision && <span className="m-cap-chip">vision</span>}
            {metadata.capabilities.tools?.function_calling && <span className="m-cap-chip">tool-use</span>}
            {metadata.capabilities.thinking && <span className="m-cap-chip">reasoning</span>}
          </div>
        </div>
      )}

      <div className="dp-section">
        <p className="dp-desc">
          {alias.source === 'model'
            ? 'Auto-discovered from local cache. Alias is derived from org/repo:quant and is read-only.'
            : 'User-created alias with custom system prompt and parameters.'}
        </p>
      </div>
    </>
  );
}

function ApiRailBody({ alias }: { alias: ApiAliasResponse }) {
  return (
    <>
      <div className="dp-status-row">
        <span className={`m-conn ${alias.has_api_key ? 'ok' : 'warn'}`} data-testid="model-detail-status">
          {alias.has_api_key ? 'connected' : 'no key'}
        </span>
      </div>
      <div className="dp-section">
        <div className="dp-sec-lbl">Connection</div>
        <div className="dp-rows">
          <Row k="base URL" v={alias.base_url} />
          <Row k="provider" v={alias.api_format} />
          <Row k="models" v={`${alias.models.length} exposed`} />
        </div>
      </div>
      <div className="dp-section">
        <div className="dp-sec-lbl">Models</div>
        <div className="m-model-list" data-testid="model-detail-models">
          {alias.models.map((m) => (
            <div key={modelId(m)} className="m-model-item mono">
              {modelId(m)}
            </div>
          ))}
        </div>
      </div>
    </>
  );
}

function modelId(m: ApiAliasResponse['models'][number]): string {
  // OpenAI/Anthropic models expose `id`; Gemini exposes `name`.
  if ('id' in m && m.id) return m.id;
  if ('name' in m && m.name) return m.name;
  return JSON.stringify(m);
}

function FallbackRailBody({ alias }: { alias: ModelRouterResponse }) {
  const enabledCount = alias.targets.filter((t) => t.enabled !== false).length;
  return (
    <>
      <div className="dp-status-row">
        <span className="m-conn ok">
          {enabledCount} of {alias.targets.length} steps active
        </span>
      </div>
      <div className="dp-section">
        <div className="dp-sec-lbl">Routing chain</div>
        <RoutingChainPreview
          testId="model-detail-chain"
          disabledLabel="disabled"
          items={alias.targets.map((t) => ({ alias: t.alias, model: t.model, enabled: t.enabled !== false }))}
        />
      </div>
      <div className="dp-section">
        <div className="dp-sec-lbl">Behavior</div>
        <div className="dp-rows">
          <Row k="on error" v="try next step" />
          <Row k="on success" v="return immediately" />
          <Row k="strategy" v={alias.strategy.strategy} />
        </div>
      </div>
    </>
  );
}
