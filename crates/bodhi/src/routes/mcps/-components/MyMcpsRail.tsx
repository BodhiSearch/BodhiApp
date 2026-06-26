import type { Mcp, McpAuthConfigResponse, McpServerResponse } from '@bodhiapp/ts-client';
import { Link } from '@tanstack/react-router';

import { ShellIcon } from '@/components/shell';
import { Skeleton } from '@/components/ui/skeleton';
import { ROUTE_MCP_SERVERS } from '@/lib/constants';
import { buildAuthMechanisms } from '@/routes/mcps/-shared/auth-mechanisms';
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
  const mechanisms = buildAuthMechanisms(authConfigs);
  const serverDisabled = !server.enabled;

  return (
    <div className="dp-panel" data-testid={`my-mcps-detail-${server.id}`}>
      <div className="dp-body">
        <div className="dp-section">
          <div className="dp-sec-lbl">Server</div>
          <div className="dp-rows" data-testid="my-mcps-detail-server">
            <Row k="Endpoint" v={server.url} />
            <Row k="Status" v={server.enabled ? 'Enabled' : 'Disabled'} />
          </div>
        </div>

        {server.description && (
          <div className="dp-section">
            <div className="dp-sec-lbl">Description</div>
            <div className="cat-sub" data-testid="my-mcps-detail-description">
              {server.description}
            </div>
          </div>
        )}

        <div className="dp-section">
          <div className="dp-sec-lbl">My Instances</div>
          {instances.length === 0 ? (
            <div className="cat-sub" data-testid="my-mcps-detail-no-instances">
              No instances yet. Connect below to create one.
            </div>
          ) : (
            <div className="mcp-inst-list" data-testid="my-mcps-detail-instances">
              {instances.map((inst) => (
                <div
                  key={inst.id}
                  className="mcp-inst-row"
                  data-testid={`my-mcps-instance-${inst.id}`}
                  data-test-instance-name={inst.name}
                  data-test-uuid={inst.id}
                >
                  <span className={`mcp-inst-dot mcp-inst-dot-${inst.enabled ? 'on' : 'off'}`} />
                  <div className="mcp-inst-body">
                    <div className="mcp-inst-name">{inst.name}</div>
                    <div className="mcp-inst-sub mono">{inst.auth_type}</div>
                  </div>
                  <div className="mcp-inst-actions">
                    <Link
                      to="/mcps/playground/"
                      search={{ id: inst.id }}
                      className="mcp-inst-act"
                      title={serverDisabled ? 'Server is disabled' : `Open playground for ${inst.name}`}
                      aria-disabled={serverDisabled}
                      data-testid={`my-mcps-instance-play-${inst.id}`}
                    >
                      <ShellIcon name="play" size={14} />
                    </Link>
                    <Link
                      to="/mcps/new/"
                      search={{ id: inst.id }}
                      className="mcp-inst-act"
                      title={`Edit ${inst.name}`}
                      data-testid={`my-mcps-instance-edit-${inst.id}`}
                    >
                      <ShellIcon name="pencil" size={14} />
                    </Link>
                    <button
                      type="button"
                      className="mcp-inst-act mcp-inst-act-danger"
                      title={`Delete ${inst.name}`}
                      onClick={() => onDeleteInstance(inst)}
                      data-testid={`my-mcps-instance-delete-${inst.id}`}
                    >
                      <ShellIcon name="trash-2" size={14} />
                    </button>
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>

        {!serverDisabled && (
          <div className="dp-section">
            <div className="dp-sec-lbl">Connect with</div>
            {authConfigsLoading ? (
              <Skeleton className="h-12 w-full" data-testid="my-mcps-detail-mechs-skeleton" />
            ) : (
              <div className="mcp-mech-list" data-testid="my-mcps-detail-mechanisms">
                {mechanisms.map((m) => (
                  <Link
                    key={m.id}
                    to="/mcps/new/"
                    search={{ server: server.id, auth: m.id }}
                    className="mcp-mech-row"
                    data-testid={`my-mcps-connect-${m.id}`}
                  >
                    <div className="mcp-mech-body">
                      <div className="mcp-mech-name">{m.label}</div>
                      {m.detail && <div className="mcp-mech-sub">{m.detail}</div>}
                    </div>
                    <ShellIcon name="chevron-right" size={15} />
                  </Link>
                ))}
              </div>
            )}
          </div>
        )}
      </div>

      {isAdmin && (
        <div className="dp-foot">
          <Link
            to={`${ROUTE_MCP_SERVERS}view/`}
            search={{ id: server.id }}
            className="dp-btn dp-btn-outline"
            data-testid="my-mcps-configure-server"
          >
            <ShellIcon name="settings" size={15} /> Configure server
          </Link>
        </div>
      )}
    </div>
  );
}
