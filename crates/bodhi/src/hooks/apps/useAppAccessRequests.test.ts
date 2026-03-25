import type { AccessRequestReviewResponse, OpenAiApiError } from '@bodhiapp/ts-client';
import { act, renderHook, waitFor } from '@testing-library/react';
import { AxiosError } from 'axios';
import { afterEach, describe, expect, it, vi } from 'vitest';

import { useGetAppAccessRequestReview, useApproveAppAccessRequest, useDenyAppAccessRequest } from '@/hooks/apps';
import { mockDraftReviewResponse, MOCK_REQUEST_ID } from '@/test-fixtures/apps';
import {
  mockAppAccessRequestReview,
  mockAppAccessRequestReviewError,
  mockAppAccessRequestApprove,
  mockAppAccessRequestApproveError,
  mockAppAccessRequestDeny,
  mockAppAccessRequestDenyError,
} from '@/test-utils/msw-v2/handlers/apps';
import { setupMswV2, server } from '@/test-utils/msw-v2/setup';
import { createWrapper } from '@/tests/wrapper';

setupMswV2();

afterEach(() => server.resetHandlers());

describe('useGetAppAccessRequestReview', () => {
  it('fetches review data successfully for draft status', async () => {
    server.use(...mockAppAccessRequestReview(mockDraftReviewResponse));

    const { result } = renderHook(() => useGetAppAccessRequestReview(MOCK_REQUEST_ID), {
      wrapper: createWrapper(),
    });

    await waitFor(() => {
      expect(result.current.isSuccess).toBe(true);
    });

    expect(result.current.data?.id).toBe(MOCK_REQUEST_ID);
    expect(result.current.data?.status).toBe('draft');
    expect(result.current.data?.app_name).toBe('Test Application');
    expect(result.current.data?.mcps_info).toHaveLength(1);
  });

  it('does not fetch when id is null (disabled query)', async () => {
    const { result } = renderHook(() => useGetAppAccessRequestReview(null), {
      wrapper: createWrapper(),
    });

    // Query should not be fetching because enabled: !!id = false
    expect(result.current.isFetching).toBe(false);
    expect(result.current.data).toBeUndefined();
  });

  it('handles error response', async () => {
    server.use(
      ...mockAppAccessRequestReviewError(MOCK_REQUEST_ID, {
        message: 'Access request not found',
        status: 404,
      })
    );

    const { result } = renderHook(() => useGetAppAccessRequestReview(MOCK_REQUEST_ID), {
      wrapper: createWrapper(),
    });

    await waitFor(() => {
      expect(result.current.isError).toBe(true);
    });

    const error = result.current.error as AxiosError<OpenAiApiError>;
    expect(error.response?.status).toBe(404);
  });
});

describe('useApproveAppAccessRequest', () => {
  it('approves request successfully and calls onSuccess', async () => {
    const onSuccess = vi.fn();
    server.use(...mockAppAccessRequestApprove(MOCK_REQUEST_ID));

    const { result } = renderHook(() => useApproveAppAccessRequest({ onSuccess }), {
      wrapper: createWrapper(),
    });

    await act(async () => {
      await result.current.mutateAsync({
        id: MOCK_REQUEST_ID,
        body: {
          approved_role: 'scope_user_user',
          approved: {
            version: '1' as const,
            mcps: [],
          },
        },
      });
    });

    expect(onSuccess).toHaveBeenCalledWith(
      expect.objectContaining({
        status: 'approved',
        flow_type: 'popup',
      })
    );
  });

  it('calls onError on approval failure', async () => {
    const onError = vi.fn();
    server.use(
      ...mockAppAccessRequestApproveError(MOCK_REQUEST_ID, {
        message: 'Conflict: request already processed',
        status: 409,
      })
    );

    const { result } = renderHook(() => useApproveAppAccessRequest({ onError }), {
      wrapper: createWrapper(),
    });

    await act(async () => {
      try {
        await result.current.mutateAsync({
          id: MOCK_REQUEST_ID,
          body: {
            approved_role: 'scope_user_user',
            approved: {
              version: '1' as const,
              mcps: [],
            },
          },
        });
      } catch {
        /* expected */
      }
    });

    expect(onError).toHaveBeenCalledWith('Conflict: request already processed');
  });
});

describe('useDenyAppAccessRequest', () => {
  it('denies request successfully and calls onSuccess', async () => {
    const onSuccess = vi.fn();
    server.use(...mockAppAccessRequestDeny(MOCK_REQUEST_ID));

    const { result } = renderHook(() => useDenyAppAccessRequest({ onSuccess }), {
      wrapper: createWrapper(),
    });

    await act(async () => {
      await result.current.mutateAsync({ id: MOCK_REQUEST_ID });
    });

    expect(onSuccess).toHaveBeenCalledWith(
      expect.objectContaining({
        status: 'denied',
        flow_type: 'popup',
      })
    );
  });

  it('calls onError on denial failure', async () => {
    const onError = vi.fn();
    server.use(
      ...mockAppAccessRequestDenyError(MOCK_REQUEST_ID, {
        message: 'Failed to deny access request',
        status: 500,
      })
    );

    const { result } = renderHook(() => useDenyAppAccessRequest({ onError }), {
      wrapper: createWrapper(),
    });

    await act(async () => {
      try {
        await result.current.mutateAsync({ id: MOCK_REQUEST_ID });
      } catch {
        /* expected */
      }
    });

    expect(onError).toHaveBeenCalledWith('Failed to deny access request');
  });
});
