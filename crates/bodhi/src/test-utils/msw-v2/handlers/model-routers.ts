import { ENDPOINT_MODEL_ROUTERS, ENDPOINT_MODEL_ROUTER_ID } from '@/hooks/models';

import { typedHttp, type components } from '../setup';

export function mockCreateModelRouter(
  {
    source = 'model_router',
    id = 'router-123',
    alias = 'my-stack',
    targets = [],
    strategy = { strategy: 'fallback', cooldown_secs: 30, max_attempts: 0, honor_retry_after: true },
    created_at = new Date().toISOString(),
    updated_at = new Date().toISOString(),
  }: Partial<components['schemas']['ModelRouterResponse']> = {},
  { stub }: { stub?: boolean } = {}
) {
  let called = false;
  return [
    typedHttp.post(ENDPOINT_MODEL_ROUTERS, async ({ response }) => {
      if (called && !stub) return;
      called = true;
      const body: components['schemas']['ModelRouterResponse'] = {
        source,
        id,
        alias,
        targets,
        strategy,
        created_at,
        updated_at,
      };
      return response(201 as const).json(body);
    }),
  ];
}

export function mockGetModelRouter(
  id: string,
  data: Partial<components['schemas']['ModelRouterResponse']> = {},
  { stub }: { stub?: boolean } = {}
) {
  let called = false;
  return [
    typedHttp.get(ENDPOINT_MODEL_ROUTER_ID, async ({ response }) => {
      if (called && !stub) return;
      called = true;
      const body: components['schemas']['ModelRouterResponse'] = {
        source: 'model_router',
        id,
        alias: 'my-stack',
        targets: [],
        strategy: { strategy: 'fallback', cooldown_secs: 30, max_attempts: 0, honor_retry_after: true },
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString(),
        ...data,
      };
      return response(200 as const).json(body);
    }),
  ];
}
