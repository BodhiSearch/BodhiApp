import type { McpServerDetail } from '@bodhiapp/reference-api-types';
import type { Mcp, McpAuthConfigResponse } from '@bodhiapp/ts-client';
import { Link } from '@tanstack/react-router';

import { ShellIcon } from '@/components/shell';
import { Skeleton } from '@/components/ui/skeleton';
import { ROUTE_MCP_SERVERS } from '@/lib/constants';
import { AuthBadge } from '@/routes/mcps/-shared/auth-badges';
import {
  McpConfigureServerFooter,
  McpConnectWithSection,
  McpInstancesSection,
} from '@/routes/mcps/-shared/McpRailSections';
import { type McpJoinedRow } from '@/routes/mcps/explore/-shared/instance-join';
import { monogram, tintIndex } from '@/routes/models/explore/-shared/catalog-format';

import { McpServerLogo } from './McpServerLogo';

const TRANSPORT_LABEL: Record<string, string> = {
  'streamable-http': 'Streamable HTTP',
  sse: 'SSE (deprecated)',
  stdio: 'stdio',
};

export function ExploreMcpRailHeader({ server, onClose }: { server: McpJoinedRow; onClose: () => void }) {
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
  server: McpJoinedRow;
  detail: McpServerDetail | undefined;
  loading: boolean;
  instances: Mcp[];
  authConfigs: McpAuthConfigResponse[] | undefined;
  authConfigsLoading: boolean;
  isAdmin: boolean;
  onDeleteInstance: (mcp: Mcp) => void;
}

export function ExploreMcpRail({
  server,
  detail,
  loading,
  instances,
  authConfigs,
  authConfigsLoading,
  isAdmin,
  onDeleteInstance,
}: RailProps) {
  const description = detail?.details ?? server.description;
  const registered = server.registered;
  const serverDisabled = registered ? !registered.enabled : false;

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
          <div className="dp-sec-lbl">Server</div>
          {loading && !detail ? (
            <Skeleton className="h-16 w-full" data-testid="cat-mcp-detail-skeleton" />
          ) : (
            <div className="dp-rows" data-testid="cat-mcp-detail-server">
              <Row k="URL" v={server.endpoint_url} />
              <Row k="Transport" v={TRANSPORT_LABEL[server.transport] ?? server.transport} />
              <Row k="Publisher" v={detail?.publisher} />
              <div className="dp-row dp-row-auth">
                <span className="dp-row-k">Supported auth</span>
                <span className="dp-row-auth-badges">
                  <AuthBadge type={server.auth_type} />
                </span>
              </div>
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

        {registered ? (
          <>
            <McpInstancesSection
              prefix="cat-mcp"
              instances={instances}
              serverDisabled={serverDisabled}
              onDeleteInstance={onDeleteInstance}
            />
            {!serverDisabled && (
              <McpConnectWithSection
                prefix="cat-mcp"
                serverId={registered.id}
                authConfigs={authConfigs}
                loading={authConfigsLoading}
              />
            )}
          </>
        ) : (
          <div
            className="connect-note"
            data-testid={isAdmin ? 'cat-mcp-detail-not-configured-admin' : 'cat-mcp-detail-not-configured'}
          >
            <ShellIcon name={isAdmin ? 'settings-2' : 'info'} size={15} />
            <div>
              <div className="connect-note-title">{isAdmin ? 'Not configured yet' : 'Not in this workspace yet'}</div>
              <div className="connect-note-sub">
                {isAdmin
                  ? 'Register this server to let users connect. The URL is pre-filled for you.'
                  : "This server hasn't been added. Ask an admin to configure it."}
              </div>
            </div>
          </div>
        )}
      </div>

      {/* Footer: admin registers an unregistered server (one click → New Server prefilled), or
          configures a registered one. */}
      {!registered && isAdmin && (
        <div className="dp-foot">
          <Link
            to={`${ROUTE_MCP_SERVERS}new/`}
            search={{ url: server.endpoint_url ?? undefined, name: server.name, auth: server.auth_type }}
            className="dp-btn dp-btn-lotus"
            data-testid="cat-mcp-connect-server"
          >
            <ShellIcon name="plus" size={15} /> Connect Server
          </Link>
        </div>
      )}
      {registered && isAdmin && <McpConfigureServerFooter prefix="cat-mcp" serverId={registered.id} />}
    </div>
  );
}
