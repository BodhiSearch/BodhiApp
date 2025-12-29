import { useAppInfo, useSetupApp } from '@/hooks/useInfo';
import { createWrapper } from '@/tests/wrapper';
import { createMockLoggedInUser } from '@/test-utils/mock-user';
import { OpenAiApiError, AppInfo } from '@bodhiapp/ts-client';
import { act, renderHook, waitFor } from '@testing-library/react';
import { setupMswV2, server } from '@/test-utils/msw-v2/setup';
import { mockAppInfo } from '@/test-utils/msw-v2/handlers/info';
import { mockSetup, mockSetupError } from '@/test-utils/msw-v2/handlers/setup';
import { mockUserLoggedIn } from '@/test-utils/msw-v2/handlers/user';
import { useUser } from '@/hooks/useUsers';
import { beforeAll, describe, expect, it, vi } from 'vitest';

// Mock data
const mockAppInfoData: AppInfo = {
  commit_sha: 'not-set',
  version: '0.1.0',
  status: 'ready',
};

const mockUserInfoData = {
  ...createMockLoggedInUser(),
  role: 'resource_user' as const,
};

setupMswV2();

// Setup default handlers
beforeAll(() => {
  server.use(...mockAppInfo(mockAppInfoData), ...mockUserLoggedIn(mockUserInfoData), ...mockSetup({ status: 'ready' }));
});

describe('useAppInfo', () => {
  it('fetches app info successfully', async () => {
    const { result } = renderHook(() => useAppInfo(), {
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

    // Setup initial queries and wait for data
    const { result: appInfoResult } = renderHook(() => useAppInfo(), {
      wrapper,
    });
    const { result: userResult } = renderHook(() => useUser({ enabled: true }), {
      wrapper,
    });

    await waitFor(() => {
      expect(appInfoResult.current.isSuccess).toBe(true);
      expect(userResult.current.isSuccess).toBe(true);
    });

    const initialAppInfoUpdatedAt = appInfoResult.current.dataUpdatedAt;
    const initialUserUpdatedAt = userResult.current.dataUpdatedAt;

    // Perform setup
    const { result: setupResult } = renderHook(() => useSetupApp(), {
      wrapper,
    });

    await act(async () => {
      await setupResult.current.mutateAsync({ name: 'Test App' });
    });

    // Verify both queries were invalidated and refetched
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
