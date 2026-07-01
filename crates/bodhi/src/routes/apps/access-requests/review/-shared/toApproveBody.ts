import type { RequestedResources } from '@bodhiapp/ts-client';

import type { AccessMode } from '@/components/access-picker';
import type { ApproveAccessRequest } from '@/hooks/apps';

type Approved = ApproveAccessRequest['approved'];
type Grant = Approved['models_access'];

/** The app's requested UI-driver flags (from `reviewData.requested`). */
export type RequestedFlags = Pick<
  RequestedResources,
  'version' | 'models_list' | 'models_access' | 'mcps_list' | 'mcps_access'
>;

/** Minimal shape of a requested MCP server + its candidate instances. */
export interface McpInfoLike {
  url: string;
  instances: { id: string; path: string }[];
}

/** The owner's consent-screen decisions. */
export interface ApproveGrantState {
  listModels: boolean;
  modelMode: AccessMode;
  models: string[];
  listMcps: boolean;
  mcpExtraMode: AccessMode;
  mcpsExtra: string[];
  approvedMcps: Record<string, boolean>;
  selectedMcpInstances: Record<string, string>;
}

const grant = (mode: AccessMode, ids: string[]): Grant =>
  mode === 'all' ? { type: 'all' as const } : { type: 'specific' as const, ids };

/** Least-privilege grant used when the app did not request a selector. */
const DENY: Grant = { type: 'specific', ids: [] };

/**
 * Build the `approved` envelope for an access-request approval from the app's
 * requested flags and the owner's decisions. Pure + exported so the branch matrix
 * (which decides what a 3rd-party app is granted on a hurried Approve) is unit-tested,
 * mirroring `toCreateTokenRequest`.
 *
 * Defaults are fail-closed: a list toggle the app didn't request stays off, and a
 * grant the app didn't request defaults to deny (empty `specific`), never all-access.
 */
export function toApproveBody(req: RequestedFlags, mcpsInfo: McpInfoLike[], state: ApproveGrantState): Approved {
  return {
    version: req.version,
    models_list: req.models_list ? state.listModels : false,
    models_access: req.models_access ? grant(state.modelMode, state.models) : DENY,
    mcps_list: req.mcps_list ? state.listMcps : false,
    mcps: mcpsInfo.map((mcp) => ({
      url: mcp.url,
      status: state.approvedMcps[mcp.url] ? ('approved' as const) : ('denied' as const),
      instance:
        state.approvedMcps[mcp.url] && state.selectedMcpInstances[mcp.url]
          ? (() => {
              const inst = mcp.instances.find((i) => i.id === state.selectedMcpInstances[mcp.url]);
              return inst ? { id: inst.id, path: inst.path } : undefined;
            })()
          : undefined,
    })),
    mcps_access: req.mcps_access ? grant(state.mcpExtraMode, state.mcpsExtra) : DENY,
  };
}
