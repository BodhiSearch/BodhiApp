/**
 * MSW v2 handlers for app access request review/approve/deny endpoints
 */

import {
  ENDPOINT_ACCESS_REQUESTS_APPROVE,
  ENDPOINT_ACCESS_REQUESTS_DENY,
  ENDPOINT_ACCESS_REQUESTS_REVIEW,
} from '@/hooks/useAppAccessRequests';
import type { AccessRequestReviewResponse } from '@/hooks/useAppAccessRequests';

import { INTERNAL_SERVER_ERROR, typedHttp, type components } from '../setup';

// =============================================================================
// Review endpoint handlers
// =============================================================================

/**
 * Mock handler for GET /access-requests/:id/review - success case
 */
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

/**
 * Mock handler for GET /access-requests/:id/review - error case
 */
export function mockAppAccessRequestReviewError(
  id: string,
  {
    code = INTERNAL_SERVER_ERROR.code,
    message = INTERNAL_SERVER_ERROR.message,
    type = INTERNAL_SERVER_ERROR.type,
    status = INTERNAL_SERVER_ERROR.status,
    ...rest
  }: Partial<components['schemas']['ErrorBody']> & { status?: 400 | 401 | 403 | 404 | 410 | 500 } = {},
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

// =============================================================================
// Approve endpoint handlers
// =============================================================================

/**
 * Mock handler for PUT /access-requests/:id/approve - success case
 * Optionally captures the request body for assertion via onBody callback
 */
export function mockAppAccessRequestApprove(
  id: string,
  { stub, onBody }: { stub?: boolean; onBody?: (body: unknown) => void } = {}
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

      return response(200 as const).empty();
    }),
  ];
}

/**
 * Mock handler for PUT /access-requests/:id/approve - error case
 */
export function mockAppAccessRequestApproveError(
  id: string,
  {
    code = INTERNAL_SERVER_ERROR.code,
    message = INTERNAL_SERVER_ERROR.message,
    type = INTERNAL_SERVER_ERROR.type,
    status = INTERNAL_SERVER_ERROR.status,
    ...rest
  }: Partial<components['schemas']['ErrorBody']> & { status?: 400 | 401 | 403 | 404 | 409 | 500 } = {},
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

// =============================================================================
// Deny endpoint handlers
// =============================================================================

/**
 * Mock handler for POST /access-requests/:id/deny - success case
 */
export function mockAppAccessRequestDeny(id: string, { stub }: { stub?: boolean } = {}) {
  let hasBeenCalled = false;

  return [
    typedHttp.post(ENDPOINT_ACCESS_REQUESTS_DENY, async ({ params, response }) => {
      if (params.id !== id) return;
      if (hasBeenCalled && !stub) return;
      hasBeenCalled = true;

      return response(200 as const).empty();
    }),
  ];
}

/**
 * Mock handler for POST /access-requests/:id/deny - error case
 */
export function mockAppAccessRequestDenyError(
  id: string,
  {
    code = INTERNAL_SERVER_ERROR.code,
    message = INTERNAL_SERVER_ERROR.message,
    type = INTERNAL_SERVER_ERROR.type,
    status = INTERNAL_SERVER_ERROR.status,
    ...rest
  }: Partial<components['schemas']['ErrorBody']> & { status?: 400 | 401 | 403 | 404 | 409 | 500 } = {},
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
