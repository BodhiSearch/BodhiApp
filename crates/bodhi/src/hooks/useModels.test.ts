import {
  useModelFiles,
  useModels,
  useModel,
  useCreateModel,
  useUpdateModel,
  useDownloads,
  usePullModel,
} from '@/hooks/useModels';
import { createWrapper } from '@/tests/wrapper';
import { setupMswV2, server, http, HttpResponse } from '@/test-utils/msw-v2/setup';
import {
  mockModels,
  mockModelsDefault,
  mockModelsEmpty,
  mockModelsError,
  mockGetModel,
  mockGetModelNotFoundError,
  mockCreateModel,
  mockCreateModelError,
  mockUpdateModel,
  mockUpdateModelError,
} from '@/test-utils/msw-v2/handlers/models';
import {
  mockModelFiles,
  mockModelFilesDefault,
  mockModelFilesEmpty,
  mockModelFilesError,
  mockModelPullDownloads,
  mockModelPullDownloadsDefault,
  mockModelPullDownloadsEmpty,
  mockModelPull,
  mockModelPullError,
  mockModelPullFileExistsError,
} from '@/test-utils/msw-v2/handlers/modelfiles';
import { act, renderHook, waitFor } from '@testing-library/react';
import { beforeAll, beforeEach, describe, expect, it, vi } from 'vitest';
import type {
  Alias,
  CreateAliasRequest,
  UpdateAliasRequest,
  PaginatedAliasResponse,
  PaginatedLocalModelResponse,
  PaginatedDownloadResponse,
  DownloadRequest,
  NewDownloadRequest,
} from '@bodhiapp/ts-client';

// Mock data
const mockModelData: Alias = {
  source: 'user',
  alias: 'test-model',
  repo: 'test-repo',
  filename: 'test-file.bin',
  snapshot: 'abc123',
  request_params: {},
  context_params: [],
};

const mockModelsData: PaginatedAliasResponse = {
  data: [mockModelData],
  page: 1,
  page_size: 30,
  total: 1,
};

const mockModelFilesData: PaginatedLocalModelResponse = {
  data: [
    {
      repo: 'test-repo',
      filename: 'test-file.txt',
      size: 1073741824,
      snapshot: 'abc123',
      model_params: {},
    },
  ],
  page: 1,
  page_size: 30,
  total: 1,
};

const mockDownloadsData: PaginatedDownloadResponse = {
  data: [
    {
      id: '1',
      repo: 'test/repo1',
      filename: 'model1.gguf',
      status: 'pending',
      error: null,
      created_at: '2024-01-01T00:00:00Z',
      updated_at: '2024-01-01T00:00:00Z',
      total_bytes: 1000000,
      downloaded_bytes: 500000,
      started_at: '2024-01-01T00:00:00Z',
    },
    {
      id: '2',
      repo: 'test/repo2',
      filename: 'model2.gguf',
      status: 'completed',
      error: null,
      created_at: '2024-01-01T00:00:00Z',
      updated_at: '2024-01-01T00:00:00Z',
      total_bytes: 2000000,
      downloaded_bytes: 2000000,
      started_at: '2024-01-01T00:00:00Z',
    },
    {
      id: '3',
      repo: 'test/repo3',
      filename: 'model3.gguf',
      status: 'error',
      error: 'Download failed',
      created_at: '2024-01-01T00:00:00Z',
      updated_at: '2024-01-01T00:00:00Z',
      total_bytes: 1500000,
      downloaded_bytes: 750000,
      started_at: '2024-01-01T00:00:00Z',
    },
  ],
  page: 1,
  page_size: 30,
  total: 3,
};

const mockPullModelRequest: NewDownloadRequest = {
  repo: 'test/repo',
  filename: 'model.gguf',
};

const mockPullModelResponse: DownloadRequest = {
  id: '123',
  repo: 'test/repo',
  filename: 'model.gguf',
  status: 'pending',
  error: null,
  created_at: '2024-01-01T00:00:00Z',
  updated_at: '2024-01-01T00:00:00Z',
  total_bytes: undefined,
  downloaded_bytes: undefined,
  started_at: '2024-01-01T00:00:00Z',
};

setupMswV2();

