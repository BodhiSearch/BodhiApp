import { ReactNode } from 'react';

import { ShellIcon, useShell } from '@/components/shell';
import { ChainItem, RoutingChainPreview } from '@/routes/models/-components/RoutingChainPreview';

export function RouterRailHeader() {
  const { collapseRail } = useShell();
  return (
    <div className="rf-rail-head">
      <ShellIcon name="info" size={13} />
      <span className="rf-rail-head-title">Routing &amp; help</span>
      <button
        type="button"
        className="rf-rail-close"
        title="Close panel"
        onClick={collapseRail}
        data-testid="router-rail-close"
      >
        <ShellIcon name="x" size={14} />
      </button>
    </div>
  );
}

export interface RailStatus {
  tone: 'warn';
  message: string;
}

interface RouterInfoRailProps {
  alias: string;
  chain: ChainItem[];
  honorRetryAfter: boolean;
  /** Display-only warn status (resilience invalid). Never gates submit. */
  status?: RailStatus;
}

const HOW_IT_WORKS = (alias: string, honorRetryAfter: boolean): ReactNode[] => [
  <>
    Client sends a request with <code className="rf-code">model={alias || '<router>'}</code>.
  </>,
  'Step 1 is tried first. If it returns a successful response, we stop and return it.',
  'If Step 1 errors (timeout, 5xx, rate limit, model down…), Step 2 is tried — and so on.',
  honorRetryAfter
    ? 'A failed target is put on cooldown; an upstream Retry-After is honored when present.'
    : 'A failed target is put on cooldown before it is retried again.',
  'A final error is only surfaced when every step has failed.',
];

const TIPS = [
  'Put fastest / cheapest targets first to minimize cost and latency.',
  'End with a local model alias for a self-hosted last resort.',
  'Cooldown keeps a failing target out of rotation so requests are not wasted retrying it.',
  'Set max attempts to cap how many targets a single request will try before giving up.',
  'Toggle a step off to skip it temporarily — the sequence and config are preserved.',
];

export function RouterInfoRail({ alias, chain, honorRetryAfter, status }: RouterInfoRailProps) {
  return (
    <div className="rf-rail" data-testid="router-rail">
      {status && (
        <div className="rf-rail-status rf-rail-status-warn" data-testid="router-rail-status">
          <ShellIcon name="alert-circle" size={13} />
          <div>{status.message}</div>
        </div>
      )}

      <div className="rf-side-card">
        <div className="rf-side-card-head">
          <ShellIcon name="route" size={11} /> Routing chain
        </div>
        <div className="rf-side-card-body">
          <RoutingChainPreview items={chain} testId="router-rail-chain" />
        </div>
      </div>

      <div className="rf-side-card">
        <div className="rf-side-card-head">
          <ShellIcon name="help-circle" size={11} /> How it works
        </div>
        <div className="rf-side-card-body">
          {HOW_IT_WORKS(alias, honorRetryAfter).map((text, i) => (
            <div key={i} className="rf-how-item">
              <span className="rf-how-num">{i + 1}</span>
              <span className="rf-how-text">{text}</span>
            </div>
          ))}
        </div>
      </div>

      <div className="rf-side-card">
        <div className="rf-side-card-head">
          <ShellIcon name="lightbulb" size={11} /> Tips
        </div>
        <div className="rf-side-card-body">
          {TIPS.map((tip, i) => (
            <div key={i} className="rf-tip-row">
              <span className="rf-tip-bullet">›</span>
              <span>{tip}</span>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}
