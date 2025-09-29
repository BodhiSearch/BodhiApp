import { useUser, useAuthenticatedUser, useAllUsers, useChangeUserRole, useRemoveUser } from '@/hooks/useUsers';
import { createWrapper } from '@/tests/wrapper';
import { createMockLoggedInUser, createMockLoggedOutUser } from '@/test-utils/mock-user';
import { mockUsersResponse } from '@/test-fixtures/users';
import { OpenAiApiError, UserListResponse } from '@bodhiapp/ts-client';
import { act, renderHook, waitFor } from '@testing-library/react';
import { AxiosError } from 'axios';
import { setupMswV2, server, http, HttpResponse } from '@/test-utils/msw-v2/setup';
import {
  mockUserLoggedIn,
  mockUserLoggedOut,
  mockUserInfoError,
  mockUsers,
  mockUsersError,
  mockUserRoleChange,
  mockUserRoleChangeError,
  mockUserRemove,
  mockUserRemoveError,
} from '@/test-utils/msw-v2/handlers/user';
import { afterAll, afterEach, beforeAll, describe, expect, it, vi } from 'vitest';

// Mock next/navigation
const mockPush = vi.fn();
vi.mock('next/navigation', () => ({
  useRouter: () => ({
    push: mockPush,
  }),
}));

// Type aliases for compatibility
type ApiError = OpenAiApiError;

const mockUserLoggedInData = createMockLoggedInUser({
  user_id: 'test-user-id',
  username: 'test@example.com',
  first_name: 'Test',
  last_name: 'User',
  role: 'resource_user',
});

const mockUserLoggedOutData = createMockLoggedOutUser();

setupMswV2();

// Setup default handlers using beforeEach to ensure they persist after resetHandlers
beforeEach(() => {
  server.use(
    // User info endpoint
    http.get('/bodhi/v1/user', () => {
      return HttpResponse.json({
        auth_status: 'logged_in',
        user_id: 'test-user-id',
        username: 'test@example.com',
        first_name: 'Test',
        last_name: 'User',
        role: 'resource_user',
      });
    }),

    // User role change endpoint
    http.put('/bodhi/v1/users/:userId/role', () => {
      return HttpResponse.json(null, { status: 200 });
    }),

    // User removal endpoint
    http.delete('/bodhi/v1/users/:userId', () => {
      return HttpResponse.json(null, { status: 200 });
    })
  );
});

afterEach(() => {
  mockPush.mockClear();
});

