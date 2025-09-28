import {
  ENDPOINT_APP_INFO,
  ENDPOINT_APP_SETUP,
  ENDPOINT_SETTINGS,
  ENDPOINT_USER_INFO,
  useAppInfo,
  useDeleteSetting,
  useSettings,
  useSetupApp,
  useUpdateSetting,
  useUser,
} from '@/hooks/useQuery';
import { createWrapper } from '@/tests/wrapper';
import { createMockLoggedInUser } from '@/test-utils/mock-user';
import { OpenAiApiError, AppInfo, SettingInfo } from '@bodhiapp/ts-client';

// Type aliases for compatibility
type ApiError = OpenAiApiError;
type Setting = SettingInfo;
import { act, renderHook, waitFor } from '@testing-library/react';
import { AxiosError } from 'axios';
import { setupMswV2, server } from '@/test-utils/msw-v2/setup';
import {
  mockSettings,
  mockSettingsInternalError,
  mockUpdateSetting,
  mockUpdateSettingInvalidError,
  mockDeleteSetting,
  mockDeleteSettingNotFoundError,
} from '@/test-utils/msw-v2/handlers/settings';
import { mockAppInfo } from '@/test-utils/msw-v2/handlers/info';
import { mockSetup, mockSetupError } from '@/test-utils/msw-v2/handlers/setup';
import { mockUserLoggedIn } from '@/test-utils/msw-v2/handlers/user';
import { afterAll, afterEach, beforeAll, describe, expect, it, vi } from 'vitest';

const mockSettingsData: Setting[] = [
  {
    key: 'BODHI_LOG_LEVEL',
    current_value: 'info',
    default_value: 'warn',
    source: 'settings_file',
    metadata: {
      type: 'option',
      options: ['error', 'warn', 'info', 'debug', 'trace'],
    },
  },
  {
    key: 'BODHI_PORT',
    current_value: 1135,
    default_value: 1135,
    source: 'default',
    metadata: {
      type: 'number',
      min: 1025,
      max: 65535,
    },
  },
];

const mockAppInfoData: AppInfo = {
  version: '0.1.0',
  status: 'ready',
};

const mockUserInfoData = {
  ...createMockLoggedInUser(),
  role: 'resource_user',
};

setupMswV2();

// Setup default handlers
beforeAll(() => {
  server.use(
    ...mockSettings(mockSettingsData),
    ...mockUpdateSetting('BODHI_LOG_LEVEL', {
      current_value: 'info',
      default_value: 'warn',
      source: 'settings_file',
      metadata: {
        type: 'option',
        options: ['error', 'warn', 'info', 'debug', 'trace'],
      },
    }),
    ...mockDeleteSetting('BODHI_LOG_LEVEL', {
      current_value: 'warn',
      default_value: 'warn',
      source: 'default',
      metadata: {
        type: 'option',
        options: ['error', 'warn', 'info', 'debug', 'trace'],
      },
    }),
    ...mockAppInfo(mockAppInfoData),
    ...mockUserLoggedIn(mockUserInfoData),
    ...mockSetup({ status: 'ready' })
  );
});

