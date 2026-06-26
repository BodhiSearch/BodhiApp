import type { McpServerDetail, McpServerSummary } from '@bodhiapp/reference-api-types';

import { ShellIcon } from '@/components/shell';
import { Skeleton } from '@/components/ui/skeleton';
import { monogram, tintIndex } from '@/routes/models/explore/-shared/catalog-format';

import { McpServerLogo } from './McpServerLogo';

/** Epoch-ms → short date (the MCP catalog uses numeric timestamps, unlike the ISO model catalog). */
function fmtEpoch(ms: number | null | undefined): string | undefined {
  if (ms == null) return undefined;
  const d = new Date(ms);
  if (Number.isNaN(d.getTime())) return undefined;
  return d.toLocaleDateString(undefined, { year: 'numeric', month: 'short', day: 'numeric' });
}

export function ExploreMcpRailHeader({ server, onClose }: { server: McpServerSummary; onClose: () => void }) {
  return (
    <div className="dp-head">
      <McpServerLogo
        src={server.logo_url}
        className={`dp-head-icon cat-logo cat-tint-${tintIndex(server.slug)}`}
        fallback={monogram(server.name)}
      />
      <div className="dp-head-body">
        <div className="dp-head-title">{server.name}</div>
        <div className="dp-head-sub">{server.featured ? 'Featured' : 'MCP server'}</div>
      </div>
      <button className="dp-close" onClick={onClose} title="Close" data-testid="cat-mcp-detail-close">
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
  server: McpServerSummary;
  detail: McpServerDetail | undefined;
  loading: boolean;
}

export function ExploreMcpRail({ server, detail, loading }: RailProps) {
  const description = detail?.details ?? server.description;
  const sources = detail?.sources?.length ? detail.sources.join(', ') : detail?.source;

  return (
    <div className="dp-panel" data-testid={`cat-mcp-detail-${server.id}`}>
      <div className="dp-body">
        {description && (
          <div className="dp-section">
            <div className="dp-sec-lbl">Description</div>
            <div className="cat-sub" data-testid="cat-mcp-detail-description">
              {description}
            </div>
          </div>
        )}

        <div className="dp-section">
          <div className="dp-sec-lbl">Connection</div>
          {loading && !detail ? (
            <Skeleton className="h-16 w-full" data-testid="cat-mcp-detail-skeleton" />
          ) : (
            <div className="dp-rows" data-testid="cat-mcp-detail-connection">
              <Row k="Endpoint" v={server.endpoint_url} />
              <Row k="Transport" v={server.transport} />
              <Row k="Auth" v={server.auth_type} />
              {server.external_link && (
                <div className="cat-servedby-links">
                  <a
                    href={server.external_link}
                    target="_blank"
                    rel="noreferrer"
                    className="cat-doc-link"
                    data-testid="cat-mcp-detail-external"
                  >
                    <ShellIcon name="external-link" size={13} /> Official docs
                  </a>
                </div>
              )}
            </div>
          )}
        </div>

        <div className="dp-section">
          <div className="dp-sec-lbl">Metadata</div>
          <div className="dp-rows" data-testid="cat-mcp-detail-metadata">
            <Row k="Verified" v={server.verified ? 'Yes' : undefined} />
            <Row k="Featured" v={server.featured ? 'Yes' : undefined} />
            <Row k="Source" v={sources} />
            <Row k="First seen" v={fmtEpoch(detail?.first_seen_at)} />
            <Row k="Last scraped" v={fmtEpoch(detail?.last_scraped_at)} />
          </div>
        </div>
      </div>
    </div>
  );
}