describe('User Hooks', () => {
  describe('useUser', () => {
    it('fetches user info successfully when logged in', async () => {
      const { result } = renderHook(() => useUser(), {
        wrapper: createWrapper(),
      });

      await waitFor(() => {
        expect(result.current.isSuccess).toBe(true);
      });

      expect(result.current.data).toEqual(mockUserLoggedInData);
    });

    it('handles logged out user', async () => {
      server.use(...mockUserLoggedOut());

      const { result } = renderHook(() => useUser(), {
        wrapper: createWrapper(),
      });

      await waitFor(() => {
        expect(result.current.isSuccess).toBe(true);
      });

      expect(result.current.data).toEqual(mockUserLoggedOutData);
    });

    it('handles error response', async () => {
      server.use(...mockUserInfoError({ message: 'Server error' }));

      const { result } = renderHook(() => useUser(), {
        wrapper: createWrapper(),
      });

      await waitFor(() => {
        expect(result.current.isError).toBe(true);
      });

      const error = result.current.error as AxiosError<ApiError>;
      expect(error.response?.data.error?.message).toBe('Server error');
    });

    it('can be disabled with enabled option', async () => {
      const { result } = renderHook(() => useUser({ enabled: false }), {
        wrapper: createWrapper(),
      });

      // Query should not have run
      expect(result.current.isIdle).toBe(true);
      expect(result.current.data).toBeUndefined();
    });
  });

  describe('useAuthenticatedUser', () => {
    it('returns authenticated user data when logged in', async () => {
      const { result } = renderHook(() => useAuthenticatedUser(), {
        wrapper: createWrapper(),
      });

      await waitFor(() => {
        expect(result.current.isSuccess).toBe(true);
      });

      expect(result.current.data).toEqual(mockUserLoggedInData);
      expect(mockPush).not.toHaveBeenCalled();
    });

    it('redirects to login when user is logged out', async () => {
      server.use(...mockUserLoggedOut());

      const { result } = renderHook(() => useAuthenticatedUser(), {
        wrapper: createWrapper(),
      });

      await waitFor(() => {
        expect(result.current.isSuccess).toBe(true);
      });

      expect(result.current.data).toBeUndefined();
      expect(mockPush).toHaveBeenCalledWith('/ui/login');
    });

    it('redirects to login when user auth_status is not logged_in', async () => {
      server.use(...mockUserLoggedIn({ auth_status: 'setup_required' as any }));

      const { result } = renderHook(() => useAuthenticatedUser(), {
        wrapper: createWrapper(),
      });

      await waitFor(() => {
        expect(result.current.isSuccess).toBe(true);
      });

      expect(result.current.data).toBeUndefined();
      expect(mockPush).toHaveBeenCalledWith('/ui/login');
    });

    it('does not redirect while loading', async () => {
      const { result } = renderHook(() => useAuthenticatedUser(), {
        wrapper: createWrapper(),
      });

      // During initial load
      expect(result.current.isLoading).toBe(true);
      expect(mockPush).not.toHaveBeenCalled();
    });
  });

  describe('useAllUsers', () => {
    it('fetches users list successfully', async () => {
      server.use(
        http.get('/bodhi/v1/users', () => {
          return HttpResponse.json({
            client_id: 'test-client-id',
            users: mockUsersResponse.users,
            page: 1,
            page_size: 10,
            total_pages: 1,
            total_users: mockUsersResponse.total_users,
            has_next: false,
            has_previous: false,
          });
        })
      );

      const { result } = renderHook(() => useAllUsers(1, 10), {
        wrapper: createWrapper(),
      });

      await waitFor(() => {
        expect(result.current.isSuccess).toBe(true);
      });

      expect(result.current.data?.users).toEqual(mockUsersResponse.users);
      expect(result.current.data?.total_users).toBe(mockUsersResponse.total_users);
    });

    it('handles pagination parameters', async () => {
      server.use(
        http.get('/bodhi/v1/users', () => {
          return HttpResponse.json({
            client_id: 'test-client-id',
            users: mockUsersResponse.users,
            page: 2,
            page_size: 5,
            total_pages: 1,
            total_users: mockUsersResponse.total_users,
            has_next: false,
            has_previous: true,
          });
        })
      );

      const { result } = renderHook(() => useAllUsers(2, 5), {
        wrapper: createWrapper(),
      });

      await waitFor(() => {
        expect(result.current.isSuccess).toBe(true);
      });

      // The query key should reflect the pagination parameters
      expect(result.current.data).toBeDefined();
    });

    it('handles error response', async () => {
      server.use(
        http.get('/bodhi/v1/users', () => {
          return HttpResponse.json(
            {
              error: {
                code: 'access_denied',
                message: 'Access denied',
                type: 'forbidden_error',
              },
            },
            { status: 500 }
          );
        })
      );

      const { result } = renderHook(() => useAllUsers(), {
        wrapper: createWrapper(),
      });

      await waitFor(
        () => {
          expect(result.current.isError).toBe(true);
        },
        { timeout: 10000 }
      );

      const error = result.current.error as AxiosError<ApiError>;
      expect(error.response?.data.error?.message).toBe('Access denied');
    });

    it('uses default pagination values', async () => {
      server.use(
        http.get('/bodhi/v1/users', () => {
          return HttpResponse.json({
            client_id: 'test-client-id',
            users: mockUsersResponse.users,
            page: 1,
            page_size: 10,
            total_pages: 1,
            total_users: mockUsersResponse.total_users,
            has_next: false,
            has_previous: false,
          });
        })
      );

      const { result } = renderHook(() => useAllUsers(), {
        wrapper: createWrapper(),
      });

      await waitFor(() => {
        expect(result.current.isSuccess).toBe(true);
      });

      expect(result.current.data).toBeDefined();
    });
  });

  describe('useChangeUserRole', () => {
    it('changes user role successfully', async () => {
      const onSuccess = vi.fn();
      const { result } = renderHook(() => useChangeUserRole({ onSuccess }), {
        wrapper: createWrapper(),
      });

      await act(async () => {
        await result.current.mutateAsync({
          userId: 'test-user-id',
          newRole: 'resource_manager',
        });
      });

      expect(result.current.isSuccess).toBe(true);
      expect(onSuccess).toHaveBeenCalled();
    });

    it('handles error response', async () => {
      server.use(
        ...mockUserRoleChangeError('test-user-id', {
          message: 'Permission denied',
          status: 403,
        })
      );

      const onError = vi.fn();
      const { result } = renderHook(() => useChangeUserRole({ onError }), {
        wrapper: createWrapper(),
      });

      await act(async () => {
        try {
          await result.current.mutateAsync({
            userId: 'test-user-id',
            newRole: 'resource_admin',
          });
        } catch (error) {
          const axiosError = error as AxiosError<ApiError>;
          expect(axiosError.response?.status).toBe(403);
          expect(axiosError.response?.data.error?.message).toBe('Permission denied');
        }
      });

      expect(onError).toHaveBeenCalledWith('Permission denied');
    });

    it('calls onError with fallback message when error message is missing', async () => {
      server.use(
        http.put('/bodhi/v1/users/:userId/role', () => {
          return HttpResponse.json(
            {
              error: {
                type: 'internal_server_error',
                // No message field - should trigger fallback
              },
            },
            { status: 500 }
          );
        })
      );

      const onError = vi.fn();
      const { result } = renderHook(() => useChangeUserRole({ onError }), {
        wrapper: createWrapper(),
      });

      await act(async () => {
        try {
          await result.current.mutateAsync({
            userId: 'test-user-id',
            newRole: 'resource_admin',
          });
        } catch (error) {
          // Error caught
        }
      });

      expect(onError).toHaveBeenCalledWith('Failed to change user role');
    });

    it('invalidates users queries on successful role change', async () => {
      server.use(
        http.get('/bodhi/v1/users', () => {
          return HttpResponse.json({
            client_id: 'test-client-id',
            users: mockUsersResponse.users,
            page: 1,
            page_size: 10,
            total_pages: 1,
            total_users: mockUsersResponse.total_users,
            has_next: false,
            has_previous: false,
          });
        })
      );

      const wrapper = createWrapper();

      // Setup users hook and wait for initial data
      const { result: usersResult } = renderHook(() => useAllUsers(), {
        wrapper,
      });

      await waitFor(() => {
        expect(usersResult.current.isSuccess).toBe(true);
      });

      const initialDataUpdatedAt = usersResult.current.dataUpdatedAt;

      // Change user role
      const { result: changeRoleResult } = renderHook(() => useChangeUserRole(), {
        wrapper,
      });

      await act(async () => {
        await changeRoleResult.current.mutateAsync({
          userId: 'test-user-id',
          newRole: 'resource_manager',
        });
      });

      // Verify users query was invalidated and refetched
      await waitFor(() => {
        expect(usersResult.current.dataUpdatedAt).toBeGreaterThan(initialDataUpdatedAt);
      });
    });
  });

  describe('useRemoveUser', () => {
    it('removes user successfully', async () => {
      const onSuccess = vi.fn();
      const { result } = renderHook(() => useRemoveUser({ onSuccess }), {
        wrapper: createWrapper(),
      });

      await act(async () => {
        await result.current.mutateAsync('test-user-id');
      });

      expect(result.current.isSuccess).toBe(true);
      expect(onSuccess).toHaveBeenCalled();
    });

    it('handles error response', async () => {
      server.use(
        ...mockUserRemoveError('test-user-id', {
          message: 'Cannot remove admin user',
          status: 400,
        })
      );

      const onError = vi.fn();
      const { result } = renderHook(() => useRemoveUser({ onError }), {
        wrapper: createWrapper(),
      });

      await act(async () => {
        try {
          await result.current.mutateAsync('test-user-id');
        } catch (error) {
          const axiosError = error as AxiosError<ApiError>;
          expect(axiosError.response?.status).toBe(400);
          expect(axiosError.response?.data.error?.message).toBe('Cannot remove admin user');
        }
      });

      expect(onError).toHaveBeenCalledWith('Cannot remove admin user');
    });

    it('calls onError with fallback message when error message is missing', async () => {
      server.use(
        http.delete('/bodhi/v1/users/:userId', () => {
          return HttpResponse.json(
            {
              error: {
                type: 'internal_server_error',
                // No message field - should trigger fallback
              },
            },
            { status: 500 }
          );
        })
      );

      const onError = vi.fn();
      const { result } = renderHook(() => useRemoveUser({ onError }), {
        wrapper: createWrapper(),
      });

      await act(async () => {
        try {
          await result.current.mutateAsync('test-user-id');
        } catch (error) {
          // Error caught
        }
      });

      expect(onError).toHaveBeenCalledWith('Failed to remove user');
    });

    it('invalidates users queries on successful user removal', async () => {
      server.use(
        http.get('/bodhi/v1/users', () => {
          return HttpResponse.json({
            client_id: 'test-client-id',
            users: mockUsersResponse.users,
            page: 1,
            page_size: 10,
            total_pages: 1,
            total_users: mockUsersResponse.total_users,
            has_next: false,
            has_previous: false,
          });
        })
      );

      const wrapper = createWrapper();

      // Setup users hook and wait for initial data
      const { result: usersResult } = renderHook(() => useAllUsers(), {
        wrapper,
      });

      await waitFor(() => {
        expect(usersResult.current.isSuccess).toBe(true);
      });

      const initialDataUpdatedAt = usersResult.current.dataUpdatedAt;

      // Remove user
      const { result: removeUserResult } = renderHook(() => useRemoveUser(), {
        wrapper,
      });

      await act(async () => {
        await removeUserResult.current.mutateAsync('test-user-id');
      });

      // Verify users query was invalidated and refetched
      await waitFor(() => {
        expect(usersResult.current.dataUpdatedAt).toBeGreaterThan(initialDataUpdatedAt);
      });
    });
  });
});
