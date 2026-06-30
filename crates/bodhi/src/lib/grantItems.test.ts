import type { AliasResponse } from '@bodhiapp/ts-client';
import { describe, expect, it } from 'vitest';

import { grantableMcpItems, grantableModelItems } from './grantItems';

describe('grantableModelItems', () => {
  it('classifies api / local / router aliases distinctly', () => {
    const aliases = [
      {
        source: 'api',
        id: 'a',
        name: 'A',
        api_format: 'openai',
        base_url: 'http://x',
        models: [{ id: 'gpt-4o' }],
        prefix: 'oa-',
      },
      { source: 'user', alias: 'local-1', repo: 'r', filename: 'f', snapshot: 's' },
      // ModelRouterResponse also carries `alias` — must NOT be bucketed as local.
      { source: 'model_router', id: 'mr', alias: 'router-1', targets: [], strategy: { type: 'fallback' } },
    ] as unknown as AliasResponse[];

    const byId = Object.fromEntries(grantableModelItems(aliases).map((i) => [i.id, i]));

    expect(byId['oa-gpt-4o'].type).toBe('api');
    expect(byId['local-1'].type).toBe('local');
    expect(byId['router-1']).toBeDefined();
    expect(byId['router-1'].type).toBeUndefined();
  });

  it('dedupes by request-facing id', () => {
    const aliases = [
      { source: 'api', id: 'a', models: [{ id: 'm' }], prefix: '' },
      { source: 'api', id: 'b', models: [{ id: 'm' }], prefix: '' },
    ] as unknown as AliasResponse[];
    expect(grantableModelItems(aliases)).toHaveLength(1);
  });
});

describe('grantableMcpItems', () => {
  it('maps instances to id + label', () => {
    const items = grantableMcpItems([{ id: 'i1', name: 'Inst 1' }] as never);
    expect(items).toEqual([{ id: 'i1', label: 'Inst 1' }]);
  });
});