describe('Settings Hooks', () => {
  describe('useSettings', () => {
    it('fetches settings successfully', async () => {
      const { result } = renderHook(() => useSettings(), {
        wrapper: createWrapper(),
      });

      await waitFor(() => {
        expect(result.current.isSuccess).toBe(true);
      });

      expect(result.current.data).toEqual(mockSettingsData);
    });

    it('handles error response', async () => {
      server.use(...mockSettingsInternalError());

      const { result } = renderHook(() => useSettings(), {
        wrapper: createWrapper(),
      });

      await waitFor(() => {
        expect(result.current.isError).toBe(true);
      });
    });
  });

  describe('useUpdateSetting', () => {
    const updateData = {
      key: 'BODHI_LOG_LEVEL',
      value: 'debug',
    };

    const mockUpdatedSetting: Setting = {
      key: 'BODHI_LOG_LEVEL',
      current_value: 'debug',
      default_value: 'warn',
      source: 'settings_file',
      metadata: {
        type: 'option',
        options: ['error', 'warn', 'info', 'debug', 'trace'],
      },
    };

    beforeEach(() => {
      server.use(
        ...mockUpdateSetting('BODHI_LOG_LEVEL', {
          current_value: 'debug',
          default_value: 'warn',
          source: 'settings_file',
          metadata: {
            type: 'option',
            options: ['error', 'warn', 'info', 'debug', 'trace'],
          },
        }),
        ...mockSettings(mockSettingsData), // For initial load
        ...mockSettings(mockSettingsData) // For refetch after update
      );
    });

    it('updates setting successfully', async () => {
      const { result } = renderHook(() => useUpdateSetting(), {
        wrapper: createWrapper(),
      });

      await act(async () => {
        await result.current.mutateAsync(updateData);
      });

      expect(result.current.data?.data).toEqual(mockUpdatedSetting);
    });

    it('handles error response', async () => {
      server.use(...mockUpdateSettingInvalidError('BODHI_LOG_LEVEL'));

      const { result } = renderHook(() => useUpdateSetting(), {
        wrapper: createWrapper(),
      });

      await act(async () => {
        try {
          await result.current.mutateAsync(updateData);
          // Should not reach here
          expect(false).toBe(true);
        } catch (error) {
          const axiosError = error as AxiosError<ApiError>;
          expect(axiosError.response?.status).toBe(400);
          expect(axiosError.response?.data.error?.message).toBe('Invalid setting value');
        }
      });
    });

    it('invalidates settings query on successful update', async () => {
      const wrapper = createWrapper();

      // Setup settings hook and wait for initial data
      const { result: settingsResult } = renderHook(() => useSettings(), {
        wrapper,
      });

      await waitFor(() => {
        expect(settingsResult.current.isSuccess).toBe(true);
      });

      const initialDataUpdatedAt = settingsResult.current.dataUpdatedAt;

      // Update setting
      const { result: updateResult } = renderHook(() => useUpdateSetting(), {
        wrapper,
      });

      await act(async () => {
        await updateResult.current.mutateAsync(updateData);
      });

      // Verify settings query was invalidated and refetched
      await waitFor(() => {
        expect(settingsResult.current.dataUpdatedAt).toBeGreaterThan(initialDataUpdatedAt);
      });
    });
  });

  describe('useDeleteSetting', () => {
    const deleteData = {
      key: 'BODHI_LOG_LEVEL',
    };

    const mockDeletedSetting: Setting = {
      key: 'BODHI_LOG_LEVEL',
      current_value: 'warn', // Reset to default value
      default_value: 'warn',
      source: 'default',
      metadata: {
        type: 'option',
        options: ['error', 'warn', 'info', 'debug', 'trace'],
      },
    };

    beforeEach(() => {
      server.use(
        ...mockDeleteSetting('BODHI_LOG_LEVEL', {
          current_value: 'warn', // Reset to default value
          default_value: 'warn',
          source: 'default',
          metadata: {
            type: 'option',
            options: ['error', 'warn', 'info', 'debug', 'trace'],
          },
        }),
        ...mockSettings(mockSettingsData), // For initial load
        ...mockSettings(mockSettingsData) // For refetch after delete
      );
    });

    it('deletes setting successfully', async () => {
      const { result } = renderHook(() => useDeleteSetting(), {
        wrapper: createWrapper(),
      });

      await act(async () => {
        await result.current.mutateAsync(deleteData);
      });

      expect(result.current.data?.data).toEqual(mockDeletedSetting);
    });

    it('handles error response', async () => {
      server.use(...mockDeleteSettingNotFoundError('BODHI_LOG_LEVEL'));

      const { result } = renderHook(() => useDeleteSetting(), {
        wrapper: createWrapper(),
      });

      await act(async () => {
        try {
          await result.current.mutateAsync(deleteData);
          // Should not reach here
          expect(false).toBe(true);
        } catch (error) {
          const axiosError = error as AxiosError<ApiError>;
          expect(axiosError.response?.status).toBe(404);
          expect(axiosError.response?.data.error?.message).toBe('Cannot delete required setting');
        }
      });
    });

    it('invalidates settings query on successful delete', async () => {
      const wrapper = createWrapper();

      // Setup settings hook and wait for initial data
      const { result: settingsResult } = renderHook(() => useSettings(), {
        wrapper,
      });

      await waitFor(() => {
        expect(settingsResult.current.isSuccess).toBe(true);
      });

      const initialDataUpdatedAt = settingsResult.current.dataUpdatedAt;

      // Delete setting
      const { result: deleteResult } = renderHook(() => useDeleteSetting(), {
        wrapper,
      });

      await act(async () => {
        await deleteResult.current.mutateAsync(deleteData);
      });

      // Verify settings query was invalidated and refetched
      await waitFor(() => {
        expect(settingsResult.current.dataUpdatedAt).toBeGreaterThan(initialDataUpdatedAt);
      });
    });
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
