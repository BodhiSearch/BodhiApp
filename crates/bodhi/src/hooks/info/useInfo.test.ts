import { AppInfo } from '@bodhiapp/ts-client';
import { act, renderHook, waitFor } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import { useGetAppInfo, useSetupApp } from '@/hooks/info';
import { useGetUser } from '@/hooks/users';
import { createMockLoggedInUser } from '@/test-utils/mock-user';
import { mockAppInfo } from '@/test-utils/msw-v2/handlers/info';
import { mockSetup, mockSetupError } from '@/test-utils/msw-v2/handlers/setup';
import { mockUserLoggedIn } from '@/test-utils/msw-v2/handlers/user';
import { setupMswV2, server } from '@/test-utils/msw-v2/setup';
import { createWrapper } from '@/tests/wrapper';

const mockAppInfoData: AppInfo = {
  commit_sha: 'not-set',
  version: '0.1.0',
  status: 'ready',
  deployment: 'standalone',
  url: 'http://localhost:1135',
};

const mockUserInfoData = {
  ...createMockLoggedInUser(),
  role: 'resource_user' as const,
};

setupMswV2();

beforeEach(() => {
  server.use(...mockAppInfo(mockAppInfoData), ...mockUserLoggedIn(mockUserInfoData), ...mockSetup({ status: 'ready' }));
});

describe('useGetAppInfo', () => {
  it('fetches app info successfully', async () => {
    const { result } = renderHook(() => useGetAppInfo(), {
      wrapper: createWrapper(),
    });

    await waitFor(() => {
      expect(result.current.isSuccess).toBe(true);
    });

    expect(result.current.data).toEqual(mockAppInfoData);
  });
});

describe('useSetupApp', () => {
  beforeEach(() => {
    server.use(
      ...mockAppInfo(mockAppInfoData),
      ...mockUserLoggedIn(mockUserInfoData),
      ...mockSetup({ status: 'ready' }),
      // Additional mocks for query invalidation refetches
      ...mockAppInfo(mockAppInfoData),
      ...mockUserLoggedIn(mockUserInfoData)
    );
  });

  it('invalidates appInfo and user queries on successful setup', async () => {
    const wrapper = createWrapper();

    const { result: appInfoResult } = renderHook(() => useGetAppInfo(), {
      wrapper,
    });
    const { result: userResult } = renderHook(() => useGetUser({ enabled: true }), {
      wrapper,
    });

    await waitFor(() => {
      expect(appInfoResult.current.isSuccess).toBe(true);
      expect(userResult.current.isSuccess).toBe(true);
    });

    const initialAppInfoUpdatedAt = appInfoResult.current.dataUpdatedAt;
    const initialUserUpdatedAt = userResult.current.dataUpdatedAt;

    const { result: setupResult } = renderHook(() => useSetupApp(), {
      wrapper,
    });

    await act(async () => {
      await setupResult.current.mutateAsync({ name: 'Test App' });
    });

    await waitFor(() => {
      expect(appInfoResult.current.dataUpdatedAt).toBeGreaterThan(initialAppInfoUpdatedAt);
      expect(userResult.current.dataUpdatedAt).toBeGreaterThan(initialUserUpdatedAt);
    });
  });

  it('calls onSuccess with the response data', async () => {
    const onSuccess = vi.fn();
    const { result } = renderHook(() => useSetupApp({ onSuccess }), {
      wrapper: createWrapper(),
    });

    await act(async () => {
      await result.current.mutateAsync({ name: 'Test App' });
    });

    expect(onSuccess).toHaveBeenCalledWith({
      status: 'ready',
    });
  });

  it('calls onError with error message on failure', async () => {
    const onError = vi.fn();
    server.use(...mockSetupError({ code: 'internal_error', message: 'Setup failed', type: 'internal_server_error' }));

    const { result } = renderHook(() => useSetupApp({ onError }), {
      wrapper: createWrapper(),
    });

    await act(async () => {
      try {
        await result.current.mutateAsync({ name: 'Test App' });
      } catch (error) {
        // Expected error
      }
    });

    expect(onError).toHaveBeenCalledWith('Setup failed');
  });
});
