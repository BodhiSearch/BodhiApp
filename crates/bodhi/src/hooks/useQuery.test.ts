import { ENDPOINT_SETTINGS, useDeleteSetting, useSettings, useUpdateSetting } from '@/hooks/useQuery';
import { createWrapper } from '@/tests/wrapper';
import { ApiError, Setting } from '@/types/models';
import { act, renderHook, waitFor } from '@testing-library/react';
import { AxiosError } from 'axios';
import { rest } from 'msw';
import { setupServer } from 'msw/node';
import { afterAll, afterEach, beforeAll, describe, expect, it } from 'vitest';

const mockSettings: Setting[] = [
  {
    key: 'BODHI_LOG_LEVEL',
    current_value: 'info',
    default_value: 'warn',
    source: 'settings_file',
    metadata: {
      type: 'option',
      options: ['error', 'warn', 'info', 'debug', 'trace']
    }
  },
  {
    key: 'BODHI_PORT',
    current_value: 1135,
    default_value: 1135,
    source: 'default',
    metadata: {
      type: 'number',
      range: {
        min: 1025,
        max: 65535
      }
    }
  }
];

const server = setupServer(
  rest.get(`*${ENDPOINT_SETTINGS}`, (_, res, ctx) => {
    return res(ctx.status(200), ctx.json(mockSettings));
  }),
  rest.put(`*${ENDPOINT_SETTINGS}/:key`, (_, res, ctx) => {
    return res(ctx.status(200), ctx.json(mockSettings[0]));
  }),
  rest.delete(`*${ENDPOINT_SETTINGS}/:key`, (_, res, ctx) => {
    return res(ctx.status(200), ctx.json(mockSettings[0]));
  })
);

beforeAll(() => server.listen());
afterAll(() => server.close());
afterEach(() => server.resetHandlers());

