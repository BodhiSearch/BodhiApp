/**
 * Shared fallback routing-chain renderer, used by both the My-Models detail rail
 * (`ModelDetailRail` → FallbackRailBody, persisted routers) and the Model Router form's live
 * preview rail. Reuses the `m-chain-*` styles from `models.css`. The form passes in-progress
 * items that may lack an alias or a model; the detail rail passes resolved persisted targets.
 */
export interface ChainItem {
  /** Display name of the referenced alias, or undefined if not yet selected (form only). */
  alias?: string;
  /** Pinned model, if any. */
  model?: string;
  enabled: boolean;
  /** Form-only: this enabled step still needs a model (e.g. an API step with empty model). */
  missingModel?: boolean;
}

interface RoutingChainPreviewProps {
  items: ChainItem[];
  testId?: string;
  /** Word shown on a disabled step. The persisted detail rail says "disabled"; the live form "skipped". */
  disabledLabel?: string;
}

export function RoutingChainPreview({ items, testId, disabledLabel = 'skipped' }: RoutingChainPreviewProps) {
  if (items.length === 0) {
    return <div className="rf-chain-empty">No steps yet — add one to start.</div>;
  }
  return (
    <div className="m-chain" data-testid={testId}>
      {items.map((it, i) => (
        <div key={i} className={`m-chain-step${it.enabled ? '' : ' disabled'}`}>
          <span className="m-chain-num">{i + 1}</span>
          <div className="m-chain-body">
            <div className="m-chain-alias mono">
              {it.alias ?? <span className="rf-chain-empty-name">(not selected)</span>}
            </div>
            {it.model && <div className="m-chain-model mono">→ {it.model}</div>}
            {it.missingModel && <div className="m-chain-model rf-chain-err">→ model required</div>}
          </div>
          {!it.enabled && <span className="m-chain-disabled">{disabledLabel}</span>}
        </div>
      ))}
    </div>
  );
}
