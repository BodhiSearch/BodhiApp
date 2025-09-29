import { useSettings, useUpdateSetting, useDeleteSetting } from '@/hooks/useSettings';
import { createWrapper } from '@/tests/wrapper';
import { createMockLoggedInUser } from '@/test-utils/mock-user';
import { OpenAiApiError, SettingInfo } from '@bodhiapp/ts-client';
import { setupMswV2, server } from '@/test-utils/msw-v2/setup';
import {
  mockSettings,
  mockSettingsInternalError,
  mockUpdateSetting,
  mockUpdateSettingInvalidError,
  mockDeleteSetting,
  mockDeleteSettingNotFoundError,
} from '@/test-utils/msw-v2/handlers/settings';
import { mockUserLoggedIn } from '@/test-utils/msw-v2/handlers/user';
import { act, renderHook, waitFor } from '@testing-library/react';
import { AxiosError } from 'axios';
import { afterAll, afterEach, beforeAll, describe, expect, it, vi } from 'vitest';

// Type aliases for compatibility
type ApiError = OpenAiApiError;
type Setting = SettingInfo;

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
    ...mockUserLoggedIn(mockUserInfoData)
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
