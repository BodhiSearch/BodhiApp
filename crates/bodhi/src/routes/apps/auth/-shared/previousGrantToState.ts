import type { ConsentPriorGrant, McpGrant, ModelGrant, UserScope } from '@bodhiapp/ts-client';

import type { AccessMode } from '@/components/access-picker';

/** Consent-form state pre-selected from a prior grant (reauthorization). */
export interface PreselectState {
  listModels: boolean;
  modelMode: AccessMode;
  models: string[];
  listMcps: boolean;
  mcpExtraMode: AccessMode;
  mcpsExtra: string[];
  approvedRole: UserScope;
}

// Serde defaults make these optional; a missing grant means least-privilege deny.
const DENY: ModelGrant = { type: 'specific', ids: [] };

const fromGrant = (g: ModelGrant | McpGrant | undefined): { mode: AccessMode; ids: string[] } => {
  const grant = g ?? DENY;
  return grant.type === 'all' ? { mode: 'all', ids: [] } : { mode: 'specific', ids: grant.ids };
};

export function previousGrantToState(previous: ConsentPriorGrant): PreselectState {
  const approved = previous.approved;
  const modelGrant = fromGrant(approved.models_access);
  const mcpGrant = fromGrant(approved.mcps_access);

  return {
    listModels: approved.models_list ?? false,
    modelMode: modelGrant.mode,
    models: modelGrant.ids,
    listMcps: approved.mcps_list ?? false,
    mcpExtraMode: mcpGrant.mode,
    mcpsExtra: mcpGrant.ids,
    approvedRole: previous.approved_role,
  };
}
