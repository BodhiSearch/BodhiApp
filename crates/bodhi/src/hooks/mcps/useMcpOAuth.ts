import {
  OAuthDiscoverMcpRequest,
  OAuthDiscoverMcpResponse,
  DynamicRegisterRequest,
  DynamicRegisterResponse,
  OAuthLoginRequest,
  OAuthLoginResponse,
  OAuthTokenExchangeRequest,
  OAuthTokenResponse,
  BodhiErrorResponse,
} from '@bodhiapp/ts-client';
import { AxiosError, AxiosResponse } from 'axios';

import { useMutationQuery } from '@/hooks/useQuery';
import { UseMutationResult } from '@/hooks/useQuery';

import {
  MCPS_AUTH_CONFIGS_ENDPOINT,
  MCPS_OAUTH_DISCOVER_MCP_ENDPOINT,
  MCPS_OAUTH_DYNAMIC_REGISTER_STANDALONE_ENDPOINT,
} from './constants';

// ============================================================================
// Mutation Hooks - OAuth Discovery & Login & Token
// ============================================================================

export function useDiscoverMcp(options?: {
  onSuccess?: (response: OAuthDiscoverMcpResponse) => void;
  onError?: (message: string) => void;
}): UseMutationResult<
  AxiosResponse<OAuthDiscoverMcpResponse>,
  AxiosError<BodhiErrorResponse>,
  OAuthDiscoverMcpRequest
> {
  return useMutationQuery<OAuthDiscoverMcpResponse, OAuthDiscoverMcpRequest>(
    () => MCPS_OAUTH_DISCOVER_MCP_ENDPOINT,
    'post',
    {
      onSuccess: (response) => options?.onSuccess?.(response.data),
      onError: (error: AxiosError<BodhiErrorResponse>) => {
        const message = error?.response?.data?.error?.message || 'Failed to discover MCP OAuth endpoints';
        options?.onError?.(message);
      },
    }
  );
}

export function useStandaloneDynamicRegister(options?: {
  onSuccess?: (response: DynamicRegisterResponse) => void;
  onError?: (message: string) => void;
}): UseMutationResult<AxiosResponse<DynamicRegisterResponse>, AxiosError<BodhiErrorResponse>, DynamicRegisterRequest> {
  return useMutationQuery<DynamicRegisterResponse, DynamicRegisterRequest>(
    () => MCPS_OAUTH_DYNAMIC_REGISTER_STANDALONE_ENDPOINT,
    'post',
    {
      onSuccess: (response) => options?.onSuccess?.(response.data),
      onError: (error: AxiosError<BodhiErrorResponse>) => {
        const message = error?.response?.data?.error?.message || 'Failed to register dynamic client';
        options?.onError?.(message);
      },
    }
  );
}

export function useOAuthLogin(options?: {
  onSuccess?: (response: OAuthLoginResponse) => void;
  onError?: (message: string) => void;
}): UseMutationResult<
  AxiosResponse<OAuthLoginResponse>,
  AxiosError<BodhiErrorResponse>,
  OAuthLoginRequest & { id: string }
> {
  return useMutationQuery<OAuthLoginResponse, OAuthLoginRequest & { id: string }>(
    ({ id }) => `${MCPS_AUTH_CONFIGS_ENDPOINT}/${id}/login`,
    'post',
    {
      onSuccess: (response) => options?.onSuccess?.(response.data),
      onError: (error: AxiosError<BodhiErrorResponse>) => {
        const message = error?.response?.data?.error?.message || 'Failed to initiate OAuth login';
        options?.onError?.(message);
      },
    },
    { transformBody: ({ id: _id, ...body }) => body }
  );
}

export function useOAuthTokenExchange(options?: {
  onSuccess?: (response: OAuthTokenResponse) => void;
  onError?: (message: string) => void;
}): UseMutationResult<
  AxiosResponse<OAuthTokenResponse>,
  AxiosError<BodhiErrorResponse>,
  OAuthTokenExchangeRequest & { id: string }
> {
  return useMutationQuery<OAuthTokenResponse, OAuthTokenExchangeRequest & { id: string }>(
    ({ id }) => `${MCPS_AUTH_CONFIGS_ENDPOINT}/${id}/token`,
    'post',
    {
      onSuccess: (response) => options?.onSuccess?.(response.data),
      onError: (error: AxiosError<BodhiErrorResponse>) => {
        const message = error?.response?.data?.error?.message || 'Failed to exchange OAuth token';
        options?.onError?.(message);
      },
    },
    { transformBody: ({ id: _id, ...body }) => body }
  );
}

// Re-export types for consumers
export type {
  OAuthDiscoverMcpRequest,
  OAuthDiscoverMcpResponse,
  DynamicRegisterRequest,
  DynamicRegisterResponse,
  OAuthLoginRequest,
  OAuthLoginResponse,
  OAuthTokenExchangeRequest,
  OAuthTokenResponse,
};
