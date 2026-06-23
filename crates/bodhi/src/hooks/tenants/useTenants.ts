import { TenantListResponse, CreateTenantRequest, CreateTenantResponse, BodhiErrorResponse } from '@bodhiapp/ts-client';
import { AxiosError, AxiosResponse } from 'axios';

import { appInfoKeys } from '@/hooks/info/constants';
import { UseMutationResult, useQuery, useMutationQuery, useQueryClient } from '@/hooks/useQuery';
import { userKeys } from '@/hooks/users/constants';
import { extractErrorMessage } from '@/lib/errorUtils';

import { tenantKeys, ENDPOINT_TENANTS } from './constants';

export function useListTenants(options?: { enabled?: boolean }) {
  return useQuery<TenantListResponse>(tenantKeys.all, ENDPOINT_TENANTS, undefined, {
    enabled: options?.enabled ?? true,
  });
}

export function useCreateTenant(options?: {
  onSuccess?: (response: CreateTenantResponse) => void;
  onError?: (message: string) => void;
}): UseMutationResult<AxiosResponse<CreateTenantResponse>, AxiosError<BodhiErrorResponse>, CreateTenantRequest> {
  const queryClient = useQueryClient();

  return useMutationQuery<CreateTenantResponse, CreateTenantRequest>(ENDPOINT_TENANTS, 'post', {
    onSuccess: (response) => {
      queryClient.invalidateQueries({ queryKey: tenantKeys.all });
      options?.onSuccess?.(response.data);
    },
    onError: (error: AxiosError<BodhiErrorResponse>) => {
      options?.onError?.(extractErrorMessage(error, 'Failed to create tenant'));
    },
  });
}

export function useTenantActivate(options?: {
  onSuccess?: () => void;
  onError?: (message: string) => void;
}): UseMutationResult<AxiosResponse<void>, AxiosError<BodhiErrorResponse>, { client_id: string }> {
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
      onError: (error: AxiosError<BodhiErrorResponse>) => {
        options?.onError?.(extractErrorMessage(error, 'Failed to activate tenant'));
      },
    },
    {
      noBody: true,
    }
  );
}
