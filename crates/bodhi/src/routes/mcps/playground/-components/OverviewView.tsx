import type { Mcp } from '@bodhiapp/ts-client';

import { ShellIcon } from '@/components/shell';
import type { McpCapabilityCounts, McpConnectionStatus } from '@/hooks/mcps/useMcpClient';
import { monogram, tintIndex } from '@/routes/models/explore/-shared/catalog-format';

export type Feature = 'overview' | 'tools' | 'prompts' | 'resources' | 'templates';

interface OverviewCard {
  id: Exclude<Feature, 'overview'>;
  icon: string;
  label: string;
  blurb: string;
  count: number;
}

export interface OverviewViewProps {
  mcp: Mcp;
  status: McpConnectionStatus;
  counts: McpCapabilityCounts;
  onSelectFeature: (feature: Exclude<Feature, 'overview'>) => void;
}

export function OverviewView({ mcp, status, counts, onSelectFeature }: OverviewViewProps) {
  const cards: OverviewCard[] = [
    { id: 'tools', icon: 'wrench', label: 'Tools', blurb: 'Actions you can run', count: counts.tools },
    {
      id: 'prompts',
      icon: 'message-square-quote',
      label: 'Prompts',
      blurb: 'Ready-made requests',
      count: counts.prompts,
    },
    { id: 'resources', icon: 'folder-open', label: 'Resources', blurb: 'Data you can read', count: counts.resources },
    {
      id: 'templates',
      icon: 'layout-template',
      label: 'Templates',
      blurb: 'Fill-in-the-blank reads',
      count: counts.resourceTemplates,
    },
  ];

  const server = mcp.mcp_server;
  const serverName = server?.name || mcp.name;
  const serverUrl = server?.url || mcp.path || '';

  const connected = status === 'connected';
  const statusLabel = connected
    ? 'Connected'
    : status === 'connecting' || status === 'refreshing'
      ? 'Connecting…'
      : status === 'error'
        ? 'Error'
        : 'Disconnected';

  return (
    <div className="pg-overview" data-testid="mcp-playground-overview">
      <div className="pg-ov-hero">
        <div className={`pg-ov-glyph cat-tint-${tintIndex(mcp.id)}`} aria-hidden>
          {monogram(serverName)}
        </div>
        <div className="pg-ov-hero-text">
          <div className="pg-ov-title-row">
            <h1 className="pg-ov-title" data-testid="mcp-playground-overview-title">
              {mcp.name}
            </h1>
            <span className={`pg-pill ${connected ? 'ok' : 'warn'}`} data-testid="mcp-playground-overview-status">
              <ShellIcon name={connected ? 'circle-check' : 'loader-2'} size={11} />
              {statusLabel}
            </span>
          </div>
          <div className="pg-ov-sub">{serverName}</div>
        </div>
      </div>

      <div className="pg-ov-facts">
        <div className="pg-fact">
          <span className="pg-fact-k">
            <ShellIcon name="link" size={12} /> Endpoint
          </span>
          <span className="pg-fact-v mono" data-testid="mcp-playground-overview-endpoint">
            {serverUrl || mcp.path}
          </span>
        </div>
        <div className="pg-fact">
          <span className="pg-fact-k">
            <ShellIcon name="radio" size={12} /> Transport
          </span>
          <span className="pg-fact-v">Streamable HTTP</span>
        </div>
      </div>

      <div className="pg-ov-section-label">What you can do here</div>
      <div className="pg-ov-cards">
        {cards.map((c) => {
          const disabled = c.count === 0;
          return (
            <button
              key={c.id}
              type="button"
              className={'pg-ov-card' + (disabled ? ' disabled' : '')}
              onClick={() => {
                if (!disabled) onSelectFeature(c.id);
              }}
              disabled={disabled}
              data-testid={`mcp-playground-overview-card-${c.id}`}
              data-test-count={c.count}
            >
              <div className="pg-ov-card-top">
                <span className="pg-ov-card-ico">
                  <ShellIcon name={c.icon} size={16} />
                </span>
                <span className="pg-ov-card-n">{c.count}</span>
              </div>
              <div className="pg-ov-card-label">{c.label}</div>
              <div className="pg-ov-card-blurb">{c.blurb}</div>
            </button>
          );
        })}
      </div>
    </div>
  );
}
