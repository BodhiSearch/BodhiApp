import type { ConsentPriorGrant } from '@bodhiapp/ts-client';
import { describe, expect, it } from 'vitest';

import { previousGrantToState } from './previousGrantToState';

const grant = (
  approved: ConsentPriorGrant['approved'],
  role: ConsentPriorGrant['approved_role'] = 'scope_user_user'
): ConsentPriorGrant => ({
  id: 'prior-grant-1',
  approved_role: role,
  approved,
  source: 'explicit',
});

describe('previousGrantToState', () => {
  it('maps an all-access grant to mode "all" with no ids', () => {
    const state = previousGrantToState(
      grant({
        version: '1',
        models_list: true,
        models_access: { type: 'all' },
        mcps_list: true,
        mcps: [],
        mcps_access: { type: 'all' },
      })
    );
    expect(state.listModels).toBe(true);
    expect(state.modelMode).toBe('all');
    expect(state.models).toEqual([]);
    expect(state.listMcps).toBe(true);
    expect(state.mcpExtraMode).toBe('all');
    expect(state.mcpsExtra).toEqual([]);
    expect(state.approvedRole).toBe('scope_user_user');
  });

  it('maps a specific grant to mode "specific" carrying the ids', () => {
    const state = previousGrantToState(
      grant(
        {
          version: '1',
          models_list: false,
          models_access: { type: 'specific', ids: ['model-a', 'model-b'] },
          mcps_list: false,
          mcps: [],
          mcps_access: { type: 'specific', ids: ['mcp-x'] },
        },
        'scope_user_power_user'
      )
    );
    expect(state.listModels).toBe(false);
    expect(state.modelMode).toBe('specific');
    expect(state.models).toEqual(['model-a', 'model-b']);
    expect(state.mcpExtraMode).toBe('specific');
    expect(state.mcpsExtra).toEqual(['mcp-x']);
    expect(state.approvedRole).toBe('scope_user_power_user');
  });

  it('missing grants default to least-privilege deny (empty specific)', () => {
    const state = previousGrantToState(grant({ version: '1' }));
    expect(state.listModels).toBe(false);
    expect(state.modelMode).toBe('specific');
    expect(state.models).toEqual([]);
    expect(state.listMcps).toBe(false);
    expect(state.mcpExtraMode).toBe('specific');
    expect(state.mcpsExtra).toEqual([]);
  });
});
