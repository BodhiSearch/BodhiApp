import type { McpServerDetail } from '@bodhiapp/reference-api-types';
import type { Mcp, McpAuthConfigResponse } from '@bodhiapp/ts-client';
import { Link } from '@tanstack/react-router';

import { ShellIcon } from '@/components/shell';
import { Skeleton } from '@/components/ui/skeleton';
import { ROUTE_MCP_SERVERS } from '@/lib/constants';
import {
  McpConfigureServerFooter,
  McpConnectWithSection,
  McpInstancesSection,
} from '@/routes/mcps/-shared/McpRailSections';
import { type McpJoinedRow, INSTALL_LABEL } from '@/routes/mcps/explore/-shared/instance-join';
import { monogram, tintIndex } from '@/routes/models/explore/-shared/catalog-format';

import { McpServerLogo } from './McpServerLogo';

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
        <div className="dp-section">
          <div className="dp-sec-lbl">Status</div>
          <div className="cat-servedby-links" data-testid="cat-mcp-detail-status">
            <span className={`mcp-install mcp-install-${server.install}`}>{INSTALL_LABEL[server.install]}</span>
            {/* Unregistered catalog server: admin can register it (url prefilled); non-admin sees a note. */}
            {!registered &&
              (isAdmin ? (
                <Link
                  to={`${ROUTE_MCP_SERVERS}new/`}
                  search={{ url: server.endpoint_url ?? undefined, name: server.name }}
                  className="cat-doc-link"
                  data-testid="cat-mcp-detail-register"
                >
                  <ShellIcon name="circle-plus" size={13} /> Add this server
                </Link>
              ) : (
                <span className="cat-sub" data-testid="cat-mcp-detail-not-configured">
                  Not in this workspace — ask an admin
                </span>
              ))}
          </div>
        </div>

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

        {registered && (
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
        )}
      </div>

      {registered && isAdmin && <McpConfigureServerFooter prefix="cat-mcp" serverId={registered.id} />}
    </div>
  );
}
