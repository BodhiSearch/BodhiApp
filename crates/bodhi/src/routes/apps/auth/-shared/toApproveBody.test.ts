import { describe, expect, it } from 'vitest';

import type { ConsentScopeInfo } from '@bodhiapp/ts-client';

import { toApproveBody, type ApproveGrantState } from './toApproveBody';

const reqAll: ConsentScopeInfo = { role: 'scope_user_user', llms: true, mcps: true, passthrough: [] };

const reqNone: ConsentScopeInfo = { role: 'scope_user_user', llms: false, mcps: false, passthrough: [] };

const baseState: ApproveGrantState = {
  listModels: false,
  modelMode: 'specific',
  models: [],
  listMcps: false,
  mcpExtraMode: 'specific',
  mcpsExtra: [],
};

describe('toApproveBody — models_access', () => {
  it('requested + All → {type:all}', () => {
    const out = toApproveBody(reqAll, { ...baseState, modelMode: 'all' });
    expect(out.models_access).toEqual({ type: 'all' });
  });

  it('requested + Specific → {type:specific, ids}', () => {
    const out = toApproveBody(reqAll, { ...baseState, modelMode: 'specific', models: ['m1', 'm2'] });
    expect(out.models_access).toEqual({ type: 'specific', ids: ['m1', 'm2'] });
  });

  it('NOT requested → deny (empty specific), never all', () => {
    // Fail-closed: owner state alone must not grant access outside the requested scope.
    const out = toApproveBody(reqNone, { ...baseState, modelMode: 'all', models: ['m1'] });
    expect(out.models_access).toEqual({ type: 'specific', ids: [] });
  });
});

describe('toApproveBody — mcps_access', () => {
  it('requested + All → {type:all}', () => {
    const out = toApproveBody(reqAll, { ...baseState, mcpExtraMode: 'all' });
    expect(out.mcps_access).toEqual({ type: 'all' });
  });

  it('requested + Specific (empty) → {type:specific, ids:[]}', () => {
    const out = toApproveBody(reqAll, { ...baseState, mcpExtraMode: 'specific', mcpsExtra: [] });
    expect(out.mcps_access).toEqual({ type: 'specific', ids: [] });
  });

  it('requested + Specific (non-empty) → {type:specific, ids}', () => {
    const out = toApproveBody(reqAll, { ...baseState, mcpExtraMode: 'specific', mcpsExtra: ['mcp-1'] });
    expect(out.mcps_access).toEqual({ type: 'specific', ids: ['mcp-1'] });
  });

  it('NOT requested → deny (empty specific)', () => {
    const out = toApproveBody(reqNone, { ...baseState, mcpExtraMode: 'all', mcpsExtra: ['mcp-1'] });
    expect(out.mcps_access).toEqual({ type: 'specific', ids: [] });
  });
});

describe('toApproveBody — list flags gated by the scope', () => {
  it('list toggles pass through only when the scope requested them', () => {
    const on = toApproveBody(reqAll, { ...baseState, listModels: true, listMcps: true });
    expect(on.models_list).toBe(true);
    expect(on.mcps_list).toBe(true);

    const off = toApproveBody(reqNone, { ...baseState, listModels: true, listMcps: true });
    expect(off.models_list).toBe(false);
    expect(off.mcps_list).toBe(false);
  });

  it('emits envelope version 1', () => {
    expect(toApproveBody(reqAll, baseState).version).toBe('1');
  });
});

describe('toApproveBody — per-url mcps always empty', () => {
  it('emits an empty mcps array regardless of state', () => {
    expect(toApproveBody(reqAll, baseState).mcps).toEqual([]);
    expect(toApproveBody(reqNone, baseState).mcps).toEqual([]);
  });
});