describe('Settings Hooks', () => {
  describe('useSettings', () => {
    it('fetches settings successfully', async () => {
      const { result } = renderHook(() => useSettings(), {
        wrapper: createWrapper()
      });

      await waitFor(() => {
        expect(result.current.isSuccess).toBe(true);
      });

      expect(result.current.data).toEqual(mockSettings);
    });

    it('handles error response', async () => {
      server.use(
        rest.get(`*${ENDPOINT_SETTINGS}`, (_, res, ctx) => {
          return res(ctx.status(500));
        })
      );

      const { result } = renderHook(() => useSettings(), {
        wrapper: createWrapper()
      });

      await waitFor(() => {
        expect(result.current.isError).toBe(true);
      });
    });
  });

  describe('useUpdateSetting', () => {
    const updateData = {
      key: 'BODHI_LOG_LEVEL',
      value: 'debug'
    };

    const mockUpdatedSetting: Setting = {
      key: 'BODHI_LOG_LEVEL',
      current_value: 'debug',
      default_value: 'warn',
      source: 'settings_file',
      metadata: {
        type: 'option',
        options: ['error', 'warn', 'info', 'debug', 'trace']
      }
    };

    beforeEach(() => {
      server.use(
        rest.put(`*${ENDPOINT_SETTINGS}/${updateData.key}`, (_, res, ctx) => {
          return res(ctx.status(200), ctx.json(mockUpdatedSetting));
        })
      );
    });

    it('updates setting successfully', async () => {
      const { result } = renderHook(() => useUpdateSetting(), {
        wrapper: createWrapper()
      });

      await act(async () => {
        await result.current.mutateAsync(updateData);
      });

      expect(result.current.data?.data).toEqual(mockUpdatedSetting);
    });

    it('handles error response', async () => {
      server.use(
        rest.put(`*${ENDPOINT_SETTINGS}/${updateData.key}`, (_, res, ctx) => {
          return res(
            ctx.status(400),
            ctx.json({
              error: 'Bad Request',
              message: 'Invalid setting value'
            })
          );
        })
      );

      const { result } = renderHook(() => useUpdateSetting(), {
        wrapper: createWrapper()
      });

      await act(async () => {
        try {
          await result.current.mutateAsync(updateData);
          // Should not reach here
          expect(false).toBe(true);
        } catch (error) {
          const axiosError = error as AxiosError<ApiError>;
          expect(axiosError.response?.status).toBe(400);
          expect(axiosError.response?.data.message).toBe('Invalid setting value');
        }
      });
    });

    it('invalidates settings query on successful update', async () => {
      const wrapper = createWrapper();

      // Setup settings hook and wait for initial data
      const { result: settingsResult } = renderHook(() => useSettings(), {
        wrapper
      });

      await waitFor(() => {
        expect(settingsResult.current.isSuccess).toBe(true);
      });

      const initialDataUpdatedAt = settingsResult.current.dataUpdatedAt;

      // Update setting
      const { result: updateResult } = renderHook(() => useUpdateSetting(), {
        wrapper
      });

      await act(async () => {
        await updateResult.current.mutateAsync(updateData);
      });

      // Verify settings query was invalidated and refetched
      await waitFor(() => {
        expect(settingsResult.current.dataUpdatedAt).toBeGreaterThan(
          initialDataUpdatedAt
        );
      });
    });
  });

  describe('useDeleteSetting', () => {
    const deleteData = {
      key: 'BODHI_LOG_LEVEL'
    };

    const mockDeletedSetting: Setting = {
      key: 'BODHI_LOG_LEVEL',
      current_value: 'warn',  // Reset to default value
      default_value: 'warn',
      source: 'default',
      metadata: {
        type: 'option',
        options: ['error', 'warn', 'info', 'debug', 'trace']
      }
    };

    beforeEach(() => {
      server.use(
        rest.delete(`*${ENDPOINT_SETTINGS}/${deleteData.key}`, (_, res, ctx) => {
          return res(ctx.status(200), ctx.json(mockDeletedSetting));
        })
      );
    });

    it('deletes setting successfully', async () => {
      const { result } = renderHook(() => useDeleteSetting(), {
        wrapper: createWrapper()
      });

      await act(async () => {
        await result.current.mutateAsync(deleteData);
      });

      expect(result.current.data?.data).toEqual(mockDeletedSetting);
    });

    it('handles error response', async () => {
      server.use(
        rest.delete(`*${ENDPOINT_SETTINGS}/${deleteData.key}`, (_, res, ctx) => {
          return res(
            ctx.status(400),
            ctx.json({
              error: 'Bad Request',
              message: 'Cannot delete required setting'
            })
          );
        })
      );

      const { result } = renderHook(() => useDeleteSetting(), {
        wrapper: createWrapper()
      });

      await act(async () => {
        try {
          await result.current.mutateAsync(deleteData);
          // Should not reach here
          expect(false).toBe(true);
        } catch (error) {
          const axiosError = error as AxiosError<ApiError>;
          expect(axiosError.response?.status).toBe(400);
          expect(axiosError.response?.data.message).toBe('Cannot delete required setting');
        }
      });
    });

    it('invalidates settings query on successful delete', async () => {
      const wrapper = createWrapper();

      // Setup settings hook and wait for initial data
      const { result: settingsResult } = renderHook(() => useSettings(), {
        wrapper
      });

      await waitFor(() => {
        expect(settingsResult.current.isSuccess).toBe(true);
      });

      const initialDataUpdatedAt = settingsResult.current.dataUpdatedAt;

      // Delete setting
      const { result: deleteResult } = renderHook(() => useDeleteSetting(), {
        wrapper
      });

      await act(async () => {
        await deleteResult.current.mutateAsync(deleteData);
      });

      // Verify settings query was invalidated and refetched
      await waitFor(() => {
        expect(settingsResult.current.dataUpdatedAt).toBeGreaterThan(
          initialDataUpdatedAt
        );
      });
    });
  });
}); 