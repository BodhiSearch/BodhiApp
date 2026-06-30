import { describe, expect, it } from 'vitest';

import { toApproveBody, type ApproveGrantState, type RequestedFlags } from './toApproveBody';

const reqAll: RequestedFlags = {
  version: '1',
  models_list: true,
  models_access: true,
  mcps_list: true,
  mcps_access: true,
};

const reqNone: RequestedFlags = {
  version: '1',
  models_list: false,
  models_access: false,
  mcps_list: false,
  mcps_access: false,
};

const baseState: ApproveGrantState = {
  listModels: false,
  modelMode: 'specific',
  models: [],
  listMcps: false,
  mcpExtraMode: 'specific',
  mcpsExtra: [],
  approvedMcps: {},
  selectedMcpInstances: {},
};

describe('toApproveBody — models_access', () => {
  it('requested + All → {type:all}', () => {
    const out = toApproveBody(reqAll, [], { ...baseState, modelMode: 'all' });
    expect(out.models_access).toEqual({ type: 'all' });
  });

  it('requested + Specific → {type:specific, ids}', () => {
    const out = toApproveBody(reqAll, [], { ...baseState, modelMode: 'specific', models: ['m1', 'm2'] });
    expect(out.models_access).toEqual({ type: 'specific', ids: ['m1', 'm2'] });
  });

  it('NOT requested → deny (empty specific), never all', () => {
    // Even if the owner state says modelMode:'all', a model selector the app did not
    // request must not silently grant all models.
    const out = toApproveBody(reqNone, [], { ...baseState, modelMode: 'all', models: ['m1'] });
    expect(out.models_access).toEqual({ type: 'specific', ids: [] });
  });
});

describe('toApproveBody — mcps_access', () => {
  it('requested + All → {type:all}', () => {
    const out = toApproveBody(reqAll, [], { ...baseState, mcpExtraMode: 'all' });
    expect(out.mcps_access).toEqual({ type: 'all' });
  });

  it('requested + Specific (empty) → {type:specific, ids:[]}', () => {
    const out = toApproveBody(reqAll, [], { ...baseState, mcpExtraMode: 'specific', mcpsExtra: [] });
    expect(out.mcps_access).toEqual({ type: 'specific', ids: [] });
  });

  it('requested + Specific (non-empty) → {type:specific, ids}', () => {
    const out = toApproveBody(reqAll, [], { ...baseState, mcpExtraMode: 'specific', mcpsExtra: ['mcp-1'] });
    expect(out.mcps_access).toEqual({ type: 'specific', ids: ['mcp-1'] });
  });

  it('NOT requested → deny (empty specific)', () => {
    const out = toApproveBody(reqNone, [], { ...baseState, mcpExtraMode: 'all', mcpsExtra: ['mcp-1'] });
    expect(out.mcps_access).toEqual({ type: 'specific', ids: [] });
  });
});

describe('toApproveBody — list flags gated by the request', () => {
  it('list toggles pass through only when the app requested them', () => {
    const on = toApproveBody(reqAll, [], { ...baseState, listModels: true, listMcps: true });
    expect(on.models_list).toBe(true);
    expect(on.mcps_list).toBe(true);

    const off = toApproveBody(reqNone, [], { ...baseState, listModels: true, listMcps: true });
    expect(off.models_list).toBe(false);
    expect(off.mcps_list).toBe(false);
  });

  it('passes through the requested version', () => {
    expect(toApproveBody(reqAll, [], baseState).version).toBe('1');
  });
});

describe('toApproveBody — mcps approvals', () => {
  const mcpsInfo = [
    { url: 'https://a/mcp', instances: [{ id: 'a1', path: '/p/a1' }] },
    { url: 'https://b/mcp', instances: [{ id: 'b1', path: '/p/b1' }] },
  ];

  it('approved with a selected instance carries {id, path}; denied carries no instance', () => {
    const out = toApproveBody(reqAll, mcpsInfo, {
      ...baseState,
      approvedMcps: { 'https://a/mcp': true, 'https://b/mcp': false },
      selectedMcpInstances: { 'https://a/mcp': 'a1' },
    });
    expect(out.mcps).toEqual([
      { url: 'https://a/mcp', status: 'approved', instance: { id: 'a1', path: '/p/a1' } },
      { url: 'https://b/mcp', status: 'denied', instance: undefined },
    ]);
  });

  it('approved but no instance selected → approved with undefined instance', () => {
    const out = toApproveBody(reqAll, mcpsInfo.slice(0, 1), {
      ...baseState,
      approvedMcps: { 'https://a/mcp': true },
      selectedMcpInstances: {},
    });
    expect(out.mcps).toEqual([{ url: 'https://a/mcp', status: 'approved', instance: undefined }]);
  });
});