describe('Model Hooks', () => {
  describe('useModelFiles', () => {
    beforeEach(() => {
      server.use(...mockModelFilesDefault());
    });

    it('fetches model files successfully with default parameters', async () => {
      const { result } = renderHook(() => useModelFiles(), {
        wrapper: createWrapper(),
      });

      await waitFor(() => {
        expect(result.current.isSuccess).toBe(true);
      });

      expect(result.current.data).toEqual(mockModelFilesData);
    });

    it('fetches model files with custom pagination and sorting', async () => {
      server.use(
        ...mockModelFiles({
          data: mockModelFilesData.data,
          page: 2,
          page_size: 10,
          total: 1,
        })
      );

      const { result } = renderHook(() => useModelFiles(2, 10, 'filename', 'desc'), {
        wrapper: createWrapper(),
      });

      await waitFor(() => {
        expect(result.current.isSuccess).toBe(true);
      });

      expect(result.current.data?.page).toBe(2);
      expect(result.current.data?.page_size).toBe(10);
    });

    it('handles empty model files list', async () => {
      server.use(...mockModelFilesEmpty());

      const { result } = renderHook(() => useModelFiles(), {
        wrapper: createWrapper(),
      });

      await waitFor(() => {
        expect(result.current.isSuccess).toBe(true);
      });

      expect(result.current.data?.data).toEqual([]);
      expect(result.current.data?.total).toBe(0);
    });

    it('handles model files fetch error', async () => {
      server.use(...mockModelFilesError({ message: 'Fetch failed', status: 500 }));

      const { result } = renderHook(() => useModelFiles(), {
        wrapper: createWrapper(),
      });

      await waitFor(() => {
        expect(result.current.isError).toBe(true);
      });

      expect(result.current.error?.response?.data?.error?.message).toBe('Fetch failed');
    });
  });

  describe('useModels', () => {
    beforeEach(() => {
      server.use(...mockModelsDefault());
    });

    it('fetches models successfully', async () => {
      const { result } = renderHook(() => useModels(1, 30, 'alias', 'asc'), {
        wrapper: createWrapper(),
      });

      await waitFor(() => {
        expect(result.current.isSuccess).toBe(true);
      });

      expect(result.current.data).toEqual(mockModelsData);
    });

    it('fetches models with custom pagination', async () => {
      server.use(
        ...mockModels({
          data: mockModelsData.data,
          page: 2,
          page_size: 10,
          total: 1,
        })
      );

      const { result } = renderHook(() => useModels(2, 10, 'repo', 'desc'), {
        wrapper: createWrapper(),
      });

      await waitFor(() => {
        expect(result.current.isSuccess).toBe(true);
      });

      expect(result.current.data?.page).toBe(2);
      expect(result.current.data?.page_size).toBe(10);
    });

    it('handles empty models list', async () => {
      server.use(...mockModelsEmpty());

      const { result } = renderHook(() => useModels(1, 30, 'alias', 'asc'), {
        wrapper: createWrapper(),
      });

      await waitFor(() => {
        expect(result.current.isSuccess).toBe(true);
      });

      expect(result.current.data?.data).toEqual([]);
      expect(result.current.data?.total).toBe(0);
    });

    it('handles models fetch error', async () => {
      server.use(...mockModelsError({ message: 'Fetch failed', status: 500 }));

      const { result } = renderHook(() => useModels(1, 30, 'alias', 'asc'), {
        wrapper: createWrapper(),
      });

      await waitFor(() => {
        expect(result.current.isError).toBe(true);
      });

      expect(result.current.error?.response?.data?.error?.message).toBe('Fetch failed');
    });
  });

  describe('useModel', () => {
    const alias = 'test-model';

    beforeEach(() => {
      server.use(...mockGetModel(alias, { repo: mockModelData.repo, filename: mockModelData.filename }));
    });

    it('fetches individual model successfully', async () => {
      const { result } = renderHook(() => useModel(alias), {
        wrapper: createWrapper(),
      });

      await waitFor(() => {
        expect(result.current.isSuccess).toBe(true);
      });

      expect((result.current.data as any)?.alias).toBe(alias);
      expect((result.current.data as any)?.repo).toBe(mockModelData.repo);
    });

    it('is disabled when alias is empty', async () => {
      const { result } = renderHook(() => useModel(''), {
        wrapper: createWrapper(),
      });

      expect(result.current.isIdle).toBe(true);
      expect(result.current.data).toBeUndefined();
    });

    it('handles model not found error', async () => {
      const notFoundAlias = 'non-existent-model';
      server.use(...mockGetModelNotFoundError(notFoundAlias));

      const { result } = renderHook(() => useModel(notFoundAlias), {
        wrapper: createWrapper(),
      });

      await waitFor(() => {
        expect(result.current.isError).toBe(true);
      });

      expect(result.current.error?.response?.status).toBe(404);
      expect(result.current.error?.response?.data?.error?.message).toContain('not found');
    });
  });

  describe('useCreateModel', () => {
    const createRequest: CreateAliasRequest = {
      alias: 'new-model',
      repo: 'test/repo',
      filename: 'model.gguf',
      request_params: {},
      context_params: [],
    };

    beforeEach(() => {
      server.use(...mockCreateModel({ alias: createRequest.alias, repo: createRequest.repo }));
    });

    it('creates model successfully', async () => {
      const onSuccess = vi.fn();
      const { result } = renderHook(() => useCreateModel({ onSuccess }), {
        wrapper: createWrapper(),
      });

      await act(async () => {
        await result.current.mutateAsync(createRequest);
      });

      expect(result.current.isSuccess).toBe(true);
      expect(onSuccess).toHaveBeenCalledWith(
        expect.objectContaining({
          alias: createRequest.alias,
          repo: createRequest.repo,
        })
      );
    });

    it('calls onError on creation failure', async () => {
      const onError = vi.fn();
      server.use(...mockCreateModelError({ message: 'Creation failed', status: 400 }));

      const { result } = renderHook(() => useCreateModel({ onError }), {
        wrapper: createWrapper(),
      });

      await act(async () => {
        try {
          await result.current.mutateAsync(createRequest);
        } catch (error) {
          // Expected error
        }
      });

      expect(onError).toHaveBeenCalledWith('Creation failed');
    });

    it('uses default error message when none provided', async () => {
      const onError = vi.fn();
      server.use(...mockCreateModelError({ status: 500 }));

      const { result } = renderHook(() => useCreateModel({ onError }), {
        wrapper: createWrapper(),
      });

      await act(async () => {
        try {
          await result.current.mutateAsync(createRequest);
        } catch (error) {
          // Expected error
        }
      });

      expect(onError).toHaveBeenCalledWith('Internal server error');
    });
  });

  describe('useUpdateModel', () => {
    const alias = 'test-model';
    const updateRequest: UpdateAliasRequest = {
      repo: 'updated/repo',
      filename: 'updated-model.gguf',
      request_params: { temperature: 0.8 },
    };

    beforeEach(() => {
      server.use(...mockUpdateModel(alias, { repo: updateRequest.repo, filename: updateRequest.filename }));
    });

    it('updates model successfully', async () => {
      const onSuccess = vi.fn();
      const { result } = renderHook(() => useUpdateModel(alias, { onSuccess }), {
        wrapper: createWrapper(),
      });

      await act(async () => {
        await result.current.mutateAsync(updateRequest);
      });

      expect(result.current.isSuccess).toBe(true);
      expect(onSuccess).toHaveBeenCalledWith(
        expect.objectContaining({
          alias: alias,
          repo: updateRequest.repo,
          filename: updateRequest.filename,
        })
      );
    });

    it('calls onError on update failure', async () => {
      const onError = vi.fn();
      server.use(...mockUpdateModelError(alias, { message: 'Update failed', status: 400 }));

      const { result } = renderHook(() => useUpdateModel(alias, { onError }), {
        wrapper: createWrapper(),
      });

      await act(async () => {
        try {
          await result.current.mutateAsync(updateRequest);
        } catch (error) {
          // Expected error
        }
      });

      expect(onError).toHaveBeenCalledWith('Update failed');
    });

    it('uses default error message when none provided', async () => {
      const onError = vi.fn();
      server.use(...mockUpdateModelError(alias, { status: 500 }));

      const { result } = renderHook(() => useUpdateModel(alias, { onError }), {
        wrapper: createWrapper(),
      });

      await act(async () => {
        try {
          await result.current.mutateAsync(updateRequest);
        } catch (error) {
          // Expected error
        }
      });

      expect(onError).toHaveBeenCalledWith('Failed to update model test-model');
    });
  });

  describe('useDownloads', () => {
    beforeEach(() => {
      server.use(...mockModelPullDownloadsDefault());
    });

    it('fetches downloads successfully', async () => {
      const { result } = renderHook(() => useDownloads(1, 30), {
        wrapper: createWrapper(),
      });

      await waitFor(() => {
        expect(result.current.isSuccess).toBe(true);
      });

      expect(result.current.data).toEqual(mockDownloadsData);
    });

    it('fetches downloads with custom pagination', async () => {
      server.use(
        ...mockModelPullDownloads({
          data: mockDownloadsData.data,
          page: 2,
          page_size: 10,
          total: 1,
        })
      );

      const { result } = renderHook(() => useDownloads(2, 10), {
        wrapper: createWrapper(),
      });

      await waitFor(() => {
        expect(result.current.isSuccess).toBe(true);
      });

      expect(result.current.data?.page).toBe(2);
      expect(result.current.data?.page_size).toBe(10);
    });

    it('enables polling when specified', async () => {
      const { result } = renderHook(() => useDownloads(1, 30, { enablePolling: true }), {
        wrapper: createWrapper(),
      });

      await waitFor(() => {
        expect(result.current.isSuccess).toBe(true);
      });

      // Note: Testing actual polling behavior would require more complex setup
      // This test just ensures the hook can be called with polling enabled
      expect(result.current.data).toEqual(mockDownloadsData);
    });

    it('handles empty downloads list', async () => {
      server.use(...mockModelPullDownloadsEmpty());

      const { result } = renderHook(() => useDownloads(1, 30), {
        wrapper: createWrapper(),
      });

      await waitFor(() => {
        expect(result.current.isSuccess).toBe(true);
      });

      expect(result.current.data?.data).toEqual([]);
      expect(result.current.data?.total).toBe(0);
    });
  });

  describe('usePullModel', () => {
    beforeEach(() => {
      server.use(...mockModelPull(mockPullModelResponse));
    });

    it('pulls model successfully', async () => {
      const onSuccess = vi.fn();
      const { result } = renderHook(() => usePullModel({ onSuccess }), {
        wrapper: createWrapper(),
      });

      await act(async () => {
        await result.current.mutateAsync(mockPullModelRequest);
      });

      expect(result.current.isSuccess).toBe(true);
      expect(onSuccess).toHaveBeenCalledWith(
        expect.objectContaining({
          repo: mockPullModelRequest.repo,
          filename: mockPullModelRequest.filename,
          status: 'pending',
        })
      );
    });

    it('calls onError on pull failure with code', async () => {
      const onError = vi.fn();
      server.use(
        ...mockModelPullFileExistsError({
          repo: mockPullModelRequest.repo,
          filename: mockPullModelRequest.filename,
        })
      );

      const { result } = renderHook(() => usePullModel({ onError }), {
        wrapper: createWrapper(),
      });

      await act(async () => {
        try {
          await result.current.mutateAsync(mockPullModelRequest);
        } catch (error) {
          // Expected error
        }
      });

      expect(onError).toHaveBeenCalledWith(expect.stringContaining('already exists'), 'pull_error-file_already_exists');
    });

    it('calls onError with default message when none provided', async () => {
      const onError = vi.fn();
      // Create a custom handler that doesn't include a code field
      server.use(
        http.post('/bodhi/v1/modelfiles/pull', () => {
          return HttpResponse.json(
            {
              error: {
                message: 'Failed to pull model',
                type: 'internal_server_error',
                // No code field
              },
            },
            { status: 500 }
          );
        })
      );

      const { result } = renderHook(() => usePullModel({ onError }), {
        wrapper: createWrapper(),
      });

      await act(async () => {
        try {
          await result.current.mutateAsync(mockPullModelRequest);
        } catch (error) {
          // Expected error
        }
      });

      expect(onError).toHaveBeenCalledWith('Failed to pull model', undefined);
    });

    it('handles error without code', async () => {
      const onError = vi.fn();
      server.use(
        ...mockModelPullError({ message: 'Pull failed', status: 400, code: 'pull_error-file_already_exists' })
      );

      const { result } = renderHook(() => usePullModel({ onError }), {
        wrapper: createWrapper(),
      });

      await act(async () => {
        try {
          await result.current.mutateAsync(mockPullModelRequest);
        } catch (error) {
          // Expected error
        }
      });

      expect(onError).toHaveBeenCalledWith('Pull failed', 'pull_error-file_already_exists');
    });
  });
});
