import type { McpGrant, ModelGrant, PreviousGrantInfo, UserScope } from '@bodhiapp/ts-client';

import type { AccessMode } from '@/components/access-picker';

/** Review-form state pre-selected from a prior grant (upgrade/exchange mode). */
export interface PreselectState {
  listModels: boolean;
  modelMode: AccessMode;
  models: string[];
  listMcps: boolean;
  mcpExtraMode: AccessMode;
  mcpsExtra: string[];
  approvedMcps: Record<string, boolean>;
  selectedMcpInstances: Record<string, string>;
  approvedRole: UserScope;
}

// Serde defaults make these optional; a missing grant means least-privilege deny.
const DENY: ModelGrant = { type: 'specific', ids: [] };

const fromGrant = (g: ModelGrant | McpGrant | undefined): { mode: AccessMode; ids: string[] } => {
  const grant = g ?? DENY;
  return grant.type === 'all' ? { mode: 'all', ids: [] } : { mode: 'specific', ids: grant.ids };
};

/** Map a prior approved grant onto the review form's initial state. */
export function previousGrantToState(previous: PreviousGrantInfo): PreselectState {
  const approved = previous.approved;
  const modelGrant = fromGrant(approved.models_access);
  const mcpGrant = fromGrant(approved.mcps_access);

  const approvedMcps: Record<string, boolean> = {};
  const selectedMcpInstances: Record<string, string> = {};
  for (const mcp of approved.mcps ?? []) {
    approvedMcps[mcp.url] = mcp.status === 'approved';
    if (mcp.status === 'approved' && mcp.instance) {
      selectedMcpInstances[mcp.url] = mcp.instance.id;
    }
  }

  return {
    listModels: approved.models_list ?? false,
    modelMode: modelGrant.mode,
    models: modelGrant.ids,
    listMcps: approved.mcps_list ?? false,
    mcpExtraMode: mcpGrant.mode,
    mcpsExtra: mcpGrant.ids,
    approvedMcps,
    selectedMcpInstances,
    approvedRole: previous.approved_role,
  };
}
