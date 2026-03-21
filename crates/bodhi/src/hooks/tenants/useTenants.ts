// External imports
import { TenantListResponse, CreateTenantRequest, CreateTenantResponse, OpenAiApiError } from '@bodhiapp/ts-client';
import { AxiosError, AxiosResponse } from 'axios';

// Internal imports
import { appInfoKeys } from '@/hooks/info/constants';
import { UseMutationResult, useQuery, useMutationQuery, useQueryClient } from '@/hooks/useQuery';
import { userKeys } from '@/hooks/users/constants';

import { tenantKeys, ENDPOINT_TENANTS } from './constants';

// Type alias
type ErrorResponse = OpenAiApiError;

// List tenants
export function useListTenants(options?: { enabled?: boolean }) {
  return useQuery<TenantListResponse>(tenantKeys.all, ENDPOINT_TENANTS, undefined, {
    enabled: options?.enabled ?? true,
  });
}

// Create tenant
export function useCreateTenant(options?: {
  onSuccess?: (response: CreateTenantResponse) => void;
  onError?: (message: string) => void;
}): UseMutationResult<AxiosResponse<CreateTenantResponse>, AxiosError<ErrorResponse>, CreateTenantRequest> {
  const queryClient = useQueryClient();

  return useMutationQuery<CreateTenantResponse, CreateTenantRequest>(ENDPOINT_TENANTS, 'post', {
    onSuccess: (response) => {
      queryClient.invalidateQueries({ queryKey: tenantKeys.all });
      options?.onSuccess?.(response.data);
    },
    onError: (error: AxiosError<ErrorResponse>) => {
      const message = error?.response?.data?.error?.message || 'Failed to create tenant';
      options?.onError?.(message);
    },
  });
}

// Activate tenant
export function useTenantActivate(options?: {
  onSuccess?: () => void;
  onError?: (message: string) => void;
}): UseMutationResult<AxiosResponse<void>, AxiosError<ErrorResponse>, { client_id: string }> {
  const queryClient = useQueryClient();

  return useMutationQuery<void, { client_id: string }>(
    ({ client_id }) => `${ENDPOINT_TENANTS}/${client_id}/activate`,
    'post',
    {
      onSuccess: () => {
        queryClient.invalidateQueries({ queryKey: tenantKeys.all });
        queryClient.invalidateQueries({ queryKey: appInfoKeys.all });
        queryClient.invalidateQueries({ queryKey: userKeys.current });
        options?.onSuccess?.();
      },
      onError: (error: AxiosError<ErrorResponse>) => {
        const message = error?.response?.data?.error?.message || 'Failed to activate tenant';
        options?.onError?.(message);
      },
    },
    {
      noBody: true,
    }
  );
}
