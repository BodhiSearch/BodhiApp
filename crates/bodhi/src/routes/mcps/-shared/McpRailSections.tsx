import type { Mcp, McpAuthConfigResponse } from '@bodhiapp/ts-client';
import { Link } from '@tanstack/react-router';

import { ShellIcon } from '@/components/shell';
import { Skeleton } from '@/components/ui/skeleton';
import { ROUTE_MCP_SERVERS } from '@/lib/constants';
import { buildAuthMechanisms } from '@/routes/mcps/-shared/auth-mechanisms';
import '@/routes/mcps/-components/my-mcps.css';
import '@/routes/mcps/-shared/auth-badges.css';

/**
 * The connect/instance/configure sections shared by the My MCPs rail and the Explore rail. Both
 * surface the same per-server actions once a catalog entry resolves to a registered server:
 * My Instances (play/edit/delete), Connect with (auth mechanisms → New-Instance deep-link), and the
 * admin Configure-server footer. `prefix` namespaces the testids per host screen.
 */

export function McpInstancesSection({
  prefix,
  instances,
  serverDisabled,
  onDeleteInstance,
}: {
  prefix: string;
  instances: Mcp[];
  serverDisabled: boolean;
  onDeleteInstance: (mcp: Mcp) => void;
}) {
  return (
    <div className="dp-section">
      <div className="dp-sec-lbl">My Instances</div>
      {instances.length === 0 ? (
        <div className="cat-sub" data-testid={`${prefix}-detail-no-instances`}>
          No instances yet. Connect below to create one.
        </div>
      ) : (
        <div className="mcp-inst-list" data-testid={`${prefix}-detail-instances`}>
          {instances.map((inst) => (
            <div
              key={inst.id}
              className="mcp-inst-row"
              data-testid={`${prefix}-instance-${inst.id}`}
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
                  data-testid={`${prefix}-instance-play-${inst.id}`}
                >
                  <ShellIcon name="play" size={14} />
                </Link>
                <Link
                  to="/mcps/new/"
                  search={{ id: inst.id }}
                  className="mcp-inst-act"
                  title={`Edit ${inst.name}`}
                  data-testid={`${prefix}-instance-edit-${inst.id}`}
                >
                  <ShellIcon name="pencil" size={14} />
                </Link>
                <button
                  type="button"
                  className="mcp-inst-act mcp-inst-act-danger"
                  title={`Delete ${inst.name}`}
                  onClick={() => onDeleteInstance(inst)}
                  data-testid={`${prefix}-instance-delete-${inst.id}`}
                >
                  <ShellIcon name="trash-2" size={14} />
                </button>
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}

const KIND_ICON: Record<string, string> = { oauth: 'lock', key: 'key', public: 'unlock', http: 'shield' };

export function McpConnectWithSection({
  prefix,
  serverId,
  authConfigs,
  loading,
}: {
  prefix: string;
  serverId: string;
  authConfigs: McpAuthConfigResponse[] | undefined;
  loading: boolean;
}) {
  const mechanisms = buildAuthMechanisms(authConfigs);
  return (
    <div className="dp-section">
      <div className="dp-sec-lbl">Connect with</div>
      <div className="mcp-mech-hint">Pick an auth mechanism to create a new instance.</div>
      {loading ? (
        <Skeleton className="h-12 w-full" data-testid={`${prefix}-detail-mechs-skeleton`} />
      ) : (
        <div className="mcp-mech-list" data-testid={`${prefix}-detail-mechanisms`}>
          {mechanisms.map((m) => (
            <Link
              key={m.id}
              to="/mcps/new/"
              search={{ server: serverId, auth: m.id }}
              className="mcp-mech-row"
              data-testid={`${prefix}-connect-${m.id}`}
            >
              <div className={`mcp-mech-icon auth-badge-${m.kind}`}>
                <ShellIcon name={KIND_ICON[m.kind]} size={14} />
              </div>
              <div className="mcp-mech-body">
                <div className="mcp-mech-name">{m.title}</div>
                {m.detail && <div className="mcp-mech-sub">{m.detail}</div>}
              </div>
              <ShellIcon name="chevron-right" size={15} />
            </Link>
          ))}
        </div>
      )}
    </div>
  );
}

export function McpConfigureServerFooter({ prefix, serverId }: { prefix: string; serverId: string }) {
  return (
    <div className="dp-foot">
      <Link
        to={`${ROUTE_MCP_SERVERS}view/`}
        search={{ id: serverId }}
        className="dp-btn dp-btn-outline"
        data-testid={`${prefix}-configure-server`}
      >
        <ShellIcon name="settings" size={15} /> Configure server
      </Link>
    </div>
  );
}
