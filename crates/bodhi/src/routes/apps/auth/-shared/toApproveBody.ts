import type { ApprovedResources, ConsentScopeInfo } from '@bodhiapp/ts-client';

import type { AccessMode } from '@/components/access-picker';

type Grant = ApprovedResources['models_access'];

/** The owner's consent-screen decisions. */
export interface ApproveGrantState {
  listModels: boolean;
  modelMode: AccessMode;
  models: string[];
  listMcps: boolean;
  mcpExtraMode: AccessMode;
  mcpsExtra: string[];
}

const grant = (mode: AccessMode, ids: string[]): Grant =>
  mode === 'all' ? { type: 'all' as const } : { type: 'specific' as const, ids };

/** Least-privilege grant used when the scope did not request a section. */
const DENY: Grant = { type: 'specific', ids: [] };

// Pure + exported so the grant branch matrix is unit-tested (mirrors `toCreateTokenRequest`).
// Fail-closed: anything outside the requested scope defaults to deny, never all-access.
export function toApproveBody(scope: ConsentScopeInfo, state: ApproveGrantState): ApprovedResources {
  return {
    version: '1',
    models_list: scope.llms ? state.listModels : false,
    models_access: scope.llms ? grant(state.modelMode, state.models) : DENY,
    mcps_list: scope.mcps ? state.listMcps : false,
    // Per-URL MCP requests are gone from the consent flow; the envelope keeps the field.
    mcps: [],
    mcps_access: scope.mcps ? grant(state.mcpExtraMode, state.mcpsExtra) : DENY,
  };
}
