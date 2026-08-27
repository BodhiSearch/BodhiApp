import {
  AppAccessSummary,
  BodhiErrorResponse,
  ConsentAppInfo,
  ConsentContextResponse,
  ConsentDecision,
  ConsentPriorGrant,
  ConsentScopeInfo,
  ListAppAccessResponse,
  SubmitConsentRequest,
  SubmitConsentResponse,
} from '@bodhiapp/ts-client';
import { AxiosError, AxiosResponse } from 'axios';

import { useMutationQuery, useQuery, useQueryClient } from '@/hooks/useQuery';
import { UseMutationResult, UseQueryResult } from '@/hooks/useQuery';
import { extractErrorMessage } from '@/lib/errorUtils';

import {
  appAccessRequestKeys,
  ENDPOINT_ACCESS_REQUESTS,
  ENDPOINT_ACCESS_REQUESTS_APPS,
  ENDPOINT_APPS_ACCESS_REQUESTS,
  ENDPOINT_APPS_ACCESS_REQUESTS_CONSENT,
} from './constants';

export type {
  AppAccessSummary,
  ConsentAppInfo,
  ConsentContextResponse,
  ConsentDecision,
  ConsentPriorGrant,
  ConsentScopeInfo,
  ListAppAccessResponse,
  SubmitConsentRequest,
  SubmitConsentResponse,
};

/**
 * `search` is the page's raw, already-encoded query string (leading '?').
 * It is appended verbatim — axios `params` would re-encode and corrupt it.
 */
export function useGetConsentContext(
  search: string
): UseQueryResult<ConsentContextResponse, AxiosError<BodhiErrorResponse>> {
  const endpoint = search ? `${ENDPOINT_APPS_ACCESS_REQUESTS_CONSENT}${search}` : ENDPOINT_APPS_ACCESS_REQUESTS_CONSENT;
  return useQuery<ConsentContextResponse>(appAccessRequestKeys.consent(search), endpoint, undefined, {
    retry: false,
  });
}

export function useSubmitConsent(options?: {
  onSuccess?: (data: SubmitConsentResponse) => void;
  onError?: (message: string) => void;
}): UseMutationResult<AxiosResponse<SubmitConsentResponse>, AxiosError<BodhiErrorResponse>, SubmitConsentRequest> {
  const queryClient = useQueryClient();
  return useMutationQuery<SubmitConsentResponse, SubmitConsentRequest>(ENDPOINT_APPS_ACCESS_REQUESTS, 'post', {
    onSuccess: (response) => {
      // Only the grants list — invalidating `.all` would refetch the consent query mid-redirect.
      queryClient.invalidateQueries({ queryKey: appAccessRequestKeys.list() });
      options?.onSuccess?.(response.data);
    },
    onError: (error: AxiosError<BodhiErrorResponse>) => {
      options?.onError?.(extractErrorMessage(error, 'Failed to submit consent decision'));
    },
  });
}

export function useListAppAccess(options?: {
  enabled?: boolean;
}): UseQueryResult<ListAppAccessResponse, AxiosError<BodhiErrorResponse>> {
  return useQuery<ListAppAccessResponse>(appAccessRequestKeys.list(), ENDPOINT_ACCESS_REQUESTS_APPS, undefined, {
    retry: false,
    ...options,
  });
}

export function useRevokeAppAccess(options?: {
  onSuccess?: (data: AppAccessSummary) => void;
  onError?: (message: string) => void;
}): UseMutationResult<AxiosResponse<AppAccessSummary>, AxiosError<BodhiErrorResponse>, { id: string }> {
  const queryClient = useQueryClient();
  return useMutationQuery<AppAccessSummary, { id: string }>(
    ({ id }) => `${ENDPOINT_ACCESS_REQUESTS}/${id}/revoke`,
    'post',
    {
      onSuccess: (response) => {
        queryClient.invalidateQueries({ queryKey: appAccessRequestKeys.all });
        options?.onSuccess?.(response.data);
      },
      onError: (error: AxiosError<BodhiErrorResponse>) => {
        options?.onError?.(extractErrorMessage(error, 'Failed to revoke app access'));
      },
    },
    {
      transformBody: () => undefined,
    }
  );
}
