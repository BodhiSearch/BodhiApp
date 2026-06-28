import type { Mcp, McpAuthConfigResponse, McpServerResponse } from '@bodhiapp/ts-client';

import { DetailRail, DetailRailBody, DetailRailRow, DetailRailRows, DetailRailSection } from '@/components/detail-rail';
import { ShellIcon } from '@/components/shell';
import { AuthBadge, authKind } from '@/routes/mcps/-shared/auth-badges';
import {
  McpConfigureServerFooter,
  McpConnectWithSection,
  McpInstancesSection,
} from '@/routes/mcps/-shared/McpRailSections';
import { monogram, tintIndex } from '@/routes/models/explore/-shared/catalog-format';

export function MyMcpsRailHeader({ server, onClose }: { server: McpServerResponse; onClose: () => void }) {
  return (
    <div className="dp-head">
      <div className={`dp-head-icon cat-logo cat-tint-${tintIndex(server.id)}`}>{monogram(server.name)}</div>
      <div className="dp-head-body">
        <div className="dp-head-title">{server.name}</div>
        <div className="dp-head-sub">{server.enabled ? 'MCP server' : 'Disabled by admin'}</div>
      </div>
      <button className="dp-close" onClick={onClose} title="Close" data-testid="my-mcps-detail-close">
        <ShellIcon name="x" size={15} />
      </button>
    </div>
  );
}

interface RailProps {
  server: McpServerResponse;
  instances: Mcp[];
  authConfigs: McpAuthConfigResponse[] | undefined;
  authConfigsLoading: boolean;
  isAdmin: boolean;
  onDeleteInstance: (mcp: Mcp) => void;
}

export function MyMcpsRail({
  server,
  instances,
  authConfigs,
  authConfigsLoading,
  isAdmin,
  onDeleteInstance,
}: RailProps) {
  const serverDisabled = !server.enabled;
  // Supported auth = the unique kinds the server has configured, plus Public (always available).
  const supportedKinds = Array.from(new Set<string>(['public', ...(authConfigs ?? []).map((c) => authKind(c.type))]));

  return (
    <DetailRail testId={`my-mcps-detail-${server.id}`}>
      <DetailRailBody>
        {server.description && (
          <DetailRailSection label="Description">
            <div className="cat-sub" data-testid="my-mcps-detail-description">
              {server.description}
            </div>
          </DetailRailSection>
        )}

        <DetailRailSection label="Server">
          <DetailRailRows testId="my-mcps-detail-server">
            <DetailRailRow k="URL" v={server.url} />
            <DetailRailRow k="Status" v={server.enabled ? 'Enabled' : 'Disabled'} />
            {!authConfigsLoading && (
              <div className="dp-row dp-row-auth">
                <span className="dp-row-k">Supported auth</span>
                <span className="dp-row-auth-badges">
                  {supportedKinds.map((k) => (
                    <AuthBadge key={k} type={k} />
                  ))}
                </span>
              </div>
            )}
          </DetailRailRows>
        </DetailRailSection>

        <McpInstancesSection
          prefix="my-mcps"
          instances={instances}
          serverDisabled={serverDisabled}
          onDeleteInstance={onDeleteInstance}
        />

        {!serverDisabled && (
          <McpConnectWithSection
            prefix="my-mcps"
            serverId={server.id}
            authConfigs={authConfigs}
            loading={authConfigsLoading}
          />
        )}
      </DetailRailBody>

      {isAdmin && <McpConfigureServerFooter prefix="my-mcps" serverId={server.id} />}
    </DetailRail>
  );
}
