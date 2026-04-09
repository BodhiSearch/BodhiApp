/**
 * Type-safe MSW v2 handlers for tenants endpoints using openapi-msw
 */
import { ENDPOINT_TENANTS } from '@/hooks/tenants';

import { typedHttp, type components, INTERNAL_SERVER_ERROR } from '../setup';

export function mockTenantsList(
  tenants: components['schemas']['TenantListItem'][] = [],
  { stub }: { stub?: boolean } = {}
) {
  let hasBeenCalled = false;
  return [
    typedHttp.get(ENDPOINT_TENANTS, ({ response }) => {
      if (hasBeenCalled && !stub) return;
      hasBeenCalled = true;
      return response(200 as const).json({ tenants });
    }),
  ];
}

export function mockTenantsListError(
  {
    code = INTERNAL_SERVER_ERROR.code,
    message = 'Failed to fetch tenants',
    type = INTERNAL_SERVER_ERROR.type,
    status = INTERNAL_SERVER_ERROR.status,
    ...rest
  }: Partial<components['schemas']['BodhiErrorBody']> & { status?: 400 | 401 | 403 | 500 } = {},
  { stub }: { stub?: boolean } = {}
) {
  let hasBeenCalled = false;
  return [
    typedHttp.get(ENDPOINT_TENANTS, ({ response }) => {
      if (hasBeenCalled && !stub) return;
      hasBeenCalled = true;
      const errorData = {
        code,
        message,
        type,
        ...rest,
      };
      return response(status).json({ error: errorData });
    }),
  ];
}
