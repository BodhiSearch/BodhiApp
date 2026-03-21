import type { PaginatedUserAccessResponse, UserAccessStatusResponse, OpenAiApiError } from '@bodhiapp/ts-client';
import { act, renderHook, waitFor } from '@testing-library/react';
import { AxiosError } from 'axios';
import { afterEach, describe, expect, it, vi } from 'vitest';

import {
  useGetRequestStatus,
  useSubmitAccessRequest,
  useListPendingRequests,
  useListAllRequests,
  useApproveRequest,
  useRejectRequest,
} from '@/hooks/users';
import {
  mockUserRequestStatus,
  mockUserRequestStatusError,
  mockUserRequestStatusPending,
  mockUserRequestStatusApproved,
  mockUserRequestStatusRejected,
  mockUserRequestAccess,
  mockUserRequestAccessError,
  mockAccessRequestsPending,
  mockAccessRequestsPendingDefault,
  mockAccessRequestsPendingEmpty,
  mockAccessRequests,
  mockAccessRequestsDefault,
  mockAccessRequestApprove,
  mockAccessRequestApproveError,
  mockAccessRequestReject,
} from '@/test-utils/msw-v2/handlers/user-access-requests';
import { setupMswV2, server } from '@/test-utils/msw-v2/setup';
import { createWrapper } from '@/tests/wrapper';

setupMswV2();

afterEach(() => server.resetHandlers());

describe('useGetRequestStatus', () => {
  it('fetches pending status successfully', async () => {
    server.use(...mockUserRequestStatusPending());

    const { result } = renderHook(() => useGetRequestStatus(), {
      wrapper: createWrapper(),
    });

    await waitFor(() => {
      expect(result.current.isSuccess).toBe(true);
    });

    expect(result.current.data).toEqual<UserAccessStatusResponse>({
      status: 'pending',
      username: 'user@example.com',
      created_at: '2024-01-01T00:00:00Z',
      updated_at: '2024-01-01T00:00:00Z',
    });
  });

  it('fetches approved status successfully', async () => {
    server.use(...mockUserRequestStatusApproved());

    const { result } = renderHook(() => useGetRequestStatus(), {
      wrapper: createWrapper(),
    });

    await waitFor(() => {
      expect(result.current.isSuccess).toBe(true);
    });

    expect(result.current.data?.status).toBe('approved');
  });

  it('fetches rejected status successfully', async () => {
    server.use(...mockUserRequestStatusRejected());

    const { result } = renderHook(() => useGetRequestStatus(), {
      wrapper: createWrapper(),
    });

    await waitFor(() => {
      expect(result.current.isSuccess).toBe(true);
    });

    expect(result.current.data?.status).toBe('rejected');
  });

  it('handles 404 error without retrying', async () => {
    server.use(
      ...mockUserRequestStatusError({
        message: 'No request found',
        status: 404,
      })
    );

    const { result } = renderHook(() => useGetRequestStatus(), {
      wrapper: createWrapper(),
    });

    await waitFor(() => {
      expect(result.current.isError).toBe(true);
    });

    const error = result.current.error as AxiosError<OpenAiApiError>;
    expect(error.response?.status).toBe(404);
  });
});

describe('useSubmitAccessRequest', () => {
  it('submits request successfully and calls onSuccess', async () => {
    const onSuccess = vi.fn();
    server.use(...mockUserRequestAccess());

    const { result } = renderHook(() => useSubmitAccessRequest({ onSuccess }), {
      wrapper: createWrapper(),
    });

    await act(async () => {
      await result.current.mutateAsync();
    });

    expect(onSuccess).toHaveBeenCalled();
  });

  it('calls onError on failure', async () => {
    const onError = vi.fn();
    server.use(
      ...mockUserRequestAccessError({
        message: 'Access request already exists',
        status: 409,
      })
    );

    const { result } = renderHook(() => useSubmitAccessRequest({ onError }), {
      wrapper: createWrapper(),
    });

    await act(async () => {
      try {
        await result.current.mutateAsync();
      } catch {
        /* expected */
      }
    });

    expect(onError).toHaveBeenCalledWith('Access request already exists');
  });
});

describe('useListPendingRequests', () => {
  it('fetches pending requests with data', async () => {
    server.use(...mockAccessRequestsPendingDefault());

    const { result } = renderHook(() => useListPendingRequests(), {
      wrapper: createWrapper(),
    });

    await waitFor(() => {
      expect(result.current.isSuccess).toBe(true);
    });

    expect(result.current.data?.requests).toHaveLength(1);
    expect(result.current.data?.requests[0].status).toBe('pending');
    expect(result.current.data?.total).toBe(1);
  });

  it('fetches empty pending requests list', async () => {
    server.use(...mockAccessRequestsPendingEmpty());

    const { result } = renderHook(() => useListPendingRequests(), {
      wrapper: createWrapper(),
    });

    await waitFor(() => {
      expect(result.current.isSuccess).toBe(true);
    });

    expect(result.current.data?.requests).toHaveLength(0);
    expect(result.current.data?.total).toBe(0);
  });
});

describe('useListAllRequests', () => {
  it('fetches all requests successfully', async () => {
    server.use(...mockAccessRequestsDefault());

    const { result } = renderHook(() => useListAllRequests(), {
      wrapper: createWrapper(),
    });

    await waitFor(() => {
      expect(result.current.isSuccess).toBe(true);
    });

    expect(result.current.data?.requests).toHaveLength(3);
    expect(result.current.data?.total).toBe(3);
  });
});

describe('useApproveRequest', () => {
  it('approves request successfully and calls onSuccess', async () => {
    const onSuccess = vi.fn();
    const requestId = 'test-request-1';
    server.use(...mockAccessRequestApprove(requestId));

    const { result } = renderHook(() => useApproveRequest({ onSuccess }), {
      wrapper: createWrapper(),
    });

    await act(async () => {
      await result.current.mutateAsync({ id: requestId, role: 'user' });
    });

    expect(onSuccess).toHaveBeenCalled();
  });

  it('calls onError on approval failure', async () => {
    const onError = vi.fn();
    const requestId = 'test-request-1';
    server.use(
      ...mockAccessRequestApproveError(requestId, {
        message: 'Request already processed',
        status: 400,
      })
    );

    const { result } = renderHook(() => useApproveRequest({ onError }), {
      wrapper: createWrapper(),
    });

    await act(async () => {
      try {
        await result.current.mutateAsync({ id: requestId, role: 'user' });
      } catch {
        /* expected */
      }
    });

    expect(onError).toHaveBeenCalledWith('Request already processed');
  });
});

describe('useRejectRequest', () => {
  it('rejects request successfully and calls onSuccess', async () => {
    const onSuccess = vi.fn();
    const requestId = 'test-request-1';
    server.use(...mockAccessRequestReject(requestId));

    const { result } = renderHook(() => useRejectRequest({ onSuccess }), {
      wrapper: createWrapper(),
    });

    await act(async () => {
      await result.current.mutateAsync(requestId);
    });

    expect(onSuccess).toHaveBeenCalled();
  });
});
