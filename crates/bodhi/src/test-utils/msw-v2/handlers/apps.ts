import {
  ENDPOINT_ACCESS_REQUESTS_APPS,
  ENDPOINT_ACCESS_REQUESTS_REVOKE,
  ENDPOINT_APPS_ACCESS_REQUESTS,
  ENDPOINT_APPS_ACCESS_REQUESTS_CONSENT,
} from '@/hooks/apps';
import type {
  AppAccessSummary,
  ConsentContextResponse,
  ListAppAccessResponse,
  SubmitConsentResponse,
} from '@/hooks/apps';
import { INTERNAL_SERVER_ERROR, typedHttp, type components } from '@/test-utils/msw-v2/setup';

export function mockListAppAccess(data: ListAppAccessResponse, { stub = true }: { stub?: boolean } = {}) {
  let hasBeenCalled = false;
  return [
    typedHttp.get(ENDPOINT_ACCESS_REQUESTS_APPS, async ({ response }) => {
      if (hasBeenCalled && !stub) return;
      hasBeenCalled = true;
      return response(200 as const).json(data);
    }),
  ];
}

export function mockRevokeAppAccess(revoked: AppAccessSummary, { stub }: { stub?: boolean } = {}) {
  let hasBeenCalled = false;
  return [
    typedHttp.post(ENDPOINT_ACCESS_REQUESTS_REVOKE, async ({ params, response }) => {
      if (params.id !== revoked.id) return;
      if (hasBeenCalled && !stub) return;
      hasBeenCalled = true;
      return response(200 as const).json(revoked);
    }),
  ];
}

/**
 * onUrl, when provided, captures the full request URL so tests can assert the raw
 * query string reached the server without re-encoding.
 */
export function mockConsentContext(
  body: ConsentContextResponse,
  { stub = true, onUrl }: { stub?: boolean; onUrl?: (url: string) => void } = {}
) {
  let hasBeenCalled = false;
  return [
    typedHttp.get(ENDPOINT_APPS_ACCESS_REQUESTS_CONSENT, async ({ request, response }) => {
      if (hasBeenCalled && !stub) return;
      hasBeenCalled = true;
      onUrl?.(request.url);
      return response(200 as const).json(body);
    }),
  ];
}

export function mockConsentContextError(
  {
    code = INTERNAL_SERVER_ERROR.code,
    message = INTERNAL_SERVER_ERROR.message,
    type = INTERNAL_SERVER_ERROR.type,
    status = INTERNAL_SERVER_ERROR.status,
    ...rest
  }: Partial<components['schemas']['BodhiError']> & { status?: 400 | 401 | 403 | 500 } = {},
  { stub }: { stub?: boolean } = {}
) {
  let hasBeenCalled = false;
  return [
    typedHttp.get(ENDPOINT_APPS_ACCESS_REQUESTS_CONSENT, async ({ response }) => {
      if (hasBeenCalled && !stub) return;
      hasBeenCalled = true;
      const errorData = { code, message, type, ...rest };
      return response(status).json({ error: errorData });
    }),
  ];
}

/**
 * onBody, when provided, captures the request body for assertion.
 */
export function mockSubmitConsent(
  data: SubmitConsentResponse,
  { stub, onBody, status = 200 }: { stub?: boolean; onBody?: (body: unknown) => void; status?: 200 | 201 } = {}
) {
  let hasBeenCalled = false;
  return [
    typedHttp.post(ENDPOINT_APPS_ACCESS_REQUESTS, async ({ request, response }) => {
      if (hasBeenCalled && !stub) return;
      hasBeenCalled = true;
      if (onBody) {
        const body = await request.json();
        onBody(body);
      }
      return response(status).json(data);
    }),
  ];
}

export function mockSubmitConsentError(
  {
    code = INTERNAL_SERVER_ERROR.code,
    message = INTERNAL_SERVER_ERROR.message,
    type = INTERNAL_SERVER_ERROR.type,
    status = INTERNAL_SERVER_ERROR.status,
    ...rest
  }: Partial<components['schemas']['BodhiError']> & { status?: 400 | 401 | 403 | 500 } = {},
  { stub }: { stub?: boolean } = {}
) {
  let hasBeenCalled = false;
  return [
    typedHttp.post(ENDPOINT_APPS_ACCESS_REQUESTS, async ({ response }) => {
      if (hasBeenCalled && !stub) return;
      hasBeenCalled = true;
      const errorData = { code, message, type, ...rest };
      return response(status).json({ error: errorData });
    }),
  ];
}
