import {
  ENDPOINT_ACCESS_REQUESTS_APPROVE,
  ENDPOINT_ACCESS_REQUESTS_APPS,
  ENDPOINT_ACCESS_REQUESTS_DENY,
  ENDPOINT_ACCESS_REQUESTS_REVIEW,
  ENDPOINT_ACCESS_REQUESTS_REVOKE,
} from '@/hooks/apps';
import type { AccessRequestReviewResponse, AppAccessSummary, ListAppAccessResponse } from '@/hooks/apps';
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

export function mockAppAccessRequestReview(reviewData: AccessRequestReviewResponse, { stub }: { stub?: boolean } = {}) {
  let hasBeenCalled = false;

  return [
    typedHttp.get(ENDPOINT_ACCESS_REQUESTS_REVIEW, async ({ params, response }) => {
      if (params.id !== reviewData.id) return;
      if (hasBeenCalled && !stub) return;
      hasBeenCalled = true;

      return response(200 as const).json(reviewData);
    }),
  ];
}

export function mockAppAccessRequestReviewError(
  id: string,
  {
    code = INTERNAL_SERVER_ERROR.code,
    message = INTERNAL_SERVER_ERROR.message,
    type = INTERNAL_SERVER_ERROR.type,
    status = INTERNAL_SERVER_ERROR.status,
    ...rest
  }: Partial<components['schemas']['BodhiError']> & { status?: 400 | 401 | 403 | 404 | 410 | 500 } = {},
  { stub }: { stub?: boolean } = {}
) {
  let hasBeenCalled = false;

  return [
    typedHttp.get(ENDPOINT_ACCESS_REQUESTS_REVIEW, async ({ params, response }) => {
      if (params.id !== id) return;
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
export function mockAppAccessRequestApprove(
  id: string,
  {
    stub,
    onBody,
    flowType = 'popup',
    redirectUrl,
  }: { stub?: boolean; onBody?: (body: unknown) => void; flowType?: 'redirect' | 'popup'; redirectUrl?: string } = {}
) {
  let hasBeenCalled = false;

  return [
    typedHttp.put(ENDPOINT_ACCESS_REQUESTS_APPROVE, async ({ params, request, response }) => {
      if (params.id !== id) return;
      if (hasBeenCalled && !stub) return;
      hasBeenCalled = true;

      if (onBody) {
        const body = await request.json();
        onBody(body);
      }

      return response(200 as const).json({
        status: 'approved',
        flow_type: flowType,
        ...(redirectUrl ? { redirect_url: redirectUrl } : {}),
      });
    }),
  ];
}

export function mockAppAccessRequestApproveError(
  id: string,
  {
    code = INTERNAL_SERVER_ERROR.code,
    message = INTERNAL_SERVER_ERROR.message,
    type = INTERNAL_SERVER_ERROR.type,
    status = INTERNAL_SERVER_ERROR.status,
    ...rest
  }: Partial<components['schemas']['BodhiError']> & { status?: 400 | 401 | 403 | 404 | 409 | 500 } = {},
  { stub }: { stub?: boolean } = {}
) {
  let hasBeenCalled = false;

  return [
    typedHttp.put(ENDPOINT_ACCESS_REQUESTS_APPROVE, async ({ params, response }) => {
      if (params.id !== id) return;
      if (hasBeenCalled && !stub) return;
      hasBeenCalled = true;

      const errorData = { code, message, type, ...rest };
      return response(status).json({ error: errorData });
    }),
  ];
}

export function mockAppAccessRequestDeny(
  id: string,
  {
    stub,
    flowType = 'popup',
    redirectUrl,
  }: { stub?: boolean; flowType?: 'redirect' | 'popup'; redirectUrl?: string } = {}
) {
  let hasBeenCalled = false;

  return [
    typedHttp.post(ENDPOINT_ACCESS_REQUESTS_DENY, async ({ params, response }) => {
      if (params.id !== id) return;
      if (hasBeenCalled && !stub) return;
      hasBeenCalled = true;

      return response(200 as const).json({
        status: 'denied',
        flow_type: flowType,
        ...(redirectUrl ? { redirect_url: redirectUrl } : {}),
      });
    }),
  ];
}

export function mockAppAccessRequestDenyError(
  id: string,
  {
    code = INTERNAL_SERVER_ERROR.code,
    message = INTERNAL_SERVER_ERROR.message,
    type = INTERNAL_SERVER_ERROR.type,
    status = INTERNAL_SERVER_ERROR.status,
    ...rest
  }: Partial<components['schemas']['BodhiError']> & { status?: 400 | 401 | 403 | 404 | 409 | 500 } = {},
  { stub }: { stub?: boolean } = {}
) {
  let hasBeenCalled = false;

  return [
    typedHttp.post(ENDPOINT_ACCESS_REQUESTS_DENY, async ({ params, response }) => {
      if (params.id !== id) return;
      if (hasBeenCalled && !stub) return;
      hasBeenCalled = true;

      const errorData = { code, message, type, ...rest };
      return response(status).json({ error: errorData });
    }),
  ];
}
