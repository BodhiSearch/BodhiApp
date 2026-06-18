import * as React from 'react';

import { useGetAppInfo } from '@/hooks/info';
import { useGetUser } from '@/hooks/users';
import { createReferenceApiClient, type ReferenceApiClient } from '@/lib/referenceApiClient';

/**
 * Builds a {@link ReferenceApiClient} pointed at `AppInfo.reference_api_url`, authenticated with the
 * logged-in user's OIDC `id_token`. Returns `null` until both the app-info and user queries resolve
 * with a usable base URL, so callers can gate their own queries on a non-null client.
 *
 * Domain hooks (e.g. an MCP catalog hook) should wrap this in `useQuery` with their own key factory,
 * `enabled: !!client`. The reference APIs themselves are built separately; see
 * docs/claude-plans/202606/screen-v2/reference-api.md.
 */
export function useReferenceApi(): ReferenceApiClient | null {
  const { data: appInfo } = useGetAppInfo();
  const { data: userEnvelope } = useGetUser();

  const baseUrl = appInfo?.reference_api_url;
  const idToken =
    userEnvelope && userEnvelope.auth_status === 'logged_in' ? (userEnvelope.id_token ?? undefined) : undefined;

  return React.useMemo(() => {
    if (!baseUrl) return null;
    return createReferenceApiClient(baseUrl, idToken);
  }, [baseUrl, idToken]);
}
