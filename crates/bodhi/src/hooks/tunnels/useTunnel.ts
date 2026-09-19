import type {
  BodhiErrorResponse,
  EnableTunnelRequest,
  TunnelSetupRequest,
  TunnelStatus,
  UpdateTunnelPreferencesRequest,
} from '@bodhiapp/ts-client';
import type { AxiosError, AxiosResponse } from 'axios';

import {
  useMutationQuery,
  useQuery,
  useQueryClient,
  type UseMutationResult,
  type UseQueryResult,
} from '@/hooks/useQuery';
import { extractErrorMessage } from '@/lib/errorUtils';

import { ENDPOINT_TUNNEL, ENDPOINT_TUNNEL_SETUP, ENDPOINT_TUNNEL_SYNC, tunnelKeys } from './constants';

export function useTunnelStatus(): UseQueryResult<TunnelStatus, AxiosError<BodhiErrorResponse>> {
  return useQuery<TunnelStatus>(tunnelKeys.all, ENDPOINT_TUNNEL, undefined, {
    refetchInterval: (query) => (query.state.data?.state === 'connecting' ? 1000 : 10_000),
  });
}

function useTunnelMutation<V>(
  endpoint: string,
  method: 'post' | 'put' | 'patch' | 'delete',
  options?: { onSuccess?: (status: TunnelStatus) => void; onError?: (message: string) => void }
): UseMutationResult<AxiosResponse<TunnelStatus>, AxiosError<BodhiErrorResponse>, V> {
  const queryClient = useQueryClient();
  return useMutationQuery<TunnelStatus, V>(
    endpoint,
    method,
    {
      onSuccess: ({ data }) => {
        queryClient.setQueryData(tunnelKeys.all, data);
        options?.onSuccess?.(data);
      },
      onError: (error) => {
        // A refused call still changes server-side runtime state — a DNS conflict
        // records its code without starting a connector — so re-read the status
        // rather than waiting for the next poll to reveal it.
        queryClient.invalidateQueries({ queryKey: tunnelKeys.all });
        options?.onError?.(extractErrorMessage(error, 'Tunnel operation failed'));
      },
    },
    method === 'delete' ? { noBody: true } : undefined
  );
}

export function useEnableTunnel(options?: {
  onSuccess?: (status: TunnelStatus) => void;
  onError?: (message: string) => void;
}) {
  return useTunnelMutation<EnableTunnelRequest>(ENDPOINT_TUNNEL, 'put', options);
}

export function useDisableTunnel(options?: {
  onSuccess?: (status: TunnelStatus) => void;
  onError?: (message: string) => void;
}) {
  return useTunnelMutation<void>(ENDPOINT_TUNNEL, 'delete', options);
}

export function useTunnelSetup(options?: {
  onSuccess?: (status: TunnelStatus) => void;
  onError?: (message: string) => void;
}) {
  return useTunnelMutation<TunnelSetupRequest>(ENDPOINT_TUNNEL_SETUP, 'put', options);
}

export function useUpdateTunnelPreferences(options?: {
  onSuccess?: (status: TunnelStatus) => void;
  onError?: (message: string) => void;
}) {
  return useTunnelMutation<UpdateTunnelPreferencesRequest>(ENDPOINT_TUNNEL, 'patch', options);
}

export function useSyncTunnelAuthorization(options?: {
  onSuccess?: (status: TunnelStatus) => void;
  onError?: (message: string) => void;
}) {
  return useTunnelMutation<void>(ENDPOINT_TUNNEL_SYNC, 'post', options);
}
